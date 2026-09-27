//! twilight でDiscordのgatewayにつなぎ、出来事を受けて `state` を更新する（B-43〜B-46）。
//! 対象のVCに人が入ったら songbird で入り、`beep.wav` を1回鳴らす（AT-01の形。まだ読み上げの文は作らない）。
//! `beep.wav` は2周目だけの仮の音。4周目で `speech`・`tts` を使う形に置き換えて外す。

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex as StdMutex},
};

use anyhow::{Context as _, Result, bail};
use async_trait::async_trait;
use songbird::{
    CoreEvent, Event as SongbirdEvent, EventContext, EventHandler, Songbird, TrackEvent,
    error::JoinError, shards::TwilightMap,
};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};
use twilight_model::{
    gateway::payload::incoming::{GuildCreate, MessageCreate, VoiceStateUpdate},
    id::{
        Id,
        marker::{ChannelMarker, GuildMarker, UserMarker},
    },
};

use crate::state::{Config, State};

/// gatewayが受ける出来事の種類（`GatewayClose` はフラグに関わらず常に来る）。
const WANTED_EVENTS: EventTypeFlags = EventTypeFlags::GUILD_CREATE
    .union(EventTypeFlags::VOICE_STATE_UPDATE)
    .union(EventTypeFlags::VOICE_SERVER_UPDATE)
    .union(EventTypeFlags::MESSAGE_CREATE);

/// 2周目だけの仮の音（AT-01の確認用）。
const BEEP_WAV: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/beep.wav");

/// 参加を試みている途中のサーバーの一覧。`songbird.get` に登録されるまでの隙間で、
/// 同じサーバーへ二重に `join_and_beep` を起動しないためのもの。
type JoiningGuilds = Arc<StdMutex<HashSet<Id<GuildMarker>>>>;

/// Discordにつなぐのに必要な intent（B-45 のもとになる Message Content Intent を含む）。
fn intents() -> Intents {
    Intents::GUILDS
        | Intents::GUILD_MESSAGES
        | Intents::GUILD_VOICE_STATES
        | Intents::MESSAGE_CONTENT
}

/// songbirdのdriverの接続状態の変化をログに出す（test-items ⑮・4節の5番）。
/// `DriverDisconnect` の `reason` で、切断（`Requested`・`WsClosed`）と
/// ネットの不調（`Io`・`TimedOut`）を見分けられるか、手動テストで確かめる。
struct VoiceConnectionLogger {
    guild_id: Id<GuildMarker>,
}

#[async_trait]
impl EventHandler for VoiceConnectionLogger {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<SongbirdEvent> {
        match ctx {
            EventContext::DriverConnect(data) => {
                tracing::info!(guild_id = %self.guild_id, ?data, "songbird: DriverConnect");
            }
            EventContext::DriverReconnect(data) => {
                tracing::info!(guild_id = %self.guild_id, ?data, "songbird: DriverReconnect");
            }
            EventContext::DriverDisconnect(data) => {
                tracing::info!(
                    guild_id = %self.guild_id,
                    kind = ?data.kind,
                    reason = ?data.reason,
                    "songbird: DriverDisconnect（B-12・B-14・B-15のどれかは4節5番で判定）"
                );
            }
            _ => {}
        }
        None
    }
}

/// トラックの再生の失敗（`TrackEvent::Error`）をログに出す（B-47）。
/// `play_input` は失敗を返さず、songbird の中でトラックを `PlayMode::Errored` にするだけなので、知らせで受ける。
struct TrackErrorLogger {
    guild_id: Id<GuildMarker>,
}

#[async_trait]
impl EventHandler for TrackErrorLogger {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<SongbirdEvent> {
        if let EventContext::Track(tracks) = ctx {
            for (state, _) in *tracks {
                tracing::warn!(
                    guild_id = %self.guild_id,
                    playing = ?state.playing,
                    "beep.wav を再生できなかった（B-47）"
                );
            }
        }
        None
    }
}

/// 対象のVCに、スピーカーミュートで入り、`beep.wav` を1回鳴らす（AT-01）。
/// wavはメモリ上の `Vec<u8>` として songbird に渡す。一時ファイルは作らない（AT-37・4節の4番・P-6）。
/// `songbird.process` を呼ぶループとは別のタスクで呼ぶこと（デッドロックを避けるため）。
async fn join_and_beep(
    songbird: &Songbird,
    guild_id: Id<GuildMarker>,
    voice_channel_id: Id<ChannelMarker>,
) -> Result<()> {
    let beep = std::fs::read(BEEP_WAV).with_context(|| format!("{BEEP_WAV} を読めなかった"))?;
    let call = songbird.join(guild_id, voice_channel_id).await?;
    {
        let mut handler = call.lock().await;
        handler.deafen(true).await?;
        handler.add_global_event(
            SongbirdEvent::Core(CoreEvent::DriverConnect),
            VoiceConnectionLogger { guild_id },
        );
        handler.add_global_event(
            SongbirdEvent::Core(CoreEvent::DriverReconnect),
            VoiceConnectionLogger { guild_id },
        );
        handler.add_global_event(
            SongbirdEvent::Core(CoreEvent::DriverDisconnect),
            VoiceConnectionLogger { guild_id },
        );
        // play_input より先に登録する（トラックが Errored になる前に、知らせを受ける側をそろえる）。
        handler.add_global_event(
            SongbirdEvent::Track(TrackEvent::Error),
            TrackErrorLogger { guild_id },
        );
        handler.play_input(beep.into());
    }
    tracing::info!(%guild_id, %voice_channel_id, "対象のVCに入り、beep.wavを鳴らした（AT-01）");
    Ok(())
}

/// クローズコードから、理由の分かる誤りかどうかを見る（B-45）。
fn bail_if_fatal_close(code: Option<u16>) -> Result<()> {
    if code == Some(4004) {
        bail!("Discordにつなげなかった（クローズコード4004）。トークンが間違っている");
    }
    if code == Some(4014) {
        bail!(
            "Discordにつなげなかった（クローズコード4014）。\
             Message Content Intent がオフになっている"
        );
    }
    Ok(())
}

/// まだ対象のVCに参加していなければ、`join_and_beep` を別タスクで起動する。
/// すでに参加中／参加を試みている途中なら何もしない（二重参加を防ぐ）。
fn try_join(config: &Config, songbird: &Arc<Songbird>, joining: &JoiningGuilds) {
    if songbird.get(config.guild_id).is_some() {
        return;
    }
    // すでに参加を試みている途中なら、二重に join_and_beep を起動しない
    // （このチェックと実際の songbird への登録の間に隙間があるため）。
    let already_joining = !joining.lock().unwrap().insert(config.guild_id);
    if already_joining {
        return;
    }

    let songbird = Arc::clone(songbird);
    let joining = Arc::clone(joining);
    let guild_id = config.guild_id;
    let voice_channel_id = config.voice_channel_id;
    tokio::spawn(async move {
        if let Err(error) = join_and_beep(&songbird, guild_id, voice_channel_id).await {
            tracing::warn!(error = %format!("{error:#}"), "対象のVCへの参加に失敗した（B-17）");
            // songbird の join は失敗しても Call を残す（manager.rs の join の doc）。
            // 残したままだと、上の songbird.get で次の入室でも入り直さなくなる（B-17）。
            match songbird.remove(guild_id).await {
                Ok(()) | Err(JoinError::NoCall) => {}
                Err(error) => tracing::warn!(%error, "失敗した参加の Call を消せなかった"),
            }
        }
        joining.lock().unwrap().remove(&guild_id);
    });
}

/// 起動時に対象のVCに人（botでない）がいれば、次の入室を待たずに入る（B-10・AT-09）。
fn handle_guild_create(
    guild_create: Box<GuildCreate>,
    config: &Config,
    state: &mut State,
    songbird: &Arc<Songbird>,
    joining: &JoiningGuilds,
) {
    if guild_create.id() != config.guild_id {
        return;
    }
    let GuildCreate::Available(guild) = *guild_create else {
        tracing::warn!(guild_id = %config.guild_id, "設定のサーバーが見つからない（B-43）");
        return;
    };

    let is_bot: HashMap<_, _> = guild
        .members
        .iter()
        .map(|member| (member.user.id, member.user.bot))
        .collect();
    state.voice_channel_members = guild
        .voice_states
        .iter()
        .filter(|voice_state| voice_state.channel_id == Some(config.voice_channel_id))
        .map(|voice_state| voice_state.user_id)
        .filter(|user_id| is_bot.get(user_id) != Some(&true))
        .collect();

    tracing::info!(
        guild_id = %guild.id,
        voice_states = ?guild.voice_states.iter()
            .map(|vs| (vs.user_id, vs.channel_id, vs.member.is_some()))
            .collect::<Vec<_>>(),
        "GUILD_CREATE を受けた（4節20番の確認：member があれば人かbotかが分かる）"
    );
    tracing::info!(
        members_count = guild.members.len(),
        members = ?guild.members.iter()
            .map(|member| (member.user.id, member.user.bot))
            .collect::<Vec<_>>(),
        "GUILD_CREATE の members（4節20番：voice_states の3人が含まれるか）"
    );

    if !state.voice_channel_members.is_empty() {
        tracing::info!(
            guild_id = %config.guild_id,
            count = state.voice_channel_members.len(),
            "起動時に対象のVCに人がいるので、すぐ入る（B-10）"
        );
        try_join(config, songbird, joining);
    }
}

/// 対象のVCに人（bot自身以外）が入ったら、songbirdで入って `beep.wav` を鳴らすタスクを起動する（AT-01）。
fn handle_voice_state_update(
    voice_state: &VoiceStateUpdate,
    config: &Config,
    bot_user_id: Id<UserMarker>,
    songbird: &Arc<Songbird>,
    joining: &JoiningGuilds,
) {
    if voice_state.guild_id != Some(config.guild_id) {
        return;
    }
    tracing::info!(
        user_id = %voice_state.user_id,
        channel_id = ?voice_state.channel_id,
        has_member = voice_state.member.is_some(),
        "VOICE_STATE_UPDATE を受けた"
    );

    let joined_target_channel = voice_state.channel_id == Some(config.voice_channel_id)
        && voice_state.user_id != bot_user_id;
    if joined_target_channel {
        try_join(config, songbird, joining);
    }
}

fn handle_message_create(message: &MessageCreate, config: &Config) {
    if message.guild_id != Some(config.guild_id) {
        return;
    }
    tracing::info!(
        channel_id = %message.channel_id,
        author_id = %message.author.id,
        is_text_channel = message.channel_id == config.text_channel_id,
        content_len = message.content.chars().count(),
        "MESSAGE_CREATE を受けた"
    );
}

/// Discordにつなぎ、出来事をログに出し続ける。トークンが間違っている、または
/// Message Content Intent がオフのとき（クローズコード 4004・4014）は、理由を返して終了する（B-45）。
pub async fn run(token: String, config: &Config, bot_user_id: Id<UserMarker>) -> Result<()> {
    let mut state = State::default();
    let mut shard = Shard::new(ShardId::ONE, token, intents());

    let mut senders = HashMap::new();
    senders.insert(ShardId::ONE.number(), shard.sender());
    let songbird = Arc::new(Songbird::twilight(
        Arc::new(TwilightMap::new(senders)),
        bot_user_id,
    ));
    let joining: JoiningGuilds = Arc::new(StdMutex::new(HashSet::new()));

    while let Some(item) = shard.next_event(WANTED_EVENTS).await {
        let event = match item {
            Ok(event) => event,
            Err(source) => {
                tracing::warn!(?source, "gatewayからの受信に失敗した");
                continue;
            }
        };

        songbird.process(&event).await;

        match event {
            Event::GatewayClose(close) => {
                let code = close.as_ref().map(|frame| frame.code);
                bail_if_fatal_close(code)?;
                tracing::warn!(?code, "gatewayとのつながりが閉じた。つなぎ直しを待つ");
            }
            Event::GuildCreate(guild_create) => {
                handle_guild_create(guild_create, config, &mut state, &songbird, &joining);
            }
            Event::VoiceStateUpdate(voice_state) => {
                handle_voice_state_update(&voice_state, config, bot_user_id, &songbird, &joining);
            }
            Event::MessageCreate(message) => handle_message_create(&message, config),
            _ => {}
        }
    }

    Ok(())
}

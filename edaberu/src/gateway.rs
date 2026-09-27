//! twilight でDiscordのgatewayにつなぎ、出来事を受けて `state` を更新する（B-43〜B-46）。
//! 対象のVCに人が入ったら songbird で入り、`beep.wav` を1回鳴らす（AT-01の形。まだ読み上げの文は作らない）。
//! `beep.wav` は2周目だけの仮の音。4周目で `speech`・`tts` を使う形に置き換えて外す。

use std::{collections::HashMap, sync::Arc};

use anyhow::{Result, bail};
use songbird::{Songbird, input::File as SongbirdFile, shards::TwilightMap};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};
use twilight_model::id::{
    Id,
    marker::{ChannelMarker, GuildMarker, UserMarker},
};

use crate::state::{Config, State};

/// gatewayが受ける出来事の種類（`GatewayClose` はフラグに関わらず常に来る）。
const WANTED_EVENTS: EventTypeFlags = EventTypeFlags::GUILD_CREATE
    .union(EventTypeFlags::VOICE_STATE_UPDATE)
    .union(EventTypeFlags::VOICE_SERVER_UPDATE)
    .union(EventTypeFlags::MESSAGE_CREATE);

/// 2周目だけの仮の音（AT-01の確認用）。
const BEEP_WAV: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/beep.wav");

/// Discordにつなぐのに必要な intent（B-45 のもとになる Message Content Intent を含む）。
fn intents() -> Intents {
    Intents::GUILDS
        | Intents::GUILD_MESSAGES
        | Intents::GUILD_VOICE_STATES
        | Intents::MESSAGE_CONTENT
}

/// 対象のVCに、スピーカーミュートで入り、`beep.wav` を1回鳴らす（AT-01）。
/// `songbird.process` を呼ぶループとは別のタスクで呼ぶこと（デッドロックを避けるため）。
async fn join_and_beep(
    songbird: &Songbird,
    guild_id: Id<GuildMarker>,
    voice_channel_id: Id<ChannelMarker>,
) -> Result<()> {
    let call = songbird.join(guild_id, voice_channel_id).await?;
    {
        let mut handler = call.lock().await;
        handler.deafen(true).await?;
        handler.play_input(SongbirdFile::new(BEEP_WAV).into());
    }
    tracing::info!(%guild_id, %voice_channel_id, "対象のVCに入り、beep.wavを鳴らした（AT-01）");
    Ok(())
}

/// Discordにつなぎ、出来事をログに出し続ける。トークンが間違っている、または
/// Message Content Intent がオフのとき（クローズコード 4014・確度B）は、理由を返して終了する（B-45）。
pub async fn run(token: String, config: &Config, bot_user_id: Id<UserMarker>) -> Result<()> {
    let mut state = State::default();
    let mut shard = Shard::new(ShardId::ONE, token, intents());

    let mut senders = HashMap::new();
    senders.insert(ShardId::ONE.number(), shard.sender());
    let songbird = Arc::new(Songbird::twilight(
        Arc::new(TwilightMap::new(senders)),
        bot_user_id,
    ));

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
                if code == Some(4014) {
                    bail!(
                        "Discordにつなげなかった（クローズコード4014）。\
                         トークンが間違っているか、Message Content Intent がオフになっている"
                    );
                }
                tracing::warn!(?code, "gatewayとのつながりが閉じた。つなぎ直しを待つ");
            }
            Event::GuildCreate(guild_create) => {
                if guild_create.id() != config.guild_id {
                    continue;
                }
                if let twilight_model::gateway::payload::incoming::GuildCreate::Available(guild) =
                    *guild_create
                {
                    state.voice_channel_members = guild
                        .voice_states
                        .iter()
                        .filter(|voice_state| {
                            voice_state.channel_id == Some(config.voice_channel_id)
                        })
                        .map(|voice_state| voice_state.user_id)
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
                } else {
                    tracing::warn!(guild_id = %config.guild_id, "設定のサーバーが見つからない（B-43）");
                }
            }
            Event::VoiceStateUpdate(voice_state) => {
                tracing::info!(
                    user_id = %voice_state.user_id,
                    channel_id = ?voice_state.channel_id,
                    has_member = voice_state.member.is_some(),
                    "VOICE_STATE_UPDATE を受けた"
                );

                let joined_target_channel = voice_state.channel_id == Some(config.voice_channel_id)
                    && voice_state.user_id != bot_user_id;
                if joined_target_channel && songbird.get(config.guild_id).is_none() {
                    let songbird = Arc::clone(&songbird);
                    let guild_id = config.guild_id;
                    let voice_channel_id = config.voice_channel_id;
                    tokio::spawn(async move {
                        if let Err(error) =
                            join_and_beep(&songbird, guild_id, voice_channel_id).await
                        {
                            tracing::warn!(?error, "対象のVCへの参加に失敗した（B-17）");
                        }
                    });
                }
            }
            Event::MessageCreate(message) => {
                tracing::info!(
                    channel_id = %message.channel_id,
                    author_id = %message.author.id,
                    is_text_channel = message.channel_id == config.text_channel_id,
                    content_len = message.content.chars().count(),
                    "MESSAGE_CREATE を受けた"
                );
            }
            _ => {}
        }
    }

    Ok(())
}

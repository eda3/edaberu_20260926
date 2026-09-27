//! twilight でDiscordのgatewayにつなぎ、出来事を受けて `state` を更新する（B-43〜B-46）。

use anyhow::{Result, bail};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};

use crate::state::{Config, State};

/// gatewayが受ける出来事の種類（GatewayClose はフラグに関わらず常に来る）。
const WANTED_EVENTS: EventTypeFlags = EventTypeFlags::GUILD_CREATE
    .union(EventTypeFlags::VOICE_STATE_UPDATE)
    .union(EventTypeFlags::MESSAGE_CREATE);

/// Discordにつなぐのに必要な intent（B-45 のもとになる Message Content Intent を含む）。
fn intents() -> Intents {
    Intents::GUILDS
        | Intents::GUILD_MESSAGES
        | Intents::GUILD_VOICE_STATES
        | Intents::MESSAGE_CONTENT
}

/// Discordにつなぎ、出来事をログに出し続ける。トークンが間違っている、または
/// Message Content Intent がオフのとき（クローズコード 4014・確度B）は、理由を返して終了する（B-45）。
pub async fn run(token: String, config: &Config) -> Result<()> {
    let mut state = State::default();
    let mut shard = Shard::new(ShardId::ONE, token, intents());

    while let Some(item) = shard.next_event(WANTED_EVENTS).await {
        let event = match item {
            Ok(event) => event,
            Err(source) => {
                tracing::warn!(?source, "gatewayからの受信に失敗した");
                continue;
            }
        };

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
                    "VOICE_STATE_UPDATE を受けた"
                );
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

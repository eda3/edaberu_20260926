//! edaberu: Discord のチャット読み上げbot。

mod gateway;
mod state;

use std::env;

use state::Config;
use twilight_model::id::Id;

/// 起動に必要な設定。まだ `edaberu_core::config` が無いので、環境変数から最小限だけ読む
/// （3周目で `config.toml` を読む形に置き換える）。
fn load_config() -> anyhow::Result<(String, Config)> {
    let token = env::var("DISCORD_TOKEN")
        .map_err(|_| anyhow::anyhow!("環境変数 DISCORD_TOKEN が無い（B-44）"))?;
    let guild_id = env::var("EDABERU_GUILD_ID")
        .map_err(|_| anyhow::anyhow!("環境変数 EDABERU_GUILD_ID が無い（B-41）"))?
        .parse()
        .map_err(|_| anyhow::anyhow!("EDABERU_GUILD_ID が数字でない"))?;
    let text_channel_id = env::var("EDABERU_TEXT_CHANNEL_ID")
        .map_err(|_| anyhow::anyhow!("環境変数 EDABERU_TEXT_CHANNEL_ID が無い（B-41）"))?
        .parse()
        .map_err(|_| anyhow::anyhow!("EDABERU_TEXT_CHANNEL_ID が数字でない"))?;
    let voice_channel_id = env::var("EDABERU_VOICE_CHANNEL_ID")
        .map_err(|_| anyhow::anyhow!("環境変数 EDABERU_VOICE_CHANNEL_ID が無い（B-41）"))?
        .parse()
        .map_err(|_| anyhow::anyhow!("EDABERU_VOICE_CHANNEL_ID が数字でない"))?;

    Ok((
        token,
        Config {
            guild_id: Id::new(guild_id),
            text_channel_id: Id::new(text_channel_id),
            voice_channel_id: Id::new(voice_channel_id),
        },
    ))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (token, config) = match load_config() {
        Ok(loaded) => loaded,
        Err(error) => {
            tracing::error!("{error}");
            std::process::exit(1);
        }
    };

    gateway::run(token, &config).await
}

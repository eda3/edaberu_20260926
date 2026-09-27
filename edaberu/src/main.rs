//! edaberu: Discord のチャット読み上げbot。

mod gateway;
mod state;

use std::env;

use anyhow::Context as _;
use state::Config;
use twilight_model::id::{Id, marker::UserMarker};

/// 環境変数を読み、`Id` に変換する（0は不正な値としてエラーにする。B-41）。
fn env_id<T>(name: &str) -> anyhow::Result<Id<T>> {
    let value = env::var(name).with_context(|| format!("環境変数 {name} が無い（B-41）"))?;
    let number: u64 = value
        .parse()
        .with_context(|| format!("{name} が数字でない（{value}）"))?;
    Id::new_checked(number).with_context(|| format!("{name} は0にできない"))
}

/// 起動に必要な設定。まだ `edaberu_core::config` が無いので、環境変数から最小限だけ読む
/// （3周目で `config.toml` を読む形に置き換える）。
fn load_config() -> anyhow::Result<(String, Config)> {
    let token = env::var("DISCORD_TOKEN").context("環境変数 DISCORD_TOKEN が無い（B-44）")?;

    Ok((
        token,
        Config {
            guild_id: env_id("EDABERU_GUILD_ID")?,
            text_channel_id: env_id("EDABERU_TEXT_CHANNEL_ID")?,
            voice_channel_id: env_id("EDABERU_VOICE_CHANNEL_ID")?,
        },
    ))
}

/// bot自身のユーザーIDをRESTで取る（B-45：トークンが間違っていればここで失敗する）。
async fn fetch_bot_user_id(token: &str) -> anyhow::Result<Id<UserMarker>> {
    let http = twilight_http::Client::new(token.to_owned());
    let user = http
        .current_user()
        .await
        .context("Discordに現在のユーザーを問い合わせられなかった（B-45）")?
        .model()
        .await
        .context("現在のユーザーの返事を読めなかった")?;
    Ok(user.id)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // RUST_LOG が無ければ info を既定にする（VPSの journal を空にしないため）。
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let (token, config) = match load_config() {
        Ok(loaded) => loaded,
        Err(error) => {
            tracing::error!("{error:#}");
            std::process::exit(1);
        }
    };

    let bot_user_id = match fetch_bot_user_id(&token).await {
        Ok(id) => id,
        Err(error) => {
            tracing::error!("{error:#}");
            std::process::exit(1);
        }
    };

    if let Err(error) = gateway::run(token, &config, bot_user_id).await {
        tracing::error!("{error:#}");
        std::process::exit(1);
    }

    Ok(())
}

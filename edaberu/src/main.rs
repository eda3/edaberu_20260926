//! edaberu: Discord のチャット読み上げbot。

mod gateway;
mod state;

use std::{env, io::IsTerminal as _};

use anyhow::Context as _;
use edaberu_core::config::check_token;
use state::Config;
use tracing_subscriber::{EnvFilter, filter::LevelFilter};
use twilight_model::id::{Id, marker::UserMarker};

/// 終了の理由を出すログの target。`RUST_LOG` の値に関わらず、この target の ERROR は必ず出す
/// （B-39・B-41・B-44・B-45 の「理由を表示して終了する」）。
const EXIT_TARGET: &str = "edaberu::exit";

/// 環境変数を読み、`Id` に変換する（0は不正な値としてエラーにする。B-41・B-39）。
fn env_id<T>(name: &str) -> anyhow::Result<Id<T>> {
    let value = env::var(name).with_context(|| format!("環境変数 {name} が無い（B-41）"))?;
    let number: u64 = value
        .parse()
        .with_context(|| format!("{name} が数字でない（B-39）"))?;
    Id::new_checked(number).with_context(|| format!("{name} は0にできない（B-39）"))
}

/// 起動に必要な設定。4周目の配線で `edaberu_core::config`（`config.toml`）に置き換えるまで、仮に環境変数から最小限だけ読む。
fn load_config() -> anyhow::Result<(String, Config)> {
    let token = check_token(env::var("DISCORD_TOKEN"))?;

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
    // RUST_LOG が無い・空なら info を既定にする（VPSの journal を空にしないため）。
    // RUST_LOG に何が入っていても、終了の理由（EXIT_TARGET の ERROR）は捨てない。
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive(format!("{EXIT_TARGET}=error").parse()?);
    // 端末でないとき（systemd の journal など）は、色の制御文字を出さない。
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_ansi(std::io::stdout().is_terminal())
        .init();

    let (token, config) = match load_config() {
        Ok(loaded) => loaded,
        Err(error) => {
            tracing::error!(target: EXIT_TARGET, "{error:#}");
            std::process::exit(1);
        }
    };

    let bot_user_id = match fetch_bot_user_id(&token).await {
        Ok(id) => id,
        Err(error) => {
            tracing::error!(target: EXIT_TARGET, "{error:#}");
            std::process::exit(1);
        }
    };

    if let Err(error) = gateway::run(token, &config, bot_user_id).await {
        tracing::error!(target: EXIT_TARGET, "{error:#}");
        std::process::exit(1);
    }

    Ok(())
}

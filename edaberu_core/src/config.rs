use std::{env::VarError, fmt};

use serde::Deserialize;

/// 設定ファイルの名前。作業ディレクトリに置く（docs/boundaries.md 1節）。
pub const FILE_NAME: &str = "config.toml";

/// `max_chars` を書かなかったときの、本文を切る文字数（external-design 3節）。
pub const DEFAULT_MAX_CHARS: usize = 30;

/// `voicevox_url` を書かなかったときの、VOICEVOXのエンジンの場所（external-design 3節）。
pub const DEFAULT_VOICEVOX_URL: &str = "http://127.0.0.1:50021";

/// 本文の前の名前の読み方（external-design 3節の `name_mode`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NameMode {
    /// 毎回読む（`always`）。
    #[default]
    Always,
    /// 同じ人が続いたら省略する（`omit_consecutive`）。
    OmitConsecutive,
    /// 読まない（`never`）。
    Never,
}

/// 設定（external-design 3節）。トークンは入れない（ログに出さないため。[`check_token`] で別に受け取る）。
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    /// 使うサーバーのID。
    pub guild_id: u64,
    /// 読むテキストチャンネルのID。
    pub text_channel_id: u64,
    /// 対象のVCのID。
    pub voice_channel_id: u64,
    /// 本文の前の名前の読み方。
    pub name_mode: NameMode,
    /// 本文を切る文字数（1以上。B-42）。
    pub max_chars: usize,
    /// VOICEVOXの話者のID。
    pub speaker_id: u32,
    /// VOICEVOXのエンジンの場所。
    pub voicevox_url: String,
}

/// 設定ファイルに書かれたままの形。書かれなかった項目は `None` で、必須の誤り（B-41）と既定値は [`parse`] で決める。
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    guild_id: Option<u64>,
    text_channel_id: Option<u64>,
    voice_channel_id: Option<u64>,
    name_mode: Option<NameMode>,
    max_chars: Option<usize>,
    speaker_id: Option<u32>,
    voicevox_url: Option<String>,
}

/// 設定やトークンを使えない理由。表示（`Display`）が、そのまま終了の理由の文になる。
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// 設定ファイルが無い（B-38）。
    FileMissing,
    /// 書き方が壊れている・値の形が違う（B-39）、または知らない項目がある（B-40）。
    Invalid {
        /// 悪い所の行（1から数える）。
        line: Option<usize>,
        /// toml が返した説明。知らない項目なら、その名前を含む。
        detail: String,
    },
    /// 必須の項目が無い（B-41）。中身は項目の名前。
    MissingField(&'static str),
    /// `max_chars` が1より小さい（B-42）。
    MaxCharsTooSmall,
    /// 環境変数 `DISCORD_TOKEN` が無い（B-44）。
    TokenMissing,
    /// 環境変数 `DISCORD_TOKEN` が UTF-8 でない。値は持たない（誤りの文に出さないため）。
    TokenNotUnicode,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileMissing => write!(
                f,
                "設定ファイル {FILE_NAME} が無い（B-38）。見本をコピーして値を書き換える：cp config.example.toml {FILE_NAME}"
            ),
            Self::Invalid {
                line: Some(line),
                detail,
            } => write!(
                f,
                "{FILE_NAME} の{line}行目に誤りがある（B-39・B-40）: {detail}"
            ),
            Self::Invalid { line: None, detail } => {
                write!(f, "{FILE_NAME} に誤りがある（B-39・B-40）: {detail}")
            }
            Self::MissingField(name) => {
                write!(f, "{FILE_NAME} に必須の項目 {name} が無い（B-41）")
            }
            Self::MaxCharsTooSmall => {
                write!(f, "{FILE_NAME} の max_chars が1より小さい（B-42）")
            }
            Self::TokenMissing => f.write_str("環境変数 DISCORD_TOKEN が無い（B-44）"),
            Self::TokenNotUnicode => f.write_str("環境変数 DISCORD_TOKEN が UTF-8 でない"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// 設定ファイルの中身を確かめて、[`Config`] にする。
///
/// `text` は設定ファイルの中身で、ファイルが無いときは `None`。ファイルを読むのは呼び出し側で、
/// ここでは外の物に触らない（ファイルも作らない。B-38）。
///
/// # Errors
///
/// ファイルが無い（B-38）、書き方や値の形が違う（B-39）、知らない項目がある（B-40）、
/// 必須の項目が無い（B-41）、`max_chars` が1より小さい（B-42）とき、その理由を返す。
pub fn parse(text: Option<&str>) -> Result<Config, ConfigError> {
    let text = text.ok_or(ConfigError::FileMissing)?;
    let file: FileConfig = toml::from_str(text).map_err(|error| invalid(text, &error))?;
    let config = Config {
        guild_id: file.guild_id.ok_or(ConfigError::MissingField("guild_id"))?,
        text_channel_id: file
            .text_channel_id
            .ok_or(ConfigError::MissingField("text_channel_id"))?,
        voice_channel_id: file
            .voice_channel_id
            .ok_or(ConfigError::MissingField("voice_channel_id"))?,
        name_mode: file.name_mode.unwrap_or_default(),
        max_chars: file.max_chars.unwrap_or(DEFAULT_MAX_CHARS),
        speaker_id: file
            .speaker_id
            .ok_or(ConfigError::MissingField("speaker_id"))?,
        voicevox_url: file
            .voicevox_url
            .unwrap_or_else(|| DEFAULT_VOICEVOX_URL.to_owned()),
    };
    if config.max_chars < 1 {
        return Err(ConfigError::MaxCharsTooSmall);
    }
    Ok(config)
}

/// toml の誤りを、悪い所の行と説明にする（B-39：どこが悪いかを表示する）。
fn invalid(text: &str, error: &toml::de::Error) -> ConfigError {
    let line = error
        .span()
        .and_then(|span| text.get(..span.start))
        .map(|before| before.matches('\n').count() + 1);
    ConfigError::Invalid {
        line,
        detail: error.message().to_owned(),
    }
}

/// `DISCORD_TOKEN` の読み取りの結果を確かめる。誤りの文には、トークンの値を含めない
/// （`VarError::NotUnicode` の Display は値そのものを含むため、つながずに置き換える）。
///
/// # Errors
///
/// 環境変数が無い（B-44）、または UTF-8 でないとき、その理由を返す。
pub fn check_token(read: Result<String, VarError>) -> Result<String, ConfigError> {
    read.map_err(|error| match error {
        VarError::NotPresent => ConfigError::TokenMissing,
        VarError::NotUnicode(_) => ConfigError::TokenNotUnicode,
    })
}

#[cfg(test)]
mod tests {
    use std::{env::VarError, error::Error as _, ffi::OsString};

    use super::{Config, ConfigError, NameMode, check_token, parse};
    use crate::TestResult;

    /// リポジトリの直下にある見本の設定ファイル。
    const EXAMPLE: &str = include_str!("../../config.example.toml");

    /// 必須の項目だけを書いた設定。1行に1項目（4行）。
    const REQUIRED_ONLY: &str =
        "guild_id = 11\ntext_channel_id = 12\nvoice_channel_id = 13\nspeaker_id = 14\n";

    const SECRET: &str = "bogus-token-12345";

    /// 末尾に UTF-8 として不正な1単位を付けた値（`VarError::NotUnicode` の中身の見本）。
    #[cfg(windows)]
    fn not_unicode(prefix: &str) -> OsString {
        use std::os::windows::ffi::OsStringExt as _;
        let mut units: Vec<u16> = prefix.encode_utf16().collect();
        units.push(0xD800);
        OsString::from_wide(&units)
    }

    /// 末尾に UTF-8 として不正な1単位を付けた値（`VarError::NotUnicode` の中身の見本）。
    #[cfg(unix)]
    fn not_unicode(prefix: &str) -> OsString {
        use std::os::unix::ffi::OsStringExt as _;
        let mut bytes = prefix.as_bytes().to_vec();
        bytes.push(0xFF);
        OsString::from_vec(bytes)
    }

    #[test]
    fn item3_example_file_loads_as_is() -> TestResult {
        let config = parse(Some(EXAMPLE))?;

        assert_eq!(
            config,
            Config {
                guild_id: 100000000000000001,
                text_channel_id: 100000000000000002,
                voice_channel_id: 100000000000000003,
                name_mode: NameMode::Always,
                max_chars: 30,
                speaker_id: 3,
                voicevox_url: "http://127.0.0.1:50021".to_owned(),
            }
        );
        Ok(())
    }

    #[test]
    fn item3_file_missing_returns_copy_steps() -> TestResult {
        assert_eq!(parse(None), Err(ConfigError::FileMissing));
        assert_eq!(
            ConfigError::FileMissing.to_string(),
            "設定ファイル config.toml が無い（B-38）。見本をコピーして値を書き換える：cp config.example.toml config.toml"
        );
        Ok(())
    }

    #[test]
    fn item3_broken_syntax_returns_line() -> TestResult {
        // 3行目の文字列に引用符が無い
        let text = "guild_id = 11\ntext_channel_id = 12\nname_mode = always\nvoice_channel_id = 13\nspeaker_id = 14\n";

        let error = parse(Some(text))
            .err()
            .ok_or("壊れた書き方なのに Ok が返った")?;

        assert!(
            matches!(error, ConfigError::Invalid { line: Some(3), .. }),
            "{error:?}"
        );
        assert!(
            error
                .to_string()
                .starts_with("config.toml の3行目に誤りがある（B-39・B-40）: "),
            "{error}"
        );
        Ok(())
    }

    #[test]
    fn item3_wrong_value_type_returns_line() -> TestResult {
        // 2行目の max_chars が数でなく文字列
        let text = "guild_id = 11\nmax_chars = \"30\"\ntext_channel_id = 12\nvoice_channel_id = 13\nspeaker_id = 14\n";

        let result = parse(Some(text));

        assert!(
            matches!(result, Err(ConfigError::Invalid { line: Some(2), .. })),
            "{result:?}"
        );
        Ok(())
    }

    #[test]
    fn item3_unknown_field_is_named() -> TestResult {
        // 5行目に、botが知らない項目
        let text = format!("{REQUIRED_ONLY}volume = 5\n");

        let error = parse(Some(&text))
            .err()
            .ok_or("知らない項目があるのに Ok が返った")?;

        let ConfigError::Invalid { line, detail } = &error else {
            return Err(format!("種類が違う: {error:?}").into());
        };
        assert_eq!(*line, Some(5));
        assert!(
            detail.contains("volume"),
            "項目の名前が出ていない: {detail}"
        );
        Ok(())
    }

    #[test]
    fn item3_missing_required_field_is_named() -> TestResult {
        for name in [
            "guild_id",
            "text_channel_id",
            "voice_channel_id",
            "speaker_id",
        ] {
            let text: String = REQUIRED_ONLY
                .lines()
                .filter(|line| !line.starts_with(name))
                .map(|line| format!("{line}\n"))
                .collect();

            assert_eq!(
                parse(Some(&text)),
                Err(ConfigError::MissingField(name)),
                "{name} を消した"
            );
        }

        // 空のファイルは、最初の必須の項目が無いと返る
        assert_eq!(parse(Some("")), Err(ConfigError::MissingField("guild_id")));

        assert_eq!(
            ConfigError::MissingField("speaker_id").to_string(),
            "config.toml に必須の項目 speaker_id が無い（B-41）"
        );
        Ok(())
    }

    #[test]
    fn item3_max_chars_zero_is_rejected() -> TestResult {
        let zero = format!("{REQUIRED_ONLY}max_chars = 0\n");
        assert_eq!(parse(Some(&zero)), Err(ConfigError::MaxCharsTooSmall));
        assert_eq!(
            ConfigError::MaxCharsTooSmall.to_string(),
            "config.toml の max_chars が1より小さい（B-42）"
        );

        // ちょうど1は通る
        let one = format!("{REQUIRED_ONLY}max_chars = 1\n");
        assert_eq!(parse(Some(&one))?.max_chars, 1);

        // 負の数は、値の形の誤りとして5行目が返る
        let negative = format!("{REQUIRED_ONLY}max_chars = -1\n");
        let result = parse(Some(&negative));
        assert!(
            matches!(result, Err(ConfigError::Invalid { line: Some(5), .. })),
            "{result:?}"
        );
        Ok(())
    }

    #[test]
    fn omitted_items_use_defaults() -> TestResult {
        let config = parse(Some(REQUIRED_ONLY))?;

        assert_eq!(
            config,
            Config {
                guild_id: 11,
                text_channel_id: 12,
                voice_channel_id: 13,
                name_mode: NameMode::Always,
                max_chars: 30,
                speaker_id: 14,
                voicevox_url: "http://127.0.0.1:50021".to_owned(),
            }
        );
        Ok(())
    }

    #[test]
    fn name_mode_accepts_three_values() -> TestResult {
        for (value, expected) in [
            ("always", NameMode::Always),
            ("omit_consecutive", NameMode::OmitConsecutive),
            ("never", NameMode::Never),
        ] {
            let text = format!("{REQUIRED_ONLY}name_mode = \"{value}\"\n");
            assert_eq!(parse(Some(&text))?.name_mode, expected, "{value}");
        }

        let text = format!("{REQUIRED_ONLY}name_mode = \"sometimes\"\n");
        let result = parse(Some(&text));
        assert!(
            matches!(result, Err(ConfigError::Invalid { line: Some(5), .. })),
            "{result:?}"
        );
        Ok(())
    }

    #[test]
    fn token_not_unicode_message_does_not_contain_value() -> TestResult {
        let value = not_unicode(SECRET);
        assert!(
            value.clone().into_string().is_err(),
            "見本の値が UTF-8 として正しくなっている"
        );

        let error = check_token(Err(VarError::NotUnicode(value)))
            .err()
            .ok_or("NotUnicode なのに Ok が返った")?;
        let text = error.to_string();

        // main は anyhow の {:#} で原因の連なりまで1行に出すので、連なりが無いことも確かめる
        assert!(error.source().is_none(), "原因の連なりが付いている");
        assert!(
            !text.contains(SECRET),
            "トークンの値が誤りの文に出た: {text}"
        );
        assert_eq!(text, "環境変数 DISCORD_TOKEN が UTF-8 でない");
        Ok(())
    }

    #[test]
    fn token_not_present_message_is_b44() -> TestResult {
        let error = check_token(Err(VarError::NotPresent))
            .err()
            .ok_or("NotPresent なのに Ok が返った")?;

        assert_eq!(error.to_string(), "環境変数 DISCORD_TOKEN が無い（B-44）");
        Ok(())
    }
}

use crate::emoji;

/// 読む名前を決める（R-36・R-41・R-42）。
///
/// ニックネーム → 表示名 → ユーザー名の順に見て、絵文字を外し（[`emoji::remove`]）、前後の空白を除いたあとに
/// 文字が残る最初の物を使う。「さん」は付けず、長くても切らない。
/// どれにも文字が残らなければ `None` を返す（そのときの読み方は、外部設計に無い）。
#[must_use]
pub fn choose(
    nickname: Option<&str>,
    display_name: Option<&str>,
    username: Option<&str>,
) -> Option<String> {
    [nickname, display_name, username]
        .into_iter()
        .flatten()
        .find_map(|name| {
            let name = emoji::remove(name);
            let name = name.trim();
            (!name.is_empty()).then(|| name.to_owned())
        })
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::choose;

    type TestResult = Result<(), Box<dyn Error>>;

    #[test]
    fn item4_r36_nickname_then_display_name_then_username() -> TestResult {
        assert_eq!(
            choose(Some("えだ"), Some("eda"), Some("eda3")),
            Some("えだ".to_owned())
        );
        assert_eq!(
            choose(None, Some("eda"), Some("eda3")),
            Some("eda".to_owned())
        );
        assert_eq!(choose(None, None, Some("eda3")), Some("eda3".to_owned()));
        Ok(())
    }

    #[test]
    fn item4_r36_long_name_is_not_cut() -> TestResult {
        // 本文を切る文字数（既定30）より長い40文字
        let long =
            "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらり";
        assert_eq!(long.chars().count(), 40);

        assert_eq!(
            choose(Some(long), None, Some("eda3")),
            Some(long.to_owned())
        );
        Ok(())
    }

    #[test]
    fn item4_r41_emoji_in_name_are_removed() -> TestResult {
        assert_eq!(
            choose(Some("🍣えだ🍣"), None, Some("eda3")),
            Some("えだ".to_owned())
        );
        Ok(())
    }

    #[test]
    fn item4_r42_name_of_only_emoji_falls_back() -> TestResult {
        assert_eq!(
            choose(Some("🍣🍣"), None, Some("eda3")),
            Some("eda3".to_owned())
        );
        Ok(())
    }

    #[test]
    fn empty_or_blank_name_falls_back() -> TestResult {
        // 空の文字列は「無い」と同じ
        assert_eq!(
            choose(Some(""), Some("eda"), Some("eda3")),
            Some("eda".to_owned())
        );
        // 絵文字を外すと空白だけになる名前も、次の名前へ
        assert_eq!(
            choose(Some("🍣 🍣"), Some("eda"), Some("eda3")),
            Some("eda".to_owned())
        );
        Ok(())
    }

    #[test]
    fn no_name_left_returns_none() -> TestResult {
        assert_eq!(choose(None, None, None), None);
        assert_eq!(choose(Some("🍣"), Some("🎉"), Some("😀")), None);
        Ok(())
    }
}

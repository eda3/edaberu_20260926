use unicode_segmentation::UnicodeSegmentation as _;

/// 文字列から、Unicode の絵文字の一覧に載る物を取り除く（R-15・R-16・R-41）。
///
/// 画面で1文字に見える単位（書記素）ごとに、絵文字の一覧（`emojis` が持つ Unicode 17.0 の一覧）に
/// 載るかを見る。肌の色・つなぎ文字（ZWJ）の組み合わせ・国旗・キーキャップも、1つの絵文字として丸ごと消える。
/// ♪☆★① のように一覧に無い記号と、数字は残す。
#[must_use]
pub fn remove(text: &str) -> String {
    text.graphemes(true)
        .filter(|grapheme| emojis::get(grapheme).is_none())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::remove;
    use crate::TestResult;

    #[test]
    fn item4_r15_emoji_in_list_are_removed() -> TestResult {
        assert_eq!(remove("やった😀🎉"), "やった");
        Ok(())
    }

    #[test]
    fn item4_r16_symbols_not_in_list_are_kept() -> TestResult {
        assert_eq!(remove("♪☆★①"), "♪☆★①");
        Ok(())
    }

    #[test]
    fn item4_sequences_are_removed_as_one_grapheme() -> TestResult {
        // 家族（ZWJ でつないだ3人）
        assert_eq!(
            remove("a\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}b"),
            "ab"
        );
        // 肌の色つきの手
        assert_eq!(remove("a\u{1F44D}\u{1F3FD}b"), "ab");
        // 国旗（地域指示記号2つ）
        assert_eq!(remove("a\u{1F1EF}\u{1F1F5}b"), "ab");
        // キーキャップの3
        assert_eq!(remove("a3\u{FE0F}\u{20E3}b"), "ab");
        // ハート（VS16 あり・なし）
        assert_eq!(remove("a\u{2764}\u{FE0F}b\u{2764}c"), "abc");
        Ok(())
    }

    #[test]
    fn item4_digits_and_signs_are_kept() -> TestResult {
        // 数字・#・* は、キーキャップの部品でも、単独では絵文字の一覧に無い
        assert_eq!(remove("eda3 #1 *"), "eda3 #1 *");
        Ok(())
    }

    #[test]
    fn empty_text_stays_empty() -> TestResult {
        assert_eq!(remove(""), "");
        Ok(())
    }
}

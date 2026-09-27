//! edaberu の純粋な部品（docs/boundaries.md 1節）。twilight・songbird には依存しない。

/// 設定ファイルと環境変数を読み、確かめる。
pub mod config;
/// 届いた発言を読むかどうかを決める。
pub mod intake;
/// 読む名前を決める。
pub mod names;
/// サーバーごとの読み上げの列。
pub mod queue;
/// 読み上げる文を作る。
pub mod speech;
/// サーバーごとの状態。
pub mod state;
/// 声を作る（trait `Tts` と VOICEVOX の実装）。
pub mod tts;
/// VCの出入りへの対応を決める。
pub mod voice_rules;

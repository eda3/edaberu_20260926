# edaberu_20260926

## Project
- Discordのチャット読み上げbot（1つのサーバー）。テキストの発言とVCへの人の出入りを、VOICEVOXの声で読み上げる。名前は「えだ」＋「駄弁る」
- 言語: Rust。ワークスペースにクレート2つ: `edaberu_core`（純粋な部品と、`Tts`・`Player` の trait、`Voicevox` 実装）／`edaberu`（twilight・songbird・main）
- 動かす場所: VPS（Ubuntu 22.04・メモリ1.9GiB）。VOICEVOXは同じVPSのDocker。手元の開発はWindows

## 文書の役割（振る舞いはコードでなく文書が正）
- `docs/external-design.md`: 振る舞い（B-nn）・読み上げる文（R-nn）・受け入れテスト（AT-nn）・参考の不具合（P-nn）。振る舞いはここにだけ書く
- `docs/boundaries.md`: 部品の境目・ライブラリの版・未確認の項目（4節）
- `docs/test-items.md`: テストで確かめる中身（①〜⑱）。変えるのは、えだが決めたときだけ
- `docs/todo.md`: 進める順番。`/next-todo` は上から順に1項目だけ進める
- `docs/measurements.md`: VPSやテスト用サーバーで測った数字。推測は書かない

## 設計の約束
- `edaberu_core` は twilight・songbird に依存しない。部品の入力は、twilight の型から写した値（ID は `u64`）にする。この線を越えるとビルドが落ちる形を保つ
- 判断する部品（`intake`・`voice_rules`・`speech`・`names`・`config`）は、外の物に触らない純粋な関数や状態にする
- `queue` は `Tts`・`Player` を trait で受け取る。テストでは偽物を渡す
- 数値（VCにつながるまで10秒・列の上限20件・VOICEVOXの打ち切り10秒・切る文字数30）は名前の付いた定数にし、テストからは短い値を注入できる形にする。数値の根拠は「未記録（経験則）」（external-design 3節）
- 自動参加の停止は記憶の中にだけ持つ。ファイルに残さない（再起動で消えるのが仕様）
- トークンは環境変数 `DISCORD_TOKEN` からだけ読む。ログ・エラーの文・パニックの文に出さない
- `!join`・`!bye` などのコマンドは作らない（読み上げを止めるのは、botをVCから切断する操作）

## Code Style
- 命名: モジュール=snake_case / 構造体=PascalCase / 定数=UPPER_SNAKE_CASE
- エラーは `?` で呼び出し元に返す。`edaberu` は `anyhow::Result`、`edaberu_core` は自前のエラー型（呼び出し側がエラーの種類で分岐できるように）
- テスト関数も `Result` を返す形にして、`?` を使う
- 公開APIには `///` ドキュメントコメントを付ける
- `///` に使用例を書く場合は ```rust ブロックで書く。例の中で `?` を使うときは、最後に隠し行 `# Ok::<(), E>(())`（E はその例のエラー型）を置く

## Workflow
- 純粋な部品の変更後は `cargo test -p edaberu_core` を実行し、doc-tests（出力の「Doc-tests」の欄）も含めて全部合格させる
- `cargo test --workspace` は songbird（Opus）をビルドできる環境でだけ回す（cmake・libopus が要る）
- `cargo clippy --workspace -- -D warnings -D clippy::pedantic -D clippy::nursery` の warning は修正してからコミット（songbird がビルドできない環境では `-p edaberu_core` で回す）
- `cargo fmt` でフォーマットを統一する
- ファイルの書き換えは Edit の道具で行う。Bash や Python で書き換えない（Bash 経由の書き換えは、チェックポイントと変更の追跡に乗らないため）
- 1項目＝1コミット。コミットの文は「何をしたか」を1行で。ブランチへの push と PR の作成（`gh pr create`）は Claude Code が行ってよい。main への合流（`gh pr merge`）とブランチの削除はえだが行う

## テスト
- テストはこのプロジェクトの検証役。既存のテストは中身を保ち、通らないときは実装の側を直す
- 期待値は `docs/external-design.md` の表の文字列を手で写す。実装の関数で期待値を作らない
- 境界（ちょうど・＋1・空・書記素・0件・上限）を必ず入れる
- 外の物（Discord・VOICEVOX・時計）は偽物か注入で置き換える。テストで実際に10秒待たない
- 点検（test-reviewer）の指摘は、実装を1か所わざと壊してテストが落ちることで確かめる

## 止まって聞く場面
次のときだけ、手を動かす前に止まって、理由を添えて確認を求める。それ以外は、テストが通るかどうかで判断して進める。
- 新しい依存クレートを足すとき（`docs/todo.md` の項目に「足してよい依存」として書いてある物は除く）
- 既存のテストを変える（消す・条件を緩める）とき、`docs/test-items.md` を変えたくなったとき
- `docs/external-design.md` に書かれていない振る舞いを決めないと進めないとき（質問一覧だけ返して止まる）
- 文書どうし、または文書とコードが食い違っているのを見つけたとき（直さずに報告する）
- `docs/todo.md` の範囲の外の機能を足すとき

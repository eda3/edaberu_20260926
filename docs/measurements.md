# 実測の記録

VPSやテスト用のサーバーで測った数字と、見えたことを書く場所です。数字は測った日と一緒に書きます。推測は書きません（書く場合は［推測］と付けます）。

## ① VPSでのビルド（`docs/todo.md` 1周目・4節の1番）

- 手元（Windows 11・rustc 1.98.0・MSVC・scoop の cmake）での確認（2026-09-27）：`cargo build -p edaberu_core`・`cargo check --workspace`・`cargo test --workspace`（songbird と opus2 を含む）が通った。VPSでの数字は下に書く

- 測った日：
- 入れた物（apt・rustup の版）：
- `cargo build --release -j 1` の結果：通った／通らなかった
- Elapsed (wall clock) time：
- Maximum resident set size（KB）：
- ビルド中のスワップの最大（`free -m` の used）：
- 通らなかったときのエラー（先頭20行）：

## ② VPSでのVOICEVOX（`docs/todo.md` 1周目・4節の2番）

- 測った日：
- イメージ：`voicevox/voicevox_engine:cpu-latest`（pull した日のダイジェストが分かれば）
- 使う話者と `id`（`/speakers` の結果から）：
- 30文字の合成 10回：最大　秒／中央値　秒
- `docker stats --no-stream` の MEM USAGE：
- 合成の直後の `free -m`（used／available）：
- 判定：VPSで使う（Q-115 a のまま）／自宅PCへ切り替える（Q-115 b）

## ③ VCへの接続（2周目・4節の3番）

- 試した日：
- つながった／つながらなかった：
- songbird のログ（つながらなかったときの先頭30行）：
- スピーカーミュートで入れたか（4節の16番）：
- メモリ上のwavを渡せたか（4節の4番）：
- 切断・移動・不調の区別（4節の5番）：それぞれ、どの知らせで分かったか
- 起動時の人／botの判定（4節の20番）：GUILD_CREATE で揃った／揃わなかった。揃わなかったときに選んだ手：

## ④ 「VCで聞く」の受け入れテスト（4周目）

| 番号 | 日 | 結果 | 見えたこと |
|---|---|---|---|
| | | | |

## ⑤ VPSでの配置（5周目）

- 置いた日：
- `systemctl stop` → VCから出て終わったか（AT-25）：
- `kill -9` → systemd が起動し直したか（AT-38）：
- `reboot` → 手を使わずにVCに入ったか（AT-38）：
- 1日動かしたあとの `journalctl` で気になった行：

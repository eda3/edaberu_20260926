# 実測の記録

VPSやテスト用のサーバーで測った数字と、見えたことを書く場所です。数字は測った日と一緒に書きます。推測は書きません（書く場合は［推測］と付けます）。

## ① VPSでのビルド（`docs/todo.md` 1周目・4節の1番）

- 手元（Windows 11・rustc 1.98.0・MSVC・scoop の cmake）での確認（2026-09-27）：`cargo build -p edaberu_core`・`cargo check --workspace`・`cargo test --workspace`（songbird と opus2 を含む）が通った。VPSでの数字は下に書く

- 測った日：2026-09-27
- 入れた物（apt・rustup の版）：apt で build-essential・cmake・pkg-config・libopus-dev・time、rustup（rustc の版は未記録）
- `cargo build --release -j 1` の結果：通った（2回目）
  - 1回目：7:55.94・最大RSS 799,048KB・libopus_sys で失敗（cmake 未導入）→ `apt install cmake` で解消
  - 2回目：6:07.13・最大RSS 918,176KB・exit 0
  - 合計：約14分
- Elapsed (wall clock) time：6:07.13（2回目）。1回目と合わせて約14分
- Maximum resident set size（KB）：918,176（2回の大きい方。約0.9GB）
- ビルド中のスワップの最大（`free -m` の used）：未計測（GNU time の Swaps 欄は Linux では常に0なので根拠にしない）
- 通らなかったときのエラー（先頭20行）：1回目は libopus_sys のビルドで失敗（cmake 未導入）。先頭20行は未記録

## ② VPSでのVOICEVOX（`docs/todo.md` 1周目・4節の2番）

- 測った日：2026-09-27
- イメージ：`voicevox/voicevox_engine:cpu-latest`（VOICEVOX ENGINE 0.25.2・supported_devices `{"cpu":true,"cuda":false,"dml":false}`）
- 使う話者と `id`（`/speakers` の結果から）：ずんだもん ノーマル・id 3
- 起動直後の `free -m`：used 517MB・available 1240MB・swap 307MB
- 30文字の合成 10回：8.26・6.05・5.64・5.68・5.68・5.60・5.65・5.32・5.65・5.41 秒。最大 8.26 秒（1回目）／中央値 5.65 秒
- 5文字の合成 3回：1.71・1.59・1.58 秒
- 14文字の合成 3回：2.99・3.17・3.13 秒
- ［推測］合成の時間 ≈ 0.8秒＋0.16秒×文字数（上の3つの長さからの見積もり）
- `docker stats --no-stream` の MEM USAGE：382.9MiB / 1.912GiB（19.55%）
- 合成の直後の `free -m`（used／available）：used 778MB・available 979MB・swap 309MB
- nproc：未計測
- 判定：
  - メモリ：同居できる（Q-115 a のまま）
  - 速さ：短い発言で1〜2秒、30文字で約5.7秒（CPU版）。まず a のまま進め、4周目でVCで聞いてから、`max_chars` を下げる／Q-115 b に切り替える／次の区切りを再生中に先に作る、のどれかを決める（えだの判断・今は未）

## ③ VCへの接続（2周目・4節の3番）

- 試した日：2026-09-27（Windows）
- つながった／つながらなかった：つながった。songbird 0.6・DAVE・暗号 Aes256Gcm・つなぎ先 c-nrt08
- beep.wav の再生：0.34秒で最後まで再生（Playable→End）
- songbird のログ（つながらなかったときの先頭30行）：（つながったため無し）
- スピーカーミュートで入れたか（4節の16番）：入れた（VCの一覧のヘッドホンの斜線・えだの目視）
- メモリ上のwavを渡せたか（4節の4番）：
- 切断・移動・不調の区別（4節の5番）：それぞれ、どの知らせで分かったか
- 起動時の人／botの判定（4節の20番）：GUILD_CREATE で揃った。動作中の VOICE_STATE_UPDATE にも member（bot判定）が付いた（has_member=true）

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

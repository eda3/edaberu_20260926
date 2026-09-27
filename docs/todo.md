# ToDo

上から順に、`/next-todo` で進める。1項目＝1ループ＋1コミット。【えだ】の付いた項目は、えだがチェックを入れる。【VPS】の付いた項目は、えだがVPSの上で手を動かす（打つコマンドは項目に書いてある。結果は `docs/measurements.md` に書く）。

- 振る舞いの正は `docs/external-design.md`（B-nn・R-nn・AT-nn・P-nn）。部品の境目は `docs/boundaries.md`。テストで確かめる中身は `docs/test-items.md`（①〜⑱）。この3つと食い違うことは書かない。食い違いを見つけたら、直さずに止まって聞く。
- 並べ方は「危ない所から」（決定表 12）。1周目と2周目に、`docs/boundaries.md` 4節の1〜5番と20番が入っている。
- 3周目（edaberu_core）は、1周目・2周目の結果を待たずに始めてよい。手元のWindowsで songbird（Opus）がビルドできないときは、2周目を飛ばして3周目を先に進め、2周目はビルドできる場所（VPS、またはクラウドセッション）で回す。どちらにするかは、えだが決める。
- 決まっていない振る舞いに当たったら、推測で埋めずに、質問一覧だけ返して止まる（外部設計を書いたときと同じ）。
- 数値（10秒・20件・30文字）は名前の付いた定数にし、テストからは短い値を注入できる形にする（テストで10秒待たない）。

## 1周目: 文書の直しと、骨組みと、危ない所の実測（VPS）

ゴール: VPSで「ビルドが通るか」「VOICEVOXが動くか」が数字で分かる。動かなければ、代替（GitHub Actions／Tailscale経由の自宅PC）に切り替える材料がそろう。

- [x] 文書の直し3点（コードはまだ書かない）
  - メモ: ① `docs/external-design.md` の AT-37 と AT-38 の並びを番号順に直す。② `docs/boundaries.md` 4節に「20. 起動した時に対象のVCにいるのが人かbotかを、GUILD_MEMBERS の特権intentなしで知れるか［C］。GUILD_CREATE の voice_states には user_id だけが入る見込み［B］。知れなければ、RESTで1人ずつ取るか、Developer Portal で GUILD_MEMBERS をオンにする。確かめ方：AT-07 の前提に『他のbotも対象のVCにいる』を足す。工程：2周目」を足す。③ `README.md` の「2. botを置く」の前に「0. VPSに Docker と Rust（rustup）を入れる（入っているかは未確認）」を足す。
  - 完了条件: 3ファイルの該当行だけが変わっている（`git diff --stat` で3ファイル）
- [x] workspace の骨組みを作る（`edaberu_core` と `edaberu`。中身は空に近い。songbird まで依存に入れる）
  - メモ: ルートの `Cargo.toml` は `[workspace] members = ["edaberu_core", "edaberu"]`、`resolver = "3"`。edition は 2024。`edaberu_core` は lib（モジュールは空で、`pub mod` の宣言だけでよい）。`edaberu` は bin（`main.rs` は「起動して終了」だけ）。依存は `docs/boundaries.md` 3節の表どおり。songbird は `default-features = false`、機能 `driver`・`gateway`・`twilight`・`rustls`・`tws`。symphonia の機能は `wav`・`pcm`（機能名は確度B。通らなければ crates.io で正しい名前を確かめて3節に書き戻す）。`.gitignore`（`target/`・`config.toml`・`*.env`）と `config.example.toml`（3節の設定項目を全部、値は見本）も作る。LICENSE は未定なので作らない。
  - 足してよい依存: `docs/boundaries.md` 3節の表にある物すべて。版は crates.io の最新の安定版を選び、選んだ版を3節の「版」の欄に書き戻す。絵文字を判定する crate はまだ入れない。
  - 完了条件: `cargo build -p edaberu_core` が手元で通る。`cargo check --workspace` も通れば、その旨をメモに書く。手元（Windows）で songbird（Opus）のビルドが通らないときは、エラーの先頭20行を `docs/measurements.md` に貼って、この項目は完了にする（`edaberu` 側のビルドは次の【VPS】①で確かめる）
- [x] 【VPS】① ビルドの実測（`docs/test-items.md` ①・4節の1番）
  - メモ: VOICEVOXのコンテナは起動しない（まだ無い）。順に打つ。
    1. `sudo apt install -y build-essential cmake pkg-config libopus-dev time`
    2. `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` → `source ~/.cargo/env`
    3. `git clone https://github.com/eda3/edaberu_20260926 ~/edaberu && cd ~/edaberu`
    4. `/usr/bin/time -v cargo build --release -j 1 2>&1 | tail -n 30`
    5. 記録する数字：`Elapsed (wall clock) time`、`Maximum resident set size`（KB）、ビルド中に別の端末で `free -m` を何度か見たときのスワップの最大。`docs/measurements.md` の①に書く
  - 通らなかったとき: エラーの先頭20行を①に貼る。GitHub Actions（runner は `ubuntu-22.04`、または `ubuntu:22.04` のコンテナの中）でビルドしてバイナリを置く形に切り替えるかは、えだが決める。決めたら、この下に「[ ] GitHub Actions でビルドする workflow を作る」を足す
- [x] 【VPS】② VOICEVOXの実測（`docs/test-items.md` ②・4節の2番）
  - メモ: ①が終わってから。順に打つ。
    1. `docker run -d --name voicevox --restart unless-stopped -p 127.0.0.1:50021:50021 voicevox/voicevox_engine:cpu-latest`
    2. 話者の一覧を見る：`curl -s http://127.0.0.1:50021/speakers | python3 -c 'import sys,json; [print(s["name"], [(t["name"], t["id"]) for t in s["styles"]]) for s in json.load(sys.stdin)]'` → 使う話者（ずんだもん など）のスタイルの `id` を控える。`config.example.toml` の `speaker_id` に書く番号になる
    3. 30文字の文を10回合成して、1回あたりの秒を測る：
       ```sh
       cat > ~/tts_bench.sh <<'EOF'
       #!/bin/sh
       SPK=${1:-3}
       TEXT='おはよう。今日はとてもいい天気ですね。散歩に行きましょうか。'
       Q=$(curl -s -G -X POST 'http://127.0.0.1:50021/audio_query' --data-urlencode "speaker=$SPK" --data-urlencode "text=$TEXT")
       curl -s -X POST -H 'Content-Type: application/json' -d "$Q" "http://127.0.0.1:50021/synthesis?speaker=$SPK" -o /dev/null
       EOF
       chmod +x ~/tts_bench.sh
       for i in $(seq 10); do /usr/bin/time -f "%e 秒" ~/tts_bench.sh 3; done   # 3 は2で控えた番号に置き換える
       ```
    4. 3の直後に `docker stats --no-stream voicevox` → `MEM USAGE` を控える。`free -m` も控える
    5. `docs/measurements.md` の②に、1回あたりの秒（10回の最大と中央値）、コンテナのメモリ、`free -m` の残りを書く
  - 動かなかったとき（起動しない・メモリ不足で落ちる・1回に10秒以上かかる）: Q-115 の b（自宅のWindows PCのVOICEVOXをTailscale経由で呼ぶ）に切り替える。設計の直しは、えだがClaude Codeに指示する
- [x] 【えだ】差分を読んで push する

## 2周目: edaberu（bot）の危ない所 — VCで聞く

ゴール: テスト用のサーバーで、botが対象のVCに入って固定の音が鳴る（AT-01 の形）。切断・移動・不調を区別できるかが分かる。

- [x] 【えだ】テスト用のDiscordサーバーとbotのアプリを用意する
  - メモ: Developer Portal で、Message Content Intent をオン、Public Bot をオフ。サーバーに、テスト用のVC 2つ（対象のVCと、それ以外のVC「X」）とテキストチャンネル1つ。botを招く（権限：チャンネルを見る・メッセージを読む・接続・発言）。控えるもの：トークン（`DISCORD_TOKEN`。手元では環境変数、VPSでは `/etc/edaberu/edaberu.env`）、サーバーID、テキストチャンネルID、対象のVCのID。テストで使う音は、短いwavを1つ用意して `edaberu/tests/fixtures/beep.wav` に置く（VOICEVOXで作った物でも、フリー素材でもよい。ライセンスを1行 README に書く）
- [x] `gateway` の骨組み：twilightでつなぎ、出来事を受けて `state` を更新する（B-43〜B-46）Issue #1・Issue #3
  - メモ: intent は GUILDS・GUILD_MESSAGES・GUILD_VOICE_STATES・MESSAGE_CONTENT。GUILD_CREATE・VOICE_STATE_UPDATE・MESSAGE_CREATE を受け、ログ（tracing）に出す。起動時の確認：トークンが無い（B-44）、間違っている／intentが許可されていない＝閉じるコード4014（B-45。確度B）、IDが見つからない（B-43）。`state` はまだ `edaberu_core` に無いので、ここでは最小の構造体を `edaberu` 側に置き、3周目で `edaberu_core::state` に移す（移すことをコメントに書く）。**起動時に対象のVCにいる人が人かbotか**（4節の20番）を、GUILD_CREATE の中身をログに出して確かめ、結果（揃う／揃わない）を4節の20番に書き戻す。揃わなければ止まって聞く（RESTで取るか、GUILD_MEMBERS を足すかは、えだが決める）
  - 完了条件: 手元でbotを起動すると、Discordにつながり、VCの出入りとテキストの発言がログに出る
- [x] `player`（songbird）と、対象のVCへの参加：`docs/test-items.md` ⑬（AT-01 の形。4節の3番・16番）Issue #2
  - メモ: 人が対象のVCに入ったら、スピーカーミュート（self deaf）で入り、`beep.wav` を1回鳴らす（まだ読み上げの文は作らない）。DAVE は songbird 0.6 の `driver` 機能に含まれる［A］。つながらないときは、songbird のログ（`RUST_LOG=songbird=debug`）の先頭30行を `docs/measurements.md` の③に貼り、止まって聞く
  - 完了条件: テスト用のアカウントで対象のVCに入ると、botが入ってきて音が鳴る。VCの一覧で、botのヘッドホンに斜線が出ている
- [x] メモリ上のwavを songbird に渡す：`docs/test-items.md` ⑭（AT-37 の形。4節の4番・P-6）
  - メモ: `beep.wav` をファイルからでなく `Vec<u8>` として渡して鳴らす。渡せたら4節の4番に「渡せる」と書く。渡せないときは、一時ファイルに書いて流し終えたら消す形に変え、P-6 と 4節の4番を書き直す（書き直しの内容は止まって聞く）
  - player と同じ実装で満たした（7b9e779）
- [ ] 切断・移動・不調を区別する：`docs/test-items.md` ⑮（AT-08・AT-09・AT-10・AT-29 の形。4節の5番）
  - メモ: 管理者のアカウントで、botを「切断」する／別のVC「X」へ移す、の2つを実際にやり、twilight の VOICE_STATE_UPDATE と songbird のドライバの知らせ（切れた・つなぎ直した）をログに出す。ネットの不調は、手元なら数十秒 Wi-Fi を切る（VPSでは AT-29 の塞ぎ方をあとで決める）。3つが区別できたら、区別の方法を4節の5番に書く。区別できないものがあれば、B-12・B-14・B-15 のどこが分かれなくなるかを書いて止まって聞く
  - 実装済み（2026-09-27）。手動テスト1〜4は4周目の⑰と一緒に行う
- [x] 点検: test-reviewer に点検させ、指摘を壊し方で確かめて直す（最大2回）
  - メモ: ここまでの `edaberu` 側は「VCで聞く」の確認が中心で、自動テストは少ない。点検役には、ログの出方・エラー時に落ちないこと（B-47）・トークンがログや例外の文に出ないことを見させる
  - 結果（2026-09-27・点検役は3点で1回・壊し方は手元で起動／読み）：
    - トークン：偽トークン（`bogus-token-12345`）で起動し、info・trace のどちらでもログと終了の文に0件（終了は「Discordに現在のユーザーを問い合わせられなかった（B-45）: response error: status code 401 …」の1行・終了コード1）。指摘1-1「DISCORD_TOKEN が UTF-8 でないと値が誤りの文に出る」を直した（`check_token`。元に戻すとテストが落ち、文に `"bogus-token-12345\u{d800}"` が出るのを確かめた）
    - B-47：エラーでbotごと終わる経路は無し。指摘2-1「join が1回失敗すると Call が残り、再起動まで入らない」→失敗したら `songbird.remove`、2-2「再生の失敗がログに出ない」→`TrackEvent::Error` を WARN に、2-3「beep.wav が無いときパスが出ない」→パスを文に入れた。3件とも Discord が要るので読みで確かめた（手動は4周目の⑰と一緒に）
    - ログ：EDABERU_GUILD_ID が無い／空／0 で、理由が1行（B-41／B-39／B-39）。指摘3-1「RUST_LOG に edaberu を含まないと終了の理由が1行も出ない」→終了の理由を target `edaberu::exit` で出し、その ERROR を常に通す（`RUST_LOG=songbird=debug`・空・`off` で0行→1行）。色の制御文字は端末のときだけ出す（ファイルへの出力で ESC の行 1→0）
- [ ] 【えだ】PR を作ってもらい、差分を読んで main へ合流する

## 3周目: edaberu_core（純粋な部品）— 1周目・2周目を待たずに始めてよい

ゴール: `cargo test -p edaberu_core` で ③〜⑫ が全部通る。twilight・songbird に依存しない。

- [x] `config` を作り、③を満たす（3節の設定項目、B-38〜B-42・B-44）
  - メモ: 設定ファイルは `config.toml`（TOML・serde）。知らない項目は誤りにする（`#[serde(deny_unknown_fields)]`）。`max_chars` は1以上。トークンは環境変数 `DISCORD_TOKEN` から。終了の理由は文字列で返し、`main` が表示する。`config.example.toml` の中身と一致させる（テストで読み込む）
- [ ] `names` と、絵文字の判定を作り、④を満たす（R-36・R-41・R-42）
  - メモ: 絵文字の判定は小さなモジュール（`emoji`）に分けて `speech` からも使う。判定は Unicode の Extended_Pictographic（または Emoji）の性質で行う。
  - 足してよい依存: 絵文字を判定する crate を1つ（候補：`unicode-properties`・`emojis`・`unic-emoji-char`。Unicode 15 以降の表を持つ物を選び、選んだ物と理由を `docs/boundaries.md` 3節に書く）
- [ ] `speech` の置き換え・読まない物を作り、⑤を満たす（R-01〜R-21）
  - メモ: 規則は5節の番号順に当てはめる。URLの範囲は P-2 の定義（`http://`か`https://`で始まり、半角の文字が続く間。空白・改行・全角文字の手前で終え、末尾の `. , : ; ! ? ) ] ' "` は外す）。期待値は R表の「読み上げる文」を手で写す（実装の関数で作らない）
- [ ] `speech` の添付・スタンプ・投票と、長文・区切りを作り、⑥⑦を満たす（R-22〜R-35・B-29）
  - メモ: 文字数は書記素で数える（`unicode-segmentation`）。`max_chars` は引数で受け、30 のほかに 10 でも試す。空白・改行のまとめ方は R-32。区切りの文字は R-34。境界（ちょうど30・31・空・置き換えた語が30文字目にかかる）を必ず入れる
- [ ] `speech` の名前の省略と、入退室の文を作り、⑧を満たす（R-37〜R-43）
  - メモ: 「直前の人」はユーザーIDで比べる。入退室の文には `name_mode` と文字数の制限を当てない（R-43）
- [ ] `intake` を作り、⑨を満たす（B-21〜B-26・B-48）
  - メモ: 入力は付帯情報だけ（本文は先頭の1文字があれば足りる）。出力は「読む／入り終えたら読む／読まない」の3値。「！すごい」は読む、「!play」は読まない
- [ ] `state` と `voice_rules` を作り、⑩を満たす（B-01〜B-18）
  - メモ: `voice_rules` は「出来事＋状態」を受けて「指示の並び」を返す純粋な関数（VCの実物には触らない）。人と他のbotを分けて数える。自動参加の停止は `state` の中にだけ持つ（ファイルに書かない）。10秒は名前の付いた定数。2周目で `edaberu` 側に置いた最小の `state` があれば、ここに移して1つにする
- [ ] 点検: test-reviewer に点検させ、指摘を壊し方で確かめて直す（最大2回）
- [ ] `queue` を作り、⑪を満たす（B-27・B-28・B-30〜B-33・P-4）
  - メモ: サーバーごとに1本。`tts` と `player` は trait で受け取り、テストでは偽物にする。上限20件（名前の付いた定数）、IDで外す、発言の途中の区切りが失敗したら残りを飛ばす。待ち時間は注入できる形にし、テストは短い値で回す。読む仕組みが止まったら作り直す（B-47）
- [ ] `tts`（trait `Tts` と `Voicevox` 実装）を作り、⑫を満たす（B-32・B-34）
  - メモ: `/audio_query` → `/synthesis` の順。打ち切りの時間は引数で受け、既定は10秒（名前の付いた定数）。やり直さない。`/speakers` から話者の名前を取る（クレジット用）。偽のHTTPサーバーは、まず `tokio::net::TcpListener` で自前に書く（返事をしないサーバー・エラーを返すサーバー・正常なサーバーの3つ）。
  - 足してよい依存: 自前で足りないときだけ、dev-dependencies に1つ（候補：`wiremock`・`httpmock`）
- [ ] 点検: test-reviewer に点検させ、指摘を壊し方で確かめて直す（最大2回）
- [ ] 【えだ】PR を作ってもらい、差分を読んで main へ合流する

## 4周目: つなぐ — 読み上げが通しで聞こえる

ゴール: テキストの発言とVCの出入りが、VOICEVOXの声で聞こえる。「VCで聞く」の受け入れテストが通る。

- [ ] `gateway` に `intake` → `names` → `speech` → `queue` → `tts` → `player` をつなぐ（B-21・B-31・2節の「部品のつながり」）
  - メモ: twilight の型から `speech` が使う値だけを写す（本文・転送元の本文・添付の数・スタンプの有無・投票の有無・書いた人の3つの名前・IDなど）。転送・投票・スタンプの欄が twilight 0.17 の型にあるかを確かめ、無ければ4節の11番に書いて止まって聞く。2周目の `beep.wav` は外す
  - ID の 0 は twilight の Id::new_checked で誤りにし、B-39 の文で終了する（main の仮の読み込みと同じ扱い）
  - 完了条件: AT-14・AT-16 が通る（VCで聞く）
- [ ] 起動時の確認と、状態の表示と、後始末（B-34〜B-37・B-52）
  - メモ: 起動時に `/speakers` で話者を確かめる（B-34・B-36）。VOICEVOXが動いていなければ警告して続ける（B-35）。状態の表示にクレジット（例「VOICEVOX:ずんだもん」。好きな文を出せるかは4節の12番）。SIGTERM・SIGINT で、読み上げを止めてVCから出てから終わる（`tokio::signal`）。設定やトークンの誤りは0でない終了コードで終わる。ログは tracing で標準出力へ。本文はふだん出さない
  - 完了条件: AT-24 と、手元での Ctrl+C（AT-25 の手元版）が通る
- [ ] 【えだ】⑰ 残りの「VCで聞く」テストを通す（AT-03〜AT-07・AT-11・AT-13・AT-15・AT-18・AT-23・AT-27・AT-28・AT-36）
  - メモ: 通らなかった番号と、見えたことを `docs/measurements.md` の④に書く。直しはこの下に項目を足す（1件1項目）
  - 先頭に⑮の手動テスト1〜4
  - 1（切断）のあと、人が出て入り直す → bot が入るか（Issue #4）
- [ ] 点検: test-reviewer に点検させ、指摘を壊し方で確かめて直す（最大2回）
- [ ] 【えだ】PR を作ってもらい、差分を読んで main へ合流する

## 5周目: VPSに置く

ゴール: VPSで systemd から動き、再起動しても戻る。

- [ ] README の手順を、実物に合わせて直す
  - メモ: 1周目の①②で分かったこと（入れた物・かかった時間・話者の番号）を反映する。「予定の手順」の断り書きを外す。動かし方の節の番号（0〜4）を整える。GitHub Actions に切り替えていれば、その手順に置き換える
- [ ] 【VPS】バイナリ・設定・トークン・systemd のサービスを置き、⑱を通す（AT-25・AT-30・AT-38）
  - メモ: README の手順どおりに置く。`sudo systemctl enable --now edaberu` → `journalctl -u edaberu -f` で起動を見る → 対象のVCに入って読み上げを聞く → `sudo systemctl stop edaberu`（VCから出て終わるか）→ `sudo systemctl start edaberu` → `sudo kill -9 $(pidof edaberu)`（systemdが起動し直すか）→ `sudo reboot`（戻るか）。見えたことを `docs/measurements.md` の⑤に書く
- [ ] 【えだ】1日動かして `journalctl -u edaberu --since yesterday` を見る。気になった行があれば項目を足す。PR を作ってもらい、差分を読んで main へ合流する

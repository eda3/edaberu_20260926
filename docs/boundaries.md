# edaberu 部品の境目の設計

- 振る舞いは `docs/external-design.md` にだけ書き、ここでは番号（B-nn・R-nn・AT-nn）で参照する。
- 部品の名前・ファイルの分け方・型の形は、すべてClaude案（えだ未確認）。ただし、crateを2つに分けることは、えだが決めた（1節）。
- 技術の事実には確度を付ける。A＝一次情報で確認、B＝一次情報からの推論、C＝未確認。

## 1 部品の一覧

方針
- Discordとsongbirdに触る部品（`gateway`・`player`）を外側に置く（Claude案）。VOICEVOXに触る部品（`tts`）は、trait越しに使う（`queue` のテストでは偽物にする）。
- 判断する部品は、外の物に触らない純粋な関数や状態にして、自動のテストで確かめる（Claude案）。
- crateは2つに分け、Cargoのworkspaceにする（えだが2026-09-27に決めた）。

| crate | 入る部品 | 依存してよい物 |
|---|---|---|
| `edaberu_core` | 純粋な部品：`config`・`names`・`speech`・`intake`・`voice_rules`・`queue`・`state`、`Tts`・`Player` のtrait、`tts` の `Voicevox` 実装 | reqwest（`Voicevox` 実装がVOICEVOXを呼ぶため）。twilight・songbirdには依存しない（部品の入力は、twilightの型から写した値にする。2節の3）。IDは `u64` で持つ |
| `edaberu` | bot：`gateway`・`player`（songbirdの実装）・`main` | `edaberu_core`、twilight、songbird |

- 純粋な部品のテストは `cargo test -p edaberu_core` で回す。songbird（Opus）をビルドしないので、cmake・libopusの無い環境でも通る見込み［B］。`cargo test --workspace` にすると、songbirdもビルドされる。
- 部品ごとにモジュールを1つ作る（`<crate>/src/<部品名>.rs`。Claude案）。
- `state` は、`voice_rules` のテストで使うため `edaberu_core` に入れる（Claude案）。
- `tts` の `Voicevox` 実装とそのテスト（偽のHTTPサーバー）は、twilightにもsongbirdにも依存しないので `edaberu_core` に置く（えだが2026-09-27に決めた）。このため `edaberu_core` はreqwestに依存する。

| 部品 | 役割 | 入力 | 出力 | 依存する部品 | テストの仕方 |
|---|---|---|---|---|---|
| `config` | 設定ファイルと環境変数を読み、確かめる（external-design 3節、B-38〜B-42・B-44） | 設定ファイルの文字列（作業ディレクトリの `config.toml`。見本は `config.example.toml`。systemdの `WorkingDirectory` を `/opt/edaberu` にする）、環境変数 `DISCORD_TOKEN`（`/etc/edaberu/edaberu.env` に書き、systemdの `EnvironmentFile` で渡す。ファイルの権限は600） | `Config`、または終了の理由の文 | なし | 自動（AT-26） |
| `names` | 読む名前を決める（R-36・R-41・R-42） | ニックネーム・表示名・ユーザー名（どれも無いことがある） | 名前の文字列 | `speech` の絵文字の判定 | 自動（AT-34） |
| `speech` | 読み上げる文を作る（R-01〜R-35・R-43）。名前の省略の判定も持つ（R-37〜R-40） | 発言を写した値（本文、転送元の本文、添付の数、スタンプの有無、投票の有無）、名前、`name_mode`、`max_chars`、直前に読んだ人の状態 | 区切りの並び（`Vec<String>`）。空なら読まない（B-29） | `names` | 自動（AT-31〜AT-35） |
| `intake` | 届いた発言を読むかどうかを決める（B-21〜B-26・B-48） | 発言の付帯情報（サーバー、チャンネル、スレッドか、書いた人がbotかwebhookか、発言の種類、本文の先頭の文字）、サーバーの状態（botのVC、入る途中か） | `読む`／`入り終えたら読む`／`読まない` | なし | 自動（AT-17） |
| `voice_rules` | VCの出入りへの対応を決める状態の機械（B-01〜B-18） | 出来事（人・他のbot・bot自身のVCの変化、つながりが切れた、入れなかった、起動した、つなぎ直した）、サーバーの状態（botのVC、各VCの人数、自動参加の停止） | 指示の並び（入る・出る・入り直す・列に文を入れる・列を捨てる・停止を立てる/消す・何もしない） | なし | 自動（AT-02・AT-12） |
| `queue` | サーバーごとに1本の列と、それを先頭から読む仕組み（B-27・B-30〜B-33・P-4） | 入れる（発言のID付き）、IDで外す、全部捨てる | 区切りを1つずつ `tts` に渡し、できた声を `player` に渡す | `tts`、`player`（どちらもtraitで受け取り、テストでは偽物にする） | 自動（AT-19〜AT-22） |
| `tts` | 声を作る。trait `Tts` 1つと、VOICEVOXの実装 `Voicevox`（参考から借りる設計） | 区切りの文字列、話者ID | wavのバイト列、またはエラー（10秒で打ち切る。B-32） | なし（外のVOICEVOX） | 自動（偽のHTTPサーバーで打ち切りを確かめる。AT-22）、VCで聞く（AT-23） |
| `player` | VCで声を流す。trait `Player` と、songbirdの実装 | wavのバイト列 | 流し終えた知らせ、またはエラー | なし（songbird） | VCで聞く（AT-21の本物版、AT-37） |
| `state` | サーバーごとの状態を持つ（決定表 1）。`HashMap<GuildId, GuildState>` | 各部品からの更新 | botのVC、入る途中か、各VCの人（人とbotを分けて数える）、自動参加の停止、直前に読んだ人、列への入口 | なし | 自動（`voice_rules` のテストの中で） |
| `gateway` | twilightでDiscordとつながり、出来事を上の部品の入力に変え、指示を実行する。起動時の確認（B-34〜B-36・B-43・B-45）と、SIGTERM・SIGINTを受け取ったときの後始末（B-37）も持つ | Discordの出来事（twilight） | songbirdへの参加・退出、twilight-httpへの状態の表示の更新 | すべて | VCで聞く（AT-01ほか） |
| `main` | 部品を組み立てて起動する。ログは tracing で標準出力に出し、systemdのjournalに残す（本文はふだんは書かず、詳しいログのときだけ書く）。設定・トークンの誤りでは、0でない終了コードで終わる（B-52） | 設定 | — | すべて | —（`gateway` と一緒に確かめる） |

## 2 部品のつながり

発言が届いてから声が出るまで
1. `gateway` がtwilightから MESSAGE_CREATE を受け取る。
2. `gateway` が付帯情報を取り出し、`intake` に渡す。`読まない` なら終わり。`入り終えたら読む` なら、`state` の「入る途中の待ち」に置く（B-18）。
3. `gateway` が発言を写した値を作る。twilightの型から、`speech` が使う形だけを抜き出す。
4. `names` が名前を決め、`speech` が区切りの並びを作る（R-nn）。空なら終わり（B-29）。
5. `queue` に、発言のIDを付けて入れる。20件あれば、一番古い物を捨ててから入れる（B-30）。
6. `queue` の読む仕組み（サーバーごとに1つのtokioのタスク）が、先頭の1件を取り出す。区切りを1つずつ `tts` に渡して声を作る。
7. できたwavを `player` に渡し、songbirdで流す。流し終えるのを待ってから次の区切りへ進む（B-31）。
8. `tts` が失敗したら、その発言の残りを飛ばしてログに書く（B-32）。

VCの出入りから声が出るまで
1. `gateway` が VOICE_STATE_UPDATE を受け取り、`state` の各VCの人数を更新する。
2. 誰の変化か（人・他のbot・bot自身）と、前後のVCを `voice_rules` に渡す。
3. `voice_rules` が指示の並びを返す。
4. 指示ごとに次のとおり実行する。
   - 「入る」：songbirdで、スピーカーミュートにして入る。10秒でつながらなければ失敗とし、`voice_rules` に「入れなかった」を渡す（B-17）。
   - 「出る」「列を捨てる」：`queue` を空にしてから出る。
   - 「列に文を入れる」：`speech` で入退室の文を作り（R-43）、`queue` に入れる。2の発言の流れの5〜8と同じに進む。
5. songbirdから「つながりが切れた」を受け取ったら、`voice_rules` に渡す（B-15）。この知らせが「人に切断された」「移された」と区別できるかは未確認［C］（4節）。

## 3 使うライブラリと版

| ライブラリ | 版 | 使う機能 | 確度 |
|---|---|---|---|
| twilight-gateway・twilight-model・twilight-http | 0.17系（最新は0.17.1） | 出来事の受け取り、状態の表示の更新、起動時のIDの確認（B-43） | A（版） |
| twilight-cache-inmemory | 0.17系 | VCの状態とメンバーの名前を覚えておく（起動時にVCにいる人を数える。B-10） | B（0.17系にあるか） |
| songbird | 0.6 | `default-features = false`、機能 `driver`・`gateway`・`twilight`・`rustls`・`tws`。DAVEは `driver` に含まれる。songbird v0.6.0 のリポジトリに、twilightで動かす例（examples/twilight）がある | A |
| songbird の `builtin-queue` | 0.6 | 使わない（Claude案）。IDで外す（B-27）、上限で古い物を捨てる（B-30）、発言の途中の区切りを飛ばす（B-32）を、自前の `queue` で持つため | A（機能がある）／B（この用途に合わない） |
| symphonia | 未確認 | songbirdでwavを流すために足す。機能名は `wav`・`pcm` | A（足す必要）／B（機能名）／C（版） |
| tokio | 未確認 | 非同期の実行、サーバーごとの読み上げのタスク、SIGTERM・SIGINTの受け取り（B-37。`signal` 機能の `tokio::signal::unix` を使う） | C（版）／B（機能名） |
| reqwest | 未確認 | VOICEVOXを非同期のHTTPで呼ぶ（P-5）。10秒の打ち切り | C |
| serde・toml | 未確認 | 設定ファイルを読む。知らない項目があれば誤りにする（B-40） | C |
| tracing・tracing-subscriber | 未確認 | ログ | C |
| unicode-segmentation | 未確認 | 書記素で文字数を数える（R-33） | C |
| 絵文字を判定するcrate（候補は未定） | 未確認 | Unicodeの絵文字の一覧に載るかどうかの判定（R-15・R-16・R-41） | C |

## 4 未確認の項目

- 次の工程のToDo一覧は、上から順に（危ない所から）並べる（決定表 12）。
- 1番と2番（`cargo build` の実測とVOICEVOXのメモリの実測）は、同時に走らせない。どちらも1.9GiBのメモリを食い合うため。1番のビルドが終わってから、2番を始める。

| 項目 | 確かめ方 | 確かめる工程 |
|---|---|---|
| 1. Ubuntu 22.04でsongbird 0.6（Opus）をビルドするのに、cmake・libopus-devが要るか［C］。VPSの上で `cargo build --release` がメモリ1.9GiB＋スワップ2GiBで通るか（`-j 1`）［C］。依存の版（tokio・reqwest・serde・toml・tracing・unicode-segmentation・symphonia）と、絵文字を判定するcrateの選定［C］ | VOICEVOXのコンテナを止めた状態で、songbirdを依存に入れた構成を `cargo build --release -j 1` でビルドし、かかった時間と、メモリ・スワップの最大の使用量を測る。通らない場合の代替の候補：GitHub Actionsでビルドし、できたバイナリをVPSに置く。そのときは、VPSとglibcをそろえるため、runnerを `ubuntu-22.04` に固定するか、`ubuntu:22.04` のコンテナの中でビルドする。これでそろうかは未確認［C］ | ToDo一覧の1番目 |
| 2. VOICEVOX ENGINE（`voicevox/voicevox_engine:cpu-latest`）が、メモリ1.9GiBのVPSで、botとnginxと一緒に動くか。公式のREADMEには必要なメモリの記載が無い［C］ | 1番のビルドが終わってから、コンテナを `-p 127.0.0.1:50021:50021` で起動し、30文字ほどの文を続けて合成させて、`docker stats` でメモリの最大の使用量を測る。動かなければ、Q-115のbに切り替える（自宅のWindows PCのVOICEVOXをTailscale経由で呼ぶ。VPSにTailscaleが要る［C］） | ToDo一覧の2番目 |
| 3. VPSから、songbird 0.6 でDAVEを使ってVCにつながるか［C］ | AT-01を通す | ToDo一覧の最初のほう |
| 4. メモリ上のwavを、一時ファイルを作らずにsongbirdに渡せるか［C］ | songbird 0.6 のドキュメントとexamplesを読み、AT-37で確かめる | ToDo一覧の最初のほう（P-6の直し方が変わるため） |
| 5. botが「人に切断された」「別のVCへ移された」「ネットの不調で切れた」を区別できるか。twilightのVOICE_STATE_UPDATEと、songbirdのドライバの知らせの組み合わせで判別する［C］ | songbirdのイベントの型を読み、AT-08・AT-10・AT-29で確かめる | ToDo一覧の最初のほう（B-12・B-14・B-15が分かれるため） |
| 6. 設定やトークンの誤りで終わり続けるとき、systemdの既定（`StartLimitIntervalSec` と `StartLimitBurst`）で起動し直しが止まるか［B］ | systemdのドキュメント（systemd.unit）を読み、わざと設定を壊して `systemctl status` を見る | systemdのサービスを作るとき |
| 7. `systemctl stop` のSIGTERMで、VCから出る後始末が、systemdの待ち時間の中に終わるか［B］ | AT-25 | VCで聞くテストの工程 |
| 8. VCの中のチャットのメッセージは、チャンネルIDがそのVCのIDになるか［B］ | AT-10 | VCで聞くテストの工程 |
| 9. Developer Portal で Message Content Intent をオンにする必要があるか。オフのときに、twilightで理由の分かる誤り（閉じるコード）が届くか［B］ | AT-28 | VCで聞くテストの工程 |
| 10. webhookの発言で、書いた人のbotの印が真になるか。ならなければwebhookのIDで判定する［B］ | twilight-model の型を読む。AT-17の入力を作るときに確かめる | 自動テストの工程 |
| 11. twilight 0.17 の型に、転送（message_snapshots）・投票・スタンプの欄があるか［C］ | twilight-model 0.17 のドキュメントを読む | R-21・R-24・R-25の自動テストの前 |
| 12. botの状態の表示に、好きな文（例「VOICEVOX:ずんだもん」）をそのまま出せるか（カスタムステータス）［C］ | twilightの状態の表示の型を読み、AT-24で確かめる | VCで聞くテストの工程 |
| 13. VOICEVOXのクレジット表記（「VOICEVOX:キャラ名」）が要ること。二次情報（解説記事）で確認しただけで、公式の規約は読んでいない［B］。botの状態の表示に出すだけで要件を満たすかは未確認［C］ | VOICEVOX公式サイトの、使う話者の利用規約を読む | 見本の設定ファイルの話者を決めるとき |
| 14. VOICEVOXのエンジンに、話者の一覧を返す `/speakers` があること［B］ | VOICEVOXのエンジンのAPIのドキュメントを読む | `tts` の自動テストの前 |
| 15. VOICEVOXの声の前後に短い無音があり、区切りの間に無音を足さなくても聞き取れるか［B］ | AT-21の本物版を聞く | VCで聞くテストの工程 |
| 16. songbirdで、スピーカーミュートの状態で入れるか［B］ | songbirdの `Call` のドキュメントを読む。AT-01で見る | VCで聞くテストの工程 |
| 17. Discordの名前（ニックネーム・表示名・ユーザー名）が最大32文字であること［B］ | Discordの開発者向けドキュメントを読む | R-36の自動テストの前 |
| 18. Discordが自動でリンクにするのは http:// か https:// で始まる形だけか［C］ | テスト用のサーバーで「www.example.com」を書いて見る | R-12の自動テストの前 |
| 19. Public Bot をオフにすると、ほかの人がbotを招けなくなるか［C］ | Discordの開発者向けドキュメントを読む | READMEの手順を仕上げるとき |

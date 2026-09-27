# edaberu

「えだ」と「駄弁る」を合わせた名前の、Discordのチャット読み上げbot（Rust製）です。1つのサーバーで、テキストの発言とVCへの人の出入りを、VOICEVOXの声で読み上げます。

## 今の状態

設計中です。コードはまだありません。

- 振る舞いと受け入れテスト：[docs/external-design.md](docs/external-design.md)
- 部品の境目：[docs/boundaries.md](docs/boundaries.md)

## 動かし方（予定）

- 動かす場所はVPS（Ubuntu 22.04）です。
- コードができるまでの予定の手順で、実際に確かめてはいません。未確認の点は docs/boundaries.md の4節にあります。

### 1. VOICEVOXを起動する

同じVPSの上で、Dockerで動かします。50021番は、VPSの中（127.0.0.1）からだけ呼べるように開けます。`--restart unless-stopped` を付けるので、VPSを起動し直すとVOICEVOXも自動で起動します。

```sh
docker run -d --name voicevox --restart unless-stopped \
  -p 127.0.0.1:50021:50021 voicevox/voicevox_engine:cpu-latest
```

### 0. VPSに Docker と Rust（rustup）を入れる

入っているかは未確認です。

```sh
sudo apt install -y build-essential cmake pkg-config libopus-dev
```

### 2. botを置く

- botは、VPSの上で `cargo build --release -j 1` でビルドします。
  - cmake・libopus-devが要るかは未確認です。
  - メモリが足りずにビルドが通らない場合は、GitHub Actionsでビルドしたバイナリを置く方法を考えています。
- 置く場所は次のとおりです。
  - バイナリ：`/opt/edaberu/edaberu`
  - 設定ファイル：`/opt/edaberu/config.toml`（`config.example.toml` をコピーして書く）

### 3. トークンを置く

トークンは、リポジトリにも設定ファイルにも書きません。

```sh
sudo mkdir -p /etc/edaberu
sudo sh -c 'echo "DISCORD_TOKEN=ここにトークン" > /etc/edaberu/edaberu.env'
sudo chmod 600 /etc/edaberu/edaberu.env
```

### 4. systemdのサービスにする

`/etc/systemd/system/edaberu.service` に次を書きます。`User=` は書かず、botはrootで動かします（えだの判断）。

```ini
[Unit]
Description=edaberu (Discord読み上げbot)
Wants=network-online.target
After=network-online.target docker.service

[Service]
WorkingDirectory=/opt/edaberu
EnvironmentFile=/etc/edaberu/edaberu.env
ExecStart=/opt/edaberu/edaberu
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now edaberu   # 今すぐ起動し、VPSの起動時にも自動で起動する
journalctl -u edaberu -f              # ログを見る
sudo systemctl restart edaberu        # 設定ファイルを書き換えたら、起動し直して反映する
sudo systemctl stop edaberu           # 止める（botはVCから出てから終わる）
```

## 読み上げを止めたいとき

コマンドはありません。botをVCから切断すると、読み上げが止まります。そのあと、人が対象のVCに入ってきても、botは自動では戻りません。

botが戻るのは、次のどちらかのときです。

- 対象のVCから人が一度いなくなり、そのあと誰かが入ったとき
- botを再起動したとき（対象のVCに人がいれば、すぐ入ります）

根拠は docs/external-design.md の B-02・B-10・B-12・B-13 です。

## 参考元

振る舞いの参考にしたリポジトリです。コードは写していません。

- [KIKUKOU/yomiagecode](https://github.com/KIKUKOU/yomiagecode)（作者 KIKUKOU、MITライセンス）

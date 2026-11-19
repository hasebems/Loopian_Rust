# CLAUDE.md

Loopian は、ライブコーディング向けのテキストベースのピアノ・シーケンサ（Rust 製）。
1行ずつ入力したテキスト（移動ドの階名、コード、コマンド）から MIDI を出力し、演奏に連動したグラフィックを表示する。
ユーザー向けの仕様は `doc/manual.md`（英語版 `doc/manual_en.md`）にある。

## ビルド・実行・テスト

```sh
cargo build                  # デバッグビルド（初回は Bevy を含むため数分かかる）
cargo build --release
cargo run                    # GUI 版。リポジトリ直下の settings.toml を読む
cargo run -- server          # CUI 版（server_loop.rs）
cargo test                   # loopian_app/src/test/ のテスト
cargo build --features verbose   # MIDI 受信などの詳細ログ
```

- `--features raspi` は Raspberry Pi 用（rppal を使う）。rppal は Linux 専用なので、macOS ではビルドもチェックもできない
- `cargo test` は 14件中 `test::test_general::pedal` の1件が以前から失敗している（期待値 dur 440 に対し実際は 480）。それ以外が通れば悪化なし
- 起動には `settings.toml` が必要。カレントディレクトリになければ、実行ファイルのディレクトリを探す（`file/settings.rs`）。例は `settings_example.toml`

## 構成

Cargo workspace（edition 2024）。

| crate | 役割 |
|---|---|
| `loopian_app` | 本体（bin 名 `loopian`）。入力、コマンド解析、演奏、MIDI、画面 |
| `loopian_graphic_api` | グラフィックパターンの共通 trait（`GenerativeView` など）、`Resize`、フォント名の定数（`font.rs`） |
| `loopian_graphics` | 各グラフィックパターン（`view_*.rs`）と、名前・番号からの生成 |

`loopian_app/src` の主なディレクトリ:

- `cmd/` 入力テキストの処理（`input_txt.rs`）、コマンド解析（`cmdparse.rs`）、テキスト → シーケンスデータ変換（`txt2seq_*.rs`）
- `elapse/` 演奏の時間管理。`stack_elapse.rs`（演奏スレッドの中心）、`tickgen.rs`（時刻 → 小節/tick）、各 Elapse オブジェクト（part, note, pedal, flow）
- `elapse_loop/` フレーズ・コンポジションのループ再生
- `midi/` MIDI 送信（`miditx.rs`）と受信（`midirx.rs`）
- `graphic/` 画面描画（`draw_graph.rs`）と、app 側のグラフィックパターン（ripple）
- `file/` 設定、ファイルの load/save、履歴
- `server/` CUI 版
- `common/lpnlib.rs` 定数、スレッド間メッセージ（`ElpsMsg`, `UiMsg`）
- `sample/` は .gitignore 済み・ビルド対象外

## スレッド構成

- **メインスレッド（GUI）**: nannou 0.20（Bevy 上）。`main.rs` の `update` / `view`。キー入力 → `InputText` → `ElpsMsg` を演奏スレッドへ送る。演奏スレッドからの `UiMsg` を受けて表示を更新する
- **演奏スレッド（"elapse"）**: `main.rs` の `gen_elapse_thread()`。`ElapseStack::periodic()` をループで呼ぶ。`ElapseStack::next_wait()` で、次のイベントが 3ms 以内でなければ `recv_timeout` で 1ms 休む
- **MIDI 受信**: midir のコールバックで `ElpsMsg::MIDIRx` に変換して演奏スレッドへ送る。受信スレッド（`MidiRxThread::run()`）は制御メッセージを待って眠っている（raspi の UART だけ 1ms ごとに見る）

演奏スレッドの中は `Rc<RefCell<...>>` で、スレッドをまたがない。GUI 側の `Model` は Bevy のリソースなので `Send + Sync` が必要（`Receiver` は `Mutex` で包んでいる）。

## nannou 0.20 / Bevy まわりの注意

詳細は `doc/record/nannou261119.md`。

- テキストのフォントはファミリー名で指定する（`loopian_graphic_api::font` の `FONT_*`）。太さ・斜体は指定できないので、フォントファイル側でファミリー名を分けてある（`assets/fonts/*-Loopian.ttf`）
- `draw.text()` でサイズを指定しないと枠が window 全体になる。0.19 と同じにするには `.w_h(200.0, 200.0)` を明示する
- キー入力は nannou のコールバックではなく、`main.rs` の `KeyInputPlugin`（Bevy の `KeyboardInput` を直接読む）で受ける。nannou の `received_character` はキーを離したときにも文字を渡すため使わない
- window サイズは `settings.toml` の pixel 指定を、起動後に拡大率で換算して合わせる（`adjust_window_size()`）
- 描画は垂直同期（`PresentMode::Fifo`）。nannou の既定（Mailbox）は macOS では使えない
- 色は Bevy の `Srgba`（`Srgba::rgb_u8()` など）

## 演奏タイミングに関わる変更

- `stack_elapse.rs` / `tickgen.rs` / 演奏ループの変更は演奏タイミングに直接影響する。変更したら、IAC Driver に出る Note On の間隔を実測して比べる（記録は `doc/record/cpu_load261119.md`）
- 音の間隔の数 ms のばらつきは、Loopian の揺らぎ（`floating_tick.rs` などの正規分布）によるもので、もともとある

## ドキュメント・メモ

- `doc/manual.md` / `doc/manual_en.md`: ユーザー向けマニュアル。コマンドや操作を変えたら更新する
- `doc/record/`: 実装内容の記録（git で管理する）。大きな対応をしたら、経緯・対応内容・確認したこと・残っていることをここに書く
    - `refactor260321.md`: cmd 層の構造的な問題の指摘と、リファクタリングの記録
    - `plus&at260907.md`: `[]+` と `@n=` の問題、追加フレーズ入力機構の再設計と実装
    - `nannou261119.md`: nannou 0.19 → 0.20 の移行
    - `cpu_load261119.md`: CPU 負荷の削減（MIDI 受信、垂直同期、演奏スレッドの休み）
- `doc/develop_memo/`: 個人の開発メモ。**フォルダごと .gitignore されている**ので、git で管理したい内容は `doc/record/` に書く
- 記録・メモのファイル名は `<トピック><yymmdd>.md` の形

## 作業の約束

- コード中のコメント、ドキュメント、コミットメッセージは日本語
- コミットメッセージは1行目に「〜した」の形で要約し、必要なら本文で理由と内容を書く
- `settings.toml` などの設定ファイルの変更はユーザー自身の作業であることが多い。依頼がない限りコミットに含めない
- 配布物は `../Loopian_Release` フォルダ（git 管理外。手順は `howto_release.md`）で作る。`assets/` を変えたらリリース時に反映が必要
- フォントなど同梱物はライセンスを確認する（現在は OFL のみ。`assets/fonts/OFL-*.txt`）

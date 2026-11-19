# CPU 負荷の削減メモ

2026/11/19 に、何もしていなくても PC が熱くなる問題に対応した。

| commit | 内容 |
|---|---|
| 6ac2070 | MIDI 受信を、届いたときだけ処理する方式にした |
| 0927f41 | 描画を垂直同期 (PresentMode::Fifo) にした |
| 132c472 | 演奏スレッドを、次のイベントが近くないときは 1ms 休むようにした |

## 結果

何もしていないときの CPU 使用率（release ビルド、M1 Pro。100% = CPU コア1個分）

| 段階 | CPU 使用率 |
|---|---|
| 対応前 | 約 250〜370% |
| MIDI 受信を変更 | 約 190〜280% |
| ＋垂直同期 | 約 150〜210% |
| ＋演奏スレッドの休み | 約 90〜120% |

再生中（4分音符を鳴らし続ける）は、MIDI 受信と垂直同期を変更した状態で、演奏スレッドの休みなし 約 260% → 休みあり 約 90〜165% だった。

## 原因

nannou / Bevy のせいではなく、主に Loopian 自身のスレッドが休まず回っていたため（nannou 0.19 のころから同じ）。

1. 演奏スレッド（elapse）: `loop { est.periodic(rxmsg.try_recv()) }` に待ちがなく、コア1個分を使い続けていた
2. MIDI 受信スレッド: midir のコールバックがバッファに積んだものを、休まず見に行っていた。コア1個分
3. 描画: nannou 0.20 の既定の `PresentMode::Mailbox` は macOS では使えず、垂直同期なしの `Immediate` になっていた。描画が上限なしで回っていた

## 対応内容

### 1. MIDI 受信（`midi/midirx.rs`, `elapse/stack_elapse.rs`）

- midir のコールバック（MIDI が届いたときに midir が別スレッドで呼ぶ）の中で、`ElpsMsg::MIDIRx` に変換して演奏スレッドに送る
    - `MidiRxBuf`（バッファ）は削除した
    - 届いてから演奏スレッドに渡るまでの遅れも、以前より短くなった
- 受信スレッドの処理は `MidiRxThread::run()`。制御用メッセージ（Quit / MidiReconnect / MidiInputCh）を `recv()` で待って眠っている
    - `MsgCtrl::Start` のときのバッファのクリアは、バッファがなくなったので不要
- 受信 ch の設定（`midi_input_ch`）は、コールバックからも読むため `Arc<AtomicU8>` で共有する
- raspi の UART は通知の仕組みがないため、`recv_timeout(1ms)` で待ちながら 1ms ごとに見に行き、届いているバイトをすべて読む（以前は1回に1バイトだけ読んでいた）
    - rppal は Linux 専用なので、macOS ではコンパイルも確認できていない。Raspberry Pi でビルドして確認が必要
- 以前の `receive_midi_event()` は、対象外の ch のメッセージが来ると `return` して2つ目のポートの確認が後回しになっていたが、コールバック方式ではポートごとに独立して処理されるので、この問題もなくなった

### 2. 垂直同期（`main.rs`）

- window の builder に `.present_mode(PresentMode::Fifo)` を指定した
- 描画回数が画面の書き換え回数（この Mac は最大 120Hz）に揃う
- 起動時の `PresentMode Mailbox requested but not available. Falling back to Immediate` のログも出なくなった

### 3. 演奏スレッドの休み（`main.rs`, `elapse/stack_elapse.rs`）

- `ElapseStack::next_wait()` が、次に `periodic()` を呼ぶまで休んでよい時間を返す
    - 再生していないとき: 1ms（`MAX_WAIT`）
    - 再生中: 次のイベントまでの tick 数を求め、現在のテンポ（`get_real_bpm()` × `get_bpm_rate()`）で時間に換算する
        - 候補は、各 Elapse Obj. の `next()` と、次の拍頭（拍頭・小節頭の処理があるため）
        - すでに処理すべきイベントがあるか、3ms（`NO_WAIT_THRESHOLD_SEC`）未満なら休まない（0 を返す）。それ以外は 1ms
        - 3ms にしたのは、macOS の 1ms の sleep が実際には少し長くなることと、elastic（swing など）で tick と時間の関係がずれることへの余裕
    - fermata 中（`imaginary_tick` があるとき）は tick と時間の関係が変わるため休まない
    - tick の計算は `msr * tick_for_onemsr + tick` を i64 で行う（`next_msr` が `FULL` = 10000 のこともあるため）
- 演奏ループ（`main.rs` の `gen_elapse_thread()`）は、休むときに `sleep` ではなく `rxmsg.recv_timeout(wait)` を使う。画面からのコマンドが届けば、すぐ起きて処理する

## 演奏タイミングの実測

IAC Driver に出る Note On の時刻を別プログラム（midir で受信）で記録し、音の間隔を比べた。
bpm 100 の4分音符（間隔 600ms）、75音、各45秒。

| 方式 | 間隔の平均 | 標準偏差 | 最大のずれ |
|---|---|---|---|
| 休みなし 1回目 | 599.97ms | 3.72ms | 12.6ms |
| 休みあり 1回目 | 599.95ms | 4.23ms | 10.7ms |
| 休みなし 2回目 | 599.99ms | 4.54ms | 13.8ms |
| 休みあり 2回目 | 599.95ms | 3.78ms | 12.5ms |

- 差は測定ごとの揺れの範囲で、休みを入れても悪化していない
- 数 ms のばらつきは休みなしでもある。Loopian の揺らぎ（`floating_tick.rs` などの正規分布）によるものと思われる
- 演奏タイミングに関わる変更をしたら、同じ方法で比べるとよい

## 残っていること・選択肢

- 残りの約 100% は Bevy（描画と、その作業用スレッド）
    - `app.set_update_rate(60.0)` で描画を毎秒60回にすれば、さらに数十％下がる見込み（MIDI 受信を変更する前の試験で、垂直同期だけ 210〜300% → 垂直同期＋60回制限＋固定 0.5ms の休み 115〜160%）。ただしアニメーションが 120Hz から 60Hz になる
- MIDI 入力（FLOW パート用）は、演奏スレッドが MIDI 受信用の別の channel から読んでいる（`check_rcv_midi()`）。休んでいる間に届くと最大で約 1ms 遅れる
    - 気になるなら、MIDI 受信も画面からのコマンドと同じ channel に流せば、届いた時点で起きるようにできる

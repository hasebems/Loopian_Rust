## copilot 指摘事項

src/cmd 配下を横断して、責務分割や依存関係、入力処理の形に構造的な問題がないかを確認します。まず関連メモを読み、対象ファイル一覧と主要な実装を把握します。

**指摘事項**

1. (済み)高優先: InputText に UI、履歴、ファイルロード、コマンド実行、オートロード状態機械が同居していて、変更波及が大きいです。状態も input_text 系、scroll_lines、load_buffer、cmd、auto_load_buffer、auto_load_state、next_msr_tick に分散しており、1 つの修正で複数の副作用を追う必要があります。特に input_txt.rs、input_txt.rs、input_txt.rs、input_txt.rs は責務境界が曖昧です。  
修正方向:
- UI 入力処理
- ファイルロードとオートロード
- コマンド実行
を別コンポーネントに分けるべきです。今の構造だと、表示仕様の変更でも再生制御に触りやすく、保守コストが高いです。

2. (済み)高優先: set 系コマンドの分岐が 1 か所に集中しすぎています。cmd_set.rs の parse_set_command は if/else の列で機能追加を受け止めており、拡張時に毎回この関数を直接編集する形です。  
問題点:
- Open/Closed に反する
- バリデーションと副作用送信が同一関数に混在
- 戻り値が String なので失敗理由が機械的に扱えない  
修正方向:
- SetCommand の enum 化
- 解析と実行の分離
- 戻り値を Result 系に統一  
この部分は構造改善の効果が最も大きいです。

3. (済み)高優先: LoopianCmd 自体がパーサ、状態保持、送信窓口、各種コマンド実装をまとめて持っており、cmd 層の中心が肥大化しています。cmdparse.rs と cmdparse.rs を見ると、入力分類、字句分解、コマンドディスパッチが同一 impl に集約されています。  
問題点:
- コマンド追加時に LoopianCmd への集中が続く
- パース仕様変更が実行経路に直結する
- テスト対象が大きすぎて局所テストしにくい  
修正方向:
- tokenizer
- command parser
- command executor
を分けるべきです。特に analyze_and_divide_to_msg は独立した純関数モジュールに切り出す価値があります。

4. (済み)高優先: エラー表現がばらばらで、構造として不安定です。cmd 配下では bool、Option、String が混在し、失敗時に what? へ潰している箇所が多いです。たとえば cmd_set.rs、seq_stock.rs、seq_stock.rs、cmdparse.rs 周辺です。  
問題点:
- 失敗理由を呼び出し元で区別できない
- UI 表示用文言と内部エラーが結合している
- 将来ログやテストで原因追跡しづらい  
修正方向:
- CmdError の導入
- Result<CmdReply, CmdError> へ寄せる
- UI 文言生成は最終段だけにする  
これは構造改善であり、同時にデバッグ性もかなり上がります。

5. 中優先: SeqDataStock が Phrase、Composition、Pedal の 3 系統を同時に抱えており、データ境界が不自然です。seq_stock.rs から seq_stock.rs と、各 setter の seq_stock.rs、seq_stock.rs がその状態です。  
問題点:
- 異なるドメインモデルを 1 ストアで握るため API が不揃い
- part の種類で分岐するコードが増える
- 将来 part 種別が増えるとさらに肥大化する  
修正方向:
- PhraseStore
- CompositionStore
- PedalStore
に分離し、LoopianCmd 側で合成する形が自然です。

6. 中優先: MessageSender が送信専用ではなく、SeqDataStock の内部構造に依存した取り出し役まで兼ねています。send_msg.rs、send_msg.rs、send_msg.rs が該当です。  
問題点:
- sender が domain storage を知りすぎている
- 送信テストで SeqDataStock 全体に依存する
- データ変換責務と transport 責務が混ざる  
修正方向:
- MessageSender は ElpsMsg を送るだけに限定
- ElpsMsg 生成は stock か executor 側で完結  
この分離をすると送信層の見通しがかなり良くなります。

7. 中優先: auto load の状態管理が散っており、仕様変更に弱いです。input_txt.rs、input_txt.rs、input_txt.rs に典型的に出ています。  
問題点:
- next_msr_tick と auto_load_state と auto_load_buffer が別管理
- set_command、load 系、auto_load_command が同じ状態を横断更新
- 実行順序依存のバグを埋め込みやすい  
修正方向:
- AutoLoadController のような専用状態機械に寄せる
- 状態遷移を enum 1 つに集約する  
今は読めば追えますが、今後の修正で壊れやすい構造です。

8. 低から中優先: 和音テーブルが完全に静的ハードコードで、拡張点として閉じています。txt2seq_cmps.rs、txt2seq_cmps.rs が中心です。  
これは直ちに悪いわけではありませんが、
- スケール追加
- ユーザー定義和音
- 外部定義化
のどれかを将来やるなら先に境界を切っておいた方がいいです。今のままだと機能追加がソース編集前提になります。

**補足**
- 変換モジュール群の分離自体は比較的良く、txt2seq_phr、txt2seq_cmps、txt2seq_ana、txt2seq_pdl という方向性は妥当です。
- ただし、それらを束ねる上位層が InputText と LoopianCmd に寄りすぎています。問題は個々の変換器より、上位オーケストレーションの密結合です。
- 現時点で最優先なのは InputText と LoopianCmd の分割です。ここを崩すと他の問題もかなり自然に解けます。

**優先順位**
1. InputText を UI と実行制御に分離
2. LoopianCmd を parser と executor に分離
3. parse_set_command を enum ベース dispatch に変更
4. エラー型を統一
5. SeqDataStock と MessageSender の責務境界を整理

**確認事項**
- 今回はレビューのみで、コード変更はしていません。
- 実行確認はしていませんが、構造レビューとしては十分に改善候補があります。

必要なら次に 2 通り進められます。  
1. 最小差分で始めるリファクタ方針を作る  
2. まず InputText と LoopianCmd の分割案を具体的なモジュール単位で設計する

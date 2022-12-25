<!--
![loopian_logo](loopian_logo.gif)
-->
<img src="loopian_logo.gif" width="50%">

loopian Alpha-version written in Rust
========================================

about loopian
--------------

'loopian' is a sequencer for piano tones with text input that we are developing for use mostly in Live Coding.


loopian とは
------------

loopian は、Live Coding などで使うために開発している、テキスト入力によるピアノ音色用シーケンサです。



<!--


用語集 (Glossary)
-------------------

- phrase : 階名にて指定する単パートの音群
- composition : phrase に適用する数小節分の Chord 情報
- loop : loopian は基本的に、phrase/composition を繰り返し演奏する。この繰り返す単位
- part : phrase は独立した４つの Loop 再生が可能である。その４つを part と呼び、各 part は left 1(L1)/left 2(L2)/right 1(R1)/right 2(R2) という名前で示される。


起動と終了
--------------

- 起動
    - 'python loopian.py'  : 通常の python スクリプトと同じ
    - './loopian.sh'       : shell script として
- 入力
    - 'L1> ' : prompt
        - L1> は Left 1 の入力状態であることを示す
        - このプロンプトの後に、コマンドやフレーズを書き込む
    - カーソルによる過去入力のヒストリー呼び出しが可能
- 終了
    - 'quit' 'exit' : 終了


音を出すための外部環境
--------------------

- 外部 MIDI 音源を繋ぐ
- マルチパートで MIDI受信するアプリを同時に起動する。以下のアプリで動作確認済。
    - Logic : Mac で MIDI 演奏するための DAW


再生コントロール
--------------

- 'play' 'start' : シーケンス開始
- 'fine' : この小節の最後でシーケンス終了
- 'stop' : 直ちにシーケンス終了
- 'sync' : 次の小節の頭で、ループ先頭に戻る
    - sync       : そのパートのみ
    - sync right : 右手パート(right1/2)
    - sync left  : 左手パート(left1/2)
    - sync all   : 全パート


Phrase 追加
-------------

- [*note&duration*][*musical expression*] : phrase 追加の書式
    - *note&duration*: 階名と音価表現を入力する
    - *musical expression*: 音楽表現を入力する
        - [*musical expression*] は省略可能
    - [] : 全データ削除
    - +[d,r,m] : 最初に + を付けることで、今まで入力したPhraseの後ろに、新しい小節を連結できる。
        - '+' を使って複数のPhraseを連結する場合、音価、音楽表現の省略は、最初のデータに従う

- 階名表現
    - d,r,m,f,s,l,t: ド、レ、ミ、ファ、ソ、ラ、シ
    - di,ri,fi,si,li: ド#、レ#、ファ#、ソ#、ラ#
    - ra,ma,sa,la,ta: レb、ミb、ソb、ラb、シb
    - -d: 1オクターブ下のド、 +d: 1オクターブ上のド、--d: 2オクターブ下、++d: 2オクターブ上
    - x: 休符
    - ',': 各音の区切り。１小節を超えたら捨てられる。区切りが連続すると休符が省略されたとみなす
    - '|' '/' : 小節区切り。区切りが連続すると休符が省略されたとみなす
    - d=m=s, : 同時演奏
    - |:d,r,m:3| : ドレミを３回繰り返し、合計４回演奏（数字がなければ1回繰り返し） <- 止める
    - <d,r,m>*4 : ドレミを４回演奏
    - d*4 : ドを４回連続して発音


- 音価表現
    - do| : ドをその小節の終わりまで伸ばす（oから|までの間の文字は無視される） 
    - d.. / d~~ : ドを基準音価の３倍伸ばす 
        - d.|.. のように小節を跨ぐこともできる（タイ）
        - do|o| は２小節伸ばす
    - [8:] : 基準音価が 8 であることを示す
        - 基準音価は任意の数値が指定可能で、全音符の長さの何分の1かを示す数値となる
        - 基準音価(n:)を省略した場合、全て四分音符とみなす

- 音楽表現
    - f,mf,mp,p,pp: 音量
    - stacc: 音価を半分にする



Composition 指定
----------------------------

- {*chord*}{*musical expression*} : Composition の書式
    - *chord*: コードを小節ごとにカンマで区切って時系列で記述
    - *musical expression*: 音楽表現
        - {*musical expression*} は省略可能
    - {} : 全データ削除
    - +{I} : 最初に + を付けることで、今まで入力したコードの後ろに、新しい小節を連結できる。
        - + を使って複数のコードを連結する場合、音楽表現の省略は、最初のデータに従う


- 長さの指定方法
    - '|' '/' : 小節区切り。区切りが連続するとコードがないとみなす
    - {I||IV|V} : １小節ごとに I -> I -> IV -> V とコードが変わる
        - 複数小節を同じコードにしたい場合、'|'のみ記述
        - 同小節内でコードを変える場合、拍ごとに','で区切る。複数拍を同じコードにしたい場合、'.' で伸ばす
        - ',,' のように何も記さずにカンマを続けた場合、その拍にコードがないとみなす
    - コード情報とピアノの Pedal 情報はリンクしている
        - コードが空白、あるいは 'thru' 指定の場合、ペダルは踏まれない
        - コードが変わるごとにペダルが踏まれる
        - 小節が変わるごとにペダルが踏まれる


- コード記述方法
    - O : original phrase
    - I : d=m=s（Iの和音)
        - ローマ数字: I, II, III, IV, V, VI, VII
    - I# : di=mi=si (数字の後に # を付けると半音高いコードになる。b は半音）
    - V : s=t=r (Ⅴの和音)
    - VIm : l=d=m (m: minor)
    - IVM7 : f=l=d=m (M7: major7th)
    - IIIm7-5 : m=s=ta=r (m7-5: minor7th -5th)
    - diatonic : d=r=m=f=s=l=t (Diatonic Scale)
    - lydian : d=r=m=fi=s=l=t (Lydian Scale)
    - Iion : Iを主音としたイオニアン(Ionian)
    - thru : 全ての音


- 音楽表現
    - para : 和音変換時、root に合わせて並行移動する 
    - noped: Pedal Off指定


入力環境コマンド
----------------

- 'right1' 'left1' : 右手２パート、左手２パートの４パートを指定可能
- 'all' : 全パートの入力モードになる
- 'midi 1' : MIDI PORT 1 を選択
- 'panic' : 今鳴っている音を消音する


調、テンポ、拍子、音量
-------------------

- 'set bpm=100' : BPM（テンポ）=100 にセット
- 'set beat=4/4' : 拍子を 4/4 にセット
- 'set key=C4' : key を C4 にセット
    - loopian にとって key とは [d]（ド） と指示されたときの音名を表す
    - デフォルト値は C4(midi note number=60)
    - 音名は C-B と大文字で表現し、必要に応じて前に #, b を足すことができる
    - 音名の後ろの数値はオクターブを指示するが、省略可能
        - 省略した場合、今設定されているオクターブがそのまま適用される
- 'set oct=+1' : 現状から１オクターブ上げる
    - set 以降に all を付け足すと、全 part に効果、付けなければ入力中の part に対してのみ効果
    - 'set oct=0,0,-1,+1' : 4つのパートのオクターブを一度に設定できる
- 'set input=fixed' : 階名を入力したときのオクターブ決定法
    - fixed は、入力する階名の位置は固定
    - closer は、指示がない限り、前回に近い音程 (default)
- 'set samenote=modeling' : 同音連打の動き方
    - modeling は、モデリング音源向けで、note off は一度しか送らない（default）
    - common は、一般的なMIDI音源向けで、note off は note on の数だけ送られる


-->
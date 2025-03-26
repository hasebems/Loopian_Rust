## [You can watch my performances on YouTube here.](https://youtube.com/playlist?list=PLUerrAh-bsOWs2xvYudPrxicXUfWKq6BX&si=wxtJ2tmU8dMsnStj)

# about Loopian

`Loopian` is a text-based piano sound sequencer being developed for use in activities like Live Coding. It has the following features:
- Text is input line by line, allowing you to specify phrases, chords, or control the overall performance with commands.
- To achieve a somewhat natural performance, the velocity, pitch during chord changes, and damper pedal usage are automatically calculated.
- Phrases are specified using movable-do solfège.
- [Loopian Reference Manual](doc/manual_en.md)

# Loopian とは

Loopian は、Live Coding などで使うために開発している、テキスト入力によるピアノ音色用シーケンサです。
以下の特徴があります
- テキストは1行単位で入力し、フレーズや和音を指定したり、演奏全体をコントロールするコマンドを指定
- ある程度自然な演奏になるように、ベロシティや和音変換時の音程、ダンパーペダルを自動算出
- フレーズは移動ドにて指定
- [Loopian Reference Manual](doc/manual.md)


# Loopian のインストールと起動方法

## インストール方法
- [Release ページ](https://github.com/hasebems/Loopian_Rust/releases)より、OSがWindowsの方は loopian_win.zip を、Macの方は loopian_mac.zip をダウンロードしてください。
- ダウンロード後、任意のフォルダで展開すればすぐに使用可能です。

## 起動方法、終了方法
- loopian というファイルをダブルクリックすれば、起動します。
- Windowsの方の場合、Microsoft GS Wavetable Synth が自動で鳴るようにセッティングされています。
- Macの方の場合、IACDriver が設定されているので、Garage Band などのアプリを起動すれば音が鳴るようになります。
- アプリ終了は、Windowを閉じるか、入力スペースに `!q` と書いて return を押してください。

# Loopian記法の紹介

## Loopianについて

Loopian は、テキストによるオリジナルの音楽記述記法と、それを動作させるアプリ、及び関連する周辺デバイスを総合した、プロジェクト全体の総称です。
アプリの中で使われるオリジナルの記法をLoopian記法と呼びます。主にLive Codingを実践するために開発しました。
Live Codingとは、リアルタイムにプログラムを書くことで音楽を生成していくパフォーマンスのことです。
観客は音楽を聴くだけでなく、プロジェクターで投影された映像を通して、プログラムがリアルタイムに書かれていく様子や音が映像に反映している様子を見ることができます。
このようにLive Codingでは、プログラムを書くことそのものを、パフォーマンスとして捉えます。

Loopianシステムも同様に、テキストで音楽情報を入力し、リアルタイムに音楽を変えていくことが可能です。
また、アプリの背景に再生中の音楽にシンクロしたグラフィックを表示します。
ただし、Loopianは一般的なLive Codingのシステムとは違い、プログラミングの形式でテキストを入力するのではなく、ターミナル等で一行ずつコマンドを入力するように、一行ごとに音楽的指示を入力していきます。
このように、Loopianには、他のLive Codingシステムとは若干趣きの違う幾つかの特徴があります。
以下に、Loopianの主な特徴を挙げてみましょう。

- 上述のように、指示は一行単位で入力し、アプリは一行ごとに文字列で反応を返します。あたかもアプリとチャットしているようなインターフェースとなっています。
- 基本的にピアノ音色のみを扱います。詳細な仕様はピアノ音色を鳴らすことを意識して設計されています。このため比較的落ち着きのあるクラシカルな雰囲気の音楽を志向します。
- 音程の指示は、C,D,E,F...といった音名や数値ではなく、移動ドの考えに基づき、ドレミファソラシドの階名によって指示します。
- Loopian記法はより音楽の仕組みに寄り添った体系となっており、将来的には教育分野でこの記法を利用するといった使い方も考えられます。

現在Loopianのシステムには、PC上で動作する専用アプリ"Loopian::APP"と、アプリと連携して動作するデバイス"Loopian::ORBIT"が存在します。
Loopian::APPは、Loopianの思想を体現する中核を成しているので、このアプリを単にLoopianと呼んでも構いません。
今後も、このLoopianの思想に基づいた様々なアプリ、デバイスが出現する可能性もあります。


## Loopian記法の基本的な考え方

これより、Loopian記法について詳細に説明しますが、まずこの記法の特徴やその背景について、最初に説明します。

### 移動ドと音程表記について

すでにLoopianの紹介の項で述べたとおり、本アプリでは音程を階名によって指示します。
音楽教育の場では、音程をドレミファソラシドで表す時、固定ドと移動ドのどちらが良いかという論争が存在します。一般的には、専門教育が固定ドで行われているため、ほとんどの教育の場で固定ドが使われることが多いようです。
しかし、音程は移動ドで表現されるべき、という開発者の背景思想があるため、本アプリでは移動ドを用います。
移動ドについては、ここでは詳細に説明しませんが、例えばヘ長調でFを演奏したとき、その音はヘ長調の主音なので、移動ドでは「ド」と呼びます。
このように、移動ドでは調によってドレミファソラシドの位置が変わります。

本アプリでは、世の中の音楽ツールではあまり一般的でない移動ドによって音程指示を行います。
また、移動ドによる音程指示の理論的背景として、トニックソルファ法、コダーイシステムによるソルミゼーションを参考にしています。

Loopian記法による階名の表記は以下の通りです。
|階名|ド|レ|ミ|ファ|ソ|ラ|シ|
|-|-|-|-|-|-|-|-|
|表記|d|r|m|f|s|l|t|
|半音上の表記|di|ri|mi|fi|si|li|ti|
|半音下の表記|da|ra|ma|fa|sa|la|ta|

### 和音の表記について

通常、和音の表記はコードネームを使うのが一般的です。
例えば、Fmaj7と書くと、Fを根音とした長七和音を意味します。しかし、音程表記が移動ドで表記するのに和音を音名で表記することは矛盾しますし、アプリ内での記述の一貫性が取れなくなります。
従って、Loopianでは和音においても相対的な表記で音程を指定します。
例えば、冒頭のFmaj7をハ長調で演奏したい場合、IVmaj7と表記します。Fはハ長調の主音のCからみて４番目の音だからです。
このように、Loopian表記では、和音の根音の表記をローマ数字風にI,Vを使って表現します。ローマ数字はその調の主音からの距離を表します。

Loopian記法による和音の根音表記は、以下の通りです。
|主音からの距離|一度|二度|三度|四度|五度|六度|七度|
|-|-|-|-|-|-|-|-|
|根音の表記|I|II|III|IV|V|VI|VII|

### ピアノ音源向けの記法

Loopian記法はピアノ１台分の音楽を再生するように考慮されています。
Loopianでは、４つのパートを独立に記述することができますが、それらは、左手を2パート、右手を2パートで構成されます。また、すべての演奏情報のMIDIチャンネルは一つしか使用しません。
さらに、ペダルは音楽全体で一つしかコントロールしません。

## 基本的な音符の表記

具体的な表記について紹介します。
楽譜において各音符は最低でも、音価と音程を記述する必要があります。音価とは、音符の長さのことで、一般には四分音符、八分音符といった名称で指定されます。
Loopian記法では、音価、音程の順で音符をテキストで表現します。
例えば、四分音符のドの音符は `qd` です。`q` は四分音符を表します。
以下に音価の表記について紹介します。

|音価|二分音符|四分音符|八分音符|十六分音符|三十二分音符|
|-|-|-|-|-|-|
|表記|h|q|e|v|w|
|付点時の表記|h'|q'|e'|v'|未対応|
|三連符の表記|3h|3q|3e|3v|3w|

音価は全ての音符に書いても良いですが、前の音符と同じであれば表記は省略することができます。

一般の楽譜では音符を時間順に並べることで、旋律を表現することができます。loopian記法では、音符を `,` （カンマ）で区切って連続して記述することで、旋律を表現します。また小節線は `/` （スラッシュ）で区切ります。
また、一つの旋律は角括弧で括ります。

上記のルールを用いて、有名なベートーヴェンの第九のメロディを、４小節分Loopian記法で記述してみましょう。

```
[qm,m,f,s/s,f,m,r/d,d,r,m/q'm,er,hr]
```

第九の有名な旋律はDから始まりますが、ニ長調なので、ドから始まります。始めから３小節はすべて四分音符なので、最初の音符に `q` を書いて後の音符は省略できます。
4小節目の頭は、付点四分音符なので `'q` となり、次は八分音符、そして最後は二分音符となります。

さらに長い音価を指定したい場合、またもう少し微妙な長さの音価を指定したいとき、`.` （ピリオド）を使って特定の音符の長さを整数倍で表現することができます。
ピリオドは階名の後ろにつけます。２倍の時は一つ、３倍の時は二つ、と長さが増えるたびにピリオドの数を足していきます。

例えば、付点四分音符は八分音符が三つ分の長さです。上で４小節目の頭にあるミの付点四分音符は `eq..` と表現することが可能です。
ピリオドを使うと、先ほどの第九のメロディは以下のようにも記述できます。
```
[qm,m,f,s/s,f,m,r/d,d,r,m/em..,r,hr]
```

## 拍子、テンポ、調の設定

通常、楽譜の冒頭で拍子やテンポ、調号などが明示されます。
これらは音楽の基本的な要素であり、どのような記譜法でも必要なものです。
以下では、Loopian記法での拍子、テンポ、調号の指定方法について紹介します。

拍子は例えば4/4拍子の場合、次のように入力します。
`set.beat(4/4)`
beatの後の括弧の中は、通常の音楽で使われる拍子と全く同じ表記です。なお何も書かない場合、4/4拍子が設定されています
次に、テンポですが、テンポが120の場合、次のように入力します。
`set.bpm(120)`
テンポは何も書かないと100が初期値として設定されています。

次に調の指定について説明します。
その前に、Loopianでは一つ重要な考え方があります。Loopianには短調という概念がありません。もう少し丁寧な言い方をするのなら、短調と長調の区別がありません。
Loopianでは、調の概念とスケールの概念を分けて考えます。
具体的な例でお話しします。
例えば、ハ長調とイ短調の関係は、平行調と言われます。平行調とは、同じ調号で表記される長調と短調の関係のことを言います。
Loopianでは、この二つを別の調ではなく `C` という同じ調として扱います。そしてこの二つの違いは、スケールの違いと捉えます。
Loopianで、ハ長調あるいはイ短調を表す場合、次のように入力します。
`set.key(C)`
これを日本語で強引に書くなら「ハ調」ということになるかと思います。この `C` の部分は、通常の記法通り、D,E,F... というようにアルファベットによる音名で記述します。半音上がったり、下がったりする場合も同様に、音名の後に `F#` `Ab` というように #,b をつけることで表現します。

## 和音の入力

Loopian記法では和音の入力をサポートします。
これらの入力により、Loopian::APPがどのように音楽を加工していくかは、またアプリの仕様にて詳細に紹介します。
ここでは、Loopian記法における和音の書き方のルールについて記述します。
冒頭でも紹介したとおり、和音の根音にあたる音は、ローマ数字によって指示します。ローマ数字と言っても、実際にはテキストの `I` と `V` の組み合わせでローマ数字を表現します。
このローマ数字の後に、和音の種類を表記します。

### 和音の表記

Loopianでサポートする和音の種類とその表記を以下に挙げます。
また、その和音の構成音をLoopian記法のドレミで記述します。
なお、以下の表では根音を _ （アンダーバー）で表記しているので、実際に使用する際はローマ数字に置き換えてください。

|和音名|和音の表記|和音の構成音|
|-|-|-|
|長三和音|_|d,m,s|
|短三和音|_m|d,ma,s|
|属七＋長三和音|_7|d,m,s,ta|
|六の和音|_6|d,m,s,l|
|属七＋短三和音|_m7|d,ma,s,ta|
|長七和音|_M7|d,m,s,t|
|↑|_maj7|d,m,s,t|
|add9th|_add9|d,r,m,s|
|属九和音|_9|d,r,m,s,ta|
|属九＋短三和音|_m9|d,r,ma,s,ta|
|長九和音|_M9|d,r,m,s,t|
|↑|_maj9|d,r,m,s,t|
|増三和音|_+5|d,m,si|
|増三和音|_aug|d,m,si|
|増七和音|_7+5|d,m,si,ta|
|増三長七和音|_aug7|d,m,si,t|
|減九＋属七和音|_7-9|d,ra,m,s,ta|
|増九＋属七和音|_7+9|d,ri,m,s,ta|
|減七和音|_dim|d,ma,fi,l|
|導七和音|_m7-5|d,ma,ri,ta|
|suspended4th|_sus4|d,f,s|
|属七suspended4th|_7sus4|d,f,s,ta|

なお、特殊な表記として、`X` と `O` があります。
いずれも、その箇所には和音がないことを表します。`X` はダンパーペダルを踏みませんが、`O` はペダルを踏みます。


### 和音進行の表記

旋律においては、角括弧の中に各音符をカンマで区切りながら連続で表記しました。
和音進行では、波括弧を使います。小節区切りは、旋律と同様 `/` （スラッシュ）を使用し、前と同じ和音の場合は `.` （ピリオド）を使用します。
例えば、ブルースのコード進行は以下のように書くことができます。
`{I7/./././IV7/./I7/./V7/IV7/I7/V7}`
和音進行を表記する場合、小節線の中に和音が一つだけ書かれていれば、その小節内はその和音で演奏されます。

小節内で和音が変わる場合、拍単位で `,` （カンマ）で区切ることで、和音が変わることを表現します。
以下はその例です。
`{I.,VIm./IV.,V7.}`
上記の例では、１小節目は `I` の和音が二拍続き、その後 `VIm` の和音が二拍続くことを意味します。

## 再生時の基本的な動作

すでに紹介したようにLoopianはピアノ再生用に4つのパートを持っています。
ピアノ演奏を表現するためには、左手と右手の二つのパートだけでも良いのですが、より複雑な声部を持った音楽を演奏するために、左手だけで2パート、右手も2パート、それぞれ独立に表記可能となっています。
以下に各パートの表記、役割、音程を図示します。

|表記|パートの役割|音程の初期値|
|-|-|-|
|L1|左手の下のパート|中央より2オクターブ下|
|L2|左手の上のパート|中央より1オクターブ下|
|R1|右手の下のパート|中央のオクターブ|
|R2|右手の上のパート|中央より1オクターブ上|

例えばL1のパートに対して旋律を指定する場合、
`L1.[d,s,d,s]`
というように、パートの後、ピリオドで繋いで旋律を表記します。
同様にL1のパートに和音を指定する場合、
`L1.{I/IV/I/V}`
と表記します。

### ループ再生について

Loopianは、基本的にループ再生します。
`R1.[d,m,s,m/s,d,m,d]`
と入力された場合、上記の2小節分の旋律を繰り返し演奏します。

上の旋律を演奏しているときに、下の和音を入力してみます。
`R1.{I/I/IV/IV}`
`I`の和音が2小節、`IV`の和音が2小節、合計4小節が旋律と同様繰り返し再生されます。
このようにLoopianでは、旋律及び和音はそれぞれ独立の繰り返し周期で進行します。

### 和音による音程変換

もう一度、先ほどの旋律と和音を再生してみましょう。
`R1.[d,m,s,m/s,d,m,d]`
`R1.{I/I/IV/IV}`
最初の旋律では、`d` `m` `s`の３つの音があるので、和音が`I`のときは、音程はそのまま再生されますが、和音が`IV`のときには、`m`や`s`の音が`IV`の構成音のどれかの音程に自動的に変換されます。
このようにLoopianでは、和音進行を指定することによって、入力した旋律を和音に合った音で演奏することができます。
また、変換の方法や、その微調整も設定できるようになっています。


### リンク

- [Loopian Reference Manual](doc/manual.md)
- [Loopianで音符を書いてみよう(note記事)](https://note.com/hasebems/n/n0cd822840e51)

# `src/board/` における連珠ルールの実装

このドキュメントは盤面の表現と、[01-renju-rules.ja.md](01-renju-rules.ja.md) のルール用語 — 連、五、長連、四、棒四、三、四四、三三、禁手 — をどのように検出しているかを説明する。現在のコードに沿って書かれており、バッククォートで囲んだ識別子は `grep` で探せる。

モジュール構成（モジュール宣言は `src/board.rs`）:

| ファイル | 役割 |
| --- | --- |
| `player.rs` | `Player`（`Black` / `White`）とその文字表現（`o` / `x`）。 |
| `point.rs` | `Point` (x, y)、`Points`、`Direction`、`Index`（線上の位置）。 |
| `line.rs` | `Line`: 縦・横・斜めの 1 本を 2 つのビットマスクで表す。 |
| `sequence.rs` | `Sequences`: `Line` 上を窓をスライドさせて石のパターンを見つけるスキャナ。 |
| `structure.rs` | `StructureKind`（Two, Three, Sword, Four, Five, ...）と `Structure`（盤上に位置づけられたパターン）。 |
| `square.rs` | `Square`: 15×15 の盤全体を 4 方向の `Line` 配列として持ち、パターン検索を提供する。 |
| `forbidden.rs` | 黒の禁手判定。 |
| `potential.rs` | 手の順序付けに使う点ごとの「ポテンシャル」評価。 |
| `zobrist.rs` | 置換表用の Zobrist ハッシュ。 |
| `board.rs` | `Board` = `Square` + Zobrist ハッシュ。ソルバーが使う公開のファサード。 |

各部品は層になっており、以下の章はこの層を下から順にたどる:

1. `Line` は盤の 1 本の線をビットマスクで保持する（§2）。
2. `Sequences` は `Line` 上で 5 マスの窓をスライドさせ、自分の石が指定の数だけある窓を見つける（§3）。
3. `Square` はすべての線を持ち、「盤上に / この点を通ってどんなパターンがあるか」に答える（§4）。
4. `StructureKind` はそのパターンにルール用語の名前を付ける — `Five`、`Four`、`Three`、…（§5）。
5. `forbidden.rs` はいくつかの `structures_on` の問い合わせを組み合わせて禁手のルールを実装する（§6）。
6. `potential.rs` と `zobrist.rs` は同じ仕組みを手の順序付けとハッシュに再利用する（§7、§8）。

---

## 1. 点と座標

盤上の点は `Point(x, y)` で表す。

- `x` と `y` はどちらも `0` 以上 `RANGE`（= 15）未満の整数。
- `x` は列を表し、`A` 列が 0、`O` 列が 14。
- `y` は行を表し、`1` 行目が 0、`15` 行目が 14。

文字列との変換には連珠でよく使う `H8` のような表記を用いる（`Display` / `FromStr` の実装）。複数の点を表す `Points` は、`H8,H7,F6` のようにカンマで区切って書く。

### 線と `Index`

各点は縦・横・右上がり斜め・右下がり斜めの 4 本の線に属する。線は `Direction` と線の番号 `i` で識別し、線上の位置を `j` で表す:

| `Direction` | 線の種類 | 線の番号 `i` | 線上の位置 `j` |
| --- | --- | --- | --- |
| `Vertical` | 縦 (`\|`) | `x` | `y` |
| `Horizontal` | 横 (`-`) | `y` | `x` |
| `Ascending` | 右上がり (`/`) | `x + 14 - y`（0 から 28 まで） | `i < 14` なら `x`、それ以外は `y` |
| `Descending` | 右下がり (`\`) | `x + y`（0 から 28 まで） | `i < 14` なら `x`、それ以外は `14 - y` |

右上がりの斜めは左上の隅（`A15` が `i = 0`）から右下の隅（`O1` が `i = 28`）へ、右下がりの斜めは `A1`（`i = 0`）から `O15`（`i = 28`）へ番号が付く。どの斜めでも `j` は最も左の点から数えるので、`j` は `x` とともに増える。表の 2 つの式の違いは、その斜めが左端から始まるか（`i < 14`）、下端（右上がり）/ 上端（右下がり）から始まるかだけである。

この `(Direction, i, j)` の組が `Index` である。関連する操作:

- `Point::to_index(d)` は点を方向 `d` の `Index` に変換し、`Index::to_point()` は点に戻す。
- `Index::walk(step)` と `walk_checked(step)` は、同じ線に沿って `step` だけ移動する。
- `Index::maxj()` はその線の最後の有効な位置を返す。斜めの線は角に近づくほど短くなるため、線ごとに値が異なる。

## 2. `Line`: 1 本の線をビットマスクで

```rust
pub struct Line { blacks: u16, whites: u16, pub size: u8 }
```

- `blacks` と `whites` はそれぞれ黒石・白石の位置を表すビットマスクで、位置 `j` にその色の石があるときビット `j` が立つ。
- `size` は線の長さ。縦横の線は 15、斜めの線は 5 から 15 までのいずれか（短い斜めは省略される。§4 参照）。

石を置く `put_mut`、取り除く `remove_mut`、位置の石を調べる `stone(j)` はいずれも単純なビット演算であり、これが探索ループを軽く保っている。

`potential_cap(r)` は、線ごとの処理を読み飛ばすための簡易的な上界である。相手の石を避けて五を置く余地すらない線では 0 を、それ以外では「自分の石数 + 1」を返す。

## 3. `Sequences`: 線上のパターン検出

`Sequences`（`sequence.rs`）は、**5 マスの窓**を線上でスライドさせ、パターンに一致した窓を順に返すイテレータである。すべてのルール概念はこの 1 つの原始操作から組み立てられている。

各窓の開始位置 `i` について、各色のマスクから 7 ビットを取り出して見る:

```
bit  :   6   |  5    4    3    2    1  |   0
cell :  i+5  | i+4  i+3  i+2  i+1   i  |  i-1
     : right |          target         | left
     : margin|        (5 cells)        | margin
```

自分の石のマスク `my` と相手の石のマスク `op` はあらかじめ 1 ビット左にシフトしてあり、`i = 0` のときも `i-1` のビットが存在する。盤外にあたるマージンは空として読まれる。

窓が**有効**と見なされるのは、次の条件をすべて満たすときである:

- 対象 5 マスに相手の石がない（`op & TARGET_MASK == 0`）。
- `exact` モードのときは、さらに左右どちらのマージンにも*自分の*石がない（`my & MARGIN_MASK == 0`）。

`exact` はプレイヤーが**黒**のときにちょうど真になる（`StructureKind::to_sequence` 参照）。これは「黒の五はちょうど五でなければならない」というルールの実装である。窓に隣接して自分の石があれば、その窓に五を作ると長連になってしまうので、五になりうる窓として数えない。白にはこの制約がないため非 exact でスキャンし、長連も通常の勝ちとして扱われる。

有効な窓について、自分の石の数を `n` と比較する。何を返すかは 3 種類の `SequenceKind` で決まる:

| `SequenceKind` | 条件 | 意味 |
| --- | --- | --- |
| `Single` | 窓 `i` にちょうど `n` 個の自分の石がある | あと `5 - n` 石で五になる場所。 |
| `Double` | 窓 `i-1` **と** 窓 `i` の両方に `n` 個の自分の石がある | 重なり合う 2 つの五候補、すなわち 6 マスの範囲 `i-1..=i+4`。両端は「どちらも石」（6 マスに `n + 1` 石）か「どちらも空」（下の `Open`）のいずれか。 |
| `Open` | `Double` のうち両端 `i-1` と `i+4` がどちらも空のもの。`n` 個の石はすべて 4 マス `i..=i+3` の中にある | **両端が開いた**パターン。活三・活四の「活（open）」。 |

図で示すと次のようになる（黒の例。各マスには線上の位置を添えている）:

```
                i-1  i  i+1 i+2 i+3 i+4
Single  n = 4        o   o   o   .   o     窓 i が 4 石を持つ                    -> Four（隙間が眼）
Double  n = 4    o   o   .   o   o   o     窓 i-1 と窓 i の両方が 4 石を持つ      -> Overlining（6 マスに 5 石）
Open    n = 3    .   o   o   .   o   .     両端が空の Double                    -> Three
```

返り値は `(i, Sequence)` の組で、`Sequence` は対象 5 マスの状態を表す 5 ビットのマスクである。

- `Sequence::stones()` と `eyes()` はマスクをオフセット `0..5` に写す。立っているビットが石、空きが「眼（eye）」である。
- `Open` の場合、閉じ側の空点 `i+4` が眼として返らないよう、5 ビット目を強制的に立てている（`LAST_MASK`）。その副作用として `stones()` にはこのマスも含まれる。したがって `Open` な構造の `stones()` は「眼でないマス」と読むこと。

`Sequences::new_on(j, …)` は、位置 `j` を含む窓だけをスキャンする変種である。`structures_on(p, …)` が「この着手はどのパターンに関わるか」を調べるときに使う。

## 4. `Square`: 盤全体

```rust
pub struct Square {
    vlines: [Line; 15],  // 列。x で索引
    hlines: [Line; 15],  // 行。y で索引
    alines: [Line; 21],  // 長さ 5 以上の右上がり斜め
    dlines: [Line; 21],  // 長さ 5 以上の右下がり斜め
}
```

石は、その点が属する 4 本の線すべてに重複して格納される（`put_mut` が各線を更新する）。

斜めの線のうち、長さが 5 未満のものには五が入りえないので、各隅の最も短い 4 本ずつを省略している:

- 斜め `i`（`4` から `24` まで）は `alines[i - 4]` に格納される（`D_LINE_OMIT = 4`、`D_LINE_NUM = 21`）。
- それ以外の斜めについては `line_idx` が `None` を返し、パターン検索は単に読み飛ばす。

主な問い合わせ:

- `stone(p)`、`stones(player)`、`empties()`、`neighbors(p, distance, only_empty)` — 石や空点の取得。
- `structures(r, kind)` — 盤全体にあるプレイヤー `r` の種別 `kind` の `Structure` をすべて返す。
- `structures_on(p, r, kind)` — 5 マス窓が点 `p` を含む構造だけを返す（`p` を通る 4 本の線のみ調べる）。「`p` に打つと何ができるか」を調べるホットパス。
- `structures_between(d, i, from, to, r, kind)` — 線 `(d, i)` のうち、5 マス窓の *開始位置* が `from..=to` にある構造だけを返す。`Sequences` が 1 つの窓を判定するのに読むのは、その 5 マスと両隣のマージン 2 マスだけなので、マス `j` の石が変えうる窓は `j - 5 ..= j + 1` から始まるものに限られる。点ごとの表を自前で持つ利用側が、一手ごとにその表を更新するために使うクエリであり、`potentials` に対する `potentials_along` に当たる。`structures` と共通の `potential_cap` による事前フィルタは、その種別の構造を 1 つも持たない線しか落とさないので、`from..=to` の外の窓はそのままで正しい。
- `line(d, i)` / `line_on(p, d)` — 格納されている `Line` そのもの。短い斜めでは `None`。`lines()` と `lines_on(p)` は `(Direction, i, &Line)` として列挙する。点ごとの表を自前で持つ利用側が、72 本の線をもう 1 組持たずに済むようにするためのもの。
- `potentials(...)` / `potentials_along(...)` — §7 参照。

文字列からのパース（`FromStr for Square`、`Board` でも利用）は次の 3 形式を受け付ける:

- 手順: `H8,H7,F6` のように黒から交互に並べたもの。
- 石のリスト: `H8,F6/H7` のように `黒/白` で区切ったもの。
- 15 行の ASCII 図: `o` が黒、`x` が白、`.` が空点で、15 行目を先頭に書く。テスト全体で使われている形式。

## 5. `StructureKind`: ルール用語の語彙

`StructureKind::to_sequence(r)` は各種別を `(SequenceKind, n, exact)` の組に写す。`exact` は原則 `r.is_black()`（黒のみ真）であるが、黒の長連系の `StructureKind` を検出する際は常に偽となる。これは長連が必ず各 5 マス窓の隣に自分の石を持ち、exact ではまさにその窓が無効になって検出できないためである:

| `StructureKind` | `SequenceKind` | `n` | `exact` | パターン（黒の例、`_` = 眼） | ルール上の概念 |
| --- | --- | --- | --- | --- | --- |
| `Five` | `Single` | 5 | 黒のみ | `ooooo` | **五連**（§3）。黒は exact マージンにより長連を除外。 |
| `Overlined` | `Double` | 5 | 常に偽 | `oooooo`（6 以上） | **長連**。 |
| `Four` | `Single` | 4 | 黒のみ | `oooo_`、`ooo_o`、`oo_oo` など | **四**: 眼に 1 石で五連。棒四は隣り合う **2 つ**の `Four` として現れる。 |
| `Straight` | `Open` | 4 | 黒のみ | `.oooo.` | **棒四**。 |
| `Sword` | `Single` | 3 | 黒のみ | `ooo__`、`o_oo_` など（5 マス窓に 3 石） | **剣先**: どちらかの眼に打てば `Four`。活三も含むので英語の "closed three" とは異なる。ルール上の定義はないが、VCF/VCT が四を作る手を列挙するのに使う。 |
| `Three` | `Open` | 3 | 黒のみ | `.ooo_.`、`.oo_o.`、`.o_oo.`、`._ooo.` | **三**: 唯一の眼に打てば `Straight`。 |
| `Two` | `Open` | 2 | 黒のみ | `.oo__.`、`.o_o_.` など | **連**（二）: 眼に打てば `Three`。 |
| `Overlining` | `Double` | 4 | 常に偽 | `oo_ooo`、`ooo_oo` など | **六腐**: 眼に打つと長連（6 以上）。 |

黒には `exact` が適用されるため、`Four` や `Three` などの種別はすでに「同時に長連を作らない」という条件を織り込んでいる。例として `o.oooo.` という並びを考える:

- 窓 `o.ooo` と窓 `.oooo` は、マージンに黒石があるため却下される。左の隙間を埋めると六になってしまうからである。
- 窓 `oooo.` だけが数えられる。右端に打てばちょうど五連になるからである。

`Structure` は `(start: Index, sequence: Sequence)` の組で、`stones()` と `eyes()` は盤上の `Point` を返す。

## 6. 禁手（`forbidden.rs`）

禁手があるのは黒だけである。入口となる関数は次の 3 つ:

```rust
pub fn forbidden_strict(q: &Square, p: Point) -> Option<ForbiddenKind>
pub fn forbidden(q: &Square, p: Point) -> Option<ForbiddenKind>
pub fn forbiddens(q: &Square) -> Vec<(ForbiddenKind, Point)>
```

`ForbiddenKind` は `Overline`（長連）、`DoubleFour`（四四）、`DoubleThree`（三三）のいずれかである。

- `forbidden_strict` はまずルール 9.2 の例外を適用する。`p` にすでに石がある場合、または `p` に打つと五ができる場合（`structures_on(p, Black, Four)` が空でない）は禁手*ではない*と判定する。それ以外の場合は `forbidden` に委譲する。
- `forbidden` は長連、四四、三三の順に調べ、複数に該当する点は最初に見つかったものを報告する。
- `forbiddens` は盤上の禁手となる空点をすべて列挙する。

ソルバー（`src/mate/game.rs` の `Game::is_forbidden_move`）は非 strict の `forbidden` を呼ぶ。五を作る手は、禁手判定が問題になる前に探索自身が勝ちとして認識するためである。

### 長連（9.2 a）

```rust
fn overline(q, p) -> bool { q.structures_on(p, Black, Overlining).next().is_some() }
```

`Overlining`（六腐）は、隣り合う 2 つの 5 マス窓がそれぞれ黒 4 石を持ち、どちらも空点 `p` を含む形である。合わせて 6 マスに 5 石があるので、`p` に打てば 6 以上の連が完成する。

### 四四（9.2 b）

```rust
fn double_four(q, p) -> bool {
    distinctive(&mut q.structures_on(p, Black, Sword).map(|s| s.start_index()))
}
```

`p` を通る各 `Sword`（剣先）は、`p` に打つと `Four` になる。`distinctive` は、イテレータが返す索引の中に「`first` と `first.walk(1)`」以外の組み合わせが 2 つ以上あるときに真を返す。この除外が必要なのは次の理由による:

- 同一線上の隣り合う窓にある 2 つの `Sword` は、1 つの棒四の両半分（`.oo_o.` → `.oooo.`）である。四としては 1 つなので二重に数えない。
- 隣接しない 2 つの窓は本物の四四である。別の線上にある場合はもちろん、同一線上でも `o.o_o.o` のように 2 つの異なる五候補になる場合が該当する。

### 三三（9.2 c および 9.3）

```rust
fn double_three(q, p) -> bool {
    // 軽い前段フィルタ: p を通る連（Two）が 2 つ以上
    if !distinctive(q.structures_on(p, Black, Two)) { return false; }
    let mut next = q.clone();
    next.put_mut(Black, p);
    truthy_double_three(&next, p)
}

fn truthy_double_three(next, p) -> bool {
    let truthy_threes = next.structures_on(p, Black, Three).filter(|s| {
        let eye = s.eyes().next().unwrap();   // 達四点
        forbidden_strict(next, eye).is_none()
    });
    distinctive(&mut truthy_threes.map(|s| s.start_index()))
}
```

判定は次の手順で進む:

1. `p` を通る `Two`（連）が 2 つ以上あることは三三の必要条件なので、これを先に調べる。ほとんどの点はここで却下され、盤をクローンせずに済む。
2. コピーした盤に着手し、`p` を通る本当の `Three` を列挙する。`Three`（`Open`, 3）の眼はちょうど 1 つで、それが達四点である。
3. ルール 9.3 に従い、三はその眼が黒にとって合法な着手であるときだけ数える。新しい局面で眼に対して `forbidden_strict` を呼ぶことで判定する（詳細は後述）。
4. *異なる*本物の三が 2 つ以上残れば、その手は禁手の三三である。

手順 3 の判定について補足する:

- 眼に対する `forbidden_strict` の呼び出しは、9.3 a（達四の手が長連や四四になる）と 9.3 b（達四の手が禁手の三三になる）の両方をカバーする。
- `forbidden_strict → forbidden → double_three → truthy_double_three → forbidden_strict → …` という再帰が、ルールの言う「以下同様」の入れ子を処理する。
- `_strict` 版を呼ぶことが重要である。眼が別の線で同時に五を作る場合、その手は四四を形成していても 9.2 により合法（かつ勝ち）なので、その三は本物として数える（`test_double_three_eye_makes_five`）。
- 判定は再帰的なので、この五は候補手自身が作った四によるものでもよい。その場合は候補手自体の判定が反転する（`test_double_three_nested_eye_makes_five`）。
- なお RIF 9.3 a) の字面は「長連または四四ができない限り」であり、9.2 の五の例外を再掲していない。本実装はこれを「*禁手*にならない限り」と読んでいる。9.2 および日本連珠社規約の三の定義と整合する解釈である。

テスト（`test_double_three`）の例として、次の局面で `H8` に打つ場合を考える:

```
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . o . . . . . . .
 . . . . x . o . o . x . . . .
 . . . . . . . o . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
```

`H8` を通る `Two` があるのは縦方向だけである（H 列の `.o_o.`。重なり合う 2 つの `Open` 窓は隣接しているので `distinctive` は 1 つと数える）。横の `x.o_o.x` は `x` に挟まれているため `Open` の窓が収まらず、`Two` ではない — この形は達四になりえない。したがって手順 1 の前段フィルタが盤をクローンせずにこの手を却下し、結果は `None` になる。2 つの `x` を取り除くと両方向に `Two` があり、`H8` に打つと両方が達四点の合法な `Three` になるので、結果は `Some(DoubleThree)` になる。参照されている Twitter スレッドの入れ子の「偽の三」の局面を含む他のケースは、`forbidden.rs` のテストにある。

## 7. ポテンシャル（`potential.rs`）

ルールの一部ではないが、同じ窓スキャンの上に作られている。`Potentials` は線上の各空点について次のように点数を求める:

1. その空点を含む 5 つの 5 マス窓を見る。窓には `Sequences` と同じ `exact` マージン規則が適用される。
2. 有効な窓ごとに「自分の石数 + 1」（そこに打った後に窓が持つ石数）を点数とする。
3. 「最大点数 × 最大点数を達成した窓の数」をその空点の値として返す。

`Square::potentials` / `potentials_along` がこれを `Index` ごとに公開し、`src/analysis/field.rs` が点ごとに集約して手の順序付けに使う。`VICTORY = 5` は五になる窓の点数である。

## 8. Zobrist ハッシュ（`zobrist.rs`）と `Board`

`Board` は `Square` と `u64` の Zobrist ハッシュを包み、`put_mut` / `remove_mut` で両者の同期を保つ。

- `CODE_TABLE` は `2 * 225` 個のランダムな 64 ビット値を持つ。索引は `2 * u8::from(point) + c` で、`c` は黒なら 0、白なら 1 である。
- 石を置く・取り除くたびに、対応する値を XOR でハッシュに出し入れする。
- `zobrist_hash_n(n)` は深さごとの値（`N_TABLE`）をさらに XOR し、ソルバーが（局面, 残り深さ）の組で置換表を引けるようにする。
- `Board::put` / `remove` はコピーを返す。ソルバーは探索ループでのクローンを避けるため `_mut` 版を使う。

## 9. 早見表: ルール → コード

| ルール | コード |
| --- | --- |
| 五連で勝ち | `structures(r, Five)`（`mate::solve` / `Game` で判定）。 |
| 長連は白の勝ち、黒は不可 | `Five` は黒だけ exact なので白の六も `Five`。黒の長連は禁手（`Overlining` = 六腐）。`mate::solve::validate` は五や黒の `Overlined` を既に含む入力局面を拒否する。 |
| 四 / 棒四 | `Four`（`Single`, 4）/ `Straight`（`Open`, 4）。棒四 = 隣接する 2 つの `Four`。 |
| 三（達四できること） | `Three`（`Open`, 3）。唯一の眼 = 達四点。 |
| 黒の「長連を作らずに」 | `Sequences` の `exact` マージン。 |
| 禁手: 長連 / 四四 / 三三 | `forbidden.rs`: `overline` / `double_four` / `double_three`。 |
| 9.2「同時に五を作る場合を除く」 | `forbidden_strict`。 |
| 9.3 本物の三と偽の三、再帰 | `truthy_double_three` が各三の眼に `forbidden_strict` を呼ぶ。 |

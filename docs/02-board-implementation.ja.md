# `src/board/` における連珠ルールの実装

盤面の表現と、[01-renju-rules.ja.md](01-renju-rules.ja.md) のルール用語（連、五、長連、四、棒四、三、四四、三三、禁手）の検出方法を説明する。バッククォートで囲んだ識別子は `grep` で探せる。

用語: docs では「連」は常に 01 §3 の row（コードの `Row`）を指す。日本語では `Two` を「連」と呼ぶこともあるが、ここでは「二連」と呼んで区別する。

モジュール（宣言は `src/board.rs`）:

| ファイル | 役割 |
| --- | --- |
| `player.rs` | `Player`（`Black` / `White`）とその文字表現（`o` / `x`）。 |
| `point.rs` | `Point` (x, y)、`Points`、`Direction`、`Index`（線上の位置）。 |
| `segment.rs` | `Segment`: 線上の連続する 5 マス（五を作れる場所 1 つ）と、その両隣 1 マスずつ。 |
| `line.rs` | `Line`: 1 本の線を 2 つのビットマスクで持つ。その上のセグメントと、そこから求める連。 |
| `row.rs` | `RowKind`（Two, Three, Sword, Four, Five, ...）と `Row`（1 人のプレイヤーの連を盤上に位置づけたもの）。 |
| `grid.rs` | `Grid`: 15×15 の盤を 4 方向の `Line` 配列で持ち、連を検索する。 |
| `forbidden.rs` | 黒の禁手判定。 |
| `zobrist.rs` | 置換表用の Zobrist ハッシュ。 |
| `board.rs` | `Board` = `Grid` + Zobrist ハッシュ。ソルバーが使うファサード。 |

層を下から順に並べると次のとおりで、各章がそれぞれを説明する:

1. `Line` は 1 本の線をビットマスクで持つ（§2）。
2. `Segment` は五を作れる場所 1 つと、各プレイヤーがそこで五にどれだけ近いかを表す（§3）。
3. `Grid` はすべての線を持ち、盤上や点を通る連を探す（§4）。
4. `RowKind` は連にルール用語の名前を付ける — `Five`、`Four`、`Three`、…。どれも点数の条件を満たす 1 つか 2 つのセグメント（§5）。
5. `forbidden.rs` は `rows_on` の問い合わせをいくつか組み合わせて禁手を判定する（§6）。
6. `zobrist.rs` は盤面をハッシュする（§7）。

---

## 1. 点と座標

盤上の点は `Point(x, y)`:

- `x`、`y` は `0..SIZE`（`SIZE` = 15）。
- `x` は列: `A` が 0、`O` が 14。
- `y` は行: `1` が 0、`15` が 14。

文字列は `H8` のような通常の表記を使う（`Display` / `FromStr`）。`Points` はカンマ区切りで `H8,H7,F6` と書く。

### 線と `Index`

各点は 4 本の線（縦、横、右上がり斜め、右下がり斜め）に属する。線は `Direction` と線の番号 `i` で、線上の位置は `j` で表す。

| `Direction` | 線 | 線の番号 `i` | 位置 `j` |
| --- | --- | --- | --- |
| `Vertical` | 縦 (`\|`) | `x` | `y` |
| `Horizontal` | 横 (`-`) | `y` | `x` |
| `Ascending` | 右上がり (`/`) | `x + 14 - y`（0〜28） | `i < 14` なら `x`、それ以外は `y` |
| `Descending` | 右下がり (`\`) | `x + y`（0〜28） | `i < 14` なら `x`、それ以外は `14 - y` |

- 右上がりの斜めは左上（`A15`、`i = 0`）から右下（`O1`、`i = 28`）へ番号が付く。
- 右下がりの斜めは `A1`（`i = 0`）から `O15`（`i = 28`）へ番号が付く。
- どの斜めでも `j` は最も左の点から数えるので、`x` とともに増える。表の 2 つの式の違いは始まる端だけ: 左端（`i < 14`）か、下端（右上がり）/ 上端（右下がり）か。

組 `(Direction, i, j)` が `Index`:

- `Point::to_index(d)` と `Index::to_point()` で相互に変換する。
- `Index::offset(step)` と `checked_offset(step)` は線に沿って `step` マス動く。
- `Index::max_j()` は線の最後の位置。斜めは角に近いほど短いので、線ごとに違う。

## 2. `Line`: 1 本の線をビットマスクで

```rust
pub struct Line { blacks: u16, whites: u16, pub size: u8 }
```

- `blacks` / `whites`: 位置 `j` にその色の石があればビット `j` が立つ。
- `size`: 線の長さ。縦横は 15、斜めは 5〜15（短い斜めは省略。§4）。

- `put`、`remove`、`stone(j)` は単純なビット演算。これが探索ループを軽くしている。
- `score_bound(r)` は線ごと読み飛ばすための上界。相手の石の間に五が収まらなければ 0、それ以外は「自分の石数 + 1」。
- `segment(j)` は 5 マスがセル `j` から始まるセグメント（§3）を切り出す。`segments()` は `j` = 0 から `size - 5` まで全部を列挙する。`rows(r, kind)`（§3.1、§5）など、`Line` が答えるほかのことはすべてこの上に作られている。

## 3. `Segment`: 五を作れる場所 1 つ

`Segment`（`segment.rs`）は線上の連続する 5 マス（五を作れる場所 1 つ）と、そのすぐ前とすぐ後の 1 マス。すべてのルール概念はこれから組み立てる。

```rust
pub struct Segment { blacks: u8, whites: u8 }
```

各色の石は 7 ビットで、`Line::segment(j)` が取り出す:

```
bit  :   6   |  5    4    3    2    1  |   0
cell :  j+5  | j+4  j+3  j+2  j+1   j  |  j-1
     : margin|     the five cells      | margin
```

- 線の端より外のマスは空として読む。
- セグメントが位置を返すときは、5 マスを 0〜4 で数える。

セグメントは、どちらのプレイヤー `r` についても次に答える:

| メソッド | 答え |
| --- | --- |
| `is_free(r)` | 5 マスに相手の石がない。 |
| `is_alive(r)` | `r` がここでまだ五を作れる: `is_free(r)` で、黒なら両マージンにも黒石がない。 |
| `count(r)` | 5 マスのうち `r` の石の数（0〜5）。生死によらない。 |
| `score(r)` | `is_alive(r)` なら `count(r)`、そうでなければ `-1`。 |
| `stones(r)` | `r` の石があるマス（0〜4）。`stone_bits(r)` は同じものの 5 ビットマスク。 |
| `eyes(r)` | ここで五を作るために `r` があと埋めるマス（0〜4）。生きていなければ空。`eye_bits(r)` は同じもののマスク。 |

マージンは黒のルールのためにある:

- 黒の五は*ちょうど*五でなければならない。
- 5 マスのすぐ隣に黒石があると、埋めたときに長連になる。そのセグメントは黒にとって死んでいる。
- 白のマージンは関係ない。白は長連でも勝ち。

### 3.1 線上の連を、全セグメントについて一度に見つける

`Line::rows(r, kind)` は、`r` がその種別の連（§5）を持つセグメントを `(j, Segment)` で返す。

- `RowKind::matches` はセグメントごとの条件を述べる。
- ソルバーはほぼ毎ノードこれを問い合わせるので、線はビット演算で全セグメントを一度に調べる。`x >> k` のビット `j` はセル `j + k` なので、シフトしたマスクで書いた条件は全セグメントについて並列に成り立つ。

マスク:

- `count_mask(r, n)`: セグメント `j` が `is_free(r)` で `count(r) == n` ならビット `j` が立つ。5 マスの和は `tally` がビットスライスで求める（下記）。
- `score_mask(r, n)`: 同じく `score(r) == n` のもの。黒なら `j - 1` と `j + 5` にも黒石がない。
- `row_starts(r, kind)`: 連がセグメント `j` にあればビット `j` が立つ。
  - `rows` はその立っているビットをたどる（`Bits`。連 1 つにつき `trailing_zeros` 1 回）。
  - `Two`・`Three`・`Straight` では `open_starts(r, n)` を使う。任意の石数 `n` を取る。
- `row_eyes(r, kind)`: それらの連の眼になっている空きマスを、セルのマスクで返す。石を置くと連が一段進むところ。
  - `eyes_of(starts, cells)` が始点のビットを各セグメントの指定のセルに広げ、空いているものを残す。
  - 追い詰めの手の並べ替えが使う（`ShapeMap`、06 §8）。
- `row_starts` と `RowKind::matches`、`row_eyes` と `rows` の眼が一致することを、長さ 9 以下のすべての線と、ランダムな長さ 15 の線でテストしている。

1 マスを通る連:

- `rows_on(i, r, kind)` はセル `i` を通る連だけを残す。セグメントの 5 マスに `i` があり、2 つのセグメントからなる連では前のセグメントにもある。つまり共有する 4 マスに `i` がある。
- `row_starts_on(i, r, kind)` は同じものをマスクで返す。`rows_on(p, …)` が「この着手はどの連に関わるか」を調べるのに使う。

<details>
<summary><code>tally</code> が全セグメントの 5 マスを一度に足し合わせる仕組み</summary>

`count_mask` は `Line::tally(r)` から始まる。戻り値は `([ones, twos, fours], free)`。どのマスクもビット `j` がセグメント `j`（セル `j`〜`j + 4`）に対応する。

| マスク | ビット `j` |
| --- | --- |
| `ones` / `twos` / `fours` | セグメント `j` にある `r` の石数（0〜5）の 2 進数の 1・2・4 の位。石 3 つなら `0b011` で、`ones` と `twos` が立つ。 |
| `free` | セグメント `j` に相手の石がなく、線の中に収まっている。 |

```rust
fn tally(&self, r: Player) -> ([u16; 3], u16) {
    let (my, op) = self.my_op(r);
    let segments = (1u16 << (self.size + 1 - FIVE)) - 1;
    let blocked = op | op >> 1 | op >> 2 | op >> 3 | op >> 4;
    let (a, b, c, d, e) = (my, my >> 1, my >> 2, my >> 3, my >> 4);
    let (s1, c1) = (a ^ b ^ c, a & b | c & (a ^ b));
    let (s2, c2) = (d ^ e, d & e);
    let (ones, c3) = (s1 ^ s2, s1 & s2);
    let twos = c1 ^ c2 ^ c3;
    let fours = c1 & c2 | c3 & (c1 ^ c2);
    ([ones, twos, fours], !blocked & segments)
}
```

**5 マスを縦に並べる。**

- `my >> k` のビット `j` はセル `j + k`。ビット `j` では `a`〜`e` がちょうどセグメント `j` の 5 マスになる。
- その和がセグメントの石数。1 つの `u16` に最大 16 セグメントが入り、ビット演算 1 回で全セグメントの足し算が進む（ビットスライス加算）。

**5 個の 1 ビット値を足す。** 和は 0〜5 で 3 ビットに収まる。加算回路と同じ手順で求める:

1. 全加算器で `a + b + c`: `s1 = a ^ b ^ c` が 1 の位。`c1 = a & b | c & (a ^ b)`（3 つの多数決）が繰り上がりで、重み 2。
2. 半加算器で `d + e`: `s2 = d ^ e`（1 の位）、`c2 = d & e`（繰り上がり、重み 2）。
3. 1 の位どうし: `ones = s1 ^ s2` が最終的な 1 の位。`c3 = s1 & s2` がもう 1 つの重み 2 の繰り上がり。
4. 重み 2 の 3 つの繰り上がり: 奇偶 `twos = c1 ^ c2 ^ c3` が 2 の位。多数決 `fours = c1 & c2 | c3 & (c1 ^ c2)` が 4 の位へ上がる。

つまり和を「1 の位 + 2 ×（`c1` + `c2` + `c3`）」に分け、後半をもう一度足している。

**相手の石と線の端。**

- `blocked = op | op >> 1 | … | op >> 4`: セル `j`〜`j + 4` のどこかに相手の石があればビット `j` が立つ（同じ並べ方で、足し算の代わりに OR）。
- `segments = (1 << (size - 4)) - 1` は開始位置 0〜`size - 5` のセグメントだけを残す。それより右の窓は線からはみ出し、シフトで入る 0 が空きマスに見えてしまう。
- `!blocked & segments` が全セグメントの `Segment::is_free`。

**具体例。** 長さ 9 の線 `-oo-o-x--` を黒から見る。マスクはセル 0 を**左端**に書く（通常の 2 進表記と逆向き）:

```
cell        012345678
my          011010000   黒石の位置
a (=my)     011010000
b (>>1)     110100000
c (>>2)     101000000
d (>>3)     010000000
e (>>4)     100000000

s1          000110000   a+b+c の 1 の位
c1          111000000   a+b+c の繰り上がり
s2          110000000   d+e の 1 の位
c2          000000000   d+e の繰り上がり
c3          000000000   s1+s2 の繰り上がり

ones        110110000
twos        111000000
fours       000000000

blocked     001111100   セル 6 の x を含む窓
segments    111110000   開始位置 0〜4 の窓
free        110000000
```

| j | 5 マス | 石数 | fours twos ones | 値 | free |
| --- | --- | --- | --- | --- | --- |
| 0 | `-oo-o` | 3 | 0 1 1 | 3 | ✓ |
| 1 | `oo-o-` | 3 | 0 1 1 | 3 | ✓ |
| 2 | `o-o-x` | 2 | 0 1 0 | 2 | ✗（x） |
| 3 | `-o-x-` | 1 | 0 0 1 | 1 | ✗ |
| 4 | `o-x--` | 1 | 0 0 1 | 1 | ✗ |

**結果の使われ方。**

- `select(digits, n)` は石数がちょうど `n` のセグメントを取り出す。各桁のマスクを、`n` のその桁が 0 なら反転して、AND する。
- `count_mask(r, n)` は `select(digits, n) & free`。
- `score_mask(r, n)` は、黒ならさらにすぐ外側に黒石のあるセグメント（`overline`）を除く。

</details>

## 4. `Grid`: 盤全体

```rust
pub struct Grid {
    vlines: [Line; 15],  // 列。x で索引
    hlines: [Line; 15],  // 行。y で索引
    alines: [Line; 21],  // 長さ 5 以上の右上がり斜め
    dlines: [Line; 21],  // 長さ 5 以上の右下がり斜め
}
```

- 石は、その点を通る 4 本の線すべてに格納する（`put` が各線を更新する）。
- 長さ 5 未満の斜めには五が入らないので、各隅の短い 4 本ずつを省略している:
  - 斜め `i`（4〜24）は `alines[i - 4]` に格納する（`DIAGONAL_LINE_SKIP = 4`、`DIAGONAL_LINE_COUNT = 21`）。
  - それ以外では `line_idx` が `None` を返し、連の検索は読み飛ばす。

主な問い合わせ:

- `stone(p)`、`stones(player)`、`empty_points()`、`neighbors(p, distance, only_empty)`: 石や空点の取得。
- `rows(r, kind)`: 盤全体にある `r` の種別 `kind` の `Row` すべて。
- `rows_on(p, r, kind)`: `p` を通る連だけ（§3.1。`p` を通る 4 本の線だけ読む）。「`p` に打つと何ができるか」を調べるホットパス。
- `line(d, i)` / `line_on(p, d)`: 格納している `Line` そのもの。短い斜めでは `None`。`lines()` と `lines_on(p)` はそれを `(Direction, i, &Line)` で列挙する。点ごとの表を自前で持つ利用側は、72 本の線の複製を持たずにこれを読める。

文字列からのパース（`FromStr for Grid`、`Board` も利用）は 3 形式を受け付ける:

- 手順（黒から交互）: `H8,H7,F6`。
- 石のリスト（`黒/白`）: `H8,F6/H7`。
- 15 行の ASCII 図: `o` が黒、`x` が白、`.` が空点、15 行目が先頭。テストで使う形式。

## 5. `RowKind`: ルール用語の語彙

`RowKind` は、点数の条件を満たす 1 つのセグメント、または隣り合う 2 つのセグメント。

- 2 つのときはセグメント `j - 1` と `j` で、6 マス `j - 1..=j + 4` にわたる。
- `RowKind::matches(r, prev, cur)` は、セグメント `cur` とその 1 つ前の `prev` について条件を述べる。
- 2 つからなる連は後ろのセグメントの位置で返す。

| `RowKind` | セグメント | パターン（黒、`_` = 眼） | ルール上の概念 |
| --- | --- | --- | --- |
| `Five` | 点数 5 が 1 つ | `ooooo` | **五連**。黒はマージンが空のときだけ点数が付くので、長連は除かれる。 |
| `Overlined` | 2 つ。どちらも `is_free` で石 5 つ | `oooooo`（6 以上） | **長連**。 |
| `Four` | 点数 4 が 1 つ | `oooo_`、`ooo_o`、`oo_oo` など | **四**: 眼に 1 石で五連。棒四は隣り合う **2 つ**の `Four` として現れる。 |
| `Straight` | 点数 4 が 2 つ。石は共有する 4 マスにある | `.oooo.` | **棒四**。 |
| `Sword` | 点数 3 が 1 つ | `ooo__`、`o_oo_` など（セグメントに 3 石） | **剣先**: どちらかの眼で `Four`。活三も含むので、英語の "closed three" とは違う。ルール用語ではなく、VCF/VCT が四を作る手を列挙するのに使う。 |
| `Three` | 点数 3 が 2 つ。石は共有する 4 マスにある | `.ooo_.`、`.oo_o.`、`.o_oo.`、`._ooo.` | **三**: 唯一の眼で `Straight`。 |
| `Two` | 点数 2 が 2 つ。石は共有する 4 マスにある | `.oo__.`、`.o_o_.` など | **二連**: 眼で `Three`。 |
| `Overlining` | 2 つ。どちらも `is_free` で石 4 つ | `oo_ooo`、`ooo_oo` など | **六腐**: 眼に打つと長連（6 以上）。 |

表の補足:

- 活きた連（`Two`、`Three`、`Straight`）の「石は共有する 4 マスにある」とは、後ろのセグメントの最後のマスが空だということ。どちらのセグメントも生きているので、6 マスの両端が空になる。
- 長連系の連は `is_alive` ではなく `is_free` なセグメントを見る:
  - 長連では、各セグメントのすぐ隣に必ず黒石がある。これはまさに黒のセグメントを死なせる条件。
  - 黒石 4 つずつの `is_free` なセグメント 2 つは、6 マスに 5 石（空きに打つと六）か、活四 `.oooo.` のどちらか。
  - 空点を通るものとして見つかるのは前者だけ。活四の 2 つのセグメントが共有するのは石だけだから。

黒のセグメントはマージンが空のときだけ生きているので、`Four` や `Three` などは「同時に長連を作らない」条件をすでに含んでいる。例: `o.oooo.`

- `o.ooo` と `.oooo` は死んでいる。マージンに黒石があり、左の隙間を埋めると六になる。
- `oooo.` だけが数えられる。右端に打つとちょうど五連。

`Row` は、（後ろの）セグメントが始まる位置（`Index`）と、石と眼のマスクを持つ。

- `stones()` と `eyes()` は盤上の `Point` を返す。
- 活きた連で眼になるのは共有する 4 マスだけ。5 マス目は開いた端で、打つ点ではない。

## 6. 禁手（`forbidden.rs`）

禁手は黒だけ。入口は 3 つ:

```rust
pub fn forbidden_strict(g: &Grid, p: Point) -> Option<ForbiddenKind>
pub fn forbidden(g: &Grid, p: Point) -> Option<ForbiddenKind>
pub fn forbidden_points(g: &Grid) -> Vec<(ForbiddenKind, Point)>
```

`ForbiddenKind` は `Overline`（長連）、`DoubleFour`（四四）、`DoubleThree`（三三）のいずれか。

- `forbidden_strict` はまず 9.2 の例外を適用し、それから `forbidden` を呼ぶ。次の場合は禁手*ではない*:
  - `p` に石がある。
  - `p` に打つと五ができる（`row_starts_on(p, Black, Four)` に始点がある）。
- `forbidden` は `p` を通る各線を 1 度だけ読み、黒の `Overlining`・`Sword`・`Two` の始点をまとめて求める。長連、四四、三三の順に調べ、最初に該当したものを返す。
- `forbidden_points` は禁手になる空点をすべて列挙する。

ソルバー（`src/mate/game.rs` の `Game::is_forbidden_move`）は非 strict の `forbidden` を呼ぶ。五を作る手は、禁手判定が問題になる前に探索自身が勝ちとして扱うため。

### 長連（9.2 a）

```rust
fn overline(overlinings: [u16; 4]) -> bool { any(overlinings) }
```

3 つの判定への入力:

- `p` を通る線ごとに、マスク `Line::row_starts_on(i, Black, kind)` を受け取る。`rows_on(p, Black, kind)` の連の始点（§3.1）を、連を組み立てずに表したもの。
- `forbidden` は各線から 3 種類を 1 パスで読む（1 種類なら `Grid::row_starts_on`）。
- 「あるか」（`any`）、「2 つ以上あるか」（後述の `has_multiple_starts`）はビットの判定で済む。

`Overlining`（六腐）は、隣り合う 2 つのセグメントがそれぞれ黒 4 石を持ち、どちらも空点 `p` を含む形。6 マスに 5 石あるので、`p` に打てば 6 以上の連になる。

### 四四（9.2 b）

```rust
fn double_four(swords: [u16; 4]) -> bool {
    has_multiple_starts(swords)
}
```

`p` を通る `Sword`（剣先）は、`p` に打つとどれも `Four` になる。

- `has_multiple_rows` は、最初の索引 `first` とその隣 `first.offset(1)` 以外の索引が現れた時点で真を返す。隣り合う 2 つのセグメントを 1 つと数えている。
- 隣を除く理由:
  - 同一線上の隣り合うセグメントの 2 つの `Sword` は、1 つの棒四の両半分（`.oo_o.` → `.oooo.`）。四としては 1 つ。
  - 隣接しない 2 つは本物の四四。別の線上、または同一線上で `o.o_o.o` のように 2 つの五候補になる形。
- `has_multiple_starts` は同じことをマスクで調べる。始点のある線が 2 本目にもあるか、最初の線に最小の始点 `j` と `j + 1` 以外の始点があれば真（`s & !(0b11 << s.trailing_zeros()) != 0`）。
- `has_multiple_rows` 自体は、1 つずつ見る必要がある下の `Three` に使う。

### 三三（9.2 c および 9.3）

```rust
fn double_three(g, p, twos: [u16; 4]) -> bool {
    if !has_multiple_starts(twos) { return false; }   // 前段フィルタ: p を通る二連が 2 つ以上
    let mut next = g.clone();
    next.put(Black, p);
    real_double_three(&next, p)
}

fn real_double_three(next, p) -> bool {
    let real_threes = next.rows_on(p, Black, Three).filter(|s| {
        let eye = s.eyes().next().unwrap();   // 達四点
        forbidden_strict(next, eye).is_none()
    });
    has_multiple_rows(&mut real_threes.map(|s| s.start_index()))
}
```

手順:

1. `p` を通る `Two`（二連）が 2 つ以上あることは三三の必要条件なので、先に調べる。ほとんどの点はここで却下され、盤をクローンせずに済む。
2. 盤のコピーに `p` を打ち、`p` を通る `Three` を列挙する。`Three` の眼はちょうど 1 つで、それが達四点。
3. 9.3 に従い、三はその眼が黒の合法手のときだけ数える。新しい局面で眼に `forbidden_strict` を呼んで判定する。
4. *異なる*本物の三が 2 つ以上残れば、禁手の三三。

手順 3 の補足:

- 眼への `forbidden_strict` は、9.3 a（達四の手が長連や四四）と 9.3 b（達四の手が禁手の三三）の両方をカバーする。
- 再帰 `forbidden_strict → forbidden → double_three → real_double_three → forbidden_strict → …` が、ルールの「以下同様」の入れ子を処理する。
- `_strict` 版であることが重要。眼が別の線で同時に五を作るなら、四四でも 9.2 により合法（かつ勝ち）なので、その三は本物（`test_double_three_eye_makes_five`）。
- 再帰的なので、この五は候補手自身が作ったものでもよい。その場合は候補手の判定が反転する（`test_double_three_nested_eye_makes_five`）。
- RIF 9.3 a) の字面は「長連または四四ができない限り」で、9.2 の五の例外を再掲していない。本実装は「*禁手*にならない限り」と読む。9.2、および日本連珠社規約の三の定義と整合する解釈。

テスト（`test_double_three`）の例: 次の局面で `H8` に打つ。

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

- 縦: H 列の `.o_o.` が `H8` を通る `Two`。隣り合う 2 つのセグメントで見つかるので、`has_multiple_rows` は 1 つと数える。
- 横: `x.o_o.x` は両側を `x` に挟まれ、セグメントの組が収まらない。棒四にはなれないので `Two` ではない。
- よって手順 1 の前段フィルタが、クローンせずにこの手を却下する。結果は `None`。
- 2 つの `x` を除くと両方向に `Two` ができる。`H8` に打つと両方が、達四点に合法に打てる `Three` になり、結果は `Some(DoubleThree)`。

入れ子の「偽の三」の局面（テストのコメントにある Twitter スレッドのもの）など、ほかのケースは `forbidden.rs` のテストにある。

## 7. Zobrist ハッシュ（`zobrist.rs`）と `Board`

`Board` は `Grid` と `u64` の Zobrist ハッシュを包み、`put` / `remove` で両者を同期させる。

- `CODE_TABLE` は `2 * 225` 個のランダムな 64 ビット値。索引は `2 * u8::from(point) + c`（`c` は黒 0、白 1）。
- 石を置く・取り除くたびに、その値をハッシュに XOR で出し入れする。
- 手番、探索の攻め方、残り深さは盤面の性質ではないので、ハッシュに含めない。ソルバーが自分で XOR して置換表のキーにする（`apply_turn`、`apply_attacker`、`LIMIT_TABLE` を使う `apply_limit`。`State::key`、04）。
- `Board::put` / `remove` は盤面をその場で変更する。`with_stone` / `without_stone` は変更したコピーを返す。ソルバーは探索ループでのクローンを避けるため、こちらは使わない。

四追い探索は各プレイヤーの剣先（`Sword`、§5）をほぼ毎ノード問い合わせるので、`SwordMap`（`src/feature/sword.rs`）にキャッシュする。

- 持つのは `Board` ではなく探索の状態（追い詰めの `ShapeMap` と同じ）: `VCFState` と、内部の四追いに渡すための `VCTState`。`State::after_play` / `after_undo` で印を付け、同期や読み出しのときに盤面を渡す。
- 中身は、プレイヤーごと・線ごと（`Grid::line_key` = `Grid::lines` での線の位置）に、剣先のセグメントがセル `j` から始まればビット `j` を立てた `u16` と、剣先のある線を表す `u128`。
- 着手は、その点を通る高々 4 本の線に「古い」印を付けるだけ（`SwordMap::mark_stale`。印は `ShapeMap` と共用の `StaleLines`、`src/feature/stale.rs`）。古い線は `SwordMap::sync` が計算し直す。
  - 探索は読むより手を打つ・戻すほうがずっと多い。着手のたびに計算し直すと、置き換えたはずの全走査より高くつく。
- 線の計算は `Line::row_starts(r, Sword)` が全セグメントを一度に行う（§3.1）。
- `SwordMap::swords(board, r)` / `swords_on(board, p, r)` は、`rows(r, Sword)` / `rows_on(p, r, Sword)` と同じものを同じ順に返す。先に `sync(board)` が必要。

## 8. 早見表: ルール → コード

| ルール | コード |
| --- | --- |
| 五連で勝ち | `rows(r, Five)`（`mate::solve` / `Game` で判定）。 |
| 長連は白の勝ち、黒は不可 | 黒のセグメントだけがマージンを見るので、白の六も `Five`。黒の長連は禁手（`Overlining` = 六腐）。`mate::solve::trivial_result` は、五や黒の `Overlined` をすでに含む入力局面に探索せず答える。 |
| 四 / 棒四 | `Four`（点数 4 のセグメント）/ `Straight`（点数 4 のセグメント 2 つ）。棒四 = 隣接する 2 つの `Four`。 |
| 三（達四できること） | `Three`（点数 3 のセグメント 2 つ）。唯一の眼が達四点。 |
| 黒の「長連を作らずに」 | `Segment::is_alive`: マージンに黒石がない。 |
| 禁手: 長連 / 四四 / 三三 | `forbidden.rs`: `overline` / `double_four` / `double_three`。 |
| 9.2「同時に五を作る場合を除く」 | `forbidden_strict`。 |
| 9.3 本物の三と偽の三、再帰 | `real_double_three` が各三の眼に `forbidden_strict` を呼ぶ。 |

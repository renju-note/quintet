# `src/board/` における連珠ルールの実装

このドキュメントは盤面の表現と、[01-renju-rules.ja.md](01-renju-rules.ja.md) のルール用語 — 連、五、長連、四、棒四、三、四四、三三、禁手 — をどのように検出しているかを説明します。現在のコードに沿って書かれており、バッククォートで囲んだ識別子は `grep` で探せます。

モジュール構成（`src/board/mod.rs`）:

| ファイル | 役割 |
| --- | --- |
| `player.rs` | `Player`（`Black` / `White`）とその文字表現（`o` / `x`）。 |
| `point.rs` | `Point` (x, y)、`Points`、`Direction`、`Index`（線上の位置）、wasm 向けの `u8` エンコード。 |
| `line.rs` | `Line`: 縦・横・斜めの 1 本を 2 つのビットマスクで表す。 |
| `sequence.rs` | `Sequences`: `Line` 上を窓をスライドさせて石のパターンを見つけるスキャナ。 |
| `structure.rs` | `StructureKind`（Two, Three, Sword, Four, Five, ...）と `Structure`（盤上に位置づけられたパターン）。 |
| `square.rs` | `Square`: 15×15 の盤全体を 4 方向の `Line` 配列として持ち、パターン検索を提供する。 |
| `forbidden.rs` | 黒の禁手判定。 |
| `potential.rs` | 手の順序付けに使う点ごとの「ポテンシャル」評価。 |
| `zobrist.rs` | 置換表用の Zobrist ハッシュ。 |
| `board.rs` | `Board` = `Square` + Zobrist ハッシュ。ソルバーが使う公開のファサード。 |

---

## 1. 点と座標

`Point(x, y)`、`0 <= x, y < RANGE (15)`。`x` は列（`A`=0 … `O`=14）、`y` は行（`1`=0 … `15`=14）。`Display`/`FromStr` は `H8` 表記、`Points` はカンマ区切り（`H8,H7,F6`）。

wasm/JS 境界では点を 1 バイトにエンコードする: `code = x * 15 + y`（`From<Point> for u8`、`TryFrom<u8> for Point`）。これは公開 API であり変更してはならない。

### 線と `Index`

各点は `Direction` ごとに 1 本ずつ、計 4 本の線に属する:

| `Direction` | 線の番号 `i` | 線上の位置 `j` |
| --- | --- | --- |
| `Vertical` | `x` | `y` |
| `Horizontal` | `y` | `x` |
| `Ascending`（`/`） | `x + 14 - y`（0..=28） | `i < 14` なら `x`、それ以外は `y` |
| `Descending`（`\`） | `x + y`（0..=28） | `i < 14` なら `x`、それ以外は `14 - y` |

`Point::to_index(d) -> Index { d, i, j }` と `Index::to_point()` で相互変換する。`Index::walk(step)` / `walk_checked(step)` は線に沿って移動し、`Index::maxj()` はその線の最後の有効位置を返す（斜めは角に向かって短くなる）。

## 2. `Line`: 1 本の線をビットマスクで

```rust
pub struct Line { blacks: u16, whites: u16, pub size: u8 }
```

`blacks` / `whites` のビット `j` は、位置 `j` にその色の石があるとき立つ。`size` は線の長さ（縦横は 15、保持する斜めは 5..=15。§4 参照）。`put_mut` / `remove_mut` / `stone(j)` は単純なビット演算で、探索ループを軽くしている。

`potential_cap(r)` は線を読み飛ばすための簡易上界: 相手の石を避けて五を置く余地すらなければ 0、それ以外は `自分の石数 + 1` を返す。

## 3. `Sequences`: 線上のパターン検出

`Sequences`（`sequence.rs`）は **5 マスの窓**を線上でスライドさせ、パターンに一致する窓を返すイテレータである。すべてのルール概念はこの 1 つの原始操作から組み立てられている。

各窓の開始位置 `i` について、各色のマスクから 7 ビットを見る:

```
ビット: 6      5 4 3 2 1     0
マス:   i+5    i+4 … i       i-1
        ^ 右マージン  ^ 対象（5 マス）  ^ 左マージン
```

（`my`/`op` は 1 ビット左シフトしてあり、`i = 0` でも `i-1` が存在する。盤外のマージンは空として読まれる。）

窓が**有効**なのは次のとき:

- 対象 5 マスに相手の石がない（`op & TARGET_MASK == 0`）、かつ
- `strict` のとき、どちらのマージンにも*自分の*石がない（`my & MARGIN_MASK == 0`）。

`strict` はプレイヤーが**黒**のときにちょうど真になる（`StructureKind::to_sequence` 参照）。これは「黒の五はちょうど五でなければならない」というルールの実装で、窓に隣接して自分の石があればその五は長連になってしまうため、五になりうる窓として数えない。白にはこの制約がないので非 strict でスキャンし、長連も通常の勝ちとして扱われる。

有効な窓の中で自分の石の数を `n` と比較する。何を返すかは 3 種類の `SequenceKind` で決まる:

| `SequenceKind` | 条件 | 意味 |
| --- | --- | --- |
| `Single` | 窓 `i` にちょうど `n` 個の自分の石 | あと `5 - n` 石で五になる場所。 |
| `Double` | 窓 `i-1` **と** `i` の両方に `n` 個の自分の石 | 重なった 2 つの五候補、つまり 6 マスに `n + 1` 石。 |
| `Compact` | `n` 個の石がすべて 4 マス `i..=i+3` にあり、`i-1` と `i+4` が空（窓 `i-1` と窓 `i` が両方有効で `n` 石） | **両端が開いた**パターン。 |

返り値は `(i, Sequence)` で、`Sequence` は対象マスの 5 ビットマスク。`Sequence::stones()` / `eyes()` はそれをオフセット `0..5` に写し、立っているビットが石、空きが眼（eye）。`Compact` では閉じ側の空点 `i+4` が眼として返らないよう 5 ビット目を強制的に立てる（`LAST_MASK`）。その副作用として `stones()` にはこのマスも含まれるので、`Compact` な構造の `stones()` は「眼でないマス」と読むこと。

`Sequences::new_on(j, …)` は位置 `j` を含む窓だけをスキャンする。`structures_on(p, …)` が「この着手はどのパターンに触れるか」を尋ねるときに使う。

## 4. `Square`: 盤全体

```rust
pub struct Square {
    vlines: [Line; 15],  // 列。x で索引
    hlines: [Line; 15],  // 行。y で索引
    alines: [Line; 21],  // 長さ 5 以上の右上がり斜め
    dlines: [Line; 21],  // 長さ 5 以上の右下がり斜め
}
```

石は属する 4 本の線すべてに重複して格納される（`put_mut` が各線を更新）。長さ 5 未満の斜めには五が入りえないので、各隅の最も短い 4 本は省略する: 斜め `i`（`4..=24`）は `alines[i - 4]` に格納され（`D_LINE_OMIT = 4`、`D_LINE_NUM = 21`）、それ以外は `line_idx` が `None` を返しパターン検索は単に読み飛ばす。

問い合わせ:

- `stone(p)`、`stones(player)`、`empties()`、`neighbors(p, distance, only_empty)`。
- `structures(r, kind)` — 盤全体にあるプレイヤー `r` の `kind` の `Structure` すべて。
- `structures_on(p, r, kind)` — 5 マス窓が点 `p` を含む構造だけ（`p` を通る 4 本の線）。「`p` に打つと何ができるか」を調べるホットパス。
- `potentials(...)` / `potentials_along(...)` — §7 参照。

パース（`FromStr for Square`、`Board` でも利用）は 3 形式を受け付ける: 手順 `H8,H7,F6`（黒から交互）、石のリスト `H8,F6/H7`（`黒/白`）、テスト全体で使われている 15 行の ASCII 図（`o` 黒、`x` 白、`.` 空点、15 行目が先頭）。

## 5. `StructureKind`: ルール用語の語彙

`StructureKind::to_sequence(r)` は各種別を `(SequenceKind, n, strict)` に写す。`strict = r.is_black()`:

| `StructureKind` | Sequence | パターン（黒の例、`_` = 眼） | ルール上の概念 |
| --- | --- | --- | --- |
| `Five` | `Single`, 5 | `ooooo` | **五**（§3）。黒は strict マージンにより長連を除外。 |
| `OverFive` | `Double`, 5, 常に非 strict | `oooooo`（6 以上） | **長連**。 |
| `Four` | `Single`, 4 | `oooo_`、`ooo_o`、`oo_oo` など | **四**: 眼に 1 石で五。棒四は隣り合う **2 つ**の `Four` として現れる。 |
| `OpenFour` | `Compact`, 4 | `.oooo.` | **棒四**。 |
| `Sword` | `Single`, 3 | `ooo__`、`o_oo_` など（5 マス窓に 3 石） | 「四になる手前」: どちらかの眼に打てば `Four`。ルール用語ではなく、VCF/VCT が四を作る手を列挙するのに使う。 |
| `Three` | `Compact`, 3 | `.ooo_.`、`.oo_o.`、`.o_oo.`、`._ooo.` | **三**: 唯一の眼に打てば `OpenFour`。 |
| `Two` | `Compact`, 2 | `.oo__.`、`.o_o_.` など | 「三になる手前」: 眼に打てば `Three`。 |
| `NextOverFive` | `Double`, 4, 常に非 strict | `oo_ooo`、`ooo_oo` など | 眼に打つと長連（6 以上）。 |

黒に `strict` が適用されるため、`Four`/`Three` などはすでに「同時に長連を作らない」を織り込んでいる。例: `o.oooo.` では窓 `o.ooo` と `.oooo` は（マージンに黒石があるため）却下され、`oooo.` だけが数えられる。左の隙間を埋めると六になり、右端に打てばちょうど五になるからである。

`Structure` は `(start: Index, sequence: Sequence)` で、`stones()` と `eyes()` は盤上の `Point` を返す。

## 6. 禁手（`forbidden.rs`）

禁手があるのは黒だけ。入口:

```rust
pub fn forbidden_strict(q: &Square, p: Point) -> Option<ForbiddenKind>
pub fn forbidden(q: &Square, p: Point) -> Option<ForbiddenKind>
pub fn forbiddens(q: &Square) -> Vec<(ForbiddenKind, Point)>
```

`ForbiddenKind` は `Overline | DoubleFour | DoubleThree`。`forbidden_strict` はまず 9.2 の例外を適用する — `p` に石がある、または `p` に打つと五ができる（`structures_on(p, Black, Four)` が空でない）なら禁手*ではない* — そのうえで `forbidden` に委譲する。ソルバー（`src/mate/game.rs` の `Game::is_forbidden_move`）は非 strict の `forbidden` を呼ぶ。五を作る手は禁手判定が問題になる前に探索自身が勝ちとして認識するためである。`forbiddens` は盤上の禁手となる空点をすべて列挙する。

`forbidden` は長連、四四、三三の順に調べる（複数に該当する点は最初のものが報告される）。

### 長連（9.2 a）

```rust
fn overline(q, p) -> bool { q.structures_on(p, Black, NextOverFive).next().is_some() }
```

`NextOverFive` = 隣り合う 2 つの 5 マス窓がそれぞれ黒 4 石を持ち、どちらも空点 `p` を含む。合わせて 6 マスに 5 石なので、`p` に打てば 6 以上の連が完成する。

### 四四（9.2 b）

```rust
fn double_four(q, p) -> bool {
    distinctive(&mut q.structures_on(p, Black, Sword).map(|s| s.start_index()))
}
```

`p` を通る各 `Sword` は、`p` に打つと `Four` になる。`distinctive` は、イテレータが `first` と `first.walk(1)` 以外の組み合わせで 2 つ以上の索引を返すときに真を返す。同一線上の隣り合う窓にある 2 つの `Sword` は 1 つの棒四の両半分（`.oo_o.` → `.oooo.`）であり四は 1 つなので二重に数えない。隣接しない 2 つの窓 — 別の線上、あるいは同一線上でも `o.o_o.o` → 2 つの異なる五候補 — は本物の四四である。

### 三三（9.2 c および 9.3）

```rust
fn double_three(q, p) -> bool {
    // 軽い前段フィルタ: p を通る「三になる手前」のパターンが 2 つ以上
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

1. `p` を通る `Two` 構造は必要条件なので、ほとんどの点は盤をクローンせずに却下される。
2. コピーした盤に着手し、`p` を通る本当の `Three` を列挙する。`Three`（`Compact`, 3）の眼はちょうど 1 つ — 達四点である。
3. ルール 9.3: 三は、その眼が黒にとって合法な着手であるときだけ数える。これは新しい局面で眼に対して `forbidden_strict` を呼ぶことで判定され、9.3 a（達四の手が長連や四四になる）と 9.3 b（禁手の三三になる）の両方をカバーする。`forbidden_strict → forbidden → double_three → truthy_double_three → forbidden_strict …` の再帰が、ルールの言う「以下同様」の入れ子を処理する。`_strict` 版であることが重要で、眼が別の線で同時に五を作る場合、その手は四四を形成していても 9.2 により合法（かつ勝ち）なので、その三は本物として数える（`test_double_three_eye_makes_five`）。判定は再帰的なので、この五は候補手自身が作った四によるものでもよく、その場合は候補手自体の判定が反転する（`test_double_three_nested_eye_makes_five`）。なお RIF 9.3 a) の字面は「長連または四四ができない限り」で、9.2 の五の例外を再掲していない。本実装はこれを「*禁手*にならない限り」と読んでおり、9.2 および日本連珠社規約の三の定義と整合する解釈である。
4. *異なる*本物の三が 2 つ以上残れば、その手は禁手の三三である。

テスト（`test_double_three`）の例: 次の局面の `H8`

```
. . . . x . o . o . x . . . .   <- 8 行目
```

（上下に石あり）では、横の `x.o_o.x` は `x` に挟まれて達四できないため、`H8` を通る本物の三は 1 つだけで、この手は合法である。2 方向とも黒だけの `.o_o.` なら `Some(DoubleThree)` になる。参照されている Twitter スレッドの入れ子の「偽の三」の局面を含む他のケースは `forbidden.rs` のテストにある。

## 7. ポテンシャル（`potential.rs`）

ルールの一部ではないが、同じ窓スキャンの上に作られている。線上の各空点について `Potentials` はそれを含む 5 つの 5 マス窓を見て、有効な窓ごとに `自分の石数 + 1`（そこに打った後に窓が持つ石数）を点数とし、`最大点数 × (最大を達成した窓の数)` を返す。窓には Sequences と同じ `strict` マージン規則が適用される。`Square::potentials` / `potentials_along` がこれを `Index` ごとに公開し、`src/analysis/field.rs` が点ごとに集約して手の順序付けに使う。`VICTORY = 5` は五になる窓の点数。

## 8. Zobrist ハッシュ（`zobrist.rs`）と `Board`

`Board` は `Square` と `u64` の Zobrist ハッシュを包み、`put_mut`/`remove_mut` で同期を保つ。`CODE_TABLE` は `2 * 225` 個のランダムな 64 ビット値を持ち、`2 * u8::from(point) + (黒なら 0 | 白なら 1)` で索引して、変更のたびに XOR で出し入れする。`zobrist_hash_n(n)` は深さごとの値（`N_TABLE`）をさらに XOR し、ソルバーが（局面, 残り深さ）で置換表を引けるようにする。`Board::put`/`remove` はコピーを返し、ソルバーは探索ループでのクローンを避けるため `_mut` 版を使う。

## 9. 早見表: ルール → コード

| ルール | コード |
| --- | --- |
| 五で勝ち | `structures(r, Five)`（`mate::solve` / `Game` で判定）。 |
| 長連は白の勝ち、黒は不可 | `Five` は黒だけ strict なので白の六も `Five`。黒の長連は禁手（`NextOverFive`）。`mate::solve::validate` は五や黒の `OverFive` を既に含む入力局面を拒否する。 |
| 四 / 棒四 | `Four`（`Single`, 4）/ `OpenFour`（`Compact`, 4）。棒四 = 隣接する 2 つの `Four`。 |
| 三（達四できること） | `Three`（`Compact`, 3）。唯一の眼 = 達四点。 |
| 黒の「長連を作らずに」 | `Sequences` の `strict` マージン。 |
| 禁手: 長連 / 四四 / 三三 | `forbidden.rs`: `overline` / `double_four` / `double_three`。 |
| 9.2「同時に五を作る場合を除く」 | `forbidden_strict`。 |
| 9.3 本物の三と偽の三、再帰 | `truthy_double_three` が各三の眼に `forbidden_strict` を呼ぶ。 |

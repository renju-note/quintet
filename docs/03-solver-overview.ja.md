# ソルバーの全体像: `src/mate/` と四追い探索

このドキュメントは詰みソルバーの入口である。次のことを説明する。

- 公開関数 `solve` とその引数
- すべてのソルバーが共有する探索状態
- 四追い（VCF、Victory by Continuous Fours）ソルバー

これらの上に作られている追い詰め（VCT、Victory by Continuous Threats）ソルバーは [04-solver-algorithm-vct.ja.md](04-solver-algorithm-vct.ja.md) で説明する。

[02-board-implementation.ja.md](02-board-implementation.ja.md) の盤面まわりの用語、特に `Four`、`Sword`、`Three`、`eyes()`、禁手、`zobrist_hash_n` を前提とする。現在のコードに沿って書いており、バッククォートで囲んだ識別子は `grep` で突き合わせられる。

コード中の英語と本文の日本語の対応:

| コード | 本文 |
| --- | --- |
| VCF | 四追い |
| VCT | 追い詰め |
| `attacker` / `defender` | 攻め方 / 受け方 |
| `Mate` / `End` / `path` | 詰み / 詰め上がり / 詰み手順（単に手順とも） |
| threat | 追い手（受け方がパスすれば四追いで詰む手） |
| `eyes()` | 眼（三や四を完成させる空点） |

モジュール構成:

| ファイル | 役割 |
| --- | --- |
| `mate/solve.rs` | 公開エントリポイント `solve` と `SolveMode`、入力の検証、ソルバーの回帰テスト。 |
| `mate/game.rs` | `Game`: 盤面 + 手順 + 手番（パス対応）、`check_event`（四の検出）、`End`（詰め上がり）。 |
| `mate/state.rs` | `State` トレイト: 残り `limit` も管理する play/undo と、置換表のキー。 |
| `mate/mate.rs` | `Mate`: 詰みの結果（詰め上がり `End` + 詰み手順 `path`）。 |
| `mate/vcf/` | 四追いソルバー: `VCFState`（四を作る手のペア）、`DFSSolver`、`IDDFSSolver`。 |
| `mate/vct/` | 追い詰めソルバー（`DFSVCTSolver`、`PNSVCTSolver`、`DFPNSVCTSolver`）。04 を参照。 |
| `analysis/field.rs` | `PotentialField`。追い詰めの手の並べ替え。04 の §8 を参照。 |

---

## 1. エントリポイント（`solve.rs`）

```rust
pub fn solve(mode: SolveMode, limit: u8, board: &Board, attacker: Player, threat_limit: u8) -> Option<Mate>
```

`attacker` は手番側であり、勝ちを証明したい側である。相手側はコード全体で *defender* と呼ばれる。

### `SolveMode`

| `SolveMode` | CLI での名前 | ソルバー | 備考 |
| --- | --- | --- | --- |
| `VCFDFS` | `vcf` | `vcf::DFSSolver` | `threat_limit` は無視される。 |
| `VCFIDDFS` | `vcf_iddfs` | — | 予約のみ。現在 `solve` は `None` を返す。 |
| `VCTDFS` | `vct` | `DFSVCTSolver` | 下の 2 つと同じ AND/OR 木を深さ優先でたどる。 |
| `VCTIDDFS` | `vct_iddfs` | — | 予約のみ。現在 `solve` は `None` を返す。 |
| `VCTPNS` | `vct_pns` | `PNSVCTSolver` | 最良優先の証明数探索。 |
| `VCTDFPNS` | `vct_dfpns` | `DFPNSVCTSolver` | df-pn（深さ優先証明数探索）。実質的なデフォルト。 |

### `limit` と `threat_limit`

- `limit` は解に含まれる **攻め方の着手数** の上限である。四も含めて攻め方の着手はすべて数える。たとえば 7 手（攻め 4 手 + 受け 3 手）の解を見つけるには `limit >= 4` が必要である（`test_vct_black` を参照）。
- `threat_limit` は、追い詰めソルバーが内部で走らせる四追い探索の深さの上限である（04 の §2）。`VCFDFS` には影響しない。

### `validate`

探索の前に、`validate` がソルバーの扱えない局面を弾く:

| 局面 | 結果 |
| --- | --- |
| どちらかの `Five` がすでに盤上にある | `None` |
| 黒の `Overlined` がすでにある | `None` |
| 攻め方がすでに `Four` を持っている | `Some(Mate { end: Unknown, path: [] })` — 証明するまでもなく勝ちとして扱う。 |

### `Mate` と `End`

```rust
pub struct Mate { pub end: End, pub path: Vec<Point> }
pub enum End { Fours(Point, Point), Forbidden(Point), Unknown }
```

`path` は詰み手順である。攻め方の手から始まり、攻め方と受け方の手が交互に並ぶ。

`end` は詰め上がりである。`path` の最後の手の後で受け方がなぜ負けているかを表す:

- `Fours(p1, p2)`: 攻め方の四の勝ち点が `p1` と `p2` の 2 つあり、1 手では止められない。白なら四四か達四である。黒は四四が禁手なので（攻め手を打つ前に必ず `is_forbidden_move` を確認する）、実際には達四である。`Square` は達四を、眼の異なる隣接した 2 つの `Four` として報告する。
- `Forbidden(p)`: 攻め方の四が 1 つあり、その唯一の止め点 `p` が受け方の禁手である。受け方が黒のときだけ起こる。
- `Unknown`: 勝ちは証明できたが、詰み手順を復元できなかった（上の `validate` と 04 の §6 の復元処理を参照）。

`Mate::n_moves()` は詰み手順の長さ、`n_times()` はそのうち攻め方の着手数である。

## 2. 共通の探索状態（`game.rs`、`state.rs`、`mate.rs`）

### `Game`

`Game` が持つもの:

- `Board`
- 探索中に打った手のリスト（`Vec<Option<Point>>`。`None` はパス）
- 手番 `turn`

`play` / `undo` は盤面をその場で書き換える。`into_play(m, f)` は `m` を打ち、`f` を実行し、手を戻して `f` の結果を返す。すべてのソルバーはこのパターンで、盤面を複製せずに木をたどる。

パス（`play(None)`）は「自分が何もしなかったら *相手* は何ができるか」を問うために使う。追い詰めで追い手を検出する仕組みがこれである（04 の §2）。

### `check_event`: 盤上の四

`Game::check_event()` は、直前に打った側（`turn` の相手）の四を見て、手番側の状況を分類する:

| 相手の四の勝ち点 | `Event` | 手番側にとっての意味 |
| --- | --- | --- |
| 異なる 2 点 | `Defeated(Fours(p1, p2))` | 負け。両方は止められない。 |
| 1 点 `p` で、`p` が手番側の禁手 | `Defeated(Forbidden(p))` | 負け。唯一の止め点が打てない。 |
| 1 点 `p` | `Forced(p)` | `p` を打つしかない。 |
| なし | `None` | 自由に選べる。 |

調べるのは最後の手を通る四だけである（`structures_on(last_move, ..., Four)`）。それより前の四は、すでに止めを強制しているはずだからだ。最後の手がパスのときはその点がないので、代わりに相手の四をすべて走査する。

2 点判定（`take_distinct_two`）は、眼の異なる 2 つの `Four` からなる達四を、四四と同じ扱いにするよう意図されている。

### `State`: limit の管理と置換表のキー

`State`（`VCFState`、`VCTState` が実装）は、`Game` に `attacker` と残り `limit` を加えたものである:

- `play(m)` は `m` を打つ。その結果として攻め方の手番に戻った（つまり受け方が打った）場合は、`limit` を 1 減らす。`undo` はその両方を戻す。
- したがって `limit` は「攻め方があと何手打てるか」である。受け方のノードでは、直前の攻め手の分がまだ含まれている。
- フック `after_play` / `after_undo` により、`VCTState` はポテンシャル場を更新する。
- `attacking()` は `turn == attacker` である。
- `zobrist_hash()` は `Board::zobrist_hash_n(limit)`、すなわち局面と残り深さの組である。ソルバー内のメモ表はすべてこの値をキーにしている。同じ局面でも残り予算が違えば別のエントリになる。
- パスは盤面を変えない。したがってパスした局面は、パス前と同じ盤面（`limit` はそのときの値）としてハッシュされる。

## 3. 四追い（`vcf/`）

四追いは、攻め方の各手が四を作る（受け方の応手が強制される）手順で、最後の手が `Fours` または `Forbidden` の詰め上がりになるものである。ソルバーは失敗メモ付きの素朴な深さ優先探索である。

### 手の生成: `Sword` の眼

`VCFState` は手番側の `Sword` 構造から攻め手を生成する。`Sword` は 5 マス窓に石 3 つと空点（*眼*）2 つがある形である。片方の眼に打つと `Four` ができ、残る勝ち点はもう片方の眼になる。したがって 1 つの剣先から、`sword_eyes_pairs` で `(attack, defence)` のペアが 2 つ得られる:

```
H8 H9 H10 . .   (column H, H7 occupied by White)  -> Sword, eyes H11, H12
pairs: (H11, H12)  play H11: four H8-H11, White must answer H12
       (H12, H11)  play H12: four H8,H9,H10,_,H12, White must answer H11
```

生成器は 3 つあり、ソルバーはこの順に試す:

- `forced_move_pair(p)`: 攻め方が `Forced(p)` のとき、攻め手がちょうど `p` であるペア。`Forced(p)` になるのは、受け方の止めが受け方自身の四を作った（ノリ手）ときである。その止めが四でなければ四追いは続かない。
- `neighbor_move_pairs()`: `last2_move`（攻め方が直前に打った石）を通る剣先から作るペア。同じ石をつないで四を続けるのが最も自然なので、最初に試す。
- `move_pairs()`: 盤上のすべての剣先から作るペア。

### `DFSSolver`

```
solve(state):
    if state.limit == 0: return None
    if deadends contains state.zobrist_hash(): return None
    result = solve_move_pairs(state)
    if result is None: deadends.insert(hash)
    return result

solve_move_pairs(state):                        # attacker to move
    match state.check_event():
        Defeated(_)  -> None                    # defender has a double four
        Forced(p)    -> forced_move_pair(p) then solve_attack
        None         -> try neighbor pairs, then all other pairs; first Some wins

solve_attack(state, attack, defence):
    if attack is forbidden: return None
    play attack; result = solve_defence(state, defence); undo
    prepend attack to the path

solve_defence(state, defence):                 # defender to move
    if check_event() is Defeated(end): return Mate { end, path: [] }
    play defence; result = solve(state); undo   # limit decrements here
    prepend defence to the path
```

補足:

- `deadends: HashSet<u64>` は四追いのない局面（`limit` 込み）を記憶する。合流した局面や、追い詰めからの繰り返し呼び出しで同じ計算をしないためである。成功は記憶せず、そのまま返す。
- 受け点は、攻め手を打った後の盤面から求め直さない。攻め手がたまたま四を 2 つ作った場合は、`defence` を打つ前に受け方側の `check_event` が `Defeated(Fours(..))` を返す。したがって、保持しているもう片方の眼が意味を持つのは四が 1 つのときだけである。
- ノリ手は攻め方側の `Forced` で扱う。受け方の止めの後、攻め方も止めを強いられることがある。その止めは四でなければならず（`forced_move_pair`）、そうでなければ手順は失敗する。`test_vcf_counter` と `test_vcf_not_opponent_double_four` がこれを検証している。

### `IDDFSSolver`

`IDDFSSolver::init(limits)` は同じ `DFSSolver` を、`limits` のうち状態自身の limit より小さいものについて順に走らせ、最後に本来の limit で走らせる。最初に見つかった解を返す。メモは `(盤面, limit)` をキーにしているので、浅いパスが深いパスを汚すことはない。

追い詰めソルバーは `IDDFSSolver::init(vec![1])`（「まず 1 手で勝てるか調べる」）を使う。

### 例

`test_vcf_counter` の盤面、黒番:

```
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . x . . . . . . .
 . . . . . . . o . o o x . . .
 . . . . . . . . o x . o . . .
 . . . . . . . . o . x . . . .
 . . . . . . . . x . . x . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
```

`solve(VCFDFS, 3, &board, Black, 0)` は手順 `I8,G8,I10,I9,J9`、詰め上がり `Fours(H11, M6)` を返す:

1. `I8` で四 `H8,I8,J8,K8` ができる（剣先 `H8,J8,K8`、眼は `G8` と `I8`）。白は `G8` に止めるしかない。
2. `I10` で四 `I6,I7,I8,_,I10` ができる。白は `I9` に止めるしかない。
3. `J9` で `I10,J9,K8,L7` ができる。`H11` と `M6` の両方が空いている達四である。2 つの `Four` として報告されるので、白に対する `check_event` は `Defeated(Fours(H11, M6))` になる。

## 4. チートシート

| 知りたいこと | 見る場所 |
| --- | --- |
| 解法モードを追加する | `SolveMode` + `FromStr` + `solve` の `match`（`solve.rs`）。 |
| なぜ深さ N で探索が止まったか | `limit` は攻め方の着手数を数え、受け方が打つたびに `State::play` で減る。 |
| 四を作る手が生成されない | `VCFState::move_pairs` は `Sword` の眼しか見ない。黒では `exact` の縁の条件により、長連になる四は除かれる。 |
| ノリ手の扱いがおかしい | 攻め方側の `Game::check_event` と `VCFState::forced_move_pair`。 |
| 置換表のメモ | `DFSSolver::deadends`。`zobrist_hash_n(limit)` がキーで、失敗だけを記憶する。 |
| 回帰テストの追加 | `solve.rs` のテストに ASCII 盤面と期待する詰み手順の文字列を追加し、関係する `SolveMode` ごとに 1 つずつ assert する。 |
| 追い詰め固有の疑問 | [04-solver-algorithm-vct.ja.md](04-solver-algorithm-vct.ja.md) のチートシートを参照。 |

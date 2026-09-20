# `src/mate/` と `src/analysis/` はどうやって詰みを探索しているか

このドキュメントは詰みソルバーの解説である。次のことを説明する。

- 局面から四追い（VCF、Victory by Continuous Fours）や追い詰め（VCT、Victory by Continuous Threats）をどう探索するか
- `limit` と `threat_limit` が何を意味するか
- `src/mate/vct/` の証明数探索がどう組み立てられているか

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
| `mate/vct/` | トレイトから組み立てる追い詰めソルバー: `Generator`、`Searcher`、`Selector`、`Traverser`、`Resolver`、`ProofTree`、`VCFHelper`。具体型は `DFSVCTSolver`、`PNSVCTSolver`、`DFPNSVCTSolver` の 3 つ。 |
| `mate/vct_lazy/` | 実験的な「遅延」追い詰めソルバー（`LazyVCTSolver`）。`vct/` と同じ構成。 |
| `analysis/field.rs` | `PotentialField`: 追い詰めの手の並べ替えに使う、差分更新される各点のポテンシャル。 |

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
| `VCTLAZY` | `vct_lazy` | `LazyVCTSolver` | 実験的（§5）。`threat_limit` は無視される。 |

### `limit` と `threat_limit`

- `limit` は解に含まれる **攻め方の着手数** の上限である。四も含めて攻め方の着手はすべて数える。たとえば 7 手（攻め 4 手 + 受け 3 手）の解を見つけるには `limit >= 4` が必要である（`test_vct_black` を参照）。
- `threat_limit` は、追い詰めソルバーが内部で走らせる四追い探索の深さの上限である（§4.2）。`VCFDFS` には影響しない。

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
- `Unknown`: 勝ちは証明できたが、詰み手順を復元できなかった（上の `validate` と §4.6 の `Resolver` を参照）。

`Mate::n_moves()` は詰み手順の長さ、`n_times()` はそのうち攻め方の着手数である。

## 2. 共通の探索状態（`game.rs`、`state.rs`、`mate.rs`）

### `Game`

`Game` が持つもの:

- `Board`
- 探索中に打った手のリスト（`Vec<Option<Point>>`。`None` はパス）
- 手番 `turn`
- `passed` フラグ

`play` / `undo` は盤面をその場で書き換える。`into_play(m, f)` は `m` を打ち、`f` を実行し、手を戻して `f` の結果を返す。すべてのソルバーはこのパターンで、盤面を複製せずに木をたどる。

パス（`play(None)`）は「自分が何もしなかったら *相手* は何ができるか」を問うために使う。追い詰めで追い手を検出する仕組みがこれである（§4.2）。

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

`State`（`VCFState`、`VCTState`、`LazyVCTState` が実装）は、`Game` に `attacker` と残り `limit` を加えたものである:

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

追い詰めソルバーは `IDDFSSolver::init(vec![1])`（「まず 1 手で勝てるか調べる」）を使う。遅延ソルバーは `(1..u8::MAX)`、つまり完全な反復深化を使う。

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

## 4. 追い詰め（`vct/`）

### 4.1 何を追い手とみなすか

このソルバーでの追い詰めは、攻め方の各手が *追い手* である手順である。追い手とは、受け方がパスしたら攻め方に高々 `threat_limit` 手の四追いがある手のことだ。

- 四は自明なケースである（受け方が `Forced` になる）。
- 三は 1 手の四追い（達四点）を持つ追い手である。
- `threat_limit >= 2` なら、四三を準備する手のような「フクミ手」も追い手になる（`test_vct_fukumi_move` を参照。`threat_limit = 3` が必要）。

受け方は、追い手が狙う四追いを止める手であれば何でも打てる。ノリ手や、受け方自身の四追いによる逆襲も含む。

探索は AND/OR 木である。攻め方のノードは OR（良い攻め手が 1 つあればよい）、受け方のノードは AND（すべての受けが負けでなければならない）である。証明数で解き、3 つの `SolveMode` は木のたどり方だけが異なる。

### 4.2 `VCTState`

`VCTState` = `Game` + `attacker` + `limit` + 攻め方の `PotentialField`（§6）である。ポテンシャル場は `PotentialField::init(attacker, 2, board)` で初期化し、`after_play` / `after_undo` で各手の 4 本の線に沿って更新する。

内部の四追い探索には、派生させた 2 種類の `VCFState` を使う（`VCFHelper`）:

| メソッド | 局面 | 四追いの攻め方 | 四追いの limit | 問い |
| --- | --- | --- | --- | --- |
| `vcf_state(max)` | そのまま | 手番側 | `min(limit, max)` | 手番側は今すぐ四追いで勝てるか？ |
| `threat_state(max)` | 手番側がパスした後 | 相手側 | 攻め方の手番なら `min(limit - 1, max)`、そうでなければ `min(limit, max)` | 自分が何もしなければ相手に四追いがあるか？ つまり直前の手は追い手か？ |

`VCFHelper` は、手番と問いの 4 通りの組み合わせに名前を付けている:

| メソッド | 手番 | 問い |
| --- | --- | --- |
| `solve_attacker_vcf` | 攻め方 | 攻め方は今すぐ四追いで勝てるか？ |
| `solve_defender_threat` | 攻め方 | 攻め方がパスしたら、受け方に四追いがあるか？ |
| `solve_attacker_threat` | 受け方 | 受け方がパスしたら、攻め方に四追いがあるか？（直前の攻め手は追い手か？） |
| `solve_defender_vcf` | 受け方 | 受け方は今すぐ四追いで勝てるか？ |

`max` 引数は、攻め方の探索では `attacker_vcf_depth`（= `threat_limit`）、受け方の探索では `defender_vcf_depth`（`solve` で `2` に固定）である。裏で動くのは各側 1 つずつの `IDDFSSolver`（`limits = [1]`）で、その `deadends` メモは追い詰め探索全体を通して保持される。

`next_zobrist_hash(m)` は、ポテンシャル場（更新が高コストな部分）に触れずに、`m` の後の子のキーを計算する。これにより未展開の子の表引きが安価になる。

### 4.3 手の生成（`generator.rs`）

どちらの生成器も `Result<Vec<Point>, Node>` を返す。内部ノードなら `Ok(候補)`、その場で決着がつくなら `Err(node)` である（`Node::zero_pn` = 攻め方の勝ちが証明済み、`Node::zero_dn` = 反証済み）。結果は `zobrist_hash()` をキーに、1000 エントリの `LruCache` 2 つにメモ化される。

`compute_attacks`（攻め方の手番）:

1. `solve_attacker_vcf` が四追いを見つけたら `Err(zero_pn)` を返す。正しさのためには不要だが（本探索でも `Forced` の応手を 1 つずつたどれば四追いは見つかる）、大幅に速くなる。
2. `solve_defender_threat` が受け方の四追いを見つけたら（攻め方が何もしなければ受け方が勝つ）、候補を `threat_defences(threat)`（後述）に絞る。攻め手はその狙いも同時に受けなければならない。
3. 候補は、攻め方のポテンシャル場でポテンシャル `>= 3` の点（`sorted_potentials(3, ..)`）を高い順に並べ、禁手を除いたものである。空なら `Err(zero_dn)`。

ここでは候補が追い手かどうかを判定していないことに注意されたい。判定は 1 手先の受け方のノードで行い、追い手でない手は `compute_defences` のステップ 1 で反証される。

`compute_defences`（受け方の手番）:

1. `solve_attacker_threat`: 受け方がパスしても攻め方に四追いがなければ、直前の攻め手は追い手ではなかった → `Err(zero_dn)`。
2. `solve_defender_vcf`: 受け方自身に（`defender_vcf_depth` 手以内の）四追いがあれば、受け方が先に勝つ → `Err(zero_dn)`。
3. 候補は、`threat_defences(threat)` を攻め方のポテンシャルで並べ替え（`sort_by_potential`）、禁手を除いたものである。空なら `Err(zero_pn)`、つまりその攻め手は受けられない。

`threat_defences(threat)` は、追い手が狙う四追いを止めうる手のヒューリスティックな集合で、次の順に並ぶ:

- 狙われている四追いの手順に含まれるすべての点。攻め方の四も、受け方の強制された止めも含む。どれかを先に占めれば手順が崩れる。
- `end_breakers(end)`: `Fours(p1, p2)` なら 2 つの勝ち点。`Forbidden(p)` ならその点と、4 本の線に沿って 5 マス以内の空点（`neighbors(p, 5, true)`）。近くの石で `p` が禁手かどうかが変わりうるためである。
- `counter_defences(threat)`: 狙われている四追いの手順を再生し、受け方が強制された各止めについて、その止めを通る受け方の `Sword` の眼を集める。手順の途中で受け方に四が生じる点であり、今打てばノリ手になりうる。
- `four_moves()`: 受け方が今すぐ打てる、四を作る手（`Sword` の眼）すべて。攻め方に応手を強いるノリ手である。

このリストには同じ点が 2 回入ることがある。後の `dedup` は、ポテンシャルで並べ替えた後の隣接する重複しか取り除かない。したがって重複が残りうるが、無害である（同じ子を 2 回引くだけだ）。

### 4.4 証明数（`proof.rs`）

```rust
pub struct Node { pub pn: u32, pub dn: u32, pub limit: u8 }
pub const INF: u32 = u32::MAX;
```

`pn` は証明数（攻め方の勝ちを証明するのに、あと何個の葉を証明する必要があるかの見積もり）、`dn` は反証数である。`pn == 0` は証明済み、`dn == 0` は反証済みを意味する:

| コンストラクタ | `(pn, dn)` | 意味 |
| --- | --- | --- |
| `Node::inf()` | `(INF, INF)` | 情報なし。根の閾値「決着するまで探索する」にも使う。 |
| `Node::zero_pn(limit)` | `(0, INF)` | 証明済み（攻め方の勝ち）。 |
| `Node::zero_dn(limit)` | `(INF, 0)` | 反証済み。 |
| `Node::unit_dn(n, limit)` | `(n, 1)` | 兄弟が `n` 個ある、未展開の攻め方の子の初期見積もり。 |
| `Node::unit_pn(n, limit)` | `(1, n)` | 兄弟が `n` 個ある、未展開の受け方の子の初期見積もり。 |

子の合成は `min_pn_sum_dn`（OR ノード: pn = min、dn = 和）と `min_dn_sum_pn`（AND ノード: pn = 和、dn = min）で行う。和は飽和加算である。

`limit` は子の最小値として一緒に運ばれ、部分木が決着した時点で予算がどれだけ残っていたかを記録する。リゾルバはこれを使って、最も粘り強い受けを選ぶ（§4.6）。

`ProofTree` は 2 つの置換表（`Table`、中身は `HashMap<u64, Node>`）へのアクセスを提供する:

- `attacker_table`: 攻め手で到達した局面（受け方の手番）の値
- `defender_table`: 受け手で到達した局面（攻め方の手番）の値

`Table::lookup_next(state, m)` は `next_zobrist_hash` で `m` の後の子を引く。

### 4.5 探索（`searcher.rs`、`selector.rs`、`traverser.rs`）

`Searcher::search` は根が証明済みかどうかを返す。`limit == 0` を確認した後、`search_attacks(state, Node::inf()).proven()` を返す。相互再帰する 2 つのノード関数は次のとおりである:

```
search_attacks(state, threshold):              # OR node, attacker to move
    Defeated(_)  -> zero_dn
    Forced(p)    -> traverse_attacks(state, [p], threshold, search_defences)
    otherwise    -> generate_attacks -> Err(node) => node
                                     | Ok(attacks) => traverse_attacks(...)

search_defences(state, threshold):             # AND node, defender to move
    Defeated(_)  -> zero_pn                     # the attacker has won
    limit <= 1   -> zero_dn                     # the attacker has no move left after this defence
    Forced(p)    -> traverse_defences(state, [p], threshold, search_attacks)
    otherwise    -> generate_defences -> Err(node) => node
                                      | Ok(defences) => traverse_defences(...)
```

`Selector` は何も展開せずに、子の表エントリからノードを評価する。`select_attack` は `Selection` を返す:

- `current`: ノード自身の `(pn, dn)`。子に対する `min_pn_sum_dn` で求める。表にない子は `unit_dn(attacks.len())` とみなす。
- `best`: `pn` が最小の子（最も証明に近い子）。
- `next1` / `next2`: 最良の子と 2 番目の子の値。

証明済みの子が見つかれば、`current` は直ちに `(0, INF)` になる。`select_defence` はその鏡像である（`dn` 最小、`min_dn_sum_pn`、`unit_pn(defences.len())`、`current.limit = limit - 1`）。

未展開の子を兄弟数で初期化する「トリック」により、候補手の少ないノードほど簡単に見える。そのため探索は、狭く強制的な手順を優先する。

`Traverser` は 3 つのソルバーに共通の展開ループである:

```
traverse_attacks(state, attacks, threshold, search_defences):
    loop:
        selection = select_attack(state, attacks)
        if selection.current.pn >= threshold.pn or selection.current.dn >= threshold.dn:
            return selection                   # backoff
        next = next_threshold_attack(selection, threshold)
        play selection.best
            attacker_table.insert(child, search_defences(child, next))
        undo
```

`traverse_defences` は、受け方の表と `next_threshold_defence` を使う以外は同じである。ノードは、その数値が親から渡された閾値を超えるまで展開される。根の閾値は `Node::inf()` なので、根は `pn == 0`（証明済み。このとき `dn` は `INF` にされる）か `pn == INF`（反証済み）になるまでループする。

ソルバー間の唯一の違いは `next_threshold_*` である:

| トレイト | 子の閾値 | 振る舞い |
| --- | --- | --- |
| `DFSTraverser` | `Node::inf()` | 選んだ子を完全に探索してから、親が次の子を見る。証明数は手の並べ替えにだけ使う、通常の深さ優先探索。 |
| `PNSTraverser` | `(next1.pn + 1, next1.dn + 1)` | 子は数値が変わり次第戻る。制御が上に戻り、各階層で最も証明に近い子が選び直される。再帰的な探索の中で、展開のたびに根から選び直す最良優先 PNS を模倣する。 |
| `DFPNSTraverser` | OR ノード: `pn = min(threshold.pn, next2.pn + 1)`、`dn = threshold.dn - current.dn + next1.dn`。AND ノードはその鏡像 | Nagai & Imai (2002) の df-pn の閾値。最良の子が最良である間はそこに留まり、親の予算を超えない。 |

`DFSVCTSolver`、`PNSVCTSolver`、`DFPNSVCTSolver`（`solver/*.rs`）は、それ以外は同一の構造体である。持っているのは `Table` 2 つ、四追い用の `IDDFSSolver` 2 つ、四追いの深さ 2 つ、生成器のキャッシュ 2 つで、振る舞いはすべてトレイトのデフォルトメソッドから来る。`VCTSolver::solve` は次のとおりである:

```rust
if self.search(state) { self.resolve(state) } else { None }
```

### 4.6 リゾルバ（`resolver.rs`）

探索は勝ちがあることを証明するだけである。`Resolver` は表をもう一度たどって手順を作る。

`resolve_attacks`（攻め方の手番）:

- `Forced` なら、それに従う。
- そうでなければ `state.empties()` を走査し、`attacker_table` のエントリが証明済みである最初の手を打つ。
- 証明済みの手がなければ（`compute_attacks` の四追いショートカットで証明されたノード）、`solve_attacker_vcf` の四追いを手順の末尾として返す。

`resolve_defences`（受け方の手番）:

- `Defeated(end)` なら、その `end` を詰め上がりとして手順を終える。
- `Forced` なら、それに従う。
- そうでなければ、攻め方の追い手に対する `threat_defences` を再計算し、証明済みの子のうち `Node::limit` が最小のものを選ぶ。これは攻め方に最も多くの手を使わせた受けである。したがって報告される手順は、最も粘り強い受けに対するものになる。
- 証明済みの候補がなければ（合法な受けが存在しないために証明されたノード）、手順は `End::Unknown` で終わる。

### 例

`test_vct_black` の盤面（黒番。`solve.rs` のコメントによれば岡部寛氏の五手詰め問題 No. 02）:

```
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . x . . . . . .
 . . . . . . . o . . . . . . .
 . . . . . . . o x o . . . . .
 . . . . . . x o . x . . . . .
 . . . . . . . x o . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
```

`solve(VCTDFS, 4, &board, Black, 1)`（`VCTPNS`、`VCTDFPNS` も同じ）は `F10,G9,I10,G10,H11,H12,G12`、詰め上がり `Fours(F13, K8)` を返す。`limit = 3` では失敗する。

- `F10` で三 `F10,_,H8,I7` ができる。これは追い手である（パスすれば `G9` が達四になる）。白の `threat_defences` には `G9` が含まれ、白はそれを打つ。
- `I10` で三 `F10,_,H10,I10` ができ、白は `G10` に止める。
- `H11` で四 `H8..H11` ができ、`H12` が `Forced` になる。
- `G12` で `G12,H11,I10,J9` ができ、`F13` と `K8` が空いている: `Fours`。

根での攻め方の候補は、ポテンシャル `>= 3` の点を高い順に並べたものである（`I10`、`G10`、`G9`、`F10`、…。オーバーレイは §6 にある）。したがって深さ優先ソルバーは `F10` より先に `I10` を試す。`I10` は即座に反証される。三ができないので、白がパスしても黒に 1 手の四追いはない。`compute_defences` が `zero_dn` を返し、その反証が `attacker_table` に保存される。

## 5. 遅延追い詰め（`vct_lazy/`）

`LazyVCTSolver` は初期の実験的な変種で、比較のために残されている。他と同じ水準では保守されていない（`vct_lazy.rs` 冒頭のコメントを参照）。アイデアは長井氏の 2011 年 GPW 論文「難解な必至問題を解くアルゴリズムとその実装」に由来する。

`vct/` と同じファイル構成と同じ `Searcher` / `Traverser` / `Resolver` 構造を持ち、次の点が異なる:

- 閾値は df-pn のもののみである（`Traverser::next_threshold_*` が df-pn の式）。候補は自身の初期 `Node` を持つ（`&[(Point, Node)]`）。
- `generate_attacks` はポテンシャルによるフィルタ（`>= 3`、禁手を除く）だけである。四追いのショートカットも、受け方の狙いによる絞り込みもない。
- `generate_defences` は、追い手の確認に別の四追いソルバーを呼ばない。代わりに「受け方がパスする」を受け方ノードの擬似的な子（手 `None`）として扱い、同じ df-pn の仕組みで探索する。この探索は四を作る手に限定される（`loop_defence_pass` → `search_limit_passed` → `search_attacks_passed`。後者は `four_moves()` だけを生成し、`Forced` の応手は四である場合だけ受け入れる）。
- パスのノードは他の子と同様に `defender_table` に保存され、その探索は親の閾値で制限される。したがって追い手の確認は、前もって完了まで走らせるのではなく、本探索と交互に進む。これが「遅延」の由来である。パスのノードが証明されていなければ、その `Node` が受け方ノードの値として返される。
- パスの部分木を証明する過程で、それを崩す点が `defences_memory: HashMap<u64, Vec<Point>>` に局面をキーとして記録される。記録されるのは、終端での `end_breakers`、各階層での勝ちの攻め手と強制された止め（`traverse_attacks_passed`、`traverse_defences_passed`）、各止めを通る受け方の剣先の眼（`next_sword_eyes`、`counter_defences` に相当）である。
- パスのノードが証明されると、受け方の候補は、記録された集合に `four_moves()` を加えてポテンシャルで並べ替えたものになる。
- `Resolver` は `solve_attacker_vcf` / `solve_attacker_threat` を必要とする。`LazyVCTSolver` はこれを、`1..u8::MAX` にわたる単一の `IDDFSSolver`（上限は状態の `limit`）で提供する。`threat_limit` は使われない。

リゾルバは `vct/` からコピーされたもので、受け方の候補を `threat_defences` で組み立て直す。これは遅延的に記録された集合と必ずしも一致しない。そのため、復元された手順が途中で `End::Unknown` で終わることがある（§4 の盤面では、他のソルバーの完全な手順に対して `F10,G9,I10`）。`solve.rs` のテストにある `VCTLAZY` の期待値は、目標ではなくこの振る舞いを記録したものである。

## 6. `PotentialField`（`analysis/field.rs`）

追い詰めの生成器には、「攻め方にとってここに石を置くとどれくらい有用か」で空点を並べる、安価で常に最新の指標が必要である。`PotentialField` は各点について方向ごとに 1 つの `u8`（`Potential { v, h, a, d }`）を保持し、その合計を返す。

方向ごとの値は、`Board::potentials(player, min, exact)` による線のポテンシャルである（02 の §7 を参照）。計算は次のとおり:

1. その点を含む 5 マス窓のうち、相手の石がないものを取る。黒ならさらに、縁に自分の石がないことも求める（`exact = player.is_black()`）。
2. 各窓について、そこに打った後に窓が持つ自分の石の数を求め、`min` 以上のものだけ残す。
3. `最大値 × その最大値に達する窓の数` を返す。

`min = 2` では、4 マス離れた石が 1 つあるだけで値が付く。

更新と問い合わせ:

- `init(player, min, board)` は場全体を埋める。
- `update_along(p, board)` は `p` を通る 4 本の線をゼロにし（`reset_along`）、`potentials_along` で計算し直す。`VCTState` は play と undo のたびにこれを呼ぶ。1 手あたりのコストは盤面全体の走査ではなく、線 4 本の走査である。
- `get(p)` は 4 方向の合計、`collect(min)` は合計が `min` 以上の点をすべて列挙する。`VCTState::sorted_potentials(3, ..)` と `sort_by_potential` は降順に並べる薄いラッパーである。
- 構築時の `min`（`2`）は窓をフィルタし、問い合わせ時の `min`（`3`）は合計をフィルタする。

`overlay(board)` はデバッグ用に場を描画する（空点は合計を表示し、`.` はゼロ）。§4 の例の盤面で `PotentialField::init(Black, 2, ..)` とすると次のようになる:

```
 . . . . . . . . . . . . . . .
 . . . 2 . . . . . . . . . . .
 . . . 2 2 2 . . . 2 . 2 . 2 .
 . . . . 4 2 4 4 . 2 2 . 4 . .
 . . . . 3 6 210 x 4 . 4 . . .
 . . . 2 41216 o18 8 8 2 . . .
 . . . 2 2 213 o x o 2 2 2 2 .
 . . . . . 2 x o12 x 8 . . . .
 . . . . 2 . 2 x o 8 2 8 2 . .
 . . . 2 . 2 . 2 4 9 4 . 4 . .
 . . . . 2 . 2 . 4 . 6 2 . 2 .
 . . . 2 . 2 . . 4 . . 3 . . .
 . . . . 2 . . . 2 . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
```

`I10`（18）と `G10`（16）は黒の線が 2 本同時に通る点で、だから攻め方の候補リストの先頭に来る。*受け* を並べるときも、場は攻め方のものである。攻め方にとってポテンシャルの高い点に置く受けが先に試される。

## 7. チートシート

| 知りたいこと | 見る場所 |
| --- | --- |
| 解法モードを追加する | `SolveMode` + `FromStr` + `solve` の `match`（`solve.rs`）。 |
| なぜ深さ N で探索が止まったか | `limit` は攻め方の着手数を数える。受け方のノードで `limit <= 1` なら `search_defences` は `zero_dn` を返す。 |
| 追い手として認識されない | `compute_defences` のステップ 1（`solve_attacker_threat`）。`attacker_vcf_depth = threat_limit` で、四追いは `Sword` の眼に限られる。 |
| 受けが足りない | `VCTState::threat_defences`（手順、`end_breakers`、`counter_defences`、`four_moves`）。 |
| 逆襲による反証が見つからない | `solve_defender_vcf` は `defender_vcf_depth = 2` に制限される。より深い逆襲の四追いは、ノリ手が `threat_defences` に現れる場合にしか見つからない。 |
| 手の並べ替え | `PotentialField`（`analysis/field.rs`）、`min = 2`、候補は合計 `>= 3` が必要。 |
| 置換表 | `ProofTree::attacker_table` / `defender_table`、`Generator::*_cache`、`DFSSolver::deadends`。すべて `zobrist_hash_n(limit)` がキー。 |
| 詰み手順の復元 | `Resolver`。`End::Unknown` は、表にたどれる証明済みの子がなかったことを意味する。 |
| 回帰テストの追加 | `solve.rs` のテストに ASCII 盤面と期待する詰み手順の文字列を追加し、関係する `SolveMode` ごとに 1 つずつ assert する。 |

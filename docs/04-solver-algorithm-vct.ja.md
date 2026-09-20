# `src/mate/vct/` はどうやって追い詰めを探索しているか

このドキュメントが説明するもの:

- `src/mate/vct/` の追い詰め（VCT、Victory by Continuous Threats）ソルバー
- それらの手を並べ替える `src/analysis/field.rs` の `PotentialField`

[03-solver-overview.ja.md](03-solver-overview.ja.md) の内容を前提とする。特に次のものは説明せずに使う。

- `solve` エントリポイント、`limit` / `threat_limit`、`Mate` / `End`
- `Game::check_event`、`State` トレイト
- 四追いソルバー（`VCFState`、`DFSSolver`、`IDDFSSolver`）。追い詰めソルバーはこれをサブルーチンとして呼ぶ
- 用語（攻め方 / 受け方、追い手、眼など）の対応表

モジュール構成:

| ファイル | 役割 |
| --- | --- |
| `vct/state.rs` | `VCTState`: `Game` + `limit` + `PotentialField`。内部の四追い探索用に派生させる `VCFState`、`threat_defences`。 |
| `vct/solver.rs` | `VCTSolver<P>`: 唯一のソルバー構造体（表、内部の四追いソルバー、キャッシュ）。`solve` = `search` してから `extract`。 |
| `vct/threshold.rs` | `ThresholdPolicy` とその実装 `DFSThreshold`、`PNSThreshold`、`DFPNSThreshold`。ソルバー間で唯一異なる部分。 |
| `vct/nested_vcf.rs` | 内部の四追い探索 4 種（攻め方 / 受け方 × 四追い / 追い手）。 |
| `vct/generator.rs` | 攻め手と受け手の候補生成（`Candidates`）と、その場で決着する場合のショートカット。 |
| `vct/proof.rs` | `Node`（証明数・反証数）、`ProofTable`（置換表）。 |
| `vct/searcher.rs` | AND/OR ノードの関数 `search_attacks` / `search_defences` と、展開ループ `expand_attacks` / `expand_defences`。 |
| `vct/selector.rs` | `select_attack` / `select_defence`: 子の表エントリからノードを評価し、最も証明に近い子を選ぶ。 |
| `vct/extractor.rs` | `extract`: 証明の後に表をたどって詰み手順を復元する。 |
| `analysis/field.rs` | `PotentialField`（§9）。 |

探索の全体像 — `solve` はまず根を証明し、次に証明をもう一度たどって手順を読み取る。各ノード関数は候補手を生成し（その途中で内部の四追いソルバーにいくつかの yes/no の問いを投げる）、最も見込みのある子を展開して、その結果を置換表に保存する:

```
VCTSolver::solve
├── search ................................................. §5
│   search_attacks（OR ノード: 攻め方の手番）
│   ├── generate_attacks ................................... §3
│   │     solve_attacker_vcf     -> Terminal(proven)?
│   │     solve_defender_threat  -> 候補を threat_defences に絞る
│   │     候補を PotentialField で並べる ...................... §9
│   └── expand_attacks: loop { select_attack; 最良の子を打つ; search_defences; attacker_table に保存 }
│
│   search_defences（AND ノード: 受け方の手番）
│   ├── generate_defences .................................. §3
│   │     solve_attacker_threat  -> Terminal(disproven)?（直前の攻め手は追い手だったか？）
│   │     solve_defender_vcf     -> Terminal(disproven)?（受け方が先に勝つか？）
│   │     threat_defences を PotentialField で並べる
│   └── expand_defences: loop { select_defence; 最良の子を打つ; search_attacks; defender_table に保存 }
│
└── extract ................................................ §6
```

`select_*` がどの子を選ぶかは証明数（§4）が決め、`expand_*` がその子にどれだけ留まってから親に戻るかは閾値ポリシー（§5）が決める。§7 で完全な例を追う。

---

## 1. 何を追い手とみなすか

このソルバーでの追い詰めは、攻め方の各手が *追い手* である手順である。追い手とは「受け方がパスしたら、攻め方に高々 `threat_limit` 手の四追いがある手」のことだ。

- 四は自明な追い手である。受け方は `Forced` になる。
- 三は追い手である。受け方がパスすれば、達四点に打つ 1 手の四追いがある。
- `threat_limit >= 2` なら、四三を準備する手のような「フクミ手」も追い手になる。`test_vct_fukumi_move` がその例で、`threat_limit = 3` が必要である。

受け方は、追い手が狙う四追いを止める手であれば何でも打てる。ノリ手や、受け方自身の四追いによる逆襲も含む。

探索は AND/OR 木である。

- 攻め方のノードは OR ノード。良い攻め手が 1 つあればよい。
- 受け方のノードは AND ノード。すべての受けが負けでなければならない。

この木を証明数で解く。3 つの `SolveMode` は木のたどり方だけが異なる。

## 2. `VCTState`

`VCTState` は次のものからなる:

- `Game`
- `attacker`
- `limit`
- 攻め方の `PotentialField`（§9）。`PotentialField::init(attacker, 2, board)` で初期化し、`after_play` / `after_undo` で各手の 4 本の線に沿って更新する

### 内部の四追い探索

追い詰めソルバーは、探索の途中で何度も「今この局面に四追いがあるか」を問う。そのために、`VCTState` から 2 種類の `VCFState` を派生させる:

| メソッド | 局面 | 四追いの攻め方 | 四追いの limit | 問い |
| --- | --- | --- | --- | --- |
| `vcf_state(max)` | そのまま | 手番側 | `min(limit, max)` | 手番側は今すぐ四追いで勝てるか？ |
| `threat_state(max)` | 手番側がパスした後 | 相手側 | 攻め方の手番なら `min(limit - 1, max)`、そうでなければ `min(limit, max)` | 自分が何もしなければ相手に四追いがあるか？ つまり直前の手は追い手か？ |

`nested_vcf.rs` は、手番と問いの 4 通りの組み合わせに名前を付けている:

| メソッド | 手番 | 問い |
| --- | --- | --- |
| `solve_attacker_vcf` | 攻め方 | 攻め方は今すぐ四追いで勝てるか？ |
| `solve_defender_threat` | 攻め方 | 攻め方がパスしたら、受け方に四追いがあるか？ |
| `solve_attacker_threat` | 受け方 | 受け方がパスしたら、攻め方に四追いがあるか？（直前の攻め手は追い手か？） |
| `solve_defender_vcf` | 受け方 | 受け方は今すぐ四追いで勝てるか？ |

四追いの深さの上限 `max` は 2 種類ある:

- 攻め方の探索: `attacker_vcf_depth`。`threat_limit` がそのまま入る。
- 受け方の探索: `defender_vcf_depth`。`SolveLimits::defender_vcf_depth` がそのまま入る（指定しなければ `2`）。

裏で動くのは各側 1 つずつの `IDDFSSolver`（`limits = [1]`）である。その `deadends` メモは、追い詰め探索全体を通して保持される。

### `next_zobrist_hash`

`next_zobrist_hash(m)` は、`m` を打った後の子のキーを計算する。ポテンシャル場の更新は高コストなので、それには触れない。これにより、未展開の子の表引きが安価になる。

## 3. 手の生成（`generator.rs`）

生成器は攻め方用と受け方用の 2 つある。どちらも `Candidates` を返す:

- `Moves(候補)`: 内部ノード。候補手のリスト。
- `Terminal(node)`: その場で決着がついた。`Node::proven` なら攻め方の勝ちが証明済み、`Node::disproven` なら反証済み。

結果は `zobrist_hash()` をキーに、1000 エントリの `LruCache` にメモ化される（攻め方用と受け方用で 1 つずつ）。

### `compute_attacks`（攻め方の手番）

1. `solve_attacker_vcf` を呼ぶ。四追いがあれば `Terminal(proven)` を返す。これは正しさのためには不要である（本探索でも `Forced` の応手を 1 つずつたどれば四追いは見つかる）。しかし大幅に速くなる。
2. `solve_defender_threat` を呼ぶ。受け方に四追いがあれば（攻め方が何もしなければ受け方が勝つ）、候補を `threat_defences(threat)`（後述）に絞る。攻め手はその狙いも同時に受けなければならない。
3. 候補を作る。攻め方のポテンシャル場でポテンシャル `>= 3` の点（`sorted_potentials(3, ..)`）を高い順に並べ、禁手を除く。空なら `Terminal(disproven)`。

ここでは、候補が追い手かどうかを判定していない。判定は 1 手先の受け方のノードで行う。追い手でない手は、`compute_defences` のステップ 1 で反証される。

### `compute_defences`（受け方の手番）

1. `solve_attacker_threat` を呼ぶ。受け方がパスしても攻め方に四追いがなければ、直前の攻め手は追い手ではなかった。`Terminal(disproven)` を返す。
2. `solve_defender_vcf` を呼ぶ。受け方自身に（`defender_vcf_depth` 手以内の）四追いがあれば、受け方が先に勝つ。`Terminal(disproven)` を返す。
3. 候補を作る。`threat_defences(threat)` を攻め方のポテンシャルで並べ替え（`sort_by_potential`）、禁手を除く。空なら `Terminal(proven)`。つまり、その攻め手は受けられない。

### `threat_defences`

`threat_defences(threat)` は、追い手が狙う四追いを止めうる手の集合である。ヒューリスティックであり、次の 4 種類をこの順に並べる:

- 狙われている四追いの手順に含まれる点すべて。攻め方の四も、受け方の強制された止めも含む。どれかを先に占めれば手順が崩れる。
- `end_breakers(end)`: 詰め上がりを崩す点。
  - `Fours(p1, p2)` なら、2 つの勝ち点。
  - `Forbidden(p)` なら、その点と、4 本の線に沿って 5 マス以内の空点（`neighbors(p, 5, true)`）。近くに石を置くと、`p` が禁手かどうかが変わりうるためである。
- `counter_defences(threat)`: ノリ手になりうる点。狙われている四追いの手順を再生し、受け方の各止めについて、その止めを通る受け方の `Sword` の眼を集める。手順の途中で受け方に四が生じる点であり、今打てばノリ手になりうる。
- `four_moves()`: 受け方が今すぐ打てる、四を作る手（`Sword` の眼）すべて。攻め方に応手を強いるノリ手である。

このリストには同じ点が 2 回入ることがある。後の `dedup` は、並べ替えた後に隣接している重複しか取り除かない。したがって重複が残りうるが、無害である（同じ子を 2 回引くだけだ）。

## 4. 証明数（`proof.rs`）

```rust
pub struct Node { pub pn: u32, pub dn: u32, pub limit: u8 }
pub const INF: u32 = u32::MAX;
```

- `pn` は証明数。攻め方の勝ちを証明するのに、あと何個の葉を証明する必要があるかの見積もりである。`pn == 0` は証明済み。
- `dn` は反証数。`dn == 0` は反証済み。

| コンストラクタ | `(pn, dn)` | 意味 |
| --- | --- | --- |
| `Node::unknown()` | `(INF, INF)` | 情報なし（表にない局面）。 |
| `Node::no_threshold()` | `(INF, INF)` | 根の閾値「決着するまで探索する」。値は `unknown` と同じだが意味が異なる。 |
| `Node::proven(limit)` | `(0, INF)` | 証明済み（攻め方の勝ち）。 |
| `Node::disproven(limit)` | `(INF, 0)` | 反証済み。 |
| `Node::unexpanded_defence(n, limit)` | `(n, 1)` | OR ノードの未展開の子（受け方の手番）の初期見積もり。`n` は兄弟の数。 |
| `Node::unexpanded_attack(n, limit)` | `(1, n)` | AND ノードの未展開の子（攻め方の手番）の初期見積もり。`n` は兄弟の数。 |

`Node::is_proven()` は `pn == 0` である。

子の値を合成する方法は 2 つある。和は飽和加算である。

- `min_pn_sum_dn`: OR ノード用。pn = min、dn = 和。
- `min_dn_sum_pn`: AND ノード用。pn = 和、dn = min。

直観的には次のとおり。OR ノードでは攻め方は子を 1 つ証明すればよいので、ノードの証明の手間は最も安い子と同じ（min）で、反証にはすべての子の反証が要る（和）。AND ノードでは役割が入れ替わる。したがって根から、OR ノードでは `pn` が最小の子、AND ノードでは `dn` が最小の子をたどって下りていくと、その結果が最も多くを決める葉 — *最も証明に近いノード* — に着く。これがセレクタのしていることである（§5）。

`limit` は子の最小値として一緒に運ばれる。部分木が決着した時点で、予算がどれだけ残っていたかを記録するためである。復元処理はこれを使って、最も粘り強い受けを選ぶ（§6）。

ソルバーは 2 つの置換表（`ProofTable`、中身は `HashMap<u64, Node>`）を持つ:

- `attacker_table`: 攻め手で到達した局面（受け方の手番）の値
- `defender_table`: 受け手で到達した局面（攻め方の手番）の値

`ProofTable::lookup_next(state, m)` は、`next_zobrist_hash` で `m` の後の子を引く。

## 5. 探索（`searcher.rs`、`selector.rs`、`threshold.rs`）

### ノード関数

`VCTSolver::search` は根が証明済みかどうかを返す。`limit == 0` を確認した後、`search_attacks(state, Node::no_threshold()).is_proven()` を返す。

相互再帰する 2 つのノード関数は次のとおりである:

```
search_attacks(state, threshold):              # OR node, attacker to move
    Defeated(_)  -> disproven
    Forced(p)    -> expand_attacks(state, [p], threshold)
    otherwise    -> generate_attacks -> Terminal(node) => node
                                     | Moves(attacks) => expand_attacks(...)

search_defences(state, threshold):             # AND node, defender to move
    Defeated(_)  -> proven                      # the attacker has won
    limit <= 1   -> disproven                   # the attacker has no move left after this defence
    Forced(p)    -> expand_defences(state, [p], threshold)
    otherwise    -> generate_defences -> Terminal(node) => node
                                      | Moves(defences) => expand_defences(...)
```

### 子の選択（`selector.rs`）

`select_attack` は何も展開しない。子の表エントリを見てノードを評価し、`Selection` を返す:

- `node`: ノード自身の `(pn, dn)`。子に対する `min_pn_sum_dn` で求める。表にない子は `unexpanded_defence(attacks.len())` とみなす。
- `best`: `pn` が最小の子（最も証明に近い子）。
- `best_child` / `second_child`: 最良の子と 2 番目の子の値。

証明済みの子が見つかれば、`node` は直ちに `(0, INF)` になる。

`select_defence` はその鏡像である。`dn` が最小の子を選び、`min_dn_sum_pn` で合成し、表にない子は `unexpanded_attack(defences.len())` とみなす。`node.limit` は `limit - 1` になる。

未展開の子を兄弟数で初期化するのは「トリック」である。候補手の少ないノードほど簡単に見えるので、探索は狭く強制的な手順を優先する。

### 展開ループ

`expand_attacks` は 3 つのソルバーに共通の展開ループである:

```
expand_attacks(state, attacks, threshold):
    loop:
        selection = select_attack(state, attacks)
        if selection.node.pn >= threshold.pn or selection.node.dn >= threshold.dn:
            return selection                   # exceeds_threshold
        next = P::next_threshold_attack(selection, threshold)
        play selection.best
            attacker_table.insert(child, search_defences(child, next))
        undo
```

`expand_defences` も同じで、受け方の表、`P::next_threshold_defence`、`search_attacks` を使う。

ノードは、その数値が親から渡された閾値を超えるまで展開される。根の閾値は `Node::no_threshold()` なので、根は決着するまでループする。決着とは `pn == 0`（証明済み。このとき `dn` は `INF` にされる）か `pn == INF`（反証済み）である。

### 閾値ポリシー（`threshold.rs`）

ソルバー間の唯一の違いは `next_threshold_*`、つまり子に渡す閾値の決め方である。この選択が `VCTSolver<P>` の型パラメータ `P: ThresholdPolicy` であり、3 つのポリシーはサイズ 0 の型である:

| ポリシー | 子の閾値 | 振る舞い |
| --- | --- | --- |
| `DFSThreshold` | `Node::no_threshold()` | 選んだ子を完全に探索してから、親が次の子を見る。通常の深さ優先探索で、証明数は手の並べ替えにだけ使う。 |
| `PNSThreshold` | `(best_child.pn + 1, best_child.dn + 1)` | 子は数値が変わり次第戻る。制御が上に戻り、各階層で最も証明に近い子が選び直される。展開のたびに根から選び直す最良優先 PNS を、再帰的な探索の中で模倣したもの。 |
| `DFPNSThreshold` | OR ノード: `pn = min(threshold.pn, second_child.pn + 1)`、`dn = threshold.dn - node.dn + best_child.dn`。AND ノードはその鏡像 | Nagai & Imai (2002) の df-pn の閾値。最良の子が最良である間はそこに留まり、親の予算を超えない。 |

### ソルバーの構造体

`VCTSolver<P>`（`solver.rs`）は 1 つの構造体で、`DFSVCTSolver`、`PNSVCTSolver`、`DFPNSVCTSolver` は 3 つのポリシーに対する型エイリアスである。持っているのは次のものだけである:

- `ProofTable` 2 つ
- 四追い用の `IDDFSSolver` 2 つと、その深さ 2 つ
- 生成器のキャッシュ 2 つ

メソッドは段階ごとに `searcher.rs`、`selector.rs`、`generator.rs`、`nested_vcf.rs`、`extractor.rs` に分かれている。`VCTSolver::solve` は次のとおりである:

```rust
if self.search(state) { self.extract(state) } else { None }
```

## 6. 手順の復元（`extractor.rs`）

探索は、勝ちがあることを証明するだけである。`extract` は表をもう一度たどって詰み手順を復元する。

`extract_attacks`（攻め方の手番）:

- `Forced` なら、それに従う。
- そうでなければ `state.empties()` を走査し、`attacker_table` のエントリが証明済みである最初の手を打つ。
- 証明済みの手がなければ、`solve_attacker_vcf` の四追いを手順の末尾として返す。これは `compute_attacks` の四追いショートカットで証明されたノードである。

`extract_defences`（受け方の手番）:

- `Defeated(end)` なら、その `end` を詰め上がりとして手順を終える。
- `Forced` なら、それに従う。
- そうでなければ、攻め方の追い手に対する `threat_defences` を再計算する。証明済みの子のうち、`Node::limit` が最小のものを選ぶ。これは攻め方に最も多くの手を使わせた受けである。したがって報告される手順は、最も粘り強い受けに対するものになる。
- 証明済みの候補がなければ、手順は `End::Unknown` で終わる。これは、合法な受けが存在しないために証明されたノードである。

## 7. 例で追う

この章では、ここまでの部品が 1 つの回帰テストの中でどう動くかを、根の局面から報告される手順まで追う。盤面は `test_vct_black` のもの（黒番。`solve.rs` のコメントによれば岡部寛氏の五手詰め問題 No. 02）:

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

`solve(VCTDFS, 4, &board, Black, 1)` は `F10,G9,I10,G10,H11,H12,G12`、詰め上がり `Fours(F13, K8)` を返す（`VCTPNS`、`VCTDFPNS` も同じ）。`limit = 3` では失敗する。

- `F10` で三 `F10,_,H8,I7` ができる。これは追い手である（パスすれば `G9` が達四になる）。白の `threat_defences` には `G9` が含まれ、白はそれを打つ。
- `I10` で三 `F10,_,H10,I10` ができ、白は `G10` に止める。
- `H11` で四 `H8..H11` ができ、`H12` が `Forced` になる。
- `G12` で `G12,H11,I10,J9` ができ、`F13` と `K8` が空いている: `Fours`。

根での攻め方の候補は、ポテンシャル `>= 3` の点を高い順に並べたものである（`I10`、`G10`、`G9`、`F10`、…。オーバーレイは §9 にある）。したがって深さ優先ソルバーは、`F10` より先に `I10` を試す。

`I10` は即座に反証される。三ができないので、白がパスしても黒に 1 手の四追いはない。`compute_defences` が `Terminal(disproven)` を返し、その反証が `attacker_table` に保存される。

## 8. 遅延追い詰め（削除済み）

かつて `vct/` の隣に、実験的な「遅延」追い詰めソルバー（`vct_lazy/`、`SolveMode::VCTLAZY`）があった。受け方の各ノードで攻め方の追い手（四追い）を先に解いてしまうのではなく、その確認を本体の df-pn 探索に織り込み、途中で見つかった受けの手を記録していくものだった。アイデアは次の論文に由来する。

> 長井歩. "難解な必至問題を解くアルゴリズムとその実装." ゲームプログラミングワークショップ 2011 論文集 2011.6 (2011): 1-8.

他のソルバーと同じ品質には至らず、[renju-note/quintet#133](https://github.com/renju-note/quintet/pull/133) で削除された。削除時点の状態と、削除の判断材料となった計測結果はその PR を参照。

## 9. `PotentialField`（`analysis/field.rs`）

追い詰めの生成器には、「攻め方にとってここに石を置くとどれくらい有用か」で空点を並べる指標が必要である。それは安価で、常に最新でなければならない。`PotentialField` は各点について方向ごとに 1 つの `u8`（`Potential { v, h, a, d }`）を保持し、その合計を返す。

### 方向ごとの値

方向ごとの値は、`Board::potentials(player, min, exact)` による線のポテンシャルである（02 の §7 を参照）。計算は次のとおり:

1. その点を含む 5 マス窓のうち、相手の石がないものを取る。黒ならさらに、縁に自分の石がないことも求める（`exact = player.is_black()`）。
2. 各窓について、そこに打った後に窓が持つ自分の石の数を求める。`min` 以上のものだけ残す。
3. `最大値 × その最大値に達する窓の数` を返す。

`min = 2` では、4 マス離れた石が 1 つあるだけで値が付く。

### 更新と問い合わせ

- `init(player, min, board)` は場全体を埋める。
- `update_along(p, board)` は `p` を通る 4 本の線をゼロにし（`reset_along`）、`potentials_along` で計算し直す。`VCTState` は play と undo のたびにこれを呼ぶ。1 手あたりのコストは、盤面全体の走査ではなく線 4 本の走査である。
- `get(p)` は 4 方向の合計を返す。`collect(min)` は合計が `min` 以上の点をすべて列挙する。
- `VCTState::sorted_potentials(3, ..)` と `sort_by_potential` は、降順に並べる薄いラッパーである。
- `min` は 2 か所にある。構築時の `min`（`2`）は窓をフィルタし、問い合わせ時の `min`（`3`）は合計をフィルタする。

### オーバーレイ

`overlay(board)` はデバッグ用に場を描画する。空点は合計を表示し、`.` はゼロである。§7 の盤面で `PotentialField::init(Black, 2, ..)` とすると次のようになる:

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

`I10`（18）と `G10`（16）は、黒の線が 2 本同時に通る点である。だから攻め方の候補リストの先頭に来る。

*受け* を並べるときも、場は攻め方のものである。攻め方にとってポテンシャルの高い点に置く受けが、先に試される。

## 10. チートシート

| 知りたいこと | 見る場所 |
| --- | --- |
| なぜ深さ N で探索が止まったか | `limit` は攻め方の着手数を数える。受け方のノードで `limit <= 1` なら `search_defences` は `disproven` を返す。 |
| 追い手として認識されない | `compute_defences` のステップ 1（`solve_attacker_threat`）。`attacker_vcf_depth = threat_limit` で、四追いは `Sword` の眼に限られる。 |
| 受けが足りない | `VCTState::threat_defences`（手順、`end_breakers`、`counter_defences`、`four_moves`）。 |
| 逆襲による反証が見つからない | `solve_defender_vcf` は `defender_vcf_depth`（既定は 2）に制限される。より深い逆襲の四追いは、ノリ手が `threat_defences` に現れる場合にしか見つからない。`SolveLimits::with_defender_vcf_depth` で深くできる。 |
| 手の並べ替え | `PotentialField`（`analysis/field.rs`）、`min = 2`、候補は合計 `>= 3` が必要。 |
| 置換表 | `VCTSolver::attacker_table` / `defender_table`、`VCTSolver::*_cache`、`DFSSolver::deadends`。すべて `zobrist_hash_n(limit)` がキー。 |
| 詰み手順の復元 | `extract`（`extractor.rs`）。`End::Unknown` は、表にたどれる証明済みの子がなかったことを意味する。 |
| 回帰テストの追加 | `solve.rs` のテストに ASCII 盤面と期待する詰み手順の文字列を追加し、関係する `SolveMode` ごとに 1 つずつ assert する（03 の §4 を参照）。 |

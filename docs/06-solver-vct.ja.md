# 追い詰め（VCT）探索（`src/mate/vct/`）

**追い詰め**（VCT: victory by continuous threats）は、攻め手がすべて**追い手**である手順である。追い手とは、四、三、あるいは受け方が何もしなければ攻め方に四追いが生じる任意の手をいう。四追いと違って受け方には応手の選択があるので、木は本物の AND/OR 木であり、ソルバーはそれを**証明数**で探索する。このドキュメントはその探索と、手を並べ替える `PotentialField` を説明する。

[04](04-solver-framework.ja.md) と [05](05-solver-vcf.ja.md) を前提にする。特に `State`、`Key`、`Memo`、`Solver` トレイト、`DFSSolver`。

```
src/mate/vct.rs         モジュールドキュメント: アルゴリズムの 1 ページ要約、再エクスポート、3 つのエイリアス
src/mate/vct/
├── state.rs            VCTState: Game + 攻め方 + limit + PotentialField。threat_defences        (§2, §3)
├── nested_vcf.rs       NestedVCF: 片側の内部四追い探索                                          (§2)
├── generator.rs        generate_attacks / generate_defences → Candidates                        (§3)
├── proof.rs            Node（証明数）、ProofTable（置換表）                                      (§4)
├── searcher.rs         search_attacks / search_defences、expand_attacks / expand_defences       (§5)
├── selector.rs         select_attack / select_defence → Selection                               (§5)
├── threshold.rs        ThresholdPolicy: DFSThreshold、PNSThreshold、DFPNSThreshold              (§5)
├── solver.rs           VCTSolver<P>: 構造体、Solver の実装                                      (§5)
└── extractor.rs        extract: 表から詰み手順を復元する                                        (§6)
src/analysis/field.rs   PotentialField                                                           (§8)
```

探索全体を 1 画面で — `solve` は根を証明し、次に証明をもう一度たどって手順を読み取る:

```
VCTSolver::solve = advance_generation; search; extract
│
├── search ──► search_attacks(root, no_threshold).is_proven()
│
│   search_attacks（OR ノード、攻め方の手番）                              §5
│   ├── check_event: Defeated → disproven; Forced(p) → attacks = [p]
│   ├── generate_attacks                                                  §3
│   │     attacker_vcf.vcf      四追いがある？           → Terminal(proven)
│   │     defender_vcf.threat   追い手を受けねばならない？ → threat_defences に絞る
│   │     ポテンシャル ≥ 3 の点、良い順                                   §8
│   └── expand_attacks: loop { select_attack; 最良の子を打つ; search_defences; attacker_table に保存 }
│
│   search_defences（AND ノード、受け方の手番）
│   ├── check_event: Defeated → proven; limit ≤ 1 → disproven; Forced(p) → defences = [p]
│   ├── generate_defences                                                 §3
│   │     attacker_vcf.threat   直前の手は追い手だった？ → でなければ Terminal(disproven)
│   │     defender_vcf.vcf      受け方が先に勝つ？        → Terminal(disproven)
│   │     threat_defences、良い順
│   └── expand_defences: loop { select_defence; 最良の子を打つ; search_attacks; defender_table に保存 }
│
└── extract ──► 表の中の証明済みの子をたどる                               §6
```

証明数（§4）が `select_*` の選ぶ子を決め、閾値ポリシー（§5）が `expand_*` がその子にどれだけ留まってから親に戻るかを決める。DFS / PNS / df-pn の 3 モードの違いはそのポリシーだけである。

## 1. 追い手と AND/OR 木

ある手が**追い手**であるとは、受け方がパスしたとすれば攻め方に `threat_limit` 個以下の四による四追いがあることをいう。

| `threat_limit` | 追い手と認識されるもの |
| --- | --- |
| 0 | 四だけ — 内部四追いに深さがないので、`Forced` の応手だけが通る |
| 1 | 加えて三: パスの後、棒四になる点が 1 手の四追い |
| 2 | 加えて四 2 つの四追いを準備する手。四三を用意する手など |
| 3 以上 | さらに深い含み — `test_vct_fukumi_move` には 3 が要る |

受け方は、狙われた四追いを止める任意の手で応じてよい。ノリ手や、受け方自身の逆襲四追いも含む。

木は 2 種類のノードが交互に現れる:

- **OR ノード**、攻め方の手番: 勝つ攻め手が **1 つ**あればよい。
- **AND ノード**、受け方の手番: **すべての**受けが負けなければならない。

## 2. `VCTState` と内部の四追い探索

```rust
pub struct VCTState { game: Game, pub attacker: Player, pub limit: u8, field: PotentialField }
```

場は攻め方の `PotentialField`（§8）で、`PotentialField::init(attacker, 2, board)` で作り、各手の 4 本の線に沿って `after_play` / `after_undo` で更新する。`next_key(m)` は `m` を打った後の子の `key()` を、場に**触れずに**計算する。未展開の子の表引きが安価なのはこのためである。

### 派生する 2 つの `VCFState`

追い詰め探索は絶えず「ここに四追いがあるか」を問う。`VCTState` は 2 つの問いのために四追い状態を派生させる:

| メソッド | 局面 | 四追いの攻め方 | 四追いの limit | 問い |
| --- | --- | --- | --- | --- |
| `vcf_state(max)` | そのまま | 手番側 | `min(limit, max)` | 手番側は今すぐ四追いで勝てるか？ |
| `threat_state(max)` | 手番側がパスした後 | 相手側 | 攻め方の手番なら `min(limit − 1, max)`、そうでなければ `min(limit, max)` | 自分が何もしなければ、相手に四追いがあるか？ |

### `NestedVCF`: 各側に 1 つ

`nested_vcf.rs` は、四追いソルバーとその深さの上限、どちら側のものかをまとめている:

```rust
pub struct NestedVCF { solver: IDDFSSolver, depth: u8, for_attacker: bool }
impl NestedVCF {
    pub fn vcf(&mut self, state: &VCTState, budget) -> Option<Mate>;     // この側は今手番で、四追いがあるか？
    pub fn threat(&mut self, state: &VCTState, budget) -> Option<Mate>;  // 相手がパスすれば、この側に四追いがあるか？
}
```

`VCTSolver` は `attacker_vcf`（深さ `threat_limit`）と `defender_vcf`（深さ `defender_vcf_depth`、既定 2）を持つ。2 つのオブジェクト × 2 つの問いで、生成器が行う 4 つの呼び出しになる:

| 呼び出し | 手番 | 問い |
| --- | --- | --- |
| `attacker_vcf.vcf` | 攻め方 | 攻め方は今すぐ四追いで勝てるか？ |
| `defender_vcf.threat` | 攻め方 | 攻め方がパスしたら受け方に四追いがあるか — この攻め手は何を同時に受けねばならないか？ |
| `attacker_vcf.threat` | 受け方 | 受け方がパスしたら攻め方に四追いがあるか — 直前の攻め手は追い手だったか？ |
| `defender_vcf.vcf` | 受け方 | 受け方は今すぐ四追いで勝てるか — 受け方が先に勝つか？ |

`vcf` は自分の側の手番であること、`threat` は相手の手番であることを assert する。逆向きの呼び出しは探索のバグであって、問いではない。中身は `IDDFSSolver`（`limits = [1]`）で、`search` を通して呼ぶ。そのため行き止まりメモ（04 の §5）は追い詰め探索 1 回の間も、探索をまたいでも生き続け、同じ `NodeBudget` から消費される。

## 3. 手の生成（`generator.rs`）

`generate_attacks` と `generate_defences` は `Candidates` を返す:

```rust
pub enum Candidates {
    Moves(Vec<Point>),   // 内部ノード: 試す手、良い順
    Terminal(Node),      // 展開せずにここで決着: Node::proven または Node::disproven
}
```

結果は生成器ごとに 1000 エントリの `LruCache` にキャッシュされる。キーは `zobrist_hash()`（局面**と** limit。内部四追いの深さが `min(limit, depth)` だから）で、予算切れのときはキャッシュしない。

**`compute_attacks`**（攻め方の手番）:

1. `attacker_vcf.vcf` — 今すぐ四追いがあるか？ → `Terminal(proven)`。正しさには不要（本探索でも `Forced` の応手を 1 つずつたどれば四を見つける）だが、大幅に速い。
2. `defender_vcf.threat` — 攻め方がパスしたら受け方に四追いがあるか？ あるなら攻め手はそれも受けねばならない: 候補を `threat_defences(threat)` に絞る。
3. 候補は攻め方の場でポテンシャル `≥ 3` の点（`sorted_potentials(3, only)`）、高い順、禁手を除く。残らなければ → `Terminal(disproven)`。

ここでは候補が追い手かどうかを**調べない**。それは 1 手下の受け方ノードで行われ、追い手でない手はそこで即座に反証される。

**`compute_defences`**（受け方の手番）:

1. `attacker_vcf.threat` — 受け方がパスしたら攻め方に四追いがあるか？ なければ直前の攻め手は追い手ではなかった → `Terminal(disproven)`。
2. `defender_vcf.vcf` — 受け方自身に（`defender_vcf_depth` 以内の）四追いがあるか？ あれば受け方が先に勝つ → `Terminal(disproven)`。
3. 候補は `threat_defences(threat)` を**攻め方の**ポテンシャルで並べたもの（`sort_by_potential`）、禁手を除く。残らなければ → `Terminal(proven)`: 追い手に応手がない。

### `threat_defences`

`VCTState::threat_defences(&threat)` は、狙われた四追いを止めうる手のヒューリスティックなリストで、この順に並ぶ:

| 部分 | 点 |
| --- | --- |
| 手順 | 追い手の四追いのすべての石 — 攻め方の四と受け方の止め。どれかを占めれば手順は崩れる |
| `end_breakers(end)` | `Fours(p1, p2)` なら 2 つの勝ち点。`Forbidden(p)` ならその点自身と、4 本の線に沿って 5 以内のすべての空点（`neighbors(p, 5, true)`）。近くの石が `p` の禁手判定を変えうるため |
| `counter_defences(threat)` | パスの後で追い手の四追いを再生し、その中の受け方の各止めについて、その止めを通る受け方の `Sword` の眼を集める — 手順の途中で受け方が四を得る点で、今打てばノリ手になりうる |
| `four_moves()` | 受け方が今持っているすべての四を作る手（`Sword` の眼） — 攻め方に応手を強いるノリ手 |

リストは同じ点を重複して含みうる。並べ替え後の `dedup` は隣接する重複しか除かないので重複は残りうるが、無害である（同じ子を 2 回引くだけ）。

## 4. 証明数（`proof.rs`）

```rust
pub struct Node { pub pn: u32, pub dn: u32, pub limit: u8 }
pub const INF: u32 = u32::MAX;
```

`pn` は**証明数**: このノードを攻め方が証明するために、少なくともあと何個の葉を勝たねばならないか。`dn` は**反証数**で、反証について同じもの。`pn == 0` が証明済み、`dn == 0` が反証済みである。

| コンストラクタ | `(pn, dn)` | 意味 |
| --- | --- | --- |
| `Node::unknown()` | `(INF, INF)` | 表が何も知らない局面 |
| `Node::no_threshold()` | `(INF, INF)` | 決して超えない閾値: 「決着するまで探索」（`unknown` と同じ値、別の役割） |
| `Node::proven(limit)` | `(0, INF)` | 攻め方の勝ち |
| `Node::disproven(limit)` | `(INF, 0)` | `limit` 以内に攻め方は勝てない |
| `Node::unexpanded_defence(n, limit)` | `(n, 1)` | OR ノードの未展開の子（受け方の手番）の初期値。`n` = 兄弟の数 |
| `Node::unexpanded_attack(n, limit)` | `(1, n)` | AND ノードの未展開の子（攻め方の手番）の初期値 |

子の合成は 2 通りで、和は飽和加算である:

- **OR** ノードでは `min_pn_sum_dn`: 証明のコストは最も安い子の分（pn の min）、反証にはすべての子の反証が要る（dn の和）。
- **AND** ノードでは `min_dn_sum_pn`: その鏡像。

根から、OR ノードでは最小の `pn`、AND ノードでは最小の `dn` の子をたどって降りると、その結果が根を最も動かす葉 — **最有望ノード**（most-proving node）— に至る。選択器はこれをたどる（§5）。`limit` は子の最小値として付いて回る。部分木が決着した場所で残っていた最小の予算である。復元器がこれを使う（§6）。

### `ProofTable`

ソルバーは 2 つの表を持つ:

- `attacker_table`: 攻め手で到達したノード（受け方の手番）
- `defender_table`: 受け手で到達したノード（攻め方の手番）

それぞれが `ProofTable` で、1 つ屋根の下の 2 つの `Memo` である:

| 半分 | キー | 持つもの |
| --- | --- | --- |
| `estimates: Memo<Node>` | `Key::hash()` — 局面と limit | 挿入されたすべてのノードをそのまま。決着に至らない証明数は、それを計算した limit に属する |
| `decided: Memo<Decided>` | `Key::position` のみ | その局面で見た、証明された最小 limit と反証された最大 limit |

`insert(state, node)` は `estimates` に常に書き、ノードが証明済みまたは反証済みなら `decided` にも書く。`lookup_next(state, m)` は `m` の後の子を `next_key` で引く。まず `estimates`、次に `decided` で、より小さい limit での証明やより大きい limit での反証がなお答えになる（04 の §3）。`decided` を通して、limit 5 の探索は limit 4 の探索が確定したものを再利用し、ある根から見つかった証明は別の根の下でもそれ以上の深さで答えになる。

`transfer_from` は `decided` が効き始める深さである。生成器は内部の四追いソルバーに `min(limit, depth)` で問うので、その深さより下では候補リストが limit とともに変わり、2 つの limit は別の木を述べている。`VCTSolver::with_carry_capacity` はこれを `max(attacker_vcf_depth, defender_vcf_depth + 1, 2)` に設定し、それより下では `decided` に書きも読みもしない。

## 5. 探索（`searcher.rs`、`selector.rs`、`threshold.rs`、`solver.rs`）

### ノード関数

`VCTSolver::search(state, budget)` は、`limit == 0` の検査の後の `search_attacks(state, Node::no_threshold(), budget).is_proven()` である。2 つのノード関数は互いを呼ぶ:

```
search_attacks(state, threshold):              # OR ノード、攻め方の手番
    budget.consume() できなければ → unknown
    Defeated(_)  → disproven(limit)
    Forced(p)    → expand_attacks([p])
    それ以外     → generate_attacks: Terminal(n) → n | Moves(a) → expand_attacks(a)

search_defences(state, threshold):             # AND ノード、受け方の手番
    budget.consume() できなければ → unknown
    Defeated(_)  → proven(limit)                # 攻め方が勝った
    limit ≤ 1    → disproven(limit)             # この受けの後、攻め手が残っていない
    Forced(p)    → expand_defences([p])
    それ以外     → generate_defences: Terminal(n) → n | Moves(d) → expand_defences(d)
```

### 選択

`select_attack(state, attacks)` は何も展開しない。子の表エントリを読み、`Selection` を返す:

| フィールド | |
| --- | --- |
| `node` | このノード自身の数値: 子に対する `min_pn_sum_dn`。表にまだない子は `unexpanded_defence(attacks.len())` として数える |
| `best` | `pn` が最小の子 — 最有望の子 |
| `best_child`、`second_child` | 最良と 2 番目の子の数値 |

証明済みの子が見つかれば `node` はその場で `(0, INF)` になる。`select_defence` はその鏡像である。最小の `dn`、`min_dn_sum_pn`、未展開の子は `unexpanded_attack(defences.len())` と数え、`node.limit` は `limit − 1` になる。

未展開の子に兄弟の数を初期値として与えるのは意図的な仕掛けである。候補の少ないノードは易しく見えるので、探索は強制的な手順を好む。

### 展開

`expand_attacks` は 3 モード共通の 1 つのループである:

```
expand_attacks(state, attacks, threshold):
    loop:
        s = select_attack(state, attacks)
        if s.node.pn ≥ threshold.pn or s.node.dn ≥ threshold.dn: return s   # exceeds_threshold
        if 予算切れ:                                              return s
        next = P::next_threshold_attack(s, threshold)
        into_play(s.best):
            result = search_defences(child, next)
            if 予算が尽きていなければ: attacker_table.insert(child, result)
```

`expand_defences` は `select_defence`、受け方の表、`P::next_threshold_defence`、`search_attacks` で同じことをする。ノードは、自分の数値が親から渡された閾値を超えるまで最有望の子を展開し続ける。根の閾値は `no_threshold` なので、根は決着するまでループする。

### 閾値ポリシー

`P: ThresholdPolicy` は `VCTSolver<P>` の型パラメータで、3 つのポリシーはサイズ 0 の型である。`DFSVCTSolver`、`PNSVCTSolver`、`DFPNSVCTSolver` がエイリアスである。

| ポリシー | 子の閾値 | 効果 |
| --- | --- | --- |
| `DFSThreshold` | `no_threshold` | 選んだ子を決着まで探索してから親が次の子を見る: 素朴な深さ優先探索。証明数は手の並べ替えにだけ使われる |
| `PNSThreshold` | `(best_child.pn + 1, best_child.dn + 1)` | 子は数値が変わり次第戻るので、展開のたびにすべての階層で最有望の子が再選択される: 再帰の中の最良優先証明数探索 |
| `DFPNSThreshold` | OR ノード: `pn = min(threshold.pn, second_child.pn + 1)`、`dn = threshold.dn − node.dn + best_child.dn`。AND ノードはその鏡像 | df-pn（Nagai & Imai, 2002）: 最良の子が最良である限りそこに留まり、親自身の閾値を決して超えない |

### ソルバーの構造体

```rust
pub struct VCTSolver<P: ThresholdPolicy> {
    attacker_table: ProofTable,  defender_table: ProofTable,
    attacker_vcf: NestedVCF,     defender_vcf: NestedVCF,
    attacks_cache: LruCache<u64, Candidates>,  defences_cache: LruCache<u64, Candidates>,
    policy: PhantomData<P>,
}
```

状態はこれで全部である。上のメソッドはこの型への `impl` ブロックで、段階ごとに 1 ファイルに分かれている。`Solver`（04 の §4）を実装し、`solve` は両方の表と両方の内部ソルバーへの `advance_generation`、`search`、成功すれば無制限の予算での `extract` である。`clear` は 6 つのフィールドをすべて空にし、`memo_len` は 4 つのメモの合計を返す（キャッシュはすでに有界）。

## 6. 手順の復元（`extractor.rs`）

`search` は勝ちがあることを証明するだけである。`extract` は表をもう一度たどって手順を復元する:

```
extract_attacks(state):                        # 攻め方の手番
    Forced(p)  → p を打ち、extract_defences
    それ以外   → attacker_table のエントリが証明済みである最初の空点: 打って extract_defences
    なければ   → attacker_vcf.vcf(state)      # compute_attacks の四追いショートカットで証明されたノード。その手順が末尾

extract_defences(state):                       # 受け方の手番
    Defeated(end) → Mate { end, path: [] }
    Forced(p)     → p を打ち、extract_attacks
    それ以外      → threat_defences(attacker_vcf.threat(state)) のうち、証明済みで Node::limit が最小の子
    証明済みなし  → Mate { end: Unknown, path: [] }   # 合法な受けがなくて証明されたノード
```

証明済みの受けのうち `limit` が最小のものを選ぶのは、攻め方に最も多くの手を使わせた受けを選ぶことである。報告される手順は最も粘る受けに対するものになる。

## 7. 手順を追う

`test_vct_black`（岡部寛氏の五手詰め問題 No. 02）、黒番:

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

`solve(VCTDFS, 4, &board, Black, 1)` — `VCTPNS`、`VCTDFPNS` も同じ — は `F10,G9,I10,G10,H11,H12,G12`、詰め上がり `Fours(F13, K8)` を返す。`limit = 3` では `None` である。

| 手 | なぜ追い手 / 強制か | その後の `limit` |
| --- | --- | --- |
| `F10`（黒） | 三 `F10,_,H8,I7`: パスの後 `G9` が棒四、1 手の四追い | 4 |
| `G9`（白） | `threat_defences` の中: 狙われた四追いの手順 | 3 |
| `I10`（黒） | 三 `F10,_,H10,I10` | 3 |
| `G10`（白） | それを止める | 2 |
| `H11`（黒） | 四 `H8..H11` | 2 |
| `H12`（白） | `Forced` | 1 |
| `G12`（黒） | `G12,H11,I10,J9`、`F13` と `K8` が空: `Fours` | 1 |

根で `generate_attacks` は空点をポテンシャル（§8）で並べる: `I10`（18）、`G10`（16）、`G9`（13）、`F10`（12）、`I8`（12）、`H11`（10）、…。深さ優先モードはしたがってまず `I10` を試す。これは 1 手で反証される。`I10` は三を作らないので、受け方ノードで `attacker_vcf.threat` は 1 手の四追いを見つけず、`compute_defences` は `Terminal(disproven)` を返す。反証は `attacker_table` に入り、`select_attack` は次へ進み、やがて `F10` が証明される。

## 8. `PotentialField`（`src/analysis/field.rs`）

生成器は「ここに石を置くことは攻め方にどれだけ役立つか」をすべての空点について、安く、常に最新の状態で必要とする。`PotentialField` は点ごと・方向ごとに `u8` を 1 つ（`Potential { v, h, a, d }`）持ち、その合計を返す。

**方向ごとの値。** `Board::potentials(player, min, exact)`（02 の §7）から: その点を通る 5 マスの窓のうち相手の石を含まないもの（黒ではさらに `exact` — 余白に自分の石がないこと。あれば長連になる）について、そこに打った後に窓が含む自分の石の数を数え、`min` 以上なら残し、`max ×（その max に達した窓の数）` を報告する。`min = 2` では、4 マス以内に自分の石が 1 つあるだけで点が付く。

**最新に保つ。** `init(player, min, board)` が場を埋め、`update_along(p, board)` は `p` を通る 4 本の線をゼロにし（`reset_along`）、再計算する（`potentials_along`）。`VCTState` は play と undo のたびにこれを呼ぶ — 盤面全体の走査ではなく、1 手あたり 4 本の線の走査である。

**問い合わせ。** `get(p)` は 4 方向の合計、`collect(min)` は合計が `min` 以上のすべての点である。`VCTState::sorted_potentials` と `sort_by_potential` は降順に並べる。`min` が 2 つあることに注意: 構築時の `2` は窓を選び、問い合わせ時の `3` は合計を選ぶ。

**オーバーレイ。** `overlay(board)` はデバッグ用に場を描く（`.` = 0）。§7 の盤面で `PotentialField::init(Black, 2, ..)` すると:

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

`I10`（18）と `G10`（16）は黒の 2 本の線に同時に乗っている。候補リストの先頭に来るのはそのためである。**受け**を並べるときも場は**攻め方の**ものである。攻め方の最良点に着地する受けが先に試される。

## 9. チートシート

| 知りたいこと | 見る場所 |
| --- | --- |
| 探索が深さ N で止まった理由 | `limit` は攻め手を数える。`search_defences` は `limit ≤ 1` で `disproven` を返す |
| 追い手として認識されない | `compute_defences` のステップ 1、深さ `threat_limit` の `attacker_vcf.threat`。内部四追いは `Sword` の眼しか見ない |
| 受けが見つからない | `VCTState::threat_defences`: 手順、`end_breakers`、`counter_defences`、`four_moves` |
| 逆襲による反証が見つからない | `defender_vcf.vcf` は `defender_vcf_depth`（2）で抑えられる。より深い逆襲四追いは、ノリ手が `threat_defences` に入っている場合にしか見つからない。`SolveLimits::with_defender_vcf_depth` で深くできる |
| 手の並べ替え | `min = 2` の `PotentialField`。攻め手の候補には合計 `≥ 3` が要る |
| どのモードが何をするか | `threshold.rs` の `ThresholdPolicy`。それ以外はすべて共通 |
| 置換表 | `attacker_table` / `defender_table`（`ProofTable`）、2 つの `LruCache`、内部ソルバーの `deadends`。すべて `State::key()` がキーで、決着は局面だけ |
| 決着が limit をまたいで効くのがある深さからである理由 | `ProofTable` の `transfer_from`（§4） |
| 手順の復元 | `extract`。`End::Unknown` はたどれる証明済みの子がなかったことを意味する |
| 回帰テストを追加する | `solve.rs` に ASCII 盤面と期待する手順。関係する `SolveMode` ごとに 1 アサーション |

*来歴。* 追い手の検査を本探索に織り込む実験的な「lazy」追い詰めソルバー（`vct_lazy/`。長井歩, GPW 2011 による）がかつて `vct/` の隣にあった。[renju-note/quintet#133](https://github.com/renju-note/quintet/pull/133) で削除された。当時の状態と、削除の根拠になった計測はその PR に記録されている。

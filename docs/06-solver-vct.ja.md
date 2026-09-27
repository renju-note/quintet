# 追い詰め（VCT）探索（`src/mate/vct/`）

**追い詰め**（VCT: victory by continuous threats）は、攻め手がすべて**追い手**である手順。追い手とは、四、三、または「受け方が何もしなければ攻め方に四追いが生じる手」。四追いと違って受け方に選択肢があるので、木は本物の AND/OR 木になり、ソルバーは**証明数**で探索する。このドキュメントはその探索と、手の並べ替えに使う `ShapeMap` を説明する。

前提: [04](04-solver-framework.ja.md) と [05](05-solver-vcf.ja.md)。特に `State`、`Key`、`Memo`、`Solver`、`DFSSolver`。

```
src/mate/vct.rs         モジュールドキュメント: アルゴリズムの 1 ページ要約、再エクスポート、3 つのエイリアス
src/mate/vct/
├── state.rs            VCTState: Game + 攻め方 + limit + ShapeMap + SwordMap。                  (§2, §3, §8)
│                       threat_defences、sorted_attacks / sorted_defences、priority
├── nested_vcf.rs       NestedVCF: 片側の内部四追い探索                                          (§2)
├── generator.rs        generate_attacks / generate_defences → Candidates                        (§3)
├── proof.rs            PnDn（証明数）、ProofTable（置換表）                                      (§4)
├── searcher.rs         search_attacks / search_defences、expand_attacks / expand_defences、     (§5)
│                       select_attack / select_defence → Selection
├── threshold.rs        ThresholdPolicy: DFSThreshold、PNSThreshold、DFPNSThreshold              (§5)
├── solver.rs           VCTSolver<P>: 構造体、Solver の実装                                      (§5)
└── extractor.rs        extract: 表から詰み手順を復元する                                        (§6)
src/feature/shape.rs       ShapeMap: 点・プレイヤー・方向ごとに、石を置くと何ができるか             (§8)
```

探索の全体像。`solve` は根を証明し、次に証明をたどって手順を読み取る。

```
VCTSolver::solve = advance_generation; search; extract
│
├── search ──► search_attacks(root, no_threshold).is_proven()
│
│   search_attacks（OR ノード、攻め方の手番）                              §5
│   ├── check_event: Defeated → disproven; Forced(p) → attacks = [p]
│   ├── generate_attacks                                                  §3
│   │     attacker_vcf.vcf      四追いがある？           → Terminal(proven)。なければ関連領域
│   │     defender_vcf.threat   追い手を受ける必要がある？ → threat_defences に絞る
│   │     二以上ができる点、priority 順                                   §8
│   │     関連領域から追い手になりえない手を除く
│   └── expand_attacks: loop { select_attack; 最良の子を打つ; search_defences; attacker_table に保存 }
│
│   search_defences（AND ノード、受け方の手番）
│   ├── check_event: Defeated → proven; limit ≤ 1 → disproven; Forced(p) → defences = [p]
│   ├── generate_defences                                                 §3
│   │     attacker_vcf.threat   直前の手は追い手？        → でなければ Terminal(disproven)
│   │     defender_vcf.vcf      受け方が先に勝つ？        → Terminal(disproven)
│   │     threat_defences、priority 順                                    §8
│   └── expand_defences: loop { select_defence; 最良の子を打つ; search_attacks; defender_table に保存 }
│
└── extract ──► 表の証明済みの子をたどる                                   §6
```

- 証明数（§4）が、`select_*` の選ぶ子を決める。
- 閾値ポリシー（§5）が、`expand_*` がその子に留まる長さを決める。DFS / PNS / df-pn の違いはこれだけ。

## 1. 追い手と AND/OR 木

ある手が**追い手**であるとは、受け方がパスしたとき攻め方に `threat_limit` 個以下の四による四追いがあること。

| `threat_limit` | 追い手と認識されるもの |
| --- | --- |
| 0 | 四だけ。内部四追いの深さが 0 なので追い手の判定は常に失敗し、受け方が `Forced`（四を止めるしかない）のノードだけが先へ進む |
| 1 | 加えて三。パスの後に棒四を作る手が、1 手の四追いになる |
| 2 | 加えて、四 2 つの四追いを準備する手（四三を用意する手など） |
| 3 以上 | さらに深い含み手。`test_vct_fukumi_move` は 3 が必要 |

受け方は、狙われた四追いを止める手なら何でも打てる。ノリ手や、受け方自身の逆襲四追いも含む。

木は 2 種類のノードが交互に現れる。

- **OR ノード**（攻め方の手番）: 勝つ攻め手が 1 つあればよい。
- **AND ノード**（受け方の手番）: すべての受けが負ける必要がある。

## 2. `VCTState` と内部の四追い探索

```rust
pub struct VCTState {
    game: Game, pub attacker: Player, pub limit: u8,
    shapes: ShapeMap, swords: SwordMap,
}
```

- `shapes` は両プレイヤーの `ShapeMap`（§8）。`after_play` / `after_undo` は着手の線に「古い」印を付けるだけ。次に読まれるとき（`sorted_attacks` / `sorted_defences`）に計算し直す。読まれるのは、候補手がキャッシュになかったときだけ。
- `VCTState` は `SwordMap`（02 §7）も同じように持つ。`vcf_state` / `threat_state` はそれを同期してからクローンを内部の `VCFState` に渡す。内部の四追いは同期済みの状態から始まり、次の四追いもこの計算を再利用できる。
- `next_key(m)` は `m` を打った後の子の `key()` を、`m` を打たずに盤面の Zobrist ハッシュへの XOR で計算する。未展開の子の表引きが安いのはこのため。

### 派生する 2 つの `VCFState`

追い詰め探索は絶えず「ここに四追いがあるか」を問う。そのために `VCTState` から 2 種類の四追い状態を派生させる。

| メソッド | 局面 | 四追いの攻め方 | 四追いの limit | 問い |
| --- | --- | --- | --- | --- |
| `vcf_state(max)` | そのまま | 手番側 | `min(limit, max)` | 手番側は今すぐ四追いで勝てるか |
| `threat_state(max)` | 手番側がパスした後 | 相手側 | 攻め方の手番なら `min(limit − 1, max)`、そうでなければ `min(limit, max)` | 何もしなければ相手に四追いがあるか |

### `NestedVCF`: 各側に 1 つ

`nested_vcf.rs` の `NestedVCF` は、四追いソルバー・深さの上限・どちら側のものかをまとめたもの。

```rust
pub struct NestedVCF { solver: IDDFSSolver, depth: u8, for_attacker: bool }
impl NestedVCF {
    pub fn vcf(&mut self, state: &mut VCTState, budget) -> Option<Mate>;     // この側の手番。今すぐ四追いがあるか
    pub fn vcf_or_zone(&mut self, state: &mut VCTState, budget) -> Result<Mate, Area>; // 同じ問い。なければどこに石を置けば生まれうるか
    pub fn threat(&mut self, state: &mut VCTState, budget) -> Option<Mate>;  // 相手がパスしたら、この側に四追いがあるか
}
```

`VCTSolver` は `attacker_vcf`（深さ `threat_limit`）と `defender_vcf`（深さ `defender_vcf_depth`、既定 2）を持つ。2 つ × 2 メソッドで、生成器が使う 4 つの問いになる。

| 呼び出し | 手番 | 問い |
| --- | --- | --- |
| `attacker_vcf.vcf` | 攻め方 | 攻め方は今すぐ四追いで勝てるか |
| `defender_vcf.threat` | 攻め方 | 攻め方がパスしたら受け方に四追いがあるか（この攻め手は何を同時に受ける必要があるか） |
| `attacker_vcf.threat` | 受け方 | 受け方がパスしたら攻め方に四追いがあるか（直前の攻め手は追い手か） |
| `defender_vcf.vcf` | 受け方 | 受け方は今すぐ四追いで勝てるか（受け方が先に勝つか） |

- `vcf` は自分の側の手番であること、`threat` は相手の手番であることを assert する。逆向きの呼び出しは探索のバグ。
- 中身は `IDDFSSolver`（`limits = [1]`）。`solve` ではなく `search` で呼ぶので、行き止まりメモ（04 §5）は追い詰め探索の間ずっと（そして探索をまたいでも）残る。予算は追い詰めと同じ `NodeBudget` から消費する。

## 3. 手の生成（`generator.rs`）

`generate_attacks` と `generate_defences` は `Candidates` を返す。

```rust
pub enum Candidates {
    Moves(Vec<Candidate>),  // 内部ノード: 試す手、良い順
    Terminal(PnDn),         // 展開せずに決着: PnDn::proven または PnDn::disproven
}
pub struct Candidate { pub point: Point, pub estimate: u32 }
```

`Candidate` の `estimate` は、表にまだ何もない間その子が始める数値。攻め手なら証明数、受け手なら反証数（§5「選択」）。

結果は生成器ごとに 65,536 エントリの `LruCache` に入れる。キーは `zobrist_hash()`（局面 + limit。内部四追いの深さが `min(limit, depth)` なので limit が要る）。予算切れのときは入れない。

df-pn は同じノードに何度も戻ってくる。容量 1000 では、攻め方ノードでの参照の 3 分の 1、受け方ノードでの参照の半分以上がミスし、ミスのたびに内部四追いも含めて手を生成し直していた。ミスの多くは初めて出会う局面で、これはどんな容量でも防げない。1 エントリの大きさは、攻め手のキャッシュで約 280 バイト（エントリ 48、表のスロット 34、約 30 手 × 8 バイト。6 分の 1 は `Terminal` で手を持たない）、受け手のキャッシュで約 110 バイト（約 6 手。半分以上が `Terminal`）。ベンチマーク（07）の全 121 ケースで:

| 容量 | 2 つとも満杯のとき | ノード数 | 時間 |
| --- | --- | --- | --- |
| 1000 | 0.4 MB | | |
| 4096 | 1.6 MB | −5% | −4% |
| 16384 | 6 MB | −3% | −6% |
| **65536** | **25 MB** | **−7%** | **−10%** |
| 262144 | 100 MB | −8% | −10% |
| 1048576 | 400 MB（実際は `vct_unstable` の 220 MB が最大） | −8% | −12% |

65536 でキャッシュが満杯になるのは 121 ケース中 18 ケースで、どれも 100 万ノードを超え、キャッシュを除いてもプロセスが 75 MB〜1 GB を使うケースである。キャッシュは空で始まり、埋まるにつれて大きくなる（`candidates_cache`）。そのため小さな探索は容量の分を払わない。`LruCache::with_hasher` だと表全体を最初に確保してしまう。キャッシュを大きくすると、ノード数だけでなく手順が変わるケースもある。生成器が返すものは内部四追いソルバーのメモの中身に左右されうる（dead end は、それを記録した探索のゾーンを返す）ので、手を生成し直したノードは最初と違う手を見ることがあるからである。

**`compute_attacks`**（攻め方の手番）

1. `attacker_vcf.vcf_or_zone`: 今すぐ四追いがあれば `Terminal(proven)`。正しさのためには不要（本探索でも `Forced` をたどれば四追いは見つかる）だが、大幅に速くなる。なければ、その探索の関連領域（05 §2）を返す。攻め方がもう 1 石置けば四追いが生まれうる点の集合である。
2. `defender_vcf.threat`: 攻め方がパスすると受け方に四追いがあるなら、攻め手はそれも受ける必要がある。候補を `threat_defences(threat)` に絞る。
3. 候補 = `ShapeMap` で攻め方がどれかの線に `Two` 以上を作る点（`sorted_attacks(only)`）を `priority`（§8）の順に並べたもの。その数が `width`。
4. そのうち追い手になりうる手（`may_threaten`）だけを残す: 関連領域にあるか、攻め方の石をすでに 2 つ含む区間にある手（`ShapeMap` でどれかの線が `Sword` 以上、§8。関連領域が呼び出し側に任せる部分）。それ以外の手は、パスされても攻め方に四追いを生まないので追い手ではない。予算切れのときは関連領域が不完全なので行わない。
   空なら `Terminal(disproven)`。
5. 各候補に、それが作るいちばん大きな形（`best_shape`。`ShapeMap` から、§8）から見積もりを付ける。四なら `width / 4`、三なら `width / 2`、それ以外は `width`（§5「選択」）。

それ以外に、ここでは候補が追い手かどうかを調べない。1 手下の受け方ノードで調べ、追い手でなければそこで反証される。その都度内部四追い探索とノード 1 つがかかるが、ベンチマークでは、ステップ 4 は追い手でない手のおよそ半分についてそれを省く（関連領域の外に追い手があったことはない）。

`width` をステップ 4 の前に取るのは、それが未展開の子の証明数の初期値になるから（§5）。追い手でない手は反証されるだけで、兄弟の証明を易しくはしない。除いた後の数を初期値にすると、探索は本来と違う手順に傾く。

**`compute_defences`**（受け方の手番）

1. `attacker_vcf.threat`: 受け方がパスしても攻め方に四追いがなければ、直前の攻め手は追い手ではない。`Terminal(disproven)`。
2. `defender_vcf.vcf`: 受け方自身に（`defender_vcf_depth` 以内の）四追いがあれば、受け方が先に勝つ。`Terminal(disproven)`。
3. 候補 = `threat_defences(threat)` を `priority`（§8）の順に並べ（`sorted_defences`）、禁手を除いたもの。空なら `Terminal(proven)`（追い手に応手がない）。
4. 各候補の見積もりは候補の数 `n`。

### `threat_defences`

`VCTState::threat_defences(&threat)` は、狙われた四追いを止めうる手のリスト（ヒューリスティック）。次の順に並ぶ。

| 部分 | 点 |
| --- | --- |
| 手順 | 四追い手順のすべての石（攻め方の四と受け方の止め）。どれかを占めれば手順が崩れる |
| `end_breakers(end)` | `Fours(p1, p2)` なら 2 つの勝ち点。`Forbidden(p)` ならその点と、4 本の線に沿って 5 以内のすべての空点（`neighbors(p, 5, true)`）。近くの石で `p` の禁手判定が変わりうるため |
| `counter_defences(threat)` | パスの後で四追い手順を再生し、受け方の各止めについて、その止めを通る受け方の `Sword` の眼を集める。手順の途中で受け方が四を作れる点で、今打てばノリ手になりうる |
| `four_moves()` | 受け方が今持っている四を作る手（`Sword` の眼）。攻め方に応手を強いるノリ手 |

同じ点が重複することがある。`sorted_defences` はそれぞれ最初の 1 つだけを残す。

## 4. 証明数（`proof.rs`）

```rust
pub struct PnDn { pub pn: u32, pub dn: u32, pub limit: u8 }
pub const INF: u32 = u32::MAX;
```

- `pn`（証明数）: このノードを証明するために、少なくともあと何個の葉を勝つ必要があるか。`pn == 0` で証明済み。
- `dn`（反証数）: 反証について同じ。`dn == 0` で反証済み。

| コンストラクタ | `(pn, dn)` | 意味 |
| --- | --- | --- |
| `PnDn::unknown()` | `(INF, INF)` | 表にない局面 |
| `PnDn::no_threshold()` | `(INF, INF)` | 決して超えない閾値（決着まで探索）。`unknown` と同じ値、別の役割 |
| `PnDn::proven(limit)` | `(0, INF)` | 攻め方の勝ち |
| `PnDn::disproven(limit)` | `(INF, 0)` | `limit` 以内に攻め方は勝てない |
| `PnDn::unexpanded_defence(n, limit)` | `(n, 1)` | OR ノードの未展開の子（受け方の手番）の初期値。`n` = その攻め手の見積もり（§3） |
| `PnDn::unexpanded_attack(n, limit)` | `(1, n)` | AND ノードの未展開の子（攻め方の手番）の初期値。`n` = その受け手の見積もり |

子の合成は 2 通り（和は飽和加算）。

- **OR** ノード: `min_pn_sum_dn`。証明のコストは最も安い子の分（pn の min）、反証にはすべての子の反証が必要（dn の和）。
- **AND** ノード: `min_dn_sum_pn`。その逆。

根から、OR ノードでは最小の `pn`、AND ノードでは最小の `dn` の子をたどると、結果が根に最も影響する葉（**最有望ノード**）に着く。選択器はこれをたどる（§5）。

`limit` は子の最小値として親へ伝わる。部分木の中で決着した地点に残っていた limit の最小値で、手順の復元に使う（§6）。

### `ProofTable`

ソルバーは 2 つの表を持つ。

- `attacker_table`: 攻め手で到達したノード（受け方の手番）。
- `defender_table`: 受け手で到達したノード（攻め方の手番）。

それぞれが `ProofTable` で、中身は `Memo` 2 つ。

| フィールド | キー | 持つもの |
| --- | --- | --- |
| `estimates: Memo<PnDn>` | `Key::hash()`（局面 + limit） | 挿入されたすべてのノードをそのまま。途中経過の証明数は、それを計算した limit に属する |
| `decided: Memo<Decided>` | `Key::position` のみ | その局面で証明された最小 limit と、反証された最大 limit |

- `insert(state, node)`: `estimates` には常に書く。ノードが証明済み / 反証済みなら `decided` にも書く。
- `lookup_next(state, m)`: `m` の後の子を `next_key` で引く。まず `estimates`、次に `decided`。`decided` では、より小さい limit での証明や、より大きい limit での反証も答えになる（04 §3）。これにより limit 5 の探索は limit 4 の結果を再利用でき、ある根で見つけた証明は別の根でも使える。
- `lookup_children(state, candidates)`: 候補それぞれに `lookup_next` したもの。展開はここから始まる（§5「展開」）。
- `transfer_from`: `decided` を使い始める limit。生成器は内部四追いを `min(limit, depth)` の深さで問うので、limit がそれより小さいと候補リストが limit によって変わり、limit ごとに別の木になる。値は `VCTSolver::with_carry_capacity` が `max(attacker_vcf_depth, defender_vcf_depth + 1, 2)` に設定する。limit がこれより小さいノードは `decided` に書きも読みもしない。

## 5. 探索（`searcher.rs`、`threshold.rs`、`solver.rs`）

### ノード関数

`VCTSolver::search(state, budget)` は、`limit == 0` の検査の後、`search_attacks(state, PnDn::no_threshold(), budget).is_proven()` を返す。2 つのノード関数は互いを呼ぶ。

```
search_attacks(state, threshold):              # OR ノード、攻め方の手番
    budget.consume() できなければ → unknown
    Defeated(_)  → disproven(limit)
    Forced(p)    → expand_attacks([p])
    それ以外     → generate_attacks: Terminal(n) → n | Moves(a) → expand_attacks(a)

search_defences(state, threshold):             # AND ノード、受け方の手番
    budget.consume() できなければ → unknown
    Defeated(_)  → proven(limit)                # 攻め方の勝ち
    limit ≤ 1    → disproven(limit)             # この受けの後、攻め手が残らない
    Forced(p)    → expand_defences([p])
    それ以外     → generate_defences: Terminal(n) → n | Moves(d) → expand_defences(d)
```

### 選択

`select_attack(limit, attacks, children)` は何も展開しない。攻め方の表にある子の数値（下の展開が持っている `children`）を読んで `Selection` を返す。

| フィールド | 内容 |
| --- | --- |
| `node` | このノード自身の数値。子に対する `min_pn_sum_dn`。表にない子は `unexpanded_defence(estimate)` とみなす（§3。強制手なら `1`） |
| `best` | `pn` が最小の子（最有望の子）。候補の中での位置 |
| `unvisited` | `best` がまだ表にないか。一度も探索されていない |
| `best_child`、`second_child` | 最良と 2 番目の子の数値 |

証明済みの子があれば、`node` はその場で `(0, INF)` になる。

`select_defence` はその逆: 最小の `dn`、`min_dn_sum_pn`、表にない子は `unexpanded_attack(estimate)`、`node.limit` は `limit − 1`。

見積もり（§3）は意図的な仕掛け。兄弟の数から始めるので、候補が少ないノードほど易しく見え、探索は強制的な手順を好む。攻め手の見積もりはさらに、その手が受け方の応手をどれだけ絞るかで下げる。四は応手を 1 つにするので兄弟の数の 4 分の 1 から、三は数手にするので半分から始める。こうして探索は、候補の並び順のどこにあっても、四と三をより深く追ってから静かな攻め手に移る。

ベンチマーク（07）で、兄弟の数だけの場合と比べて試したこと:

- 効果の大半は四の優遇によるもので、唯一の代償もそこから来る。四を優遇しない（三だけ半分にする）と `heavy` のケースは 17% 高くついた。優遇すると、ほかの重みによらず `vct_small_but_long` が約 3 倍高くつく。`limit` 255 では、探索が四をどこまでも深く追うのを止めるものがない。
- それ以外の攻め手の見積もりを兄弟の数より上げる（1.5 倍にする）のは、追い手になりえない攻め手を除く前は得になったが、除いてからは損になるだけだった。
- 証明数はノードをまたいで比べられるので、比だけでなく尺度も効く。すべての見積もりを一律に半分にすると探索は高くつく。
- 受け手を、それが作る四や三で重み付けしても得にならなかった。

この重みで、`heavy` を含む全ケースのノード数は 38%、`heavy` のケースは 47% 減る（`vct_unstable`: 109.5M → 41.6M）。デフォルトセットの合計は `vct_small_but_long`（5.8M → 16.7M）のために 6% 増えるが、このケースを除けば 26% 減る。

### 展開

`expand_attacks` は 3 モード共通のループ。

```
expand_attacks(state, attacks, threshold):
    children = attacker_table.lookup_children(state, attacks)
    loop:
        s = select_attack(limit, attacks, children)
        if s.node.pn ≥ threshold.pn or s.node.dn ≥ threshold.dn: return s   # exceeds_threshold
        if 予算切れ:                                              return s
        if s.unvisited かつ s.best が禁手:
            with_move(s.best): attacker_table.insert(child, disproven)
            children[s.best] = disproven
            continue
        next = P::next_threshold_attack(s, threshold)
        with_move(s.best):
            result = search_defences(child, next)
            if 予算が残っていれば: attacker_table.insert(child, result)
        children[s.best] = result
```

子の数値は展開の始めに一度だけ表から読み、その後ループは直前に探索した子の値だけを書き戻す。これは毎回表を読み直した場合とまったく同じ値になる。ある子を探索しても兄弟のエントリは変わらないからである。その子の下の局面はすべてその子の石を含み、兄弟の局面は含まない（VCT の探索はパスしない）。毎周すべての子の表を引き直していた頃は、`vct_small_but_long` のプロファイルでその表引きが時間の 1 割を占めていた。一度だけ読むようにすると、ノード数は変わらずにベンチマーク全体の時間が 4%（既定の集合で 5%）減る。

`compute_attacks` は禁手を候補から除かない。禁手は少なく（ベンチマークでおよそ 300 に 1 つ）、ほとんどの候補は一度も探索されないからである。攻め手が禁手かどうかは、それを初めて探索しようとするときにだけ確かめ、禁手ならノードを使わずに表で反証済みとする。それまでは、ほかの未展開の子と同じく親の数値に数えられる。そのため探索は、禁手を事前に除いていたときとは違う手順をたどる。

`expand_defences` は `select_defence`、`defender_table`、`P::next_threshold_defence`、`search_attacks` で同じことをする。ノードは、自分の数値が親から渡された閾値を超えるまで、最有望の子を展開し続ける。根の閾値は `no_threshold` なので、根は決着するまでループする。

閾値は最初の展開の前にも確かめる。初期値の時点で閾値を超えているノードは、手を生成しただけで何も展開せずに戻り、親はそれで自分の数値を知る。

#108 から #110 まで（2022 年）の `expand_attacks` はこの最初の確認を飛ばし、最有望の攻め手を必ず 1 回展開していた。これが得になるかは局面によって大きく違う。ベンチマーク（07）では、`vct_unstable` は 345 分の 1（119.5M ノード → 346k）になるが、既定の集合は 39%（24.4M → 34.0M）、`vct_black_long_short` は 53%（17.4M → 26.7M）重くなる。そのためこの動作は採用していない（issue #148）。

### 閾値ポリシー

`P: ThresholdPolicy` は `VCTSolver<P>` の型パラメータ。3 つのポリシーはサイズ 0 の型で、`DFSVCTSolver`、`PNSVCTSolver`、`DFPNSVCTSolver` がそのエイリアス。

| ポリシー | 子の閾値 | 効果 |
| --- | --- | --- |
| `DFSThreshold` | `no_threshold` | 選んだ子を決着まで探索してから、親が次の子を見る。素朴な深さ優先探索。証明数は手の並べ替えにだけ使う |
| `PNSThreshold` | `(best_child.pn + 1, best_child.dn + 1)` | 子の数値が変わるとすぐ戻る。展開のたびに全階層で最有望の子を選び直す。再帰で書いた最良優先の証明数探索 |
| `DFPNSThreshold` | OR ノード: `pn = min(threshold.pn, second_child.pn + 1 + second_child.pn / 8)`、`dn = threshold.dn − node.dn + best_child.dn`。AND ノードはその逆 | df-pn（Nagai & Imai, 2002）。最良の子が（少しの余裕をもって）最良である限りそこに留まり、親の閾値は超えない |

`second_child.pn / 8` の余裕は、ε = 1/8 の「1 + ε トリック」（Pawlewicz & Lew, 2006）である。素の df-pn では最良の子は 2 番目の子より 1 だけ大きくなるまでしか探索されないので、両者の数値が近いと探索は 2 つの子の間を行き来し、そのたびに親まで戻ってまた降りる。余裕を持たせると、今いる子にもう少し長く留まる。ベンチマーク（07）では、`heavy` を含む全ケースのノード数が 137.3M から 99.4M（−28%）になった。既定の集合で −19%、`vct_unstable` 以外の `heavy` で −27%、`vct_unstable` は 40.2M → 25.4M。6〜14% 重くなったケースが 3 つあるが、どれも 170k ノード以下である。

ほかに試した ε（全ケースでのノード数の変化）:

| ε | 全ケース | 既定の集合 | `vct_small_but_long` | `vct_unstable` |
| --- | --- | --- | --- | --- |
| 1/16 | −14% | −8% | +5% | −7% |
| 1/8 | −28% | −19% | −14% | −37% |
| 1/6 | −20% | −18% | −9% | −7% |
| 1/4 | −8% | −25% | −25% | +47% |
| 1/3 | −25% | −11% | +11% | −23% |
| 1/2 | −10% | −25% | −22% | +41% |

1/4 以上では、この 2 つ以外のケースがもう少し減る（1/8 の約 −26% に対して約 −32%）。しかしこの 2 つは大きく振れる。どちらも探索が長い四の連続をたどるケースで、展開の順序が少し変わるだけで別の手順に入ってしまう。両方でよい結果になるのが 1/8 である。

### ソルバーの構造体

```rust
pub struct VCTSolver<P: ThresholdPolicy> {
    attacker_table: ProofTable,  defender_table: ProofTable,
    attacker_vcf: NestedVCF,     defender_vcf: NestedVCF,
    attacks_cache: LruCache<u64, Candidates>,  defences_cache: LruCache<u64, Candidates>,
    policy: PhantomData<P>,
}
```

状態はこれだけ。メソッドは段階ごとに 1 ファイルずつ、この型への `impl` ブロックとして書かれている。`Solver`（04 §4）の実装は次のとおり。

- `solve`: 両方の表と両方の内部ソルバーに `advance_generation` → `search` → 成功なら無制限の予算で `extract`。
- `clear`: 6 つのフィールドをすべて空にする。
- `memo_len`: 4 つのメモの合計（キャッシュはもともと有界）。

## 6. 手順の復元（`extractor.rs`）

`search` は勝ちの存在を証明するだけ。`extract` が表をたどって手順を復元する。

```
extract_attacks(state):                        # 攻め方の手番
    Forced(p)  → p を打ち、extract_defences
    それ以外   → attacker_table で証明済みの最初の空点を打ち、extract_defences
    なければ   → attacker_vcf.vcf(state)      # compute_attacks の四追いショートカットで証明されたノード。その手順が末尾

extract_defences(state):                       # 受け方の手番
    Defeated(end) → Mate { end, path: [] }
    Forced(p)     → p を打ち、extract_attacks
    それ以外      → threat_defences(attacker_vcf.threat(state)) のうち、証明済みで PnDn::limit が最小の子
    証明済みなし  → Mate { end: Unknown, path: [] }   # 合法な受けがなくて証明されたノード
```

証明済みの受けのうち `limit` が最小のものを選ぶ = 攻め方に最も多くの手を使わせた受けを選ぶ。報告される手順は、最も粘る受けに対するものになる。

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

`solve(VCTDFS, &board, Black, SolveLimits::new(4).with_threat_limit(1))`（`VCTPNS`、`VCTDFPNS` も同じ）は手順 `F10,G9,I10,G10,H11,H12,G12`、詰め上がり `Fours(F13, K8)` で証明する。`limit = 3` では `Disproven`。

| 手 | なぜ追い手 / 強制か | その後の `limit` |
| --- | --- | --- |
| `F10`（黒） | 三 `F10,_,H8,I7`。パスの後 `G9` で棒四になる = 1 手の四追い | 4 |
| `G9`（白） | `threat_defences` に含まれる（狙われた四追いの手順上の点） | 3 |
| `I10`（黒） | 三 `F10,_,H10,I10` | 3 |
| `G10`（白） | 止め | 2 |
| `H11`（黒） | 四 `H8..H11` | 2 |
| `H12`（白） | `Forced` | 1 |
| `G12`（黒） | `G12,H11,I10,J9`。`F13` と `K8` が空 = `Fours` | 1 |

根で `generate_attacks` は候補を `priority`（§8）順に並べる: `H11`（10: 四と二で 5、四で 5）、`H12`（9）、`F10`（6: 三と二で 4、三で 2）、`G9`、`J6`、`K5`（各 5）、…。深さ優先モードはまず `H11` を試す。

白の `H12` は必然で、その四を使ってしまった黒には残り 3 手の追い詰めがないので、`H11` は反証される。反証は `attacker_table` に入り、`select_attack` は次の候補に進む。`H12` も同じく実らず、`F10` が証明される。

## 8. 手の並べ替え（`src/feature/`、`VCTState::priority`）

見積もり（§3）が同じ候補の間では、候補の順序が、探索がどの子を最初に展開するかを決める。点の特徴は `src/feature/` のキャッシュが表し、`VCTState::priority` がそれを重み付けする。

### `ShapeMap`（`src/feature/shape.rs`）

`ShapeMap` は、プレイヤーごと・空点ごとに、そこに石を置くと通る 4 本の線それぞれに何ができるか（`Shape`）を持つ:

| `Shape` | その点は | 求め方（02 §3.1） |
| --- | --- | --- |
| `Five` | `Four` の目 | `row_eyes(r, Four)` |
| `Four` | `Sword` の目 | `row_eyes(r, Sword)` |
| `Three` | `Two` の目 | `row_eyes(r, Two)` |
| `Sword` | 石を 2 つ含むセグメントの中 | `eyes_of(score_mask(r, 2), ..)` |
| `Two` | 石を 1 つ含む開いたセグメント対の共有マス | `eyes_of(open_starts(r, 1), ..)` |
| `Nothing` | 上のどれでもない、または石がある | |

当てはまるうち最も大きいものを取る。`SwordMap`（02 §7）と同じく線ごとに持ち、着手で古い印を付けて `sync` で計算し直す。線・プレイヤーごとに数回のビット演算で済む。

`get(p, r)` は 4 方向分を `Shapes` として返す。`Shapes` は数え（`count`、`count_at_least`）、格の合計を出し（`total`: `Two` の 1 から `Five` の 5 まで）、1 方向を除き（`except`）、黒がそこに打てなさそうかを推測する（`looks_forbidden`: 四が 2 つか三が 2 つで、五がない。1 本の線上の四四、長連、三が偽であることは見ない）。`forbidden_eyes(board, p, r)` は、`r` が `p` に打って作る四・三のうち、黒の石が禁手に見える点に目が残るものを数える。白にとっては黒が止められない四、止め方の減る三。黒にとっては、そこで棒四にできない三。`p` の石は、目を通る線のうち連の線にしか乗らないので、ほかの線の黒の形はそのまま読める。

### `priority`

`sorted_attacks` / `sorted_defences` は `VCTState::priority(p)` の高い順に並べる。同じ値のものは元の順を保つ。値は手番側にとってその手が作るもので、人が手を見積もるやり方にならっている:

| 項 | 値 | 理由 |
| --- | --- | --- |
| 攻めのみ: 各方向の形の格 | `Shapes::total` | 次の追い手は二や剣先からも作られる |
| `Four` 1 つごと | +5 | 相手の応手を 1 つに限定する |
| `Three` 1 つごと | +2 | 相手の応手を数手に限定する |
| `Sword` 以上になる方向、2 つ目から 1 つごと | +5 | 複数の線で同時に脅かす（四三、両狙い） |
| 白: 黒が禁手に見える点に目がある四 / 三 1 つごと | +20 / +10 | 黒はそこで受けられない |
| 黒: 達四点が禁手に見える三 1 つごと | −2 | その三は偽かもしれない |
| 黒: その点自体が `looks_forbidden` | −20 | 本当に禁手なら黒は打てない。それが分かるのは初めて探索するとき（§5）。禁手でなければ、三は見かけより少ない |

受けでは最初の項を除く。ここでの受けはどれも何らかの形で追い手を止める（`threat_defences`）。それらを攻め方がその点をどれだけ欲しいか（そこでの攻め方の石の数や形）で並べると、ベンチマーク（07）ではこれらの項だけの場合より探索が大きくなった。重みもベンチマークで調整した。

最初の項は `PotentialField`（点ごとに、そこを通る生きたセグメントにある攻め方の石を数えたもの）を置き換えたもので、以前は受けもこれで並べていた。上の四や三の項がある今では攻め手の順序をほとんど変えず、受けはないほうがよく、盤面に合わせて更新するのに探索時間の約 5 分の 1 を使っていた。

## 9. チートシート

| 知りたいこと | 見る場所 |
| --- | --- |
| 探索が深さ N で止まった理由 | `limit` は攻め手数。`search_defences` は `limit ≤ 1` で `disproven` |
| 追い手として認識されない | `compute_defences` のステップ 1（`attacker_vcf.threat`、深さ `threat_limit`）。内部四追いは `Sword` の眼しか見ない |
| 受けが見つからない | `VCTState::threat_defences`: 手順、`end_breakers`、`counter_defences`、`four_moves` |
| 攻め手が見つからない | どれかの線に `Two` 以上（`sorted_attacks`）、次に `DFSSolver::search_zone` の関連領域に対する `VCTState::may_threaten`（05 §2） |
| 逆襲による反証が見つからない | `defender_vcf.vcf` の深さは `defender_vcf_depth`（2）。より深い逆襲四追いは、ノリ手が `threat_defences` に入っている場合しか見つからない。`SolveLimits::with_defender_vcf_depth` で深くできる |
| 手の並べ替え | `ShapeMap` の上の `VCTState::priority`（§8）。攻め手の候補はどれかの線に `Two` 以上 |
| モードごとの違い | `threshold.rs` の `ThresholdPolicy`。それ以外は共通 |
| 置換表 | `attacker_table` / `defender_table`（`ProofTable`）、2 つの `LruCache`、内部ソルバーの `dead_ends`。キーはすべて `State::key()`。決着は局面のみ |
| 決着が limit をまたいで効くのが一定の深さ以上である理由 | `ProofTable` の `transfer_from`（§4） |
| 手順の復元 | `extract`。`End::Unknown` = たどれる証明済みの子がなかった |
| 回帰テストの追加 | `solve.rs` に ASCII 盤面と期待する手順。関係する `SolveMode` ごとに 1 アサーション |

*来歴*: 追い手の判定を本探索に織り込む実験的な「lazy」追い詰めソルバー（`vct_lazy/`、長井歩 GPW 2011 に基づく）がかつて `vct/` の隣にあった。[renju-note/quintet#133](https://github.com/renju-note/quintet/pull/133) で削除。当時の状態と削除の根拠となった計測はその PR にある。

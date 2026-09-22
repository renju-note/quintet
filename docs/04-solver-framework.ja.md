# `src/mate/` の内側: 探索の骨格

`src/mate/` 以下を変更する前に読む。すべてのソルバーが共有する部品 — ゲーム、状態、メモ、`Solver` トレイト — と、それらが従う規則を説明する。2 つの探索そのものは [05](05-solver-vcf.ja.md)（四追い）と [06](06-solver-vct.ja.md)（追い詰め）にある。

[03](03-solver-api.ja.md)（ソルバーへの問い、`limit`、`NodeBudget`）と、[02](02-board-implementation.ja.md) の盤の用語（`Four`、`Sword`、`eyes()`、禁手、`zobrist_hash`）を前提にする。

## 1. 構成

```
src/mate.rs          モジュールルート: 再エクスポート、全体像のドキュメントコメント
src/mate/
├── solve.rs         solve / solve_limited、SolveMode、SolveLimits、SolveResult、validate
│                    + ソルバーの回帰テスト
├── solver.rs        trait Solver                      (§4)
├── game.rs          Game、Event、End                  (§2)
├── state.rs         trait State、Key                  (§3)
├── memo.rs          Memo<V>: 世代付きメモ             (§5)
├── budget.rs        NodeBudget                        (§6)
├── mate.rs          Mate: 結果
├── vcf.rs, vcf/     四追いソルバー                    (05)
└── vct.rs, vct/     追い詰めソルバー                  (06)
src/analysis/field.rs   PotentialField: 追い詰めの手の並べ替え (06 の §8)
```

下から上へ、どう組み合わさるか:

```
Board  ──►  Game  ──►  State (VCFState | VCTState)  ──►  Solver::solve(state, budget) ──► Option<Mate>
            石、       + 攻め方、残り limit、             Memo<V> を読み書きする。
            手順、     + メモ用の Key                      キーは State::key()
            手番
```

- **`Game`** は盤面と、探索中にそこへ打たれた手である。
- **`State`** はゲームに探索が必要とするものを足したもの。どちら側のための探索か、攻め手はあと何手か、そして追い詰めでは手の並べ替えのためのポテンシャル場。
- **`Solver`** は状態を探索する。問いをまたいで生きる **`Memo`** を所有し、そのキーはすべて状態の **`Key`** である。
- **`NodeBudget`** は 1 つの問い、あるいはその連なりが使ってよい仕事量を抑える。

## 2. `Game`: 探索中の盤面

```rust
pub struct Game { board: Board, moves: Vec<Option<Point>>, pub turn: Player }
```

`Game::init(&board, turn)` が盤面を 1 回だけクローンする。それ以降、探索はクローンしない。`play(m)` は石を置いて `turn` を反転し、`undo()` はそれを戻し、`into_play(m, f)` は play → `f(self)` → undo をして `f` の結果を返す。すべてのソルバーは `into_play` で木を歩く。

手は `Option<Point>` である。`play(None)` は**パス**で、手番だけ反転し盤面は変わらない。追い手の定義は「自分が何もしなければ相手は何ができるか」なので、連珠にパスはないがここでは一級の手である。

### `check_event`: 盤上にすでにある四

どのノードも、何かを生成する前に `Game::check_event()` を問う。**直前に打った側**の四は、**これから打つ側**に何を意味するか？

| 相手の四の勝ち点が… | `Event` | これから打つ側にとって |
| --- | --- | --- |
| 異なる 2 点 `p1`、`p2` | `Defeated(Fours(p1, p2))` | 負け — 1 手では両方止まらない |
| 1 点 `p` で、それが自分の禁手 | `Defeated(Forbidden(p))` | 負け — 唯一の止めが打てない |
| 1 点 `p` | `Forced(p)` | `p` を打たなければならない |
| なし | `None` | 自由に選べる |

見るのは直前の手を通る四だけである（`structures_on(last_move, opponent, Four)`）。それより古い四はすでに応手を強いていたはずだからだ。パスの後は直前の手がないので、代わりに相手のすべての四を走査する。棒四は眼の異なる 2 つの `Four` として現れ、四四と同じ 2 点判定（`take_distinct_two`）で処理される。

`End` は `Defeated` のペイロードで、`Mate` の末尾になるものである（03 の §5）。

## 3. `State`: limit とキー

```rust
pub trait State {
    fn game(&self) -> &Game;        fn game_mut(&mut self) -> &mut Game;
    fn attacker(&self) -> Player;
    fn limit(&self) -> u8;          fn set_limit(&mut self, limit: u8);
    // 提供メソッド:
    fn play(&mut self, m: Option<Point>);   fn undo(&mut self);   fn into_play(..);
    fn attacking(&self) -> bool;    // turn == attacker
    fn key(&self) -> Key;           fn zobrist_hash(&self) -> u64;
    fn after_play(..) / after_undo(..)      // フック。VCTState が場の更新に使う
}
```

`VCFState` と `VCTState` が実装する。`State::play` は `Game::play` に limit の帳簿付けを足したもので、`limit` について覚えるべき規則はこれ 1 つである:

> `limit` は攻め方がまだ打ってよい手数である。手番が攻め方に**戻る**とき — つまり受け方の手の後に — 1 減る。したがって受け方のノードでは、直前に打った攻め手がまだ数に入っている。

手順に沿って追うと: 根（攻め方の手番）`limit = 3` → 1 手目の攻め手の後もまだ `3` → 受けの後で `2` → 2 手目の攻め手の後も `2` → … → `0` になったら攻め方はもう打てない。05 の §4 で `test_vcf_counter` の手順を盤面で追っている。

### `Key`: すべてのメモのキー

```rust
pub struct Key { pub position: u64, pub limit: u8 }
```

`State::key()` は、すべてのソルバーのすべてのメモが使うものである。`position` は 3 つのものを 1 つの Zobrist ハッシュに畳み込んでいる。石（`Board::zobrist_hash`）、手番（`apply_turn`。パスは石を動かさずに手番を変えるので、入っていなければならない）、そして**攻め方**（`apply_attacker`）である。`limit` は残り limit である。

攻め方がキーに入っているのは、1 つのソルバーで両者について問えるようにするためである。同じ石、同じ手番でも、ある問いでは「黒の詰み」、別の問いでは「白に詰みなし」であり、この 2 つがエントリを共有してはならない（`test_zobrist_hash_separates_the_attacker`）。

2 つの部分を分けているのは、覚え方が違うからである:

- **決着**（証明済み / 反証済み）は、1 つの limit についての事実ではなく**境界**である。`limit` 以内の詰みはそれより大きいどの limit でも詰みであり、`limit` 以内に詰みがなければそれより小さいどの limit でもない。だから決着は `position` だけをキーにし、limit をデータとして持つ。`DFSSolver::deadends` は四追いのない最大 limit を、`ProofTable::decided` は証明された最小 limit と反証された最大 limit を持つ。
- **決着に至らないもの** — 証明数、候補手リスト — は 1 つの limit での木についてのものなので、両方を合わせた `Key::hash()` をキーにする（`ProofTable::estimates`、追い詰めの候補手キャッシュ）。

四追いソルバーではこの境界は厳密である。生成するものが `limit` を一切読まないからだ。追い詰めでは `transfer_from` 以上で成り立つ（06 の §4）。`test_verdict_is_monotone_in_limit`（`--ignored`）は、limit を増やしても結論が「詰み」から「詰みなし」に戻らないことを確かめている。

## 4. `Solver`: 1 つの問い、1 つの世代

```rust
pub trait Solver {
    type State: State;
    fn solve(&mut self, state: &mut Self::State, budget: &mut NodeBudget) -> Option<Mate>;
    fn clear(&mut self);
    fn advance_generation(&mut self);
    fn memo_len(&self) -> usize;
}
```

どのソルバーにも入口が 2 つあり、分け方は 3 つとも同じである:

| | `solve` | `search` |
| --- | --- | --- |
| 何か | 1 つの問いの全体 | 探索そのもの |
| すること | `advance_generation()`、続けて `search`（追い詰めではさらに `extract`） | 再帰。メモの管理はしない |
| 呼ぶのは | 外の世界: `solve_limited`、エンジン | 他のソルバー: `IDDFSSolver` は `DFSSolver::search` を、追い詰めソルバーは内部四追いソルバーの `search` を |

分けている理由は、ソルバーが別のソルバーの中に入ることがあるからだ。追い詰めソルバーは 1 回の探索で内部の四追いソルバーに何百回も問う。その呼び出しのたびに世代が開いたら、内部のメモは絶えずかき回される。どこで 1 つの世代が終わり次が始まるかは、一番外側の問いだけが決める。

`clear()` はすべて忘れ、`memo_len()` は保持数を数える。どちらも呼び出し側のためのもので、ソルバーの内側は必要としない。

## 5. `Memo`: 探索をまたいで覚える

```rust
pub struct Memo<V> { entries: HashMap<u64, Entry<V>>, generation: u32, carry_capacity: usize }
```

ソルバーが持つ表はすべて `Memo` か、その 2 つ組（`ProofTable`）である。何が入り、いつ出ていくかは 3 つの規則が決める。

**入っているものは真のままである。** どのエントリもそのキーについての事実 — 行き止まり、証明、反証、証明数の見積もり — であり、キーはその事実が依存するすべてを含む（§3）。だからメモを無効化する必要はなく、大きさを抑えるだけでよい。

**世代が大きさを抑える。** `solve` ごとに 1 回呼ばれる `advance_generation()` は新しい世代を開く。メモが `carry_capacity` を超えて育っていれば、まず**直前の**世代より古いエントリをすべて捨てる。直前に終わった探索は常に残す — 同じ問いをもう一度問うのが、使い回しで本当に得られるものだからだ — ので、何回問おうとメモはおよそ探索 2 回分に落ち着く。`DEFAULT_CARRY_CAPACITY` はメモあたり `1 << 16` エントリである。

**探索の途中では何も捨てない。** `carry_capacity` は固い上限ではない。df-pn のノードは子の証明数が表に入って初めて前に進む。探索中に追い出すメモがあれば、展開ループが同じ子で永遠に回りかねない。1 回の探索の中では、ノード予算が上限である。

そして `NodeBudget` に由来する、**入れないもの**についての規則が 1 つ:

**打ち切られた探索は何も書かない。** 予算が尽きると `None` / 「不明」が上へ伝わり、途中のすべての挿入が飛ばされる。`DFSSolver` の行き止まりも、`ProofTable` のノードも、候補手キャッシュのリストも書かれない。諦めた探索から作ったメモは、事実の顔をした推測である。各挿入の隣にある `!budget.is_exhausted()` のガードがその検査である。

## 6. `NodeBudget`: 仕事量を数える

`NodeBudget::consume()` は 1 ノードを数え、上限を超えると `false` を返す。尽きた状態は `restart()` まで固定される。呼ばれるのは各探索関数の先頭 — `DFSSolver::search`、`search_attacks`、`search_defences` — だけなので、1 ノードはそのどれかへの 1 回の訪問であり、内部の四追い探索も含む。`src/` 以下のどこにも時計はない。クレートは `wasm32-unknown-unknown` でコンパイルできなければならないからだ。

`VCTSolver::solve` は `extract` を無制限の予算で走らせる。根が証明されたなら手順を読み戻す仕事は手順の長さで抑えられており、予算を惜しんで証明を捨てるのは無意味だからである。

## 7. チートシート

| 知りたいこと | 見る場所 |
| --- | --- |
| ソルバーとは最小限何か | `solver.rs` の `Solver`。`solve` = `advance_generation` + そのソルバー自身の `search` |
| 探索が深さ N で止まった理由 | `limit` は攻め手を数え、受け手のたびに `State::play` で 1 減る |
| ハッシュに攻め方が入っている理由 | `State::key` — 1 つのソルバーで両者について答える |
| 決着のキーに limit がない理由 | 決着は limit をまたぐ境界である（§3）。`DFSSolver::deadends`、`ProofTable::decided` |
| メモが育たない理由 | `Memo::advance_generation` は `carry_capacity` を超えると直前の世代以外を捨てる |
| 何かがメモされない理由 | 打ち切られた探索は何も書かない — `!budget.is_exhausted()` のガード |
| 1 ノードとは | `DFSSolver::search` / `search_attacks` / `search_defences` の 1 回の呼び出し |
| 盤上の四はどう検出されるか | `Game::check_event` → `Defeated` / `Forced` |
| パスはどう表すか | `Game::play(None)` — 手番は反転し、盤面は変わらない |
| 回帰テストを追加する | `solve.rs` のテストに ASCII 盤面と期待する手順文字列。関係する `SolveMode` ごとに 1 アサーション |

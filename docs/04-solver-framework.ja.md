# `src/mate/` の内側: 探索の骨格

対象読者: `src/mate/` を変更する人。

すべてのソルバーが共有する部品（`Game`、`State`、`Memo`、`Solver` トレイト）と、それらが守る規則を説明する。探索そのものは [05](05-solver-vcf.ja.md)（四追い）と [06](06-solver-vct.ja.md)（追い詰め）で説明する。

前提: [03](03-solver-api.ja.md)（問いの形、`limit`、`NodeBudget`）と、[02](02-board-implementation.ja.md) の用語（`Four`、`Sword`、`eyes()`、禁手、`zobrist_hash`）。

## 1. 構成

```
src/mate.rs          モジュールルート: 再エクスポート、全体像のドキュメントコメント
src/mate/
├── solve.rs         solve、SolveMode、SolveLimits、SolveResult、validate
│                    + ソルバーの回帰テスト
├── solver.rs        trait Solver                      (§4)
├── game.rs          Game、Event、End                  (§2)
├── state.rs         trait State、Key                  (§3)
├── memo.rs          Memo<V>: 世代付きメモ             (§5)
├── budget.rs        NodeBudget                        (§6)
├── mate.rs          Mate: 結果
├── vcf.rs, vcf/     四追いソルバー                    (05)
└── vct.rs, vct/     追い詰めソルバー                  (06)
src/feature/potential.rs   PotentialField: 追い詰めの手の並べ替え (06 §8)
```

部品の関係（下から上へ）:

```
Board  ──►  Game  ──►  State (VCFState | VCTState)  ──►  Solver::solve(state, budget) ──► Option<Mate>
            石、       + 攻め方、残り limit、             Memo<V> を読み書きする。
            手順、     + メモ用の Key                      キーは State::key()
            手番
```

- **`Game`**: 盤面 + 探索中に打った手。
- **`State`**: `Game` + 探索に必要な情報。どちら側の探索か、攻め手はあと何手か、追い詰めなら手の並べ替え用のポテンシャル場。
- **`Solver`**: `State` を探索する。問いをまたいで生きる **`Memo`** を持つ。メモのキーは `State` の **`Key`**。
- **`NodeBudget`**: 1 つの問い（または一連の問い）に使ってよい仕事量。

## 2. `Game`: 探索中の盤面

```rust
pub struct Game { board: Board, moves: Vec<Option<Point>>, pub turn: Player }
```

- `Game::init(&board, turn)` で盤面を 1 回だけクローンする。以後、探索中にクローンはしない。
- `play(m)`: 石を置き、`turn` を反転。`undo()`: それを戻す。
- `into_play(m, f)`: play → `f(self)` → undo をまとめて行い、`f` の結果を返す。すべてのソルバーはこれで木を歩く。
- 手は `Option<Point>`。`play(None)` は**パス**で、手番だけ反転する。連珠にパスはないが、追い手の定義（「何もしなければ相手はどうできるか」）に必要なので、ここでは普通の手として扱う。

### `check_event`: 盤上にすでにある四

各ノードは手を生成する前に `Game::check_event()` を呼ぶ。「直前に打った側の四は、これから打つ側にとって何を意味するか」を返す。

| 相手の四の勝ち点 | `Event` | これから打つ側にとって |
| --- | --- | --- |
| 異なる 2 点 `p1`、`p2` | `Defeated(Fours(p1, p2))` | 負け。1 手では両方止まらない |
| 1 点 `p`。ただし自分の禁手 | `Defeated(Forbidden(p))` | 負け。唯一の止めが打てない |
| 1 点 `p` | `Forced(p)` | `p` を打つしかない |
| なし | `None` | 自由 |

- 見るのは直前の手を通る四だけ（`structures_on(last_move, opponent, Four)`）。それより古い四は、すでに応手を強いているはず。
- パスの後は直前の手がないので、相手のすべての四を見る。
- 棒四は眼の異なる 2 つの `Four` として現れ、四四と同じ 2 点判定（`take_distinct_two`）で扱う。

`End` は `Defeated` の中身で、`Mate` の末尾になる（03 §5）。

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

`VCFState` と `VCTState` が実装する。`State::play` は `Game::play` に `limit` の管理を足したもの。`limit` の規則は 1 つだけ:

> `limit` は攻め方がまだ打てる手数。手番が攻め方に**戻る**とき（＝受けの手の後）に 1 減る。したがって受け方のノードでは、直前の攻め手がまだ数に入っている。

例: 根（攻め方の手番）`limit = 3` → 攻め手の後も `3` → 受けの後 `2` → 攻め手の後も `2` → … → `0` で攻め方は打てない。盤面つきの例は 05 §4（`test_vcf_counter`）。

### `Key`: すべてのメモのキー

```rust
pub struct Key { pub position: u64, pub limit: u8 }
```

`State::key()` は、全ソルバーの全メモが使うキー。

- `position`: 3 つを 1 つの Zobrist ハッシュにまとめたもの。
  - 石（`Board::zobrist_hash`）
  - 手番（`apply_turn`）。パスは石を動かさずに手番を変えるので、必要。
  - **攻め方**（`apply_attacker`）
- `limit`: 残り limit。

攻め方がキーに入る理由: 1 つのソルバーで両方の色について問えるようにするため。同じ石・同じ手番でも「黒の詰み」と「白の詰み」は別の問いで、エントリを共有してはいけない（`test_zobrist_hash_separates_the_attacker`）。

`position` と `limit` を分けている理由: 覚え方が違うため。

- **決着**（証明済み / 反証済み）は limit をまたいで有効。`limit` 以内の詰みはそれより大きい limit でも詰み。`limit` 以内で詰みなしなら、それより小さい limit でもなし。だから決着は `position` だけをキーにし、limit はデータとして持つ。
  - `DFSSolver::deadends`: 四追いがないと分かった最大 limit。
  - `ProofTable::decided`: 証明された最小 limit と、反証された最大 limit。
- **決着に至らないもの**（証明数の途中経過、候補手リスト）は 1 つの limit での木についての情報。両方を合わせた `Key::hash()` をキーにする（`ProofTable::estimates`、候補手キャッシュ）。

四追いソルバーでは、手の生成が `limit` を読まないので、この境界は厳密。追い詰めでは `transfer_from` 以上で成り立つ（06 §4）。`test_verdict_is_monotone_in_limit`（`--ignored`）が、limit を増やしても「詰み」が「詰みなし」に戻らないことを確認している。

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

どのソルバーにも入口が 2 つある。分け方は 3 つとも同じ。

| | `solve` | `search` |
| --- | --- | --- |
| 何か | 1 つの問いの全体 | 探索そのもの |
| すること | `advance_generation()` → `search`（追い詰めはさらに `extract`） | 再帰。メモの管理はしない |
| 誰が呼ぶか | 外部（`mate::solve`、エンジン） | 他のソルバー（`IDDFSSolver` → `DFSSolver::search`、追い詰め → 内部四追いの `search`） |

分ける理由: ソルバーは別のソルバーの中に入ることがある。追い詰めソルバーは 1 回の探索で内部の四追いソルバーを何百回も呼ぶ。そのたびに世代が変わると、内部のメモが絶えず捨てられてしまう。世代の区切りは一番外側の問いだけが決める。

`clear()` はすべて忘れる。`memo_len()` は保持数を返す。どちらも呼び出し側向けで、ソルバー内部では使わない。

## 5. `Memo`: 探索をまたいで覚える

```rust
pub struct Memo<V> { entries: HashMap<u64, Entry<V>>, generation: u32, carry_capacity: usize }
```

ソルバーの表はすべて `Memo`（または `Memo` 2 つ組の `ProofTable`）。規則は 4 つ。

- **入っているものは常に正しい。** エントリはキーについての事実（行き止まり、証明、反証、証明数の見積もり）で、キーはその事実が依存するすべてを含む（§3）。だから無効化は不要。大きさを抑えるだけでよい。
- **世代で大きさを抑える。** `solve` ごとに `advance_generation()` が新しい世代を開く。メモが `carry_capacity` を超えていれば、**直前の**世代より古いエントリをすべて捨てる。直前の探索は常に残す（同じ問いの繰り返しを安くするのが使い回しの目的だから）。結果、メモは探索 2 回分程度で安定する。`DEFAULT_CARRY_CAPACITY` はメモあたり `1 << 16`。
- **探索の途中では捨てない。** `carry_capacity` は固い上限ではない。df-pn のノードは子の証明数が表に入って初めて進むので、探索中に捨てると展開ループが同じ子で回り続けかねない。1 回の探索の中ではノード予算が上限。
- **打ち切られた探索は何も書かない。** 予算が尽きると「不明」が上へ伝わり、途中の挿入はすべてスキップされる（`DFSSolver` の行き止まり、`ProofTable` のノード、候補手キャッシュ）。諦めた探索の結果をメモすると、推測が事実として残ってしまう。各挿入の前にある `!budget.is_exhausted()` がこの検査。

## 6. `NodeBudget`: 仕事量を数える

- `consume()` が 1 ノードを数え、上限を超えると `false` を返す。尽きた状態は `restart()` まで続く。
- 呼ぶ場所は各探索関数の先頭だけ: `DFSSolver::search`、`search_attacks`、`search_defences`。つまり 1 ノード = これらの 1 回の呼び出し。内部の四追い探索も含む。
- 時計は使わない。クレートは `wasm32-unknown-unknown` でコンパイルできる必要がある。
- `VCTSolver::solve` は `extract` を無制限の予算で走らせる。根が証明できたなら、手順の復元は手順の長さで抑えられる。予算不足で証明を捨てるのは無意味。

## 7. チートシート

| 知りたいこと | 見る場所 |
| --- | --- |
| ソルバーとは何か | `solver.rs` の `Solver`。`solve` = `advance_generation` + 各ソルバーの `search` |
| 探索が深さ N で止まった理由 | `limit` は攻め手数。受けの手ごとに `State::play` で 1 減る |
| ハッシュに攻め方が入る理由 | `State::key`。1 つのソルバーで両方の色に答えるため |
| 決着のキーに limit がない理由 | 決着は limit をまたいで有効（§3）。`DFSSolver::deadends`、`ProofTable::decided` |
| メモが育たない理由 | `Memo::advance_generation`。`carry_capacity` を超えると直前の世代以外を捨てる |
| メモされない理由 | 打ち切られた探索は何も書かない。`!budget.is_exhausted()` のガード |
| 1 ノードとは | `DFSSolver::search` / `search_attacks` / `search_defences` の 1 回の呼び出し |
| 盤上の四の検出 | `Game::check_event` → `Defeated` / `Forced` |
| パスの表現 | `Game::play(None)`。手番だけ反転 |
| 回帰テストの追加 | `solve.rs` に ASCII 盤面と期待する手順文字列。関係する `SolveMode` ごとに 1 アサーション |

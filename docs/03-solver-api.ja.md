# 詰みソルバーの使い方

ソルバーを**呼び出す**コード向けのページである。対象は次のとおり。

- `wasm.rs`
- CLI（`examples/solve.rs`）
- ソルバーを保持して何度も問い合わせる Rust コード

探索の中身は [04](04-solver-framework.ja.md)、[05](05-solver-vcf.ja.md)、[06](06-solver-vct.ja.md) で説明する。ここでは知らなくてよい。

- 前提: [01](01-renju-rules.ja.md) の用語（四、棒四、三、禁手）。
- Rust のソルバー API は `src/mate/` にあり、`quintet::mate` から使える。
- JS の入口は `src/wasm.rs`。

<a id="quick-start"></a>

## まず一度解く

Rust、CLI、JavaScript の例は、同じ局面の黒の追い詰めを解く。

- 攻め手は最大 5 手。
- 追い手判定の内部四追いは最大 3 手。
- 結果は `I8,G8,K7,H10,J6,E10,I7,K5,I6`。

### Rust

```rust
use quintet::board::{Board, Player, Points};
use quintet::mate::{solve, SolveLimits, SolveMode, SolveResult};

fn main() -> Result<(), &'static str> {
    let board: Board = "H8,H7,I9,G7,J8,J9,G9,F10".parse()?;
    let limits = SolveLimits::new(5)
        .with_threat_limit(3)
        .with_max_nodes(100_000);
    match solve(SolveMode::VCTDFPNS, &board, Player::Black, limits) {
        SolveResult::Proven(mate) => println!("{}", Points(mate.path)),
        SolveResult::Disproven => println!("No mate within these limits"),
        SolveResult::Aborted => println!("Unknown: budget exhausted"),
    }
    Ok(())
}
```

### CLI

リポジトリのルートで実行する。`o` は黒、`x` は白を表す。局面の文字列は、黒から交互に打った手を表す。

```sh
cargo run --release --example solve -- vct_dfpns 5 3 o H8,H7,I9,G7,J8,J9,G9,F10
```

### JavaScript

- `npm install @renju-note/quintet` で公開パッケージを導入し、WebAssembly を読み込める環境で実行する。
- 座標は 0 始まりで、`H8` は `(7, 7)`。
- 引数の `16` は df-pn、`true` は黒を攻め方にする指定。

```js
import init, { solve, encode_xy, decode_x, decode_y } from "@renju-note/quintet";

await init();
const blacks = [[7, 7], [8, 8], [9, 7], [6, 8]].map(([x, y]) => encode_xy(x, y));
const whites = [[7, 6], [6, 6], [9, 8], [5, 9]].map(([x, y]) => encode_xy(x, y));
const path = solve(16, 5, blacks, whites, true, 3);
if (path !== undefined) {
  console.log(Array.from(path, p => [decode_x(p), decode_y(p)]));
}
```

### 結果の違い

- Rust の `Proven` は詰みあり、`Disproven` は指定した範囲で詰みなし、`Aborted` は不明。
- JS は、詰み手順か `undefined` を返す。詰みなし・予算切れ・不正な点コードなどは区別しない。
- 上の JS 例にはノード予算がない。予算を指定するには `solve_vct_auto` を使う。

結果の読み方は [§5](#section-5)、探索前の特殊な局面は [§6](#section-6) を参照。通常の利用は [§1](#section-1) と [§2](#section-2) を読めばよい。ソルバーを保持して何度も問う場合だけ [§4](#section-4) に進む。

<a id="section-1"></a>

## 1. ソルバーが答える問い

すべての呼び出しは次の問いに答える。

> この `board` で `attacker` の手番のとき、攻め方は `limit` 手以内に**詰み**（必勝手順）を持つか？

用語は次のとおり。

- **攻め方（attacker）**: 手番側で、勝ちを証明したい側。相手は**受け方（defender）**。どちらの色でも攻め方になれる。
- **詰み**: 次の 2 種類。どちらを探すかは `SolveMode` で選ぶ。
  - **VCF（四追い）**: 攻め手がすべて四。
  - **VCT（追い詰め）**: 攻め手がすべて追い手（四、または放置すると四追いになる手）。
- **`limit`**: 攻め方の手数だけを数える。7 手（攻め 4、受け 3）の手順なら `limit >= 4`。

答えが「あり」なら [`Mate`](#section-5) が返る。詰み手順と、最後に受け方がなぜ負けているかを持つ。

<a id="section-2"></a>

## 2. 使い捨てで呼ぶ: `solve`

```rust
pub fn solve(mode: SolveMode, board: &Board, attacker: Player, limits: SolveLimits) -> SolveResult
```

- ソルバーを作って 1 回走らせ、捨てる。
- `limits` で深さと、任意でノード予算を指定する。
- 答えは 3 値。`wasm.rs` もこれを呼び、答えを「詰み手順」か「なし」にまとめて返す。

完全な呼び出し例は [冒頭の Rust 例](#quick-start) を参照。

### 探索のコストも知る

`solve_with_stats` は同じ呼び出しで、探索のコストも返す。

```rust
let (result, stats) = solve_with_stats(SolveMode::VCTDFPNS, &board, Player::Black, limits);
stats.nodes      // 訪れたノード数。NodeBudget と同じ数え方（§3）
stats.memo_len   // 探索後にソルバーのメモに残るエントリ数（Solver::memo_len、§4）
```

どちらの数も決定的で、同じ引数なら実行ごと・マシンごとに同じ値になる。だから、ソルバーの変更はこれで比べる。ベンチマーク（[07](07-benchmarks.ja.md)）もそうしている。

### `SolveResult`

```rust
pub enum SolveResult { Proven(Mate), Disproven, Aborted }
```

`Option<Mate>` では、詰みがない理由を区別できない。`SolveResult` は区別する。

- `Disproven`: この上限では詰みがない。
- `Aborted`: 予算が尽きた（予約モードでは探索しなかった）。詰みの有無は不明。
  - `Aborted` を「安全」と扱うエンジンは、詰まされる。
- アクセサ: `is_proven()`、`is_disproven()`、`is_aborted()`、`mate()`、`into_mate()`。

### `SolveMode`

| `SolveMode` | コード | CLI 名 | 探索 |
| --- | --- | --- | --- |
| `VCFDFS` | 0 | `vcf` | 四追い、深さ優先。`threat_limit` は無視。 |
| `VCFIDDFS` | 1 | `vcf_iddfs` | 予約。`solve` は何も探索せず `Aborted` を返す。 |
| `VCTDFS` | 10 | `vct` | 追い詰め。証明数木を深さ優先でたどる。 |
| `VCTIDDFS` | 11 | `vct_iddfs` | 予約。`solve` は何も探索せず `Aborted` を返す。 |
| `VCTPNS` | 15 | `vct_pns` | 追い詰め、証明数探索。 |
| `VCTDFPNS` | 16 | `vct_dfpns` | 追い詰め、df-pn。**通常はこれを使う。** |
| `VCTAUTO` | 17 | `vct_auto` | 追い詰め、df-pn。ノード予算だけで解く。`limit` と `threat_limit` は無視（後述）。 |

- コードは wasm/JS 側の表現（`SolveMode::try_from(u8)`）で、公開 API である。番号を変えてはいけない。
- CLI 名は `SolveMode::from_str` で解釈する。
- 3 つの VCT モードは同じ木を違う順序で探索するので、結論は同じ。違うのは速さと、複数ある詰み手順のうちどれを返すか。

### `SolveLimits`: 探索の上限

```rust
SolveLimits::new(limit)
    .with_threat_limit(threat_limit)     // 既定 0
    .with_defender_vcf_depth(depth)      // 既定 DEFAULT_DEFENDER_VCF_DEPTH = 2
    .with_max_nodes(nodes)               // 既定: 無制限
```

| フィールド | 何の上限か | 対象 |
| --- | --- | --- |
| `limit` | 本手順の攻め手数。 | 全モード |
| `threat_limit` | 「この手は追い手か」を判定する内部四追いの攻め手数（[06 §2](06-solver-vct.ja.md#section-2)）。`0`: 四のみ。`1`: 四と三。`2` 以上: 四三を準備する手など、より深い含み手も対象になる（`test_vct_fukumi_move` は `3` が必要）。 | VCT |
| `defender_vcf_depth` | 受け方の逆襲四追いを探す内部四追いの攻め手数。 | VCT |
| `max_nodes` | 総仕事量（ノード数、[§3](#section-3)）。 | 全モード |

`with_*` は新しい値を返す。後でフィールドが増えても、この書き方なら呼び出し側は壊れない。

### 予算だけで解く: `VCTAUTO`

`VCTAUTO` は、`limit` も `threat_limit` も選ばずに追い詰めを問う。

- `with_max_nodes` を与えれば、ほかは関係ない（`defender_vcf_depth` だけは効く）。
- 中身は `DFPNSVCTSolver::solve_auto`（[06 §5](06-solver-vct.ja.md#section-5)）。
- JS からは `solve_vct_auto(blacks, whites, black, max_nodes)` で呼ぶ。

```rust
let limits = SolveLimits::new(0).with_max_nodes(1_000_000);   // 0 は無視される
let result = solve(SolveMode::VCTAUTO, &board, Player::Black, limits);
```

- 手順の長さに上限を付けない。`threat_limit` 2、3、1（`AUTO_THREAT_LIMITS`）の順に探索する。
- 各 `threat_limit` にはその回の予算の一部を割り当て、割り当ては回ごとに倍にする。
- 結果の意味は次のとおり。
  - `Proven`: 追い詰め。最短より長いことが多い。長い詰みのほうが証明が安いことが多いため。
  - `Disproven`: これらの `threat_limit` のどれでも、長さに関係なく追い詰めはない。「N 手以内にはない」とは言わない。
  - `Aborted`: いつもどおり不明。

<a id="section-3"></a>

## 3. 予算: `NodeBudget`

時計は使わない。`src/` 以下は `std::time` のない `wasm32-unknown-unknown` 向けにもコンパイルするため。時間制限をかけたい呼び出し側は、自分でノード数に換算する。

```rust
let mut budget = NodeBudget::new(100_000);   // または NodeBudget::unlimited()
budget.is_exhausted();                       // 予算切れになったか
budget.nodes();                              // これまでのノード数
budget.reset();                              // 0 に戻す。上限はそのまま
let part = budget.slice(1_000);              // 仕事の一部のための予算。残りより多くはならない
budget.absorb(&part);                        // その一部が使った分を数える
```

- 1 ノード = 探索関数の 1 回の呼び出し。内部の四追い探索も数える。
  - 四追いでは `DFSSolver::search`。
  - 追い詰めでは `search_attacks` / `search_defences`。
- 尽きた予算は、`reset()` されるまで尽きたまま。1 つの予算を共有する複数の呼び出しは、全体としてそこで止まる。

予算を安全に使える理由は 2 つある。

- **証明は偽にならない。** 予算切れ直前に見つかった詰みも本物で、そのまま返る。
- **未完了の結果を再利用しない。** 予算切れで未完了の結果はメモに書かれない。完了済みの部分木の結果は残る（[04 §5](04-solver-framework.ja.md#section-5)）ので、同じソルバーをすぐ再利用できる。
  - `test_abort_leaves_no_wrong_memo` が、小さい予算で何度も打ち切った後も正しく答えることを確認している。

<a id="section-4"></a>

## 4. ソルバーを保持する: `Solver` トレイト

`solve` はソルバーを毎回捨てる。探索で埋めた表も一緒に消える。関連する問いを何度も投げる呼び出し側（対局エンジン、変化をたどる解析画面など）は、ソルバーを 1 つ持ち続けるほうがよい。必要な部品はすべて公開されている。

```rust
use quintet::mate::{DFPNSVCTSolver, NodeBudget, Solver, VCTState, DEFAULT_DEFENDER_VCF_DEPTH};

let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
let budget = &mut NodeBudget::new(1_000_000);   // ループ全体の上限
for board in candidates {
    let state = &mut VCTState::from_board(&board, attacker, limit, threat_limit);
    match solver.solve(state, budget) {
        Some(mate) => {}                        // 詰みあり
        None if budget.is_exhausted() => break,
        None => {}                              // limit 以内に詰みなし
    }
}
```

すべてのソルバーは `Solver`（`mate/solver.rs`）を実装する。

```rust
pub trait Solver {
    type State: State;
    fn solve(&mut self, state: &mut Self::State, budget: &mut NodeBudget) -> Option<Mate>;
    fn clear(&mut self);
    fn advance_generation(&mut self);
    fn memo_len(&self) -> usize;
}
```

| ソルバー | `State` | コンストラクタ | 探すもの |
| --- | --- | --- | --- |
| `DFSSolver` | `VCFState` | `new()` | 四追い |
| `IDDFSSolver` | `VCFState` | `new(limits)` | 四追い。`limits` の各値で順に探す |
| `VCTSolver<P>`（エイリアス `DFSVCTSolver`、`PNSVCTSolver`、`DFPNSVCTSolver`） | `VCTState` | `new(defender_vcf_depth)` | 追い詰め |

状態は問いごとに作る。

- `VCFState::from_board(&board, attacker, limit)`
- `VCTState::from_board(&board, attacker, limit, threat_limit)`

持ち越すのはソルバーのほうである。

<details>
<summary>再利用の保証とメモの管理（実装の詳細）</summary>

使い回すと、次のことが得られる。

- **何をどの順で問ってもよい。** メモのキーは局面・手番・残り limit・**攻め方**。同じ盤面で黒の詰みと白の詰みを問っても混ざらない（`test_reused_solver_both_attackers`）。
  - 追い詰めソルバーのメモは、状態の `threat_limit` もキーにする。そのため、1 つのソルバーで複数の `threat_limit` に答えられる（`test_reused_solver_matches_fresh_across_threat_limits`）。
  - `threat_limit` をまたいで共有するのは内部四追いだけで、証明数表は共有しない（[06 §2](06-solver-vct.ja.md#section-2)）。
- **メモリは有界。** `solve` は毎回新しい**世代**を開く。メモが引き継ぎ容量を超えると、直前の探索より古いエントリを捨てる。メモは探索 2 回分程度で安定する（`test_reused_solver_memo_stays_bounded`）。
  - 閾値は各ソルバーの `with_carry_capacity(..)` で指定する。既定はメモあたり `1 << 16`。
  - 現在の保持数は `memo_len()` で得られる。
- **同じ問いの繰り返しはほぼ無料。** 探索し直さず、数ノードで済む（`test_reused_solver`）。
- **同じ局面を別の limit で問うと、決着を再利用できる。**
  - VCF では、すべての limit で有効。
  - VCT では、`transfer_from` 以上で有効（[06 §4](06-solver-vct.ja.md#section-4)）。
  - この範囲では、決着は limit をまたいで有効になる。4 手以内で詰みなら 5 手・6 手でも詰み。3 手以内で詰みなしなら 2 手・1 手でもなし。
  - 1 局面を limit 1〜6 で順に問うと、6 回別々に探索する場合の約 5 分の 1 で済む（`test_decisions_carry_between_limits`）。
- **別の局面どうしはあまり共有しない。** 残り limit がキーに入るため。同じ盤面に 2 つの根から到達しても、根からの距離が違えば別エントリになる。
- `clear()` はすべて忘れる。必須ではない。まっさらなソルバーを渡したいとき、メモリを返したいときに使う。
- `advance_generation()` は `solve` が最初に呼ぶ。ソルバーの低レベルの `search` / `extract` を自分で呼ぶ場合だけ必要になる（[04 §4](04-solver-framework.ja.md#section-4)）。

</details>

### ほかに問えること

- **この詰みをどう受けるか。** `VCTState::threat_defences(&mate)` が、見つかった詰みに対して試す価値のある受けを返す。内訳は、詰み手順の各点、詰め上がりを崩す点、ノリ手、受け方自身の四を作る手である。追い詰め探索も、受けの候補を同じリストから作る（[06 §3](06-solver-vct.ja.md#section-3)）。
- **どの手がこの追い詰めを止めるか。**
  - `VCTSolver` が局面を証明した直後に `proof_tree(&mut state, &mut budget)` を呼ぶと、表から証明全体を `ProofTree` として取り出せる。攻め方の手番では攻め手 1 つ、受け方の手番では試す価値のある受けすべてを持つ木になる。
  - 受け方の石を 1 つ足した局面で `proves(&mut state, &tree, &mut budget)` を呼ぶと、同じ証明が成り立つかをノードごとに確かめる。探索し直すより 100 分の 1 程度で済む。
  - `tree.zone()` の外の石なら、それも要らない。証明はそのまま成り立つ。
  - zone の外の手と、`proves` が通る手は追い詰めを止めない。探索し直すのは残りの手だけである（[06 §6](06-solver-vct.ja.md#section-6)）。
- **この手は追い手か。** 手番側をパスさせて（`Game::play(None)`）、相手の四追いを問う。パスした `game` から `VCFState::new(game, limit)` を作り、`DFSSolver::new().solve(..)` を呼ぶ。`test_threat_after_pass` が 1 手ずつ確認している。

<a id="section-5"></a>

## 5. `Mate` と `End`

```rust
pub struct Mate { pub end: End, pub path: Vec<Point> }
pub enum End { Fours(Point, Point), Forbidden(Point), Unknown }
```

- `path`: 攻め方から始まり、攻めと受けが交互に並ぶ。
- `Mate::n_moves()` は長さ、`n_attacks()` は攻め手数を返す。
- `end`: 最後の手の後、受け方がなぜ負けているか。

| `End` | 受け方の状況 | 誰に起きるか |
| --- | --- | --- |
| `Fours(p1, p2)` | 勝ち点の異なる四が 2 つ（`p1`、`p2`）。1 手では止まらない。棒四（`Grid` は眼の異なる 2 つの `Four` として返す）または四四。 | どちらも。ただし攻め方が黒なら棒四だけ。四四は黒の禁手なので打たれない。 |
| `Forbidden(p)` | 四が 1 つで、唯一の止め `p` が禁手。 | 黒のみ。 |
| `Unknown` | 勝ちは証明されたが、手順を復元できなかった。探索前から攻め方に四があった（[§6](#section-6)）か、復元でたどれる証明済みの子がなかった（[06 §6](06-solver-vct.ja.md#section-6)）。 | — |

<a id="section-6"></a>

## 6. 探索前の検査: `trivial_result`

`solve` は、ソルバーが扱わない局面に探索せず答える。予約モードは、その前に盤面によらず `Aborted` を返す。

| 局面 | 結果 |
| --- | --- |
| どちらかにすでに `Five` がある | `Disproven` |
| 黒にすでに `Overlined` がある | `Disproven` |
| 攻め方にすでに `Four` がある | `Proven(Mate { end: Unknown, path: [] })`。すでに勝ちで、証明するものがない。 |

ソルバーを直接使う場合（[§4](#section-4)）には、この検査は入らない。必要なら自分で確かめる。

<a id="section-7"></a>

## 7. チートシート

| したいこと | 使うもの |
| --- | --- |
| 1 回だけ問う | `solve` |
| 探索のコストを知る | `solve_with_stats`。`SolveStats::nodes` と `memo_len` |
| 四追いだけ探す | `SolveMode::VCFDFS` |
| 追い詰めを探す | `SolveMode::VCTDFPNS`。追い手の範囲は `threat_limit` |
| ノード予算だけで追い詰めを探す | `SolveMode::VCTAUTO`、`VCTSolver::solve_auto`、`wasm::solve_vct_auto` |
| 長すぎる探索を止める | `SolveLimits::with_max_nodes`。`Aborted` は「不明」であって「安全」ではない |
| 多くの問いを安く投げる | `DFPNSVCTSolver` を保持し、問いごとに新しい `VCTState` を渡し、`NodeBudget` は 1 つを共有する |
| 両方の色について問う | 同じソルバーでよい。攻め方はキーに入っている |
| 受けの候補を知る | `VCTState::threat_defences(&mate)` |
| どの手が追い詰めを止めるかを 1 手ずつ知る | 解いた直後に `VCTSolver::proof_tree`、手ごとに `ProofTree::zone` と `VCTSolver::proves` |
| 直前の手が追い手か知る | `Game::play(None)` でパスし、相手の四追いを問う |
| JS から詰み手順を読む | `wasm::solve` は手順を `u8` の点コード（[02 §1](02-board-implementation.ja.md#section-1)）で返す。`decode_x` / `decode_y` で読む |
| モードを追加する | `SolveMode` と、その `TryFrom<u8>`、`FromStr`、`solve` の `match`（`solve.rs`） |

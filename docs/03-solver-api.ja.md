# 詰みソルバーの使い方

対象読者: ソルバーを**呼び出す**人。`wasm.rs`、CLI（`examples/solve.rs`）、あるいはソルバーを保持して何度も問い合わせる Rust コードを書く人。

探索の中身は知らなくてよい。中身は [04](04-solver-framework.ja.md)、[05](05-solver-vcf.ja.md)、[06](06-solver-vct.ja.md) で説明する。

前提: [01](01-renju-rules.ja.md) の用語（四、棒四、三、禁手）。以下の型・関数はすべて `src/mate/` にあり、`quintet::mate` から使える。

## 1. ソルバーが答える問い

すべての呼び出しは、次の 1 つの問いに答える。

> この `board` で `attacker` の手番のとき、攻め方は `limit` 手以内に**詰み**（必勝手順）を持つか？

- **攻め方（attacker）**: 手番側。勝ちを証明したい側。相手は**受け方（defender）**と呼ぶ。黒でも白でも攻め方になれる。
- **詰み**: 次のどちらか。どちらを探すかは `SolveMode` で指定する。
  - **VCF（四追い）**: 攻め手がすべて四。
  - **VCT（追い詰め）**: 攻め手がすべて追い手（四、または放置すると四追いになる手）。
- **`limit`**: 攻め方の手数だけを数える。7 手（攻め 4、受け 3）の手順なら `limit >= 4` が必要。

答えが「あり」なら [`Mate`](#5-mate-と-end) が返る。詰み手順と、最後に受け方がなぜ負けているかを持つ。

## 2. 使い捨てで呼ぶ: `solve`

```rust
pub fn solve(mode: SolveMode, board: &Board, attacker: Player, limits: SolveLimits) -> SolveResult
```

ソルバーを作って 1 回走らせ、捨てる。`limits` で深さと（任意で）ノード予算を指定し、答えは 3 値になる。`wasm.rs` もこれを呼び、答えを詰み手順か「なし」に落とす。

```rust
use quintet::board::{Board, Player};
use quintet::mate::{SolveLimits, SolveMode, SolveResult, solve};

let board: Board = "H8,I9,J9,H7".parse().unwrap();
let limits = SolveLimits::new(5).with_threat_limit(2).with_max_nodes(100_000);
match solve(SolveMode::VCTDFPNS, &board, Player::Black, limits) {
    SolveResult::Proven(mate) => { /* mate.path が詰み手順 */ }
    SolveResult::Disproven => { /* この limit では詰みなし */ }
    SolveResult::Aborted => { /* 予算切れ。不明 */ }
}
```

### `SolveMode`

| `SolveMode` | コード | CLI 名 | 探索 |
| --- | --- | --- | --- |
| `VCFDFS` | 0 | `vcf` | 四追い、深さ優先。`threat_limit` は無視。 |
| `VCFIDDFS` | 1 | `vcf_iddfs` | 予約。`solve` は `Disproven` を返す。 |
| `VCTDFS` | 10 | `vct` | 追い詰め。証明数木を深さ優先でたどる。 |
| `VCTIDDFS` | 11 | `vct_iddfs` | 予約。`solve` は `Disproven` を返す。 |
| `VCTPNS` | 15 | `vct_pns` | 追い詰め、証明数探索。 |
| `VCTDFPNS` | 16 | `vct_dfpns` | 追い詰め、df-pn。**通常はこれ。** |

- コードは wasm/JS 側の表現（`SolveMode::try_from(u8)`）で、公開 API。番号を変えてはいけない。
- CLI 名は `SolveMode::from_str`。
- 3 つの VCT モードは同じ木を違う順序で探索し、結論は同じ。違うのは速さと、複数ある詰み手順のうちどれを先に見つけるか。

### `SolveLimits`: 探索の上限

```rust
SolveLimits::new(limit)
    .with_threat_limit(threat_limit)     // 既定 0
    .with_defender_vcf_depth(depth)      // 既定 DEFAULT_DEFENDER_VCF_DEPTH = 2
    .with_max_nodes(nodes)               // 既定: 無制限
```

| フィールド | 何の上限か | 対象 |
| --- | --- | --- |
| `limit` | 本手順の攻め手数 | 全モード |
| `threat_limit` | 「この手は追い手か」を判定する内部四追いの攻め手数（06 §2）。`0`: 四のみ。`1`: 四と三。`2` 以上: 四三を準備する手など、深い含み手も（`test_vct_fukumi_move` は `3` が必要）。 | VCT |
| `defender_vcf_depth` | 受け方の逆襲四追いを探す内部四追いの攻め手数 | VCT |
| `max_nodes` | 総仕事量（ノード数、§3） | 全モード |

`with_*` は新しい値を返す。後でフィールドが増えても、この書き方なら呼び出し側は壊れない。

### `SolveResult`

```rust
pub enum SolveResult { Proven(Mate), Disproven, Aborted }
```

`Option<Mate>` は「なぜ詰みがないか」を区別できない。`SolveResult` は区別する。

- `Disproven`: この上限では詰みがない。
- `Aborted`: 予算が尽きた。詰みの有無は不明。

この違いは重要で、`Aborted` を「安全」と扱うエンジンは詰まされる。アクセサは `is_proven()`、`is_disproven()`、`is_aborted()`、`mate()`、`into_mate()`。

## 3. 予算: `NodeBudget`

時計は使わない。`src/` 以下は `wasm32-unknown-unknown` 向けにコンパイルでき、`std::time` がないため。時間制限をかけたい呼び出し側は、自分でノード数に換算する。

```rust
let mut budget = NodeBudget::new(100_000);   // または NodeBudget::unlimited()
budget.is_exhausted();                       // 予算切れになったか
budget.nodes();                              // これまでのノード数
budget.restart();                            // 0 に戻す。上限はそのまま
```

- 1 ノード = 探索関数の 1 回の呼び出し。四追いでは `DFSSolver::search`、追い詰めでは `search_attacks` / `search_defences`。内部の四追い探索も数える。
- 尽きた予算は `restart()` まで尽きたまま。1 つの予算を複数の呼び出しで共有すると、全体としてそこで止まる。

予算を安全に使える理由は 2 つ。

- **証明は偽にならない。** 予算切れ直前に見つかった詰みも本物で、そのまま返る。
- **打ち切られた探索はメモに何も残さない。** 途中結果はどのメモにも書かれない（04 §5）。だから同じソルバーをすぐ再利用できる。`test_abort_leaves_no_wrong_memo` が、小さい予算で何度も打ち切った後でも正しい答えが返ることを確認している。

## 4. ソルバーを保持する: `Solver` トレイト

`solve` はソルバーを毎回捨てる。探索で埋めた表もいっしょに消える。対局エンジンや解析画面のように関連する問いを何度も投げるなら、ソルバーを 1 つ持ち続けるほうがよい。そのために必要な部品はすべて公開されている。

```rust
use quintet::mate::{DFPNSVCTSolver, NodeBudget, Solver, VCTState, DEFAULT_DEFENDER_VCF_DEPTH};

let mut solver = DFPNSVCTSolver::init(threat_limit, DEFAULT_DEFENDER_VCF_DEPTH);
let budget = &mut NodeBudget::new(1_000_000);   // ループ全体の上限
for board in candidates {
    let state = &mut VCTState::init(&board, attacker, limit);
    match solver.solve(state, budget) {
        Some(mate) => { /* 詰みあり */ }
        None if budget.is_exhausted() => break,
        None => { /* limit 以内に詰みなし */ }
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
| `DFSSolver` | `VCFState` | `init()` | 四追い |
| `IDDFSSolver` | `VCFState` | `init(limits)` | 四追い。`limits` の各値で順に探す |
| `VCTSolver<P>`（エイリアス `DFSVCTSolver`、`PNSVCTSolver`、`DFPNSVCTSolver`） | `VCTState` | `init(threat_limit, defender_vcf_depth)` | 追い詰め |

状態は `VCFState::init(&board, attacker, limit)` / `VCTState::init(&board, attacker, limit)` で作る。問いごとに作り直してよい。持ち越すのはソルバーのほう。

使い回すと何が起きるか:

- **何をどの順で問うてもよい。** メモのキーには局面・手番・残り limit に加えて**攻め方**が入る。同じ盤面で黒の詰みと白の詰みを問うても混ざらない（`test_reused_solver_both_attackers`）。
- **メモリは有界。** `solve` は毎回新しい**世代**を開く。メモが引き継ぎ容量を超えると、直前の探索より古いエントリを捨てる。何回問うてもメモは探索 2 回分程度で安定する（`test_reused_solver_memo_stays_bounded`）。閾値は各ソルバーの `with_carry_capacity(..)` で指定（既定はメモあたり `1 << 16`）。現在の保持数は `memo_len()`。
- **同じ問いの繰り返しはほぼ無料。** 探索し直さず、数ノードで済む（`test_reused_solver`）。
- **同じ局面を別の limit で問うのは安い。** 決着は limit をまたいで有効なため（4 手以内で詰みなら 5 手でも詰み。3 手以内で詰みなしなら 2 手でもなし）。1 局面を limit 1〜6 で順に問うと、6 回別々に探索する場合の約 5 分の 1 で済む（`test_decisions_carry_between_limits`）。
- **別の局面どうしはあまり共有しない。** 残り limit がキーに入るため。同じ盤面に別の根から到達しても、根からの距離が違えば別エントリになる。
- `clear()` はすべて忘れる。必須ではない。まっさらなソルバーを渡したいとき、メモリを返したいときに使う。
- `advance_generation()` は `solve` が最初に呼ぶ。低レベルの `search` / `extract` を自分で呼ぶ場合だけ、自分で呼ぶ（04 §4）。

### ほかに問えること

- **この詰みをどう受けるか。** `VCTState::threat_defences(&mate)` が、試す価値のある受けを返す。内訳は、詰み手順の各点、詰め上がりを崩す点、ノリ手、受け方自身の四を作る手。追い詰め探索が受けの候補として使うのと同じリスト（06 §3）。
- **この手は追い手か。** 手番側をパスさせて（`Game::play(None)`）、相手の四追いを問う。パスした `game` から `VCFState::new(game, limit)` を作り、`DFSSolver::init().solve(..)` する。`test_threat_after_pass` が 1 手ずつ確認している。

## 5. `Mate` と `End`

```rust
pub struct Mate { pub end: End, pub path: Vec<Point> }
pub enum End { Fours(Point, Point), Forbidden(Point), Unknown }
```

- `path`: 攻め方から始まり、攻めと受けが交互に並ぶ。`n_moves()` は長さ、`n_times()` は攻め手数。
- `end`: 最後の手の後、受け方がなぜ負けているか。

| `End` | 受け方の状況 | 誰に起きるか |
| --- | --- | --- |
| `Fours(p1, p2)` | 勝ち点の異なる四が 2 つ。1 手では止まらない。棒四（`Square` は眼の異なる 2 つの `Four` として報告する）または四四。 | どちらも。ただし攻め方が黒なら棒四だけ（四四は禁手なので打たれない）。 |
| `Forbidden(p)` | 四が 1 つで、唯一の止め `p` が禁手。 | 黒のみ。 |
| `Unknown` | 勝ちは証明されたが手順を復元できなかった。探索前から攻め方に四があった（§6）か、復元でたどれる証明済みの子がなかった（06 §6）。 | — |

## 6. 探索前の検査: `validate`

`solve` は最初に、ソルバーが扱えない局面を弾く。

| 局面 | 結果 |
| --- | --- |
| どちらかにすでに `Five` がある | `Disproven` |
| 黒にすでに `Overlined` がある | `Disproven` |
| 攻め方にすでに `Four` がある | `Proven(Mate { end: Unknown, path: [] })`（すでに勝ち。証明するものがない） |

ソルバーを直接使う場合（§4）にはこの検査は入らない。必要なら自分で確かめる。

## 7. チートシート

| したいこと | 使うもの |
| --- | --- |
| 1 回だけ問う | `solve` |
| 四追いだけ探す | `SolveMode::VCFDFS` |
| 追い詰めを探す | `SolveMode::VCTDFPNS`。追い手の範囲は `threat_limit` |
| 長すぎる探索を止める | `SolveLimits::with_max_nodes`。`Aborted` は「不明」であって「安全」ではない |
| 多くの問いを安く投げる | `DFPNSVCTSolver` を保持。問いごとに新しい `VCTState`。`NodeBudget` は 1 つを共有 |
| 両方の色について問う | 同じソルバーでよい。攻め方はキーに入っている |
| 受けの候補を知る | `VCTState::threat_defences(&mate)` |
| 直前の手が追い手か知る | `Game::play(None)` でパスし、相手の四追いを問う |
| JS から詰み手順を読む | `wasm::solve` は手順を `u8` の点コード（02 §1）で返す。`decode_x` / `decode_y` |
| モードを追加する | `SolveMode` と `TryFrom<u8>`、`FromStr`、`solve` の `match`（`solve.rs`） |

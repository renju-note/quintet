# 詰みソルバーの使い方

ソルバーを**呼ぶ**側のためのドキュメントである。`wasm.rs` から、`examples/solve.rs` の CLI から、あるいはソルバーを保持して何度も問いかける Rust コードから。探索の中身を知る必要はない。中身は [04](04-solver-framework.ja.md)、[05](05-solver-vcf.ja.md)、[06](06-solver-vct.ja.md) にある。

[01](01-renju-rules.ja.md) の用語（四、棒四、三、禁手）を前提にする。以下のものはすべて `src/mate/` にあり、`quintet::mate` から再エクスポートされている。

## 1. ソルバーが答える問い

どの呼び出しも同じことを問う:

> この `board` で、`attacker` の手番のとき、攻め方は `limit` 手以内の**詰み**（必勝手順）を持つか？

- **攻め方（attacker）**は手番側であり、勝ちを証明したい側である。相手はコード全体で**受け方（defender）**と呼ぶ。どちらの色も攻め方になれる。
- 詰みとは **VCF**（四追い: 攻め手がすべて四）または **VCT**（追い詰め: 攻め手がすべて追い手。四か、放置すれば四追いになる手）である。どちらを問うかが `SolveMode` である。
- `limit` は攻め方の手だけを数える。7 手（攻め 4 手、受け 3 手）の詰み手順には `limit >= 4` が要る。

肯定の答えは [`Mate`](#5-mate-と-end) である。詰み手順と、その最後で受け方がなぜ負けているかを持つ。

## 2. 使い捨て: `solve` と `solve_limited`

```rust
pub fn solve(mode: SolveMode, limit: u8, board: &Board, attacker: Player, threat_limit: u8) -> Option<Mate>
pub fn solve_limited(mode: SolveMode, board: &Board, attacker: Player, limits: SolveLimits) -> SolveResult
```

どちらもソルバーを作り、1 回走らせ、捨てる。`solve` は `wasm.rs` が呼ぶ短い形で、`solve_limited` は同じ探索にノード予算と 3 値の答えを付けたものである。`solve` は `solve_limited` を使って書かれている。

```rust
use quintet::board::{Board, Player};
use quintet::mate::{SolveLimits, SolveMode, SolveResult, solve_limited};

let board: Board = "H8,I9,J9,H7".parse().unwrap();
let limits = SolveLimits::new(5).with_threat_limit(2).with_max_nodes(100_000);
match solve_limited(SolveMode::VCTDFPNS, &board, Player::Black, limits) {
    SolveResult::Proven(mate) => { /* mate.path が詰み手順 */ }
    SolveResult::Disproven => { /* この limit では詰みなし */ }
    SolveResult::Aborted => { /* 予算切れ。まだ不明 */ }
}
```

### `SolveMode`

| `SolveMode` | コード | CLI 名 | 探索するもの |
| --- | --- | --- | --- |
| `VCFDFS` | 0 | `vcf` | 四追い、深さ優先。`threat_limit` は無視される。 |
| `VCFIDDFS` | 1 | `vcf_iddfs` | 予約。`solve` は `None` を返す。 |
| `VCTDFS` | 10 | `vct` | 追い詰め。証明数木を深さ優先でたどる。 |
| `VCTIDDFS` | 11 | `vct_iddfs` | 予約。`solve` は `None` を返す。 |
| `VCTPNS` | 15 | `vct_pns` | 追い詰め、証明数探索。 |
| `VCTDFPNS` | 16 | `vct_dfpns` | 追い詰め、深さ優先証明数探索（df-pn）。**通常はこれを使う。** |

コードは wasm/JS 側の表現（`SolveMode::try_from(u8)`）であり公開 API である。番号を振り直してはならない。CLI 名は `SolveMode::from_str` である。3 つの VCT モードは同じ木を違う順序で探索し、同じ結論に達する。違うのは速さと、複数の詰み手順のうちどれを先に報告するかである。

### `SolveLimits`: 探索の上限

```rust
SolveLimits::new(limit)
    .with_threat_limit(threat_limit)     // 既定 0
    .with_defender_vcf_depth(depth)      // 既定 DEFAULT_DEFENDER_VCF_DEPTH = 2
    .with_max_nodes(nodes)               // 既定: 無制限
```

| フィールド | 上限を課すもの | 対象 |
| --- | --- | --- |
| `limit` | 本手順の攻め手数 | 全モード |
| `threat_limit` | ある手が**追い手**かどうかを決める内部四追いの攻め手数（06 の §2）。`0` は四だけ、`1` は四と三、`2` 以上は四三を準備する手やさらに深い含みも認識する（`test_vct_fukumi_move` には `3` が要る）。 | VCT |
| `defender_vcf_depth` | **受け方**の逆襲四追いを探す内部四追いの攻め手数。 | VCT |
| `max_nodes` | 仕事量の総計。ノード数（§3） | 全モード |

`with_*` は新しい値を返すので、後からフィールドが増えてもこの書き方の呼び出し側は壊れない。

### `SolveResult`

```rust
pub enum SolveResult { Proven(Mate), Disproven, Aborted }
```

`Option<Mate>` は「なぜ詰みがないか」を言えない。`SolveResult` は言える。そしてその違いは重要である。`Disproven` は「この上限では詰みがない」、`Aborted` は「予算が尽きて、局面はまだ未決」である。`Aborted` を「安全」と読むエンジンは詰まされる。`is_proven()`、`is_disproven()`、`is_aborted()`、`mate()`、`into_mate()` がアクセサである。

## 3. 予算: `NodeBudget`

時計はない。`src/` 以下はすべて `wasm32-unknown-unknown` 向けにコンパイルでき、そこに `std::time` はない。時間制御を持つ呼び出し側は、自分でノード数に換算する。

```rust
let mut budget = NodeBudget::new(100_000);   // または NodeBudget::unlimited()
budget.is_exhausted();                       // 探索は予算切れになったか
budget.nodes();                              // これまでに数えたノード数
budget.restart();                            // ゼロに戻す。上限はそのまま
```

1 ノードは探索関数への 1 回の訪問である。四追いでは `DFSSolver::search`、追い詰めでは `search_attacks` / `search_defences` で、内部の四追い探索も含む。尽きた予算は `restart()` するまで尽きたままなので、1 つの予算を共有する一連の呼び出しは、それぞれが少しずつ余計に働くのではなく、全体として止まる。

予算を安全に使えるのは次の 2 つの保証による:

- **証明は偽にならない。** 予算が尽きる直前に見つかった詰みは本物であり、そのまま返される。
- **打ち切られた探索は偽の情報を残さない。** そこで計算したものはどのメモにも書かれない（04 の §5）ので、同じソルバーをすぐ再利用できる。`test_abort_leaves_no_wrong_memo` は、1 つのソルバーをさまざまな小さい予算で打ち切り、その後も正しい答えを返すことを確かめている。

## 4. ソルバーを保持する: `Solver` トレイト

`solve` はソルバーを捨てる。探索が埋めた表もいっしょに捨てる。関連する問いを何度も投げる側（対局エンジン、変化をたどる解析画面）は、ソルバーを 1 つ持ち続けるべきである。そのために部品はすべて公開されている:

```rust
use quintet::mate::{DFPNSVCTSolver, NodeBudget, Solver, VCTState, DEFAULT_DEFENDER_VCF_DEPTH};

let mut solver = DFPNSVCTSolver::init(threat_limit, DEFAULT_DEFENDER_VCF_DEPTH);
let budget = &mut NodeBudget::new(1_000_000);   // ループ全体の上限
for board in candidates {
    let state = &mut VCTState::init(&board, attacker, limit);
    match solver.solve(state, budget) {
        Some(mate) => { /* 証明された */ }
        None if budget.is_exhausted() => break,
        None => { /* limit 以内に詰みなし */ }
    }
}
```

すべてのソルバーは `Solver`（`mate/solver.rs`）を実装する:

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
| `IDDFSSolver` | `VCFState` | `init(limits)` | 四追い。`limits` の各値で順に |
| `VCTSolver<P>` — エイリアス `DFSVCTSolver`、`PNSVCTSolver`、`DFPNSVCTSolver` | `VCTState` | `init(threat_limit, defender_vcf_depth)` | 追い詰め |

状態は `VCFState::init(&board, attacker, limit)` または `VCTState::init(&board, attacker, limit)` で作る。問いごとに新しく作ればよい。持ち越すのはソルバーのほうである。

使い回して何が起きるか:

- **何を、どの順で問うてもよい。** すべてのメモは、局面、手番、残り limit、そして**攻め方**をキーに含む。だから 1 つのソルバーで、同じ盤面の黒の詰みと白の詰みを混同せずに問える（`test_reused_solver_both_attackers`）。
- **メモリは有界に保たれる。** `solve` は毎回新しい**世代**を開く。メモが引き継ぎ容量を超えて育つと、直前の探索より古いエントリが捨てられる。何回問おうとメモはおよそ探索 2 回分に落ち着く（`test_reused_solver_memo_stays_bounded`）。各ソルバーの `with_carry_capacity(..)` がその閾値を決め（既定はメモあたり `1 << 16` エントリ）、`memo_len()` が現在の保持数を返す。
- **同じ問いの繰り返しはほぼ無料である。** 探索まるごとではなく数ノードで済む（`test_reused_solver`）。
- **同じ局面を別の limit で問うのは安い。** 決着は境界である。4 手以内の詰みは 5 手でも 6 手でも詰みであり、3 手以内に詰みがなければ 2 手でも 1 手でもない。1 局面を limit 1〜6 で深化させると、新しい探索 6 回の約 5 分の 1 で済む（`test_decisions_carry_between_limits`）。
- **別の局面どうしはあまり共有しない。** 残り limit がキーの一部だからである。同じ盤面に 2 つの根から到達しても、根からの距離が同じでなければ別のエントリになる。
- `clear()` はすべて忘れる。必須ではない。まっさらな状態でソルバーを渡したいときや、メモリを返したいときに使う。`advance_generation()` は `solve` が最初に呼ぶものであり、ソルバーの低レベルな `search` / `extract` を自分で駆動する呼び出し側だけが必要とする（04 の §4）。

### 呼び出し側がさらに問えること

- **この詰みをどう受けるか？** `VCTState::threat_defences(&mate)` は、見つかった詰みに対して試す価値のある手を返す。詰み手順そのもの、詰め上がりを崩す点、ノリ手、受け方自身の四を作る手である。追い詰め探索が受けの候補を作るのに使うのと同じリストである（06 の §3）。
- **この手は追い手か？** 手番側にパスさせ（`Game::play(None)`）、相手の四追いを問う。パスした game で `VCFState::new(game, limit)` を作り、`DFSSolver::init().solve(..)` する。`test_threat_after_pass` が 1 手ずつ確かめている。

## 5. `Mate` と `End`

```rust
pub struct Mate { pub end: End, pub path: Vec<Point> }
pub enum End { Fours(Point, Point), Forbidden(Point), Unknown }
```

`path` は攻め方から始まり、攻め方と受け方の手が交互に並ぶ。`Mate::n_moves()` はその長さ、`n_times()` はそのうちの攻め手数である。`end` は、最後の手の後で受け方がなぜ負けているかを言う:

| `End` | 受け方が直面しているもの | 誰がこうなるか |
| --- | --- | --- |
| `Fours(p1, p2)` | 勝ち点の異なる四が 2 つ（`p1`、`p2`）。1 手では止まらない。棒四（`Square` は眼の異なる 2 つの `Four` として報告する）または四四。 | どちらも。ただし攻め方が黒のときは棒四だけである。四四は黒の禁手であり、打たれることがないため。 |
| `Forbidden(p)` | 四が 1 つで、唯一の止め `p` が禁手。 | 黒のみ。 |
| `Unknown` | 勝ちは証明されたが手順を完成できなかった。探索前から攻め方に四があった（§6）か、復元器がたどれる証明済みの子を見つけられなかった（06 の §6）。 | — |

## 6. 探索前の検査: `validate`

`solve_limited` は最初に、ソルバーが扱わない局面を弾く:

| 局面 | 結果 |
| --- | --- |
| どちらかの色にすでに `Five` がある | `Disproven` |
| 黒にすでに `Overlined` がある | `Disproven` |
| 攻め方にすでに `Four` がある | `Proven(Mate { end: Unknown, path: [] })` — すでに勝ちで、証明するものがない |

ソルバーを直接使う側（§4）にはこの検査は掛からない。必要なら自分で確かめる。

## 7. チートシート

| したいこと | 使うもの |
| --- | --- |
| 1 回だけ、あるか / ないか / 手順を得る | `solve`（予算と 3 値の答えが要るなら `solve_limited`） |
| 四追いだけを探す | `SolveMode::VCFDFS` |
| 追い詰めを探す | `SolveMode::VCTDFPNS`。何を追い手と見るかは `threat_limit` |
| 長すぎる探索を止める | `SolveLimits::with_max_nodes`。`Aborted` は「安全」ではなく「不明」と扱う |
| 多くの問いを安く投げる | `DFPNSVCTSolver` を保持し、問いごとに新しい `VCTState` を渡し、`NodeBudget` を 1 つ共有する |
| 両方の色について問う | 同じソルバーでよい。攻め方はすべてのキーに入っている |
| どの受けを試すべきか | `VCTState::threat_defences(&mate)` |
| 直前の手は追い手だったか | `Game::play(None)` でパスし、相手の四追いを問う |
| JS から詰み手順を読む | `wasm::solve` は手順を `u8` の点コード（02 の §1）で返す。`decode_x` / `decode_y` |
| モードを追加する | `SolveMode` とその `TryFrom<u8>`、`FromStr`、`solve_limited` の `match`（`solve.rs`） |

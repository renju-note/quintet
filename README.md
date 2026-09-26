# quintet

A [Renju](https://www.renju.net/rifrules/) mate solver written in Rust and
compiled to WebAssembly. Given a position and a side to move, it searches for
a forced win and returns the winning sequence.

quintet powers the analysis features of [renju-note](https://github.com/renju-note),
and is published to npm as
[`@renju-note/quintet`](https://www.npmjs.com/package/@renju-note/quintet).

## What it solves

| Mode | Meaning |
| --- | --- |
| **VCF** (Victory by Continuous Fours) | A forced win using only *fours*: every attacking move threatens to complete five, so every reply is forced. |
| **VCT** (Victory by Continuous Threats) | A forced win using *fours and threes*. The defender may answer a three in several ways, so this is a proper AND/OR tree search, solved with proof numbers. |

Renju rules are fully modelled: Black's forbidden moves (double-three,
double-four, overline) are detected — including the recursive "real vs. fake
three" rule — and can be both an obstacle for a Black attacker and a winning
resource for a White attacker. Opening restrictions and passing are not
modelled; the solver takes an arbitrary position.

## Usage

### From JavaScript

```sh
npm install @renju-note/quintet
```

```js
import init, { solve_vct_dfpn, encode_xy, decode_x, decode_y } from "@renju-note/quintet";

await init();

// Points are encoded as a single byte: x * 15 + y, with x, y in 0..15
// (x = column A..O, y = horizontal line 1..15).
const blacks = [encode_xy(7, 7), encode_xy(7, 6)]; // H8, H7
const whites = [encode_xy(6, 6)];                  // G7

// solve_vct_dfpn(blacks, whites, blackToMove, limit) -> Uint8Array | undefined
const path = solve_vct_dfpn(blacks, whites, true, 10);
if (path) {
  const moves = Array.from(path, (p) => [decode_x(p), decode_y(p)]);
  // moves alternates attacker, defender, attacker, ...
}
```

Exported functions (`src/wasm.rs`):

| Function | Description |
| --- | --- |
| `solve(mode, limit, blacks, whites, black, threat_limit)` | General entry point. `mode` is a numeric `SolveMode` code: `0` = VCF, `10` = VCT (DFS), `15` = VCT (PNS), `16` = VCT (df-pn). |
| `solve_vcf(blacks, whites, black, limit)` | Shorthand for `mode = 0`. |
| `solve_vct(blacks, whites, black, limit)` | Shorthand for `mode = 10`, `threat_limit = limit`. |
| `solve_vct_dfpn(blacks, whites, black, limit)` | Shorthand for `mode = 16`, `threat_limit = limit`. The recommended VCT solver. |
| `encode_xy(x, y)` / `decode_x(code)` / `decode_y(code)` | Point encoding helpers. |

All solvers return the winning path as encoded points, or nothing if no win
is found within the limits.

### From Rust

```toml
[dependencies]
quintet = { git = "https://github.com/renju-note/quintet" }
```

```rust
use quintet::board::*;
use quintet::mate::*;

let board: Board = "H8,H7,F6".parse()?; // moves alternating Black, White, ...
let limits = SolveLimits::new(10).with_threat_limit(3);
let result = solve(SolveMode::VCTDFPNS, &board, Player::Black, limits);
if let Some(m) = result.into_mate() {
    println!("{}", Points(m.path));
}
```

### Parameters

- **`limit`** — maximum number of *attacker* moves in the solution. Larger
  values find longer wins but take longer.
- **`threat_limit`** — depth of the nested VCF search used to decide whether
  a move is a threat (VCT only). `1` recognises only fours and threes as
  threats; higher values also find "hidden" threats that prepare a
  four-three.

### Command line

`examples/solve.rs` wraps the solver for quick experiments:

```sh
cargo run --release --example solve <mode> <limit> <threat_limit> <o|x> <position>
```

- `mode`: `vcf`, `vct`, `vct_pns` or `vct_dfpns`
- `o|x`: the attacker (`o` = Black, `x` = White)
- `position`: a move list `H8,H7,F6,...` (alternating from Black), or a
  stone list `H8,F6/H7` written as `blacks/whites`

Example — White has a 15-move VCT ending in a forbidden move for Black:

```
% cargo run --release --example solve vct_dfpns 255 3 x \
  H8,I9,H7,J8,F8,H9,G9,H10,G11,F10,I10,G10,I8,J9,E10,K9,L9,G8,J10,K7,L6,K8,K6,L10,I7,J6,M9,I5,H4,F7,G6,E6,D5,F5,G4
Mode: VCTDFPNS
Limit: 255
ThreatLimit: 3
Attacker: White
Board:
15 . . . . . . . . . . . . . . .
14 . . . . . . . . . . . . . . .
13 . . . . . . . . . . . . . . .
12 . . . . . . . . . . . . . . .
11 . . . . . . o . . . . . . . .
10 . . . . o x x x o o . x . . .
 9 . . . . . . o x x x x o o . .
 8 . . . . . o x o o x x . . . .
 7 . . . . . x . o o . x . . . .
 6 . . . . x . o . . x o o . . .
 5 . . . o . x . . x . . . . . .
 4 . . . . . . o o . . . . . . .
 3 . . . . . . . . . . . . . . .
 2 . . . . . . . . . . . . . . .
 1 . . . . . . . . . . . . . . .
   A B C D E F G H I J K L M N O

Solving...

Elapsed: 889.191875ms
End: Forbidden(L7)
Times (Length): 15 (29)
Moves: K11,K10,N12,M11,N8,H5,H6,L8,J5,J7,M5,L4,M6,K5,J4,K3,J3,J2,K4,L5,M4,L3,L2,F3,I6,E2,D1,M3,N5
```

(Original game: <https://www.renju.net/media/games.php?gameid=92337>)

## How it works

- `src/board/` — bit-packed board representation. Every rule concept (five,
  four, three, forbidden move) is detected by sliding a 5-cell window over
  bitmask lines.
- `src/mate/vcf/` — depth-first VCF search over four-making moves, with a
  transposition memo.
- `src/mate/vct/` — VCT search as a proof-number AND/OR tree. The DFS, PNS
  and df-pn variants share one solver and differ only in their threshold
  policy. Threats are recognised by running a nested VCF search after a
  hypothetical pass.
- `src/feature/` — potential-field heuristics used for move ordering.
- `src/wasm.rs` — the `wasm-bindgen` surface.

The [`docs/`](docs/README.en.md) directory has detailed documentation in
English and Japanese: the Renju rules, how the board code implements them,
and how the solvers work.

## Development

```sh
cargo test --release                      # always test in release; the solvers are slow in debug
cargo test --release -- --ignored         # also run the slow tests
cargo bench --bench solvers               # solver benchmark (see docs/07-benchmarks.en.md)
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build --target wasm32-unknown-unknown
```

Everything under `src/` must stay compilable for `wasm32-unknown-unknown`:
no threads, no filesystem, no `std::time` in library code.

## Release

Publishing a GitHub release triggers
[`.github/workflows/wasm-pack-release.yml`](.github/workflows/wasm-pack-release.yml),
which sets the crate version from the tag (`vX.Y.Z`), runs
`wasm-pack build --scope renju-note` and publishes the package to npm with
trusted publishing. To build the package locally:

```sh
wasm-pack build --scope renju-note
```

## License

[MIT](LICENSE)

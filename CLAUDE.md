# CLAUDE.md

Guidance for Claude Code when working in this repository.

## What this is

`quintet` is a Renju (five-in-a-row with forbidden-move rules for Black) mate
solver written in Rust. It is a library crate (`rlib`) that is also compiled to
WebAssembly (`cdylib` via `wasm-bindgen`) and published to npm as
`@renju-note/quintet` on GitHub release. The primary consumer is the
[renju-note](https://github.com/renju-note) web app.

## Commands

```bash
cargo test --release          # run all tests (release: solvers are too slow in debug)
cargo test --release -- --ignored   # also run the slow bench cases in src/mate/solve.rs
cargo fmt --check
cargo clippy --all-targets
cargo build --target wasm32-unknown-unknown   # verify the wasm target still compiles
wasm-pack build --scope renju-note            # what CI runs on release

# CLI for manual experiments (examples/solve.rs)
cargo run --release --example solve <mode> <limit> <threat_limit> <x|o> <moves>
#   mode: vcf | vcf_iddfs | vct | vct_iddfs | vct_pns | vct_dfpns | vct_lazy
#   moves: comma-separated like H8,H7,F6 (alternating Black, White, ...)
```

Always run tests in `--release`; the debug build is an order of magnitude
slower and some cases time out.

## Layout

- `src/board/` — board representation. `Point`/`Points` (15x15, `A1`..`O15`,
  encoded to a single `u8` for the wasm boundary), `Line`/`Square` (bit-packed
  rows and 4 line directions), `Structure`/`StructureKind` (three, four, five,
  overline...), `forbidden.rs` (Renju forbidden-move detection for Black),
  `potential.rs`, `zobrist.rs` (hashing for transposition tables).
- `src/analysis/` — `field.rs`, per-point potential evaluation used for move
  ordering.
- `src/mate/` — the solvers. `solve.rs` is the entry point (`solve`,
  `SolveMode`) and also holds the integration tests. `game.rs`/`state.rs`/
  `mate.rs` are shared search-state types.
  - `vcf/` — Victory by Continuous Fours (DFS and IDDFS).
  - `vct/` — Victory by Continuous Threats: DFS, PNS and df-pn solvers built
    from `generator` (move generation), `searcher`, `resolver`, `selector`,
    `traverser`, `proof` (proof/disproof numbers).
  - `vct_lazy/` — an experimental lazy-expansion VCT variant with the same
    shape as `vct/`.
- `src/wasm.rs` — the `#[wasm_bindgen]` surface (`solve`, `solve_vcf`,
  `solve_vct`, `solve_vct_dfpn`, `encode_xy`/`decode_x`/`decode_y`).
- `examples/solve.rs` — CLI wrapper over `mate::solve`.
- `docs/` — reference documentation for humans and AI agents: the Renju
  rules (`docs/01-renju-rules.en.md`) and how `src/board/` implements them
  (`docs/02-board-implementation.en.md`). Read these before touching rule logic
  (structures, forbidden moves) and keep them in sync when changing it.
  Every document has an English `*.en.md` and a Japanese `*.ja.md` version;
  always add or edit both together.

## Conventions and constraints

- Everything under `src/` must stay compilable for `wasm32-unknown-unknown`:
  no threads, no filesystem, no `std::time` in library code, and be careful
  adding dependencies (check they are `no_std`/wasm friendly).
- The `SolveMode` numeric codes in `src/mate/solve.rs` (`0`, `1`, `10`, ...)
  and the wasm function signatures are a public API consumed by JS. Do not
  renumber or change them without a matching consumer change.
- Point encoding (`Point -> u8`) is also part of the public API.
- Performance matters: this is a search engine. Prefer bit operations and
  fixed-size arrays over allocation in hot paths; avoid `clone()` of boards
  and states in the search loop.
- Tests are inline `#[cfg(test)] mod tests` in each file. Solver regression
  tests use ASCII boards parsed with `.parse::<Board>()` and assert the exact
  solution path string. When fixing a solver bug, add such a case to
  `src/mate/solve.rs`.
- Formatting is `rustfmt` default. Keep `cargo fmt --check` clean.
- Commit messages and PR titles are short English imperative phrases
  (e.g. `Fix VCF bug`, `Refactor VCT solver`). Squash-merged PRs get `(#N)`
  appended automatically.

## Glossary

- **VCF** — Victory by Continuous Fours: a forced win using only "four"
  threats.
- **VCT** — Victory by Continuous Threats: forced win using fours and threes.
- **Forbidden move** — under Renju rules Black may not play double-three,
  double-four or overline; White has no such restriction.
- **PNS / df-pn** — proof-number search / depth-first proof-number search,
  best-first AND/OR tree search used by the VCT solvers.
- **limit / threat_limit** — max depth for the attacker's sequence, and max
  depth of the nested VCF search used to evaluate threats.

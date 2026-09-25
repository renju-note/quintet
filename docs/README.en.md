# Documentation

Reference material for people and AI agents working on `quintet`. These
documents explain the domain (Renju) and how the domain rules map onto the
code, so that changes to the solver can be reasoned about without having to
re-derive everything from source.

Every document exists in English (`*.en.md`) and Japanese (`*.ja.md`).
Japanese index: [README.ja.md](README.ja.md).

| Document | What it covers |
| --- | --- |
| [01-renju-rules.en.md](01-renju-rules.en.md) / [ja](01-renju-rules.ja.md) | The RIF International Rules of Renju (board, terminology, win conditions, forbidden moves, opening rules) restated in Markdown. |
| [02-board-implementation.en.md](02-board-implementation.en.md) / [ja](02-board-implementation.ja.md) | How `src/board/` represents the board and implements the rules above: line encoding, sequence/structure detection, forbidden-move detection, hashing. |
| [03-solver-api.en.md](03-solver-api.en.md) / [ja](03-solver-api.ja.md) | Using the solvers: `solve`, `SolveMode`, `SolveLimits`, `SolveResult`, `NodeBudget`, keeping a solver across questions with the `Solver` trait, `Mate` / `End`. |
| [04-solver-framework.en.md](04-solver-framework.en.md) / [ja](04-solver-framework.ja.md) | Inside `src/mate/`: the layout and the pieces every solver shares — `Game` and `check_event`, `State` and `Key`, `Memo` generations, the `Solver` trait, node budgets. |
| [05-solver-vcf.en.md](05-solver-vcf.en.md) / [ja](05-solver-vcf.ja.md) | The VCF search in `src/mate/vcf/`: four-making move pairs, `DFSSolver` and its deadend memo, `IDDFSSolver`, a worked example. |
| [06-solver-vct.en.md](06-solver-vct.en.md) / [ja](06-solver-vct.ja.md) | The VCT search in `src/mate/vct/`: threats, nested VCF searches, move generation, proof numbers, the DFS / PNS / df-pn threshold policies, path extraction, a worked example, and the `PotentialField` used for move ordering. |
| [07-benchmarks.en.md](07-benchmarks.en.md) / [ja](07-benchmarks.ja.md) | The solver benchmark in `benches/`: what it measures (nodes, memo entries, time), running it, comparing two versions, and adding cases. |

Where to start:

- New to Renju? Read 01 first; the solver documents assume its vocabulary
  (four, straight four, three, forbidden move).
- Changing rule logic in `src/board/` (structures, forbidden moves)? Read 02.
- Calling the solvers from the app, the CLI or your own Rust? Read 03.
- Changing the search in `src/mate/` or `src/feature/`? Read 04, then 05
  and 06, and measure the change with the benchmark in 07.
- Looking for one specific thing? Each document ends with a cheat sheet that
  maps questions to identifiers (02 §9, 03 §7, 04 §7, 05 §5, 06 §9).

Conventions for adding documents:

- One topic per file, numbered in reading order: `NN-kebab-case.en.md`
  **and** `NN-kebab-case.ja.md`, both linked from the table above
  (`README.*` has no number). Always add and update the two languages
  together; they must say the same thing.
- Do not hard-wrap paragraphs in the Japanese files: Markdown renders a
  line break inside a paragraph as a space, which shows up as a stray gap in
  Japanese text. English files may wrap as usual.
- Prefer ASCII board diagrams in the same format the tests use
  (`o` = Black, `x` = White, `.` = empty, row 15 at the top) so examples can be
  pasted straight into a `.parse::<Board>()` test.
- When a document describes code, reference the actual identifiers
  (`Grid::structures_on`, `StructureKind::Sword`, ...) so it can be
  cross-checked with `grep`.

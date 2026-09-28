# Documentation

Reference for people and AI agents working on `quintet`. It explains the
domain (Renju) and how its rules map onto the code, so that a change to the
solver can be reasoned about without re-deriving everything from source.

Every document has an English (`*.en.md`) and a Japanese (`*.ja.md`)
version. Japanese index: [README.ja.md](README.ja.md).

| Document | What it covers |
| --- | --- |
| [01-renju-rules.en.md](01-renju-rules.en.md) / [ja](01-renju-rules.ja.md) | The RIF International Rules of Renju in Markdown: board, terms, win conditions, forbidden moves, opening rules. |
| [02-board-implementation.en.md](02-board-implementation.en.md) / [ja](02-board-implementation.ja.md) | How `src/board/` represents the board and implements the rules: line encoding, segments and rows, forbidden moves, hashing. |
| [03-solver-api.en.md](03-solver-api.en.md) / [ja](03-solver-api.ja.md) | Calling the solvers: `solve`, `SolveMode`, `SolveLimits`, `SolveResult`, `NodeBudget`, reusing a solver through the `Solver` trait, `Mate` / `End`. |
| [04-solver-framework.en.md](04-solver-framework.en.md) / [ja](04-solver-framework.ja.md) | Inside `src/mate/`: the layout and the shared pieces — `Game` and `check_event`, `State` and `Key`, `Memo` generations, the `Solver` trait, node budgets. |
| [05-solver-vcf.en.md](05-solver-vcf.en.md) / [ja](05-solver-vcf.ja.md) | The VCF search in `src/mate/vcf/`: four-making move pairs, `DFSSolver` and its dead-end memo, `IDDFSSolver`, a worked example. |
| [06-solver-vct.en.md](06-solver-vct.en.md) / [ja](06-solver-vct.ja.md) | The VCT search in `src/mate/vct/`: threats, nested VCF searches, move generation, proof numbers, the DFS / PNS / df-pn threshold policies, path extraction, a worked example, move ordering (`ShapeMap`). |
| [07-benchmarks.en.md](07-benchmarks.en.md) / [ja](07-benchmarks.ja.md) | The solver benchmark in `benches/`: what it measures (nodes, memo entries, time), running it, comparing two versions, adding cases. |
| [08-performance.en.md](08-performance.en.md) / [ja](08-performance.ja.md) | Performance history: how much faster the solvers got since 2024, each PR's effect on nodes and time, cases that got slower, where the time goes now, what was tried and what is left. |

Where to start:

- New to Renju: read 01 first. The solver documents use its terms (four,
  straight four, three, forbidden move).
- Changing rule logic in `src/board/` (rows, forbidden moves): read 02.
- Calling the solvers from the app, the CLI or Rust: read 03.
- Changing the search in `src/mate/` or `src/feature/`: read 04, then 05
  and 06, and measure with the benchmark in 07.
- Looking for what to speed up next, or whether an idea was tried: 08.
- Looking for one thing: each document ends with a cheat sheet from
  questions to identifiers (02 §8, 03 §7, 04 §7, 05 §5, 06 §9).

Conventions for writing documents:

- Files
  - One topic per file, numbered in reading order:
    `NN-kebab-case.en.md` **and** `NN-kebab-case.ja.md`, both linked from
    the table above (`README.*` has no number).
  - Add and update both languages together; they must say the same thing.
- Style
  - Be concise. Short sentences; split a long explanation into a bulleted
    list or a table.
  - Refer to the code by its real identifiers (`Grid::rows_on`,
    `RowKind::Sword`, ...) so it can be cross-checked with `grep`.
  - Draw boards in the ASCII format the tests use (`o` = Black,
    `x` = White, `.` = empty, line 15 at the top), so an example can be
    pasted into a `.parse::<Board>()` test.
- Japanese files
  - Do not hard-wrap paragraphs: Markdown renders the line break as a
    space, a stray gap in Japanese text. English files may wrap.
  - In code blocks (pseudo code, diagrams, trees), write everything in
    ASCII except a trailing comment at the end of a line. Full-width
    characters break the alignment.

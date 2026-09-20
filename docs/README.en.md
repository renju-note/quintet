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
| [03-solver-overview.en.md](03-solver-overview.en.md) / [ja](03-solver-overview.ja.md) | Overview of `src/mate/`: `solve` and its parameters (`limit`, `threat_limit`, `SolveMode`), the shared search state (`Game`, `State`, `check_event`), and the VCF depth-first search. |
| [04-solver-algorithm-vct.en.md](04-solver-algorithm-vct.en.md) / [ja](04-solver-algorithm-vct.ja.md) | The VCT solvers in `src/mate/vct/`: threats, move generation, proof numbers, the DFS / PNS / df-pn traversals, path extraction, and the `PotentialField` used for move ordering. |

Where to start:

- New to Renju? Read 01 first; the solver documents assume its vocabulary
  (four, straight four, three, forbidden move).
- Changing rule logic in `src/board/` (structures, forbidden moves)? Read 02.
- Changing the search in `src/mate/` or `src/analysis/`? Read 03, then 04.
- Looking for one specific thing? Each document ends with a cheat sheet that
  maps questions to identifiers (02 §9, 03 §4, 04 §10).

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
  (`Square::structures_on`, `StructureKind::Sword`, ...) so it can be
  cross-checked with `grep`.

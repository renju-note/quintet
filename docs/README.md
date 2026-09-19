# Documentation

Reference material for people and AI agents working on `quintet`. These
documents explain the domain (Renju) and how the domain rules map onto the
code, so that changes to the solver can be reasoned about without having to
re-derive everything from source.

| Document | What it covers |
| --- | --- |
| [renju-rules.md](renju-rules.md) | The RIF International Rules of Renju (board, terminology, win conditions, forbidden moves, opening rules) restated in Markdown. |
| [board-implementation.md](board-implementation.md) | How `src/board/` represents the board and implements the rules above: line encoding, sequence/structure detection, forbidden-move detection, hashing. |

Conventions for adding documents:

- One topic per file, `kebab-case.md`, linked from the table above.
- Prefer ASCII board diagrams in the same format the tests use
  (`o` = Black, `x` = White, `.` = empty, row 15 at the top) so examples can be
  pasted straight into a `.parse::<Board>()` test.
- When a document describes code, reference the actual identifiers
  (`Square::structures_on`, `StructureKind::Sword`, ...) so it can be
  cross-checked with `grep`.

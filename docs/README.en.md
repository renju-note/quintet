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
| [03-solver-algorithm.en.md](03-solver-algorithm.en.md) / [ja](03-solver-algorithm.ja.md) | How `src/mate/` and `src/analysis/` search for mates: `solve` and its parameters, the VCF depth-first search, the VCT proof-number search (DFS / PNS / df-pn), the lazy variant, and the potential field used for move ordering. |

Conventions for adding documents:

- One topic per file, numbered in reading order: `NN-kebab-case.en.md` **and** `NN-kebab-case.ja.md` (`README.*` has no number), both
  linked from the table above. Always add and update the two languages
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

# International Rules of Renju (RIF)

This is a Markdown restatement of the *International Rules of Renju* adopted
by the Renju International Federation (RIF) on 2 May 1996, with the
correction of 3 May 1998. The authoritative text is published at
<https://www.renju.net/rifrules/>; this page follows its section numbering
but paraphrases the wording. Where the original and this page disagree, the
original wins.

Sections that matter for a mate solver (2, 3, 4, 9, 10, 11, 12) are written
out in full. Sections about tournament procedure (clocks, scoresheets,
arbiters, conduct) are summarized only.

---

## 1. Introduction

Renju is a two-player game played on a board with black and white pieces
called *stones*.

## 2. The board

- 15 × 15 lines, giving 225 intersections.
- Five intersections are marked as reference points (star points).
- The board's colour must differ from both stone colours.

In this repository intersections are named `A1`–`O15`: a column letter
`A`–`O` (left to right) followed by a row number `1`–`15` (bottom to top).
The centre is `H8`.

## 3. Terms and definitions

All terms below refer to stones of **one colour** on **one line** (a row,
column, or either diagonal).

| Term | Definition |
| --- | --- |
| **Row** | A group of same-coloured stones on one line, bounded on each side by the edge of the board, an opponent stone, or an empty intersection, with no opponent stone between the group's own stones. (Gaps of empty intersections *inside* the row are allowed.) |
| **Unbroken row** | A row with no empty intersection between any two of its stones. |
| **Five (in a row)** | An unbroken row of exactly five stones. |
| **Overline** | An unbroken row of six or more stones. |
| **Four** | A row of four stones to which one more stone can be added to make a five. |
| **Straight four** | An unbroken row of four stones that can be turned into a five in **two** different ways (i.e. both ends are open). Also called an *open four*. |
| **Three** | A row of three stones to which one more stone can be added to make a straight four, without that added stone also making a five. |
| **Double-four** | A single move that creates two or more fours at once, all passing through the intersection played. The fours may lie on different lines or on the same line. |
| **Double-three** | A single move that creates two or more threes at once, all passing through the intersection played. |

Notes for implementers:

- A *four* is defined by the existence of a completing point, so `oooo.`,
  `ooo.o`, and `oo.oo` are all fours. Only `.oooo.` (with both ends
  playable) is a straight four.
- For Black, "can be added to make a five" must be read together with
  section 9: a stone that would produce an overline does not make a five,
  so e.g. `o.oooo` contains no four for Black (filling the gap gives six).
- A *three* must be able to become a *straight* four. `.ooo.` and `.oo.o.`
  are threes; `xooo.` is not (it can only become a plain four).

## 4. How to play

- 4.1 One player takes the black stones, the other the white stones.
- 4.2 Players move alternately, one move at a time. Black moves first and
  must play the first stone on the centre intersection.
- 4.3 "Black to play" / "White to play" means it is that side's turn.

## 5. What a move is

A move is either placing a stone on an empty intersection, or declaring a
pass (giving up the right to place a stone on this turn).

## 6. Completing a move

A stone placement is complete when the player releases the stone. A pass is
complete when it is declared.

## 7. Adjusting stones

The player to move may straighten stones on their intersections after
informing the opponent.

## 8. Disturbed positions

If stones are knocked out of place or wrongly removed/replaced, the position
is restored and play continues. If a player caused the disturbance and the
position cannot be restored, that player loses. If nobody is responsible and
the position cannot be restored, the game is void and replayed.

## 9. Winning the game

- **9.1** The first player to make a **five in a row** wins. For **White**
  an **overline also counts as a win** (White has no forbidden moves).
- **9.2** **White wins if Black makes a forbidden move.** A Black move is
  forbidden if, *without at the same time making a five*, it makes:
  - a) an overline;
  - b) a double-four;
  - c) a double-three (with the exceptions in 9.3).

  A Black move that makes a five is always legal and wins, even if the same
  stone also forms an overline, double-four or double-three.
- **9.3** A Black double-three is **allowed** (i.e. not forbidden) if at
  least one of the following holds:
  - a) At most one of the threes can actually be turned into a straight
    four by a move that does not itself create an overline or a
    double-four. To check this, imagine playing the candidate stone, then
    imagine completing each three into a straight four and see whether that
    completion would be forbidden.
  - b) At most one of the threes can be turned into a straight four by a
    move that does not itself form a forbidden double-three. Whether that
    nested double-three is forbidden is decided by the same procedure
    (first rule a), then again imagining the straight fours), recursively,
    as deep as needed.

  Informally: a three "counts" toward a double-three only if its
  straight-four point is itself a legal move for Black. If fewer than two
  of the threes are real in this sense, the move is not a double-three.
- **9.4** A player wins if they can show the opponent's time has run out, or
  that the opponent failed to make the required number of moves in time.
- **9.5** A player wins if the opponent resigns.
- **9.6** A win must be claimed by the winner, stopping both clocks at the
  same time; for wins under 9.1–9.4 the claimant's clock button must be up
  when the clocks are stopped.
- **9.7** If Black makes a five but does not notice or claim it, White
  moves, play continues, and Black later makes a forbidden move under 9.2,
  White wins (if White claims it) despite the earlier five.
- **9.8** If Black makes a forbidden double-three or double-four and White
  does not claim it but keeps playing, White loses the right to claim that
  particular double-three/double-four later. An **overline**, however, can
  still be claimed by White at any later point as long as Black has not made
  and claimed a five and the game has not otherwise ended.

## 10. Draws

- a) The game is drawn when:
  - 10.1 every intersection is occupied;
  - 10.2 both players agree;
  - 10.3 both players pass consecutively;
  - 10.4 both players' time has run out.
- b) A draw offer (10.2) may only be made together with the offerer's move,
  after which they start the opponent's clock. The opponent accepts or
  declines, verbally or by moving. The offer cannot be withdrawn while it is
  pending.

## 11. The opening patterns

Only 26 opening patterns are permitted: 13 *direct* (second move
orthogonally adjacent to the centre) and 13 *indirect* (second move
diagonally adjacent). Concretely, the second move (White) has two allowed
placements next to the centre, and for each of them the third move (Black)
has 13 allowed placements, all within the central 5 × 5 area. The first
three stones must form one of these 26 shapes.

## 12. Opening rules

- **12.1** Before the game, a *tentative Black* and a *tentative White* are
  decided.
- **12.2–12.4** The tentative Black plays the first **three** moves (Black,
  White, Black), i.e. chooses which of the 26 patterns is used.
  (Rule adopted by the General Assembly on 2 May 1996.)
- **12.5** The tentative White then decides who will play Black and who will
  play White for the rest of the game (the *swap* option).
- **12.6** The player now holding White plays the 4th move on any empty
  intersection.
- **12.7** *Black's choice*: Black proposes **two different** candidates for
  the 5th move. The proposals must differ in every respect (not be
  symmetric equivalents). White chooses one of them to become the 5th move.
  Black's clock runs until two valid proposals are given; White's clock runs
  until a proposal is accepted and the 6th move (any empty intersection) is
  played.
- **12.8** After the 5th move the special opening rules end.
- **12.9** Passing is not allowed during the first three moves.

## 13. Recording the game (summary)

Both players must keep a legible move-by-move record of the whole game on
the organizer's form (13.1). A player with five minutes or less remaining
may stop recording, but must complete the record afterwards if possible
(13.2).

## 14. Use of the clock (summary)

A set number of moves must be made within a set time, controlled by a game
clock (14.1–14.2). Black's clock starts the game; after each move the player
stops their own clock and starts the opponent's with the same hand used to
move (14.3). A move is not counted for time control until the clock is
pressed (14.4). The clock's reading is decisive unless obviously defective
(14.5). Clocks are stopped for interruptions not caused by the players
(14.6), and players may not stop the clocks themselves without immediately
calling the organizer (14.7). Only the players may point out a flag fall or
a forgotten clock press (14.8), unless time referees are used, in which case
they control the time and must be called when five minutes remain (14.9).

## 15. Late arrival (summary)

Clocks are started at the organizer's request. If both players are absent
one clock runs and its time counts against both.

## 16. Conduct (summary)

During play no written or printed material and no analysis on another board
is allowed; no analysis is allowed in the playing room while games are in
progress or adjourned; players may not distract or disturb each other and
must follow the competition rules. Violations may be punished, including by
loss of the game.

## 17. The organizer (summary)

An organizer is appointed to run the competition: ensure the rules are
applied, adjudicate disputes, provide good playing conditions, keep players
undisturbed, penalize rule violations, and schedule the resumption of
adjourned games.

## 18. Changing the rules

These rules may only be changed by decision of the RIF General Assembly.

---

## Quick reference for the solver

The parts of the rules that `quintet` encodes:

| Rule | Consequence for the engine |
| --- | --- |
| 9.1 | Five wins for either colour. Overline (6+) wins for White only. |
| 9.2 a | Black overline is forbidden (unless the same stone also makes a five). |
| 9.2 b | Black double-four is forbidden (unless it also makes a five). |
| 9.2 c + 9.3 | Black double-three is forbidden only if at least two of the threes are *real*: their straight-four point must itself be a legal Black move. Checked recursively. |
| 3 (four, three) | For Black, "makes a five" excludes overlines, so Black fours/threes are only counted when completing them yields exactly five. |
| 12 | Opening restrictions are **not** modelled; the solver takes an arbitrary position and side to move. |
| 5, 10.3 | Passing is not modelled. |

See [board-implementation.md](board-implementation.md) for how these are
implemented.

# International Rules of Renju (RIF)

This is a Markdown restatement of the *International Rules of Renju* adopted
by the Renju International Federation (RIF) on 2 May 1996, with a small
correction decided by the RIF Central Committee on 3 May 1998. The
authoritative text is published at <https://www.renju.net/rifrules/>; this
page follows its section numbering but paraphrases the wording. Where the
original and this page disagree, the original wins.

Sections that matter for a mate solver (2, 3, 4, 9, 10, 11, 12) are restated
in full; notes that are not part of the rules are marked as such. Sections
about tournament procedure (clocks, scoresheets, arbiters, conduct) are
summarized only.

---

## 1. Introduction

Renju is a two-player game played on a board with black and white pieces
called *stones*.

## 2. The board

- 15 × 15 lines, giving 225 intersections.
- Five of the intersections are reference points.
- The board's colour must differ from both stone colours.

In this repository intersections are named `A1`–`O15`: a column letter
`A`–`O` (left to right) followed by the number `1`–`15` of the horizontal
line (bottom to top).
The centre is `H8`.

## 3. Terms and definitions

All terms below refer to stones of **one colour** on **one line** (a
horizontal line, a vertical line, or either diagonal).

| Term | Definition |
| --- | --- |
| **Row** | A group of same-coloured stones on one line, bounded on each side by the edge of the board, an opponent stone, or an empty intersection, with no opponent stone between the group's own stones. (Gaps of empty intersections *inside* the row are allowed.) |
| **Unbroken row** | A row with no empty intersection between any two of its stones. |
| **Five (in a row)** | An unbroken row of exactly five stones. |
| **Overline** | An unbroken row of six or more stones. |
| **Four** | A row of four stones to which one more stone can be added to make a five. |
| **Straight four** | An unbroken row of four stones (a four) that can be turned into a five in **two** different ways. |
| **Three** | A row of three stones to which one more stone can be added to make a straight four, without that added stone also making a five. |
| **Double-four** | A single stone that makes more than one four at once, the fours meeting at the intersection played. |
| **Double-three** | A single stone that makes more than one three at once, the threes meeting at the intersection played. |

Notes for implementers:

- A *four* is defined by the existence of a completing point, so `oooo.`,
  `ooo.o`, and `oo.oo` are all fours. Only `.oooo.` (with both ends
  playable) is a straight four, also called an *open four*.
- The fours of a double-four may lie on different lines or on the same
  line (e.g. `o.o_o.o`, where `_` is the stone played).
- For Black, "can be added to make a five" must be read together with
  section 9: a stone that would produce an overline does not make a five,
  so e.g. `o.oooo` contains no four for Black (filling the gap gives six).
- A *three* must be able to become a *straight* four. `.ooo.` and `.oo.o.`
  are threes; `xooo.` is not (it can only become a plain four).

## 4. How to play

- 4.1 One player takes the black stones, the other the white stones.
- 4.2 Players move alternately, one move each time. Black begins the game
  with a move in the middle of the board (the centre intersection).
- 4.3 "Black to play" / "White to play" means it is that side's turn to
  move.

## 5. What a move is

A move is either placing a stone on an empty intersection, or declaring a
pass (giving up the right to place a stone on this turn).

## 6. Completing a move

A stone placement is complete when the player releases the stone. A pass is
complete when it is declared.

## 7. Adjusting stones

The player to move may adjust one or more stones on their intersections,
but must inform the opponent before doing so.

## 8. Disturbed positions

If stones become disarranged during a game, or are wrongly removed or
replaced, the position is reconstructed as it was before the mishap and play
continues. If a player caused the disturbance and the position cannot be
restored, that player loses. If nobody is responsible and
the position cannot be restored, the game is void and replayed.

## 9. Winning the game

- **9.1** The first player to make a **five in a row** wins. For **White**
  an **overline also counts as a win**.
- **9.2** **White wins if Black makes a forbidden move.** A Black move is
  forbidden if, *without at the same time making a five*, it makes:
  - a) an overline;
  - b) a double-four;
  - c) a double-three (with the exceptions in 9.3).

  So a Black move that makes a five wins, even if the same stone also
  forms an overline, double-four or double-three. White has no forbidden
  moves.
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
  - 10.3 both players pass, one after the other;
  - 10.4 both players' time has run out.
- b) A draw offer (10.2) may only be made together with the offerer's move,
  after which they start the opponent's clock. The opponent accepts or
  refuses orally, or refuses by making a move. Meanwhile the player who made
  the offer may not withdraw it.

## 11. The opening patterns

Only 26 opening patterns may be played: 13 *indirect* and 13 *direct*. The
second and third moves must be played as in the patterns, which the original
shows as diagrams (not reproduced here): two alternatives for the second
move, and after it 13 alternatives for the third move.

In the diagrams, the second move (White) is next to the centre,
orthogonally for the direct patterns and diagonally for the indirect ones,
and the third move (Black) is within the central 5 × 5 area.

## 12. Opening rules

- **12.1** Before the game, a *tentative Black* and a *tentative White* are
  decided.
- **12.2–12.4** The tentative Black plays all of the first **three** moves
  (two for Black and one for White), i.e. decides which of the 26 patterns
  is used.
  (Rule adopted by the General Assembly on 2 May 1996.)
- **12.5** The tentative White then decides who will play Black and who will
  play White in the game (White has the right to change sides, the *swap*).
- **12.6** The player now holding White plays the 4th move on any empty
  intersection.
- **12.7** *Black's choice*: Black makes **two different** proposals for
  the 5th stone. The proposals must be unequal in every respect (in
  practice: not symmetrically equivalent). White chooses one of them to become the 5th move.
  Black's clock runs until two valid proposals are given; White's clock runs
  until a proposal is accepted and the 6th move (any empty intersection) is
  played.
- **12.8** With the 5th move the special opening rules end.
- **12.9** Passing is not allowed during the first three moves.

## 13. Recording the game (summary)

Both players must keep a legible move-by-move record of the whole game on
the organizer's form (13.1). A player with five minutes or less remaining
need not record, but must complete the record as soon as the time
shortage is over, if possible (13.2).

## 14. Use of the clock (summary)

- 14.1–14.2 A set number of moves must be made within a set time,
  controlled by a special clock.
- 14.3 Black's clock is started when the game begins. After each move the
  player stops their own clock with the hand used to move and at the same
  time starts the opponent's, promptly so as not to disturb the opponent.
  A player may forget to stop the clock, and the opponent need not point
  it out.
- 14.4 For time control, the last move counts as made only once the player
  has stopped their clock.
- 14.5 The clock's reading is decisive unless obviously defective; a
  player who wants to point out a defect must do so immediately.
- 14.6 If the game is interrupted for a reason not caused by the players,
  the clocks are stopped until the matter is resolved.
- 14.7 Players may not stop the clocks themselves without immediately
  calling the organizer.
- 14.8 No one but the players, the organizer included, may point out that a
  player's time is up or that a player forgot to stop the clock.
- 14.9 The organizer may use time referees, who then control the time, and
  14.8 does not apply. The organizer must make sure every game can get one
  when needed, and the players must call one as soon as five minutes of
  the set time remain.

## 15. Late arrival (summary)

When the games are to start, all clocks are started at the organizer's
request. If both players are absent, one clock is started and its time runs
for both.

## 16. Conduct (summary)

During play no written or printed material and no analysis on another board
is allowed; no analysis is allowed in the playing room while games are in
progress or adjourned; players may not distract or disturb each other and
must follow the rules set for the competition. Breaking these rules can lead
to punishment and loss of the game.

## 17. The organizer (summary)

An organizer is appointed to run the competition: ensure the rules are
applied carefully (17.1), judge all disputes during the competition (17.2),
provide good conditions (17.3), keep players from being disturbed by each
other or by spectators (17.4), punish players who break the rules (17.5),
and decide the order in which interrupted games resume (17.6).

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

See [02-board-implementation.en.md](02-board-implementation.en.md) for how these are
implemented.

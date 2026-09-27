# 四追い（VCF）探索（`src/mate/vcf/`）

**四追い**（VCF: victory by continuous fours）は、攻め手がすべて四を作る手順。

- 受け方に選択肢はない。四はその場で止めるしかない。
- そのため手順は初手から強制で、木は `(攻め, 止め)` ペアの一本道になる。
- クレートで最も単純なソルバーで、追い詰めソルバーが追い手の判定に頼るものでもある。

前提: [04](04-solver-framework.ja.md)（`Game`、`State`、`Key`、`Memo`、`Solver`）。

```
src/mate/vcf.rs         module doc, re-exports
src/mate/vcf/
├── state.rs            VCFState: Game + attacker + limit; four-making move pairs   (§1)
├── dfs.rs              DFSSolver: the depth-first search and its dead-end memo      (§2)
└── iddfs.rs            IDDFSSolver: DFSSolver at increasing limits                (§3)
```

## 1. `VCFState`: 四を作る手

```rust
pub struct VCFState { game: Game, attacker: Player, limit: u8, swords: SwordMap }
```

- `VCFState::from_board(&board, attacker, limit)`: 攻め方の手番から始める。
- `VCFState::new(game, limit)`: 既存の `Game` から作る。攻め方は `game` の手番側。追い詰めソルバーが内部の四追い状態を作るときに使う。

四は **`Sword`**（剣先: 5 マスの窓に自分の石 3 つと空の**眼** 2 つ）に打って作る。

- 片方の眼に打つと、もう片方の眼を勝ち点とする `Four` ができる。
- よって 1 つの剣先から `(攻め, 受け)` ペアが 2 つ得られる（`push_eyes_pairs`）。

```
column H:  H7 = x (White),  H8 H9 H10 = o (Black),  H11 H12 = empty
            → Sword with eyes H11, H12
pairs:  (H11, H12)   play H11: four H8–H11, White must answer H12
        (H12, H11)   play H12: four H8,H9,H10,_,H12, White must answer H11
```

生成器は 3 つ。ソルバーはこの順に試す:

| 生成器 | ペアの出どころ | 使う場面 |
| --- | --- | --- |
| `forced_move_pair(p)` | 攻めの眼がちょうど `p` の剣先 | 攻め方が `Forced(p)` のとき。受け方の止めが四になった（ノリ手）。攻め方の応手はそれを止め、*かつ*四でなければならない。できなければ四追いは終わり。 |
| `neighbor_move_pairs()` | `second_last_move`（攻め方の直前の石）を通る剣先 | 最初に試す。直前の石を伸ばす手が、四を続けられる可能性が最も高い。 |
| `move_pairs()` | 手番側のすべての剣先 | 次に試す。 |

補足:

- 止めの点は剣先から取り、攻め手を打った後の盤面から求め直さない。攻め手が四を 2 つ作れば、止めを打つ前に受け方の `check_event` が `Defeated(Fours(..))` を返す。だから保持している止めが効くのは四が 1 つのときだけ。
- 3 つとも盤面を走査しない。`VCFState` が `SwordMap`（02 §7）にキャッシュしている剣先を読む:
  - `after_play` / `after_undo` で印を付け、読む前に同期して、前回から触れられた線だけ計算し直す。
  - 返す順序は `rows` / `rows_on` とまったく同じ。探索する木は全走査した場合と同じ。
- 追い詰め探索の中の四追いは、`VCTState` が持つ `SwordMap` のクローンから始まる。`VCTState` はクローンの前に同期するので、計算し直しの必要な線は残っていない。

## 2. `DFSSolver`

```
solve(state):                                  # Solver::solve: 1 つの問い
    advance_generation(); return search(state)

search(state):                                 # 攻め方の手番
    if limit == 0:                        return None
    if !budget.consume():                 return None
    if dead_ends[key.position] >= limit:  return None        # 既知: この深さでは四追いなし
    result = search_move_pairs(state)
    if result is None and budget not exhausted:
        dead_ends[key.position] = max(known, limit)
    return result

search_move_pairs(state):
    match check_event():
        Defeated(_) → None                                   # 受け方に四四がある
        Forced(p)   → forced_move_pair(p) → search_attack, or None
        None        → for (a, d) in neighbor pairs, then the rest:
                          if search_attack(a, d) is Some → return it
                      None

search_attack(state, attack, defence):
    result = with_move(attack): search_defence(defence)   then prepend attack
    if result is Some and attack is forbidden: return None
    return result

search_defence(state, defence):                # 受け方の手番
    if check_event() is Defeated(end): return Mate { end, path: [] }
    with_move(defence): search(state)            then prepend defence
                                                 # limit は with_move の中で 1 減る
```

要点:

- **メモするのは失敗だけ。**
  - 成功はそのまま返す。
  - 四追いがない局面は `dead_ends` に記録する。それが分かった最大の limit と、その探索の関連領域（後述）。
  - 手の生成が `limit` を読まないので、limit *n* の木は *n+1* の木を途中で切ったもの。「*n* 以内に四追いなし」は *n* 以下のすべての limit で正確で、キーは `Key::position` だけでよい（04 §3）。
- **禁手の攻め手では勝てない。**
  - ここで見つかる四追いで、黒は四四や長連を打たない。黒の `Fours` が常に棒四なのはこのため（03 §5）。
  - 禁手の攻め手は少ない（ベンチマークでおよそ 70 に 1 つ）。だから攻め手はまず探索し、その先で四追いが見つかったときだけ禁手かを確かめる。禁手なら失敗とする。
  - 禁手の攻め手の先の探索は無駄になるが、すべての攻め手を事前に確かめるほうが高くついた。
- **ノリ手は特別扱いしない。** 止めが四になれば、攻め方の次のノードは `Forced(p)` を見る。`forced_move_pair(p)` が成功するのは `p` 自体が四を作る眼のときだけ。`test_vcf_counter`、`test_vcf_not_opponent_double_four` が検証している。
- **攻め方のノードでの `Defeated` は枝の終わり。** 受け方の止めが四四（または黒が止められない四）を作った。攻め方に応じる四はないので、この手順は失敗。

### 関連領域（zone）

`search_zone(state, budget, &mut zone)` は、`search` に加えて**関連領域**を集める。四追いが見つからなかったとき、攻め方がもう 1 石置けば四追いが生まれうる空点の集合。

- 追い詰めソルバーはこれで、追い手になりえない攻め手を除く（06 §3）。
- 点ごとに 1 ビットの `Area`（`src/feature/area.rs`）で、探索が見たものから作る:

| どこで | 加えるもの |
| --- | --- |
| 打った攻め手 `a` ごと | `Area::around(a, 4)`: `a` を通る 4 本の線の前後 4 マス。`a` を含むどのセグメントもここに収まる |
| 攻め方のノードでの `Forced(p)` | `p`（受け方の四の眼） |
| 攻め方のノードでの `Defeated(Fours(e1, e2))` | `e1`、`e2` |
| 攻め方のノードでの `Defeated(Forbidden(e))`、禁手と分かった黒の攻め手、白が攻めるときの黒の止めそれぞれ | `Area::around(_, 5)` |

禁手の攻め手:

- 黒の攻め手が禁手と分かるのは、その先で四追いが見つかったときだけ。その探索の関連領域は不完全なので、攻め手自身の `Area::around(_, 5)` で補う。
- 先で何も見つからなかった禁手の攻め手には、何も足さなくてよい。禁手かどうかによらず、その探索が集めた関連領域の外の石は、その探索が何も見つけないことを変えない。

関連領域に入れないもの、それで正しい理由:

- 根の時点で攻め方の石を 2 つ含むセグメントの残りの点は入れない。そこに石を置くと新しい剣先ができるので、呼び出し側が自分で調べる（`VCTState::may_threaten` が `ShapeMap` から読む）。
- その両方の外にある石は、根でも木のどの攻め手の後でも、攻め方が四や五を作れるセグメントに加わらない。後のノードで加わりうるセグメントは、根の時点のものか、木の攻め手を含むものだから。
- 受け方の四も止めない。四の空きは眼だけだから。
- よって木は同じで、答えも同じ。
- 表の最後の行は `VCTState::threat_defences` と同じく近似。石は離れた場所から禁手かどうかを変えうる（そこの三の達四点がさらに禁手で、三が本物でなくなる場合など）。

メモの中の関連領域:

- 行き止まりは limit と一緒に関連領域を持つ（`Memo<(u8, Area)>`）。そこに行き当たった探索は、その関連領域を自分のものに加える。
- 行き止まりを上書きするのは、より大きな limit の探索だけ。その木（と関連領域）は小さい limit のものを含む。
- `search` は関連領域を捨てる `search_zone`。探索する木も、数えるノードも同じ。

## 3. `IDDFSSolver`

`IDDFSSolver::new(limits)` は:

- `limits` のうち状態の limit より小さい値で順に `DFSSolver::search` を走らせ、最後に状態自身の limit で走らせる。
- 最初に見つかった結果を返す。

浅い探索は、木全体をたどらずに短い四追いを見つける。行き止まりメモは limit ごとに正確なので、浅い探索は深い探索に何のコストも残さない。

`Solver` を実装し、`solve` / `search` の分け方も同じ。追い詰めソルバーは各側に `limits = [1]`（「まず 1 手で勝てるか」）のものを 1 つずつ持ち、その `search` を呼ぶ（06 §2）。

## 4. 手順を追う

`test_vcf_counter` の盤面、黒番:

```
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . x . . . . . . .
 . . . . . . . o . o o x . . .
 . . . . . . . . o x . o . . .
 . . . . . . . . o . x . . . .
 . . . . . . . . x . . x . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
 . . . . . . . . . . . . . . .
```

`solve(VCFDFS, &board, Black, SolveLimits::new(3))` は手順 `I8,G8,I10,I9,J9`、詰め上がり `Fours(H11, M6)` で証明する:

| 手 | 作るもの | その後の `limit` |
| --- | --- | --- |
| 根 | | 3 |
| `I8`（黒） | 剣先 `H8,J8,K8`（眼 `G8`、`I8`）から四 `H8,I8,J8,K8` | 3（直前の攻め手はまだ数に入る） |
| `G8`（白） | 強制の止め | 2 |
| `I10`（黒） | 四 `I6,I7,I8,_,I10` | 2 |
| `I9`（白） | 強制の止め | 1 |
| `J9`（黒） | `I10,J9,K8,L7`。`H11` と `M6` の両方が空いた棒四（2 つの `Four`） | 1 |
| | 白の `check_event` → `Defeated(Fours(H11, M6))` | |

`limit = 2` では `None` になる。`I9` の後で limit が 0 になり、`J9` を試す前に `search` が止まるため。四 3 つには `limit >= 3` が必要。

## 5. チートシート

| 知りたいこと | 見る場所 |
| --- | --- |
| 四を作る手が生成されない | `VCFState::move_pairs` は `Sword` の眼しか見ない。黒ではマージン（`Segment::is_alive`）が長連になる四を除く |
| ノリ手の扱いがおかしい | 攻め方のノードの `Game::check_event` → `Forced(p)`、次に `VCFState::forced_move_pair` |
| どの四が先に試されるか | `neighbor_move_pairs`（攻め方の直前の石を通るもの）→ `move_pairs` |
| メモ | `DFSSolver::dead_ends`: 四追いのない最大 limit とその探索の関連領域。キーは `Key::position`。予算切れ後は書かない |
| どの点で攻め方に四追いが生まれうるか | `DFSSolver::search_zone`（§2「関連領域」） |
| `solve` と `search` の違い | `solve` は世代を開く。`search` は `IDDFSSolver` と追い詰めソルバーが繰り返し呼ぶ（04 §4） |
| 四追いの深さ | 攻め手 `limit` 手。各手が四なので、四 *k* 個には `limit >= k` |

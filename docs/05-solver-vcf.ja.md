# 四追い（VCF）探索（`src/mate/vcf/`）

**四追い**（VCF: victory by continuous fours）は、攻め手がすべて四を作る手順。受け方は四をその場で止めるしかないので、手順は初手から強制で、木は `(攻め, 止め)` ペアの一本道になる。クレートで最も単純なソルバーであり、追い詰めソルバーが追い手の判定に使うものでもある。

前提: [04](04-solver-framework.ja.md)（`Game`、`State`、`Key`、`Memo`、`Solver`）。

```
src/mate/vcf.rs         モジュールドキュメント、再エクスポート
src/mate/vcf/
├── state.rs            VCFState: Game + 攻め方 + limit。四を作る手のペア         (§1)
├── dfs.rs              DFSSolver: 深さ優先探索と行き止まりメモ                  (§2)
└── iddfs.rs            IDDFSSolver: limit を増やしながら DFSSolver を走らせる    (§3)
```

## 1. `VCFState`: 四を作る手

```rust
pub struct VCFState { game: Game, pub attacker: Player, pub limit: u8 }
```

- `VCFState::from_board(&board, attacker, limit)`: 攻め方の手番から始める。
- `VCFState::new(game, limit)`: 既存の `Game` から作る。攻め方は `game` の手番側。追い詰めソルバーが内部の四追い状態を作るときに使う。

四は **`Sword`**（5 マスの窓に自分の石 3 つと空の**眼** 2 つ）に打つことで作る。片方の眼に打つと、もう片方の眼を勝ち点とする `Four` ができる。1 つの剣先から `(攻め, 受け)` ペアが 2 つ得られる（`sword_eyes_pairs`）。

```
H 列:  H7 = x（白）、H8 H9 H10 = o（黒）、H11 H12 = 空
        → 眼 H11、H12 の Sword
ペア:  (H11, H12)   H11 に打つと四 H8–H11。白は H12 に受けるしかない
       (H12, H11)   H12 に打つと四 H8,H9,H10,_,H12。白は H11 に受けるしかない
```

生成器は 3 つ。ソルバーはこの順に試す。

| 生成器 | ペアの出どころ | 使う場面 |
| --- | --- | --- |
| `forced_move_pair(p)` | 攻めの眼がちょうど `p` である剣先 | 攻め方が `Forced(p)` のとき。受け方の止めが四になった（ノリ手）ので、攻め方はそれを止めつつ四を作る必要がある。できなければ四追いは終わり |
| `neighbor_move_pairs()` | `second_last_move`（攻め方の直前の石）を通る剣先 | 最初に試す。直前の石を伸ばす手が、四を続ける可能性が最も高い |
| `move_pairs()` | 手番側のすべての剣先 | 次に試す |

止めの点は剣先から取り、攻め手を打った後の盤面から求め直さない。攻め手がたまたま四を 2 つ作った場合は、止めを打つ前に受け方の `check_event` が `Defeated(Fours(..))` を返す。したがって保持している止めが使われるのは四が 1 つのときだけ。

3 つとも盤面を走査しない。`VCFState` が `SwordMap` にキャッシュしている剣先（02 §7）を読む。`after_play` / `after_undo` で印を付けておき、読む前に同期して、前回読んでから打たれた手が触れた線だけを計算し直す。剣先を返す順序は `rows` / `rows_on` とまったく同じなので、探索する木は全走査した場合と変わらない。

追い詰め探索の中の四追いは、追い詰め側の盤面をクローンして始まる。`VCTState` はクローンの前に剣先を同期するので、クローンには計算し直しの必要な線が残っていない。

## 2. `DFSSolver`

```
solve(state):                                  # Solver::solve — 1 つの問い
    advance_generation(); return search(state)

search(state):                                 # 攻め方の手番
    if limit == 0:                       return None
    if !budget.consume():                return None
    if dead_ends[key.position] >= limit:  return None        # 既知: この深さでは四追いなし
    result = search_move_pairs(state)
    if result is None and budget not exhausted:
        dead_ends[key.position] = max(known, limit)
    return result

search_move_pairs(state):
    match check_event():
        Defeated(_) → None                                   # 受け方に四四がある
        Forced(p)   → forced_move_pair(p) → search_attack。なければ None
        None        → 近傍ペア、次に残りのペアの各 (a, d) について:
                          search_attack(a, d) が Some なら → それを返す
                      None

search_attack(state, attack, defence):
    result = with_move(attack): search_defence(defence)   → 結果の先頭に attack を付ける
    if result が Some かつ attack が禁手: return None
    return result

search_defence(state, defence):                # 受け方の手番
    if check_event() が Defeated(end): return Mate { end, path: [] }
    with_move(defence): search(state)            → 結果の先頭に defence を付ける
                                                 # limit は with_move の中で 1 減る
```

要点:

- **メモするのは失敗だけ。** 成功はそのまま返す。四追いがない局面は、それが分かった最大の limit として、その探索の関連領域（後述）と一緒に `dead_ends` に記録する。手の生成が `limit` を読まないので、limit *n* の木は limit *n+1* の木を途中で切ったもの。「*n* 以内に四追いなし」は *n* 以下のすべての limit で正しく、キーは `Key::position` だけでよい（04 §3）。
- **禁手の攻め手では勝てない。** ここで見つかる四追いで、黒は四四や長連を打たない。黒の `Fours` が常に棒四なのはこのため（03 §5）。禁手の攻め手は少ない（ベンチマークでおよそ 70 に 1 つ）ので、攻め手はまず探索し、その先で四追いが見つかったときにだけ禁手かどうかを確かめる。禁手なら失敗とする。禁手の攻め手の先の探索は無駄になるが、すべての攻め手を事前に確かめるほうが高くついた。
- **ノリ手は特別扱いしない。** 止めが四になれば、攻め方の次のノードは `Forced(p)` を見る。`forced_move_pair(p)` が成功するのは `p` が四を作る眼のときだけ。`test_vcf_counter`、`test_vcf_not_opponent_double_four` が検証している。
- **攻め方のノードでの `Defeated` は枝の終わり。** 受け方の止めが四四（または黒が止められない四）を作った。攻め方に応じる四はないので、この手順は失敗。

### 関連領域（zone）

`search_zone(state, budget, &mut zone)` は `search` に加えて**関連領域**を集める。四追いが見つからなかったとき、攻め方がもう 1 石置けば四追いが生まれうる空点の集合である。追い詰めソルバーはこれで追い手になりえない攻め手を除く（06 §3）。点ごとに 1 ビットの `Area`（`src/feature/area.rs`）で、探索が見たものから作る:

| どこで | 加えるもの |
| --- | --- |
| 打った攻め手 `a` ごと | `Area::around(a, 4)`: `a` を通る 4 本の線の前後 4 マス — `a` を含むどの区間もここに収まる |
| 攻め方のノードでの `Forced(p)` | `p`（受け方の四の眼） |
| 攻め方のノードでの `Defeated(Fours(e1, e2))` | `e1`、`e2` |
| 攻め方のノードでの `Defeated(Forbidden(e))`、禁手と分かった黒の攻め手、白が攻めるときの黒の止めそれぞれ | `Area::around(_, 5)` |

黒の攻め手が禁手と分かるのは、その先で四追いが見つかったときだけである。その探索の関連領域は不完全なので、攻め手自身の `Area::around(_, 5)` がそれを補う。先で何も見つからなかった禁手の攻め手には、何も足さなくてよい。禁手であってもなくても、その探索が集めた関連領域の外の石は、その探索が何も見つけないことを変えないからである。

入っていないのは、根の時点で攻め方の石を 2 つ含む区間の残りの点。そこに石を置けば新しい剣先ができるので、呼び出し側が自分で調べる（`VCTState::may_threaten` が `ShapeMap` から読む）。理由: その両方の外にある石は、根でも木のどの攻め手の後でも、攻め方が四や五を作れる区間に加わらない。後のノードで加わりうる区間は、根の時点の区間か、木の攻め手を含む区間のどちらかだからである。受け方の四も止めない。四の空点は眼だけだからである。よって木は同じで、答えも同じ。最後の行は `VCTState::threat_defences` と同じく近似である。三の四ノビ点がさらに禁手であるためにその三が本物の三でなくなる場合など、石は離れた場所から禁手かどうかを変えうる。

行き止まりは limit と一緒に関連領域を持ち（`Memo<(u8, Area)>`）、そこに行き当たった探索はその関連領域を自分のものに加える。行き止まりが上書きされるのは、より大きな limit の探索によってだけで、その木（と関連領域）は小さい limit のものを含む。`search` は関連領域を捨てる `search_zone` で、探索する木も数えるノードも変わらない。

## 3. `IDDFSSolver`

`IDDFSSolver::new(limits)` は次のように動く。

1. `limits` のうち状態の limit より小さい値で順に `DFSSolver::search` を走らせる。
2. 最後に状態自身の limit で走らせる。
3. 最初に見つかった結果を返す。

浅い limit での探索は、木全体をたどらずに短い四追いを見つける。行き止まりメモは「その limit 以下では四追いなし」という事実しか記録しないので、浅い探索の結果が深い探索を誤らせることはない。

`Solver` を実装し、`solve` / `search` の分け方は `DFSSolver` と同じ。追い詰めソルバーは各側に 1 つずつ `limits = [1]`（「まず 1 手で勝てるか調べる」）のものを持ち、その `search` を呼ぶ（06 §2）。

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

`solve(VCFDFS, &board, Black, SolveLimits::new(3))` は手順 `I8,G8,I10,I9,J9`、詰め上がり `Fours(H11, M6)` で証明する。

| 手 | 作るもの | その後の `limit` |
| --- | --- | --- |
| 根 | | 3 |
| `I8`（黒） | 剣先 `H8,J8,K8`（眼 `G8`、`I8`）から四 `H8,I8,J8,K8` | 3（直前の攻め手はまだ数に入る） |
| `G8`（白） | 強制の止め | 2 |
| `I10`（黒） | 四 `I6,I7,I8,_,I10` | 2 |
| `I9`（白） | 強制の止め | 1 |
| `J9`（黒） | `I10,J9,K8,L7`。`H11` と `M6` の両方が空 = 棒四（2 つの `Four`） | 1 |
| | 白の `check_event` → `Defeated(Fours(H11, M6))` | |

`limit = 2` では `None` になる。`I9` の後で limit が 0 になり、`J9` を試す前に `search` が止まるため。四 3 つの四追いには `limit >= 3` が必要。

## 5. チートシート

| 知りたいこと | 見る場所 |
| --- | --- |
| 四を作る手が生成されない | `VCFState::move_pairs` は `Sword` の眼しか見ない。黒では `exact` の余白判定が長連になる四を除く |
| ノリ手の扱いがおかしい | 攻め方のノードの `Game::check_event` → `Forced(p)`、次に `VCFState::forced_move_pair` |
| どの四が先に試されるか | `neighbor_move_pairs`（攻め方の直前の石を通るもの）→ `move_pairs` |
| メモ | `DFSSolver::dead_ends`: 四追いのない最大 limit とその探索の関連領域。キーは `Key::position`。予算切れ後は書かない |
| どの点に置けば攻め方に四追いが生まれうるか | `DFSSolver::search_zone`（§2「関連領域」） |
| `solve` と `search` の違い | `solve` は世代を開く。`search` は `IDDFSSolver` と追い詰めソルバーが繰り返し呼ぶ（04 §4） |
| 四追いの深さ | 攻め手 `limit` 手。各手が四なので、四 *k* 個には `limit >= k` が必要 |

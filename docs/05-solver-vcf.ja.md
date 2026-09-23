# 四追い（VCF）探索（`src/mate/vcf/`）

**四追い**（VCF: victory by continuous fours）は、攻め手がすべて四を作る手順。受け方は四をその場で止めるしかないので、手順は初手から強制で、木は `(攻め, 止め)` ペアの一本道になる。クレートで最も単純なソルバーであり、追い詰めソルバーが追い手の判定に使うものでもある。

前提: [04](04-solver-framework.ja.md)（`Game`、`State`、`Key`、`Memo`、`Solver`）。

```
src/mate/vcf.rs         モジュールドキュメント、再エクスポート
src/mate/vcf/
├── state.rs            VCFState: Game + 攻め方 + limit + SwordField。四を作る手のペア (§1)
├── dfs.rs              DFSSolver: 深さ優先探索と行き止まりメモ                  (§2)
└── iddfs.rs            IDDFSSolver: limit を増やしながら DFSSolver を走らせる    (§3)
```

## 1. `VCFState`: 四を作る手

```rust
pub struct VCFState { game: Game, pub attacker: Player, pub limit: u8 }
```

- `VCFState::init(&board, attacker, limit)`: 攻め方の手番から始める。
- `VCFState::new(game, limit)`: 既存の `Game` から作る。攻め方は `game` の手番側。追い詰めソルバーが内部の四追い状態を作るときに使う。

四は **`Sword`**（5 マスの窓に自分の石 3 つと空の**眼** 2 つ）に打つことで作る。片方の眼に打つと、残る勝ち点がもう片方の眼の `Four` ができる。1 つの剣から `(攻め, 受け)` ペアが 2 つ得られる（`sword_eyes_pairs`）。

```
H 列:  H7 = x（白）、H8 H9 H10 = o（黒）、H11 H12 = 空
        → 眼 H11、H12 の Sword
ペア:  (H11, H12)   H11 に打つと四 H8–H11。白は H12 に受けるしかない
       (H12, H11)   H12 に打つと四 H8,H9,H10,_,H12。白は H11 に受けるしかない
```

生成器は 3 つ。ソルバーはこの順に試す。

| 生成器 | ペアの出どころ | 使う場面 |
| --- | --- | --- |
| `forced_move_pair(p)` | 攻めの眼がちょうど `p` である剣 | 攻め方が `Forced(p)` のとき。受け方の止めが四になった（ノリ手）ので、攻め方はそれを止めつつ四を作る必要がある。できなければ四追いは終わり |
| `neighbor_move_pairs()` | `last2_move`（攻め方の直前の石）を通る剣 | 最初に試す。直前の石を伸ばす手が、四を続ける可能性が最も高い |
| `move_pairs()` | 手番側のすべての剣 | 次に試す |

止めの点は剣から取り、攻め手を打った後の盤面から求め直さない。攻め手がたまたま四を 2 つ作った場合は、止めを打つ前に受け方の `check_event` が `Defeated(Fours(..))` を返す。したがって保持している止めが使われるのは四が 1 つのときだけ。

### `SwordField`: 剣を線ごとに保持する

`neighbor_move_pairs` と `move_pairs` は盤面を走査しない。`VCFState` は攻め方の `SwordField`（`src/analysis/sword.rs`）を持つ。保存された 72 本の線それぞれについて、剣の窓が始まるセルのビットマスクと、各剣の石の並びを保持する。1 手で変わるのはその点を通る高々 4 本の線なので、`after_play` / `after_undo` はそれらの線に「古い」印を付けるだけで、2 つの生成器は読む前に古い線だけを盤面から `sync` する。ペアの順序は `Board::structures` / `structures_on` が剣を列挙する順（線は縦・横・右上がり・右下がり、窓は左から右）とまったく同じなので、探索する木は盤面を全走査した場合と変わらない。

`VCFState::new` は盤面から場を作る。`VCFState::with_swords` は、他で保持している場を受け取る。追い詰めソルバーは内部の四追い探索に自分の場のコピーを渡す（06 §2）ので、内部の四追いも盤面の走査から始めずに済む。

## 2. `DFSSolver`

```
solve(state):                                  # Solver::solve — 1 つの問い
    advance_generation(); return search(state)

search(state):                                 # 攻め方の手番
    if limit == 0:                       return None
    if !budget.consume():                return None
    if deadends[key.position] >= limit:  return None        # 既知: この深さでは四追いなし
    result = search_move_pairs(state)
    if result is None and budget not exhausted:
        deadends[key.position] = max(known, limit)
    return result

search_move_pairs(state):
    match check_event():
        Defeated(_) → None                                   # 受け方に四四がある
        Forced(p)   → forced_move_pair(p) → search_attack。なければ None
        None        → 近傍ペア、次に残りのペアの各 (a, d) について:
                          search_attack(a, d) が Some なら → それを返す
                      None

search_attack(state, attack, defence):
    if attack が禁手: return None
    into_play(attack): search_defence(defence)   → 結果の先頭に attack を付ける

search_defence(state, defence):                # 受け方の手番
    if check_event() が Defeated(end): return Mate { end, path: [] }
    into_play(defence): search(state)            → 結果の先頭に defence を付ける
                                                 # limit は into_play の中で 1 減る
```

要点:

- **メモするのは失敗だけ。** 成功はそのまま返す。四追いがない局面は、それが分かった最大の limit として `deadends: Memo<u8>` に記録する。手の生成が `limit` を読まないので、limit *n* の木は limit *n+1* の木を途中で切ったもの。「*n* 以内に四追いなし」は *n* 以下のすべての limit で正しく、キーは `Key::position` だけでよい（04 §3）。
- **禁手の攻め手は飛ばす。** 黒はここで四四や長連を打たない。黒の `Fours` が常に棒四なのはこのため（03 §5）。
- **ノリ手は特別扱いしない。** 止めが四になれば、攻め方の次のノードは `Forced(p)` を見る。`forced_move_pair(p)` が成功するのは `p` が四を作る眼のときだけ。`test_vcf_counter`、`test_vcf_not_opponent_double_four` が検証している。
- **攻め方のノードでの `Defeated` は枝の終わり。** 受け方の止めが四四（または黒が止められない四）を作った。攻め方に応じる四はないので、この手順は失敗。

## 3. `IDDFSSolver`

`IDDFSSolver::init(limits)` は次のように動く。

1. `limits` のうち状態の limit より小さい値で順に `DFSSolver::search` を走らせる。
2. 最後に状態自身の limit で走らせる。
3. 最初に見つかった結果を返す。

浅いパスは木全体を歩かずに短い四追いを見つける。行き止まりメモは limit ごとに厳密なので、浅いパスが深いパスの邪魔をすることはない。

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
| `I8`（黒） | 剣 `H8,J8,K8`（眼 `G8`、`I8`）から四 `H8,I8,J8,K8` | 3（直前の攻め手はまだ数に入る） |
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
| メモ | `DFSSolver::deadends`: 四追いのない最大 limit。キーは `Key::position`。予算切れ後は書かない |
| `solve` と `search` の違い | `solve` は世代を開く。`search` は `IDDFSSolver` と追い詰めソルバーが繰り返し呼ぶ（04 §4） |
| 四追いの深さ | 攻め手 `limit` 手。各手が四なので、四 *k* 個には `limit >= k` が必要 |

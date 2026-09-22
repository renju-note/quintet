# 四追い（VCF）探索（`src/mate/vcf/`）

**四追い**（VCF: victory by continuous fours）は、攻め手がすべて四を作る手順である。受け方に選択はない — 四はその場で止めなければならない — ので、手順全体が初手から強制であり、木は `(攻め, 止め)` のペアの連鎖になる。これがクレートで最も単純なソルバーであり、追い詰めソルバーが追い手の認識に頼るものでもある。

[04](04-solver-framework.ja.md) を前提にする: `Game`、`State`、`Key`、`Memo`、`Solver`。

```
src/mate/vcf.rs         モジュールドキュメント、再エクスポート
src/mate/vcf/
├── state.rs            VCFState: Game + 攻め方 + limit。四を作る手のペア         (§1)
├── dfs.rs              DFSSolver: 深さ優先探索と行き止まりメモ                  (§2)
└── iddfs.rs            IDDFSSolver: limit を増やしながら DFSSolver を走らせる    (§3)
```

## 1. `VCFState`: 四はどこにあるか

```rust
pub struct VCFState { game: Game, pub attacker: Player, pub limit: u8 }
```

`VCFState::init(&board, attacker, limit)` は攻め方の手番から始める。`VCFState::new(game, limit)` は既存のゲームを受け取る（追い詰めソルバーは内部の四追い状態をこれで作る。攻め方は `game` の手番側になる）。

四は **`Sword`** に打つことで作られる。5 マスの窓の中に自分の石が 3 つ、空の**眼**が 2 つあるものだ。片方の眼に打つと、残る勝ち点がもう片方の眼である `Four` ができる。したがって 1 つの剣から `(攻め, 受け)` のペアが 2 つ得られる（`sword_eyes_pairs`）:

```
H 列:  H7 = x（白）、H8 H9 H10 = o（黒）、H11 H12 = 空
        → 眼 H11、H12 の Sword
ペア:  (H11, H12)   H11 に打つ: 四 H8–H11。白は H12 に受けねばならない
       (H12, H11)   H12 に打つ: 四 H8,H9,H10,_,H12。白は H11 に受けねばならない
```

生成器は 3 つあり、ソルバーはこの順に試す:

| 生成器 | ペアの出どころ | いつ |
| --- | --- | --- |
| `forced_move_pair(p)` | 攻めの眼がちょうど `p` である剣 | 攻め方が `Forced(p)` のとき。受け方の止めが自分の四になった（ノリ手）ので、攻め方の応手はそれを止め、**かつ**四でなければならない。さもなければ四追いは終わり |
| `neighbor_move_pairs()` | `last2_move`（攻め方の直前の石）を通る剣 | 最初に — 直前に打った石を伸ばすのが、四を作り続ける最も見込みのある道 |
| `move_pairs()` | 手番側のすべての剣 | 次にそれ以外すべて |

止めは剣から取り、攻め手の後の盤面から求め直しはしない。攻め手がたまたま四を **2 つ**作った場合は、保持していた止めが打たれる前に受け方の `check_event` が `Defeated(Fours(..))` を報告するので、保持している眼が意味を持つのは四が 1 つのときだけである。

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
        Forced(p)   → forced_move_pair(p) → search_attack、なければ None
        None        → 近傍ペア、次に残りのペアの各 (a, d) について:
                          search_attack(a, d) が Some なら → それを返す
                      None

search_attack(state, attack, defence):
    if attack が禁手: return None
    into_play(attack): search_defence(defence)   その後 attack を先頭に付ける

search_defence(state, defence):                # 受け方の手番
    if check_event() が Defeated(end): return Mate { end, path: [] }
    into_play(defence): search(state)            その後 defence を先頭に付ける
                                                 # limit は into_play の中で 1 減る
```

注目すべき点:

- **メモするのは失敗だけ。** 成功はそのまま返す。四追いがないと示された局面は、示された最大の limit として `deadends: Memo<u8>` に記録される。ここでは手の生成が `limit` を読まないので、limit *n* の木は limit *n+1* の木を途中で切ったものであり、「*n* 以内に四追いなし」は *n* までのすべての limit について厳密である。だからメモのキーは `Key::position` だけでよい（04 の §3）。
- **禁手の攻め手は飛ばす**ので、黒がここで四四や長連を打つことはない。黒の `Fours` の詰め上がりが常に棒四であるのもこのためである（03 の §5）。
- **ノリ手は特別扱いしない。** 止めが四になったとき、攻め方の次のノードは `Forced(p)` を見る。`forced_move_pair(p)` が成功するのは、`p` 自身が四を作る眼であるときだけである。`test_vcf_counter` と `test_vcf_not_opponent_double_four` がこれを検証している。
- **攻め方のノードでの `Defeated` は枝を終える。** 受け方の止めが四四（あるいは黒が止められない四）を作った。それに答える四は攻め方にないので、手順は失敗する。

## 3. `IDDFSSolver`

`IDDFSSolver::init(limits)` は、`limits` のうち状態自身の limit より小さい各 `limit` で `DFSSolver::search` を走らせ、最後に状態自身の limit で走らせて、最初の結果を返す。浅いパスは木全体を歩かずに短い四追いを見つけ、行き止まりメモは limit ごとに厳密なので、深いパスにとって浅いパスは無駄にならない。

`Solver` を同じ `solve` / `search` の分け方で実装する。追い詰めソルバーは各側に 1 つずつ `limits = [1]`（「まず 1 手で勝てるか調べる」）のものを持ち、その `search` を呼ぶ（06 の §2）。

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

`solve(VCFDFS, 3, &board, Black, 0)` は `I8,G8,I10,I9,J9`、詰め上がり `Fours(H11, M6)` を返す:

| 手 | 作るもの | その後の `limit` |
| --- | --- | --- |
| 根 | | 3 |
| `I8`（黒） | 剣 `H8,J8,K8`（眼 `G8`、`I8`）から四 `H8,I8,J8,K8` | 3 — 直前に打った攻め手はまだ数に入る |
| `G8`（白） | 強制の止め | 2 |
| `I10`（黒） | 四 `I6,I7,I8,_,I10` | 2 |
| `I9`（白） | 強制の止め | 1 |
| `J9`（黒） | `I10,J9,K8,L7`、`H11` と `M6` の両方が空 — 棒四、2 つの `Four` | 1 |
| | 白の `check_event` → `Defeated(Fours(H11, M6))` | |

`limit = 2` では同じ呼び出しが `None` を返す。`I9` の後で limit が 0 になり、`J9` を試す前に `search` が止まる。3 つの四には `limit >= 3` が要る。

## 5. チートシート

| 知りたいこと | 見る場所 |
| --- | --- |
| 四を作る手が生成されない | `VCFState::move_pairs` は `Sword` の眼しか見ない。黒では `exact` の余白判定が長連になる四を除く |
| ノリ手の扱いがおかしい | 攻め方のノードでの `Game::check_event` → `Forced(p)`、続けて `VCFState::forced_move_pair` |
| どの四が先に試されるか | `neighbor_move_pairs`（攻め方の直前の石を通るもの）、次に `move_pairs` |
| メモ | `DFSSolver::deadends`: 四追いのない最大 limit。キーは `Key::position`。予算切れの後は書かれない |
| なぜ `solve` と `search` があるか | `solve` は世代を開く。`search` は `IDDFSSolver` と追い詰めソルバーが繰り返し呼ぶもの（04 の §4） |
| 四追いはどこまで深くできるか | 攻め手 `limit` 手。各手が四なので、四 *k* 個の四追いには `limit >= k` が要る |

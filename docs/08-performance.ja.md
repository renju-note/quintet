# 性能の記録

ソルバーがどれだけ速くなったか、何が効いたか、何が残っているか。

- [#149](https://github.com/renju-note/quintet/pull/149)（2026-09-23）から [#170](https://github.com/renju-note/quintet/pull/170)（2026-09-28）までの作業を、2024 年版と比べて記録する。
- 次にソルバーを調整する人のために: 何を試したか、今どこに時間がかかっているか。

前提: [07](07-benchmarks.ja.md)（ベンチマーク、ノード数と時間）、[06](06-solver-vct.ja.md)（追い詰め探索）。

## 1. 要約

`benches/cases/` の全 121 ケース（`--all`）を 4 時点で測った:

| 時点 | コミット | 時間、全件 | VCT 既定（99） | VCT `heavy`（16） | ノード数、全件 |
| --- | --- | --- | --- | --- | --- |
| 2024-03、[#120](https://github.com/renju-note/quintet/pull/120) | `d8bdf3c` | 498.3 s | 106.4 s | 391.9 s | — |
| 2026-09-23、[#149](https://github.com/renju-note/quintet/pull/149) | `bef47e5` | 433.5 s | 90.4 s | 343.1 s | 309.4M |
| 2026-09-27、[#165](https://github.com/renju-note/quintet/pull/165) | `a34e0c8` | 82.6 s | 24.1 s | 58.4 s | 137.3M |
| 2026-09-28、[#169](https://github.com/renju-note/quintet/pull/169) | `f73f4c5` | 49.6 s | 16.4 s | 33.2 s | 92.3M |

- 時間: 全体で 2024 年の 10.0 倍速い。既定の集合で 6.5 倍、`heavy` で 11.8 倍。
- ケースごと: 幾何平均で 6.5 倍、中央値で 8.1 倍（2024 年に 1 ms 以上かかった 95 ケース）。
- VCF の 6 ケース: 合計 1.25 ms → 0.36 ms。
- どのケースも 4 時点で判定が同じで、タイムアウトしたものはない。
- [#120](https://github.com/renju-note/quintet/pull/120) には `solve_with_stats` がないので、ノード数は数えられない。

測り方:

- 1 台の Apple Silicon の Mac で、1 ケース 1 プロセス、時点を 1 つずつ順に。負荷平均は約 2。
- 既定の集合は 3 回の中央値、`heavy` は 1 回。
- 時間はこの表の中でだけ比べられる。ノード数はどのマシンでも同じ。

変化の大きかったケース:

| ケース | 2024 | 今 | 倍率 |
| --- | --- | --- | --- |
| `vct_unstable` | 164.3 s | 11.0 s | 15× |
| `vct_black_long_short` | 31.8 s | 2.8 s | 11× |
| `vct_game_black_l09_3` | 16.0 s | 0.8 s | 20× |
| `vct_small_but_long` | 23.4 s | 6.4 s | 3.6×（最小） |

## 2. 何が効いたか

[#149](https://github.com/renju-note/quintet/pull/149) から [#170](https://github.com/renju-note/quintet/pull/170) までの各マージコミットを、今の `benches/` で走らせた（§7）。[#159](https://github.com/renju-note/quintet/pull/159) はケースを足しただけ。

探索の変更（ノード数、全 121 ケース）:

| PR | 内容 | ノード数、全件 | 既定の集合 | `heavy` | 詳細 |
| --- | --- | --- | --- | --- | --- |
| [#160](https://github.com/renju-note/quintet/pull/160) | 追い詰めの候補を、手が作る形で並べる | 309.4M → 285.6M（−8%） | −16% | −6% | 06 §8 |
| [#161](https://github.com/renju-note/quintet/pull/161) | 追い手になりえない攻め手を除く（VCF のゾーン） | → 221.2M（−23%） | −32% | −20% | 06 §3 |
| [#162](https://github.com/renju-note/quintet/pull/162) | 四と三の証明数の見積もりを下げる | → 137.4M（−38%） | +6% | −47% | 06 §5 |
| [#163](https://github.com/renju-note/quintet/pull/163) | `ShapeMap` が `PotentialField` に代わる | → 137.0M（−0.3%） | −1% | 0% | 06 §8 |
| [#165](https://github.com/renju-note/quintet/pull/165) | 禁手判定を減らす | → 137.3M（+0.2%） | 0% | +0.3% | 06 §5 |
| [#167](https://github.com/renju-note/quintet/pull/167) | df-pn の閾値に 1 + ε | → 99.4M（−28%） | −19% | −31% | 06 §5 |
| [#168](https://github.com/renju-note/quintet/pull/168) | 候補手キャッシュを 65536 件に | → 92.3M（−7%） | −2% | −9% | 06 §3 |

- ほかの PR では、どのケースのノード数も変わらない。
- [#162](https://github.com/renju-note/quintet/pull/162) の既定の集合での +6% は `vct_small_but_long`（5.8M → 16.7M）によるもので、残りは 26% 減った。`vct_unstable` は 109.5M → 41.6M。
- [#165](https://github.com/renju-note/quintet/pull/165) はノード数を少し変える。禁手の攻め手は初めて探索するときに見つかるので、それまでは親の数値に数えられる。

時間、全 PR（既定の集合 105 ケース、各 1 回、1 台のクラウドのマシンでコミットを 1 つずつ順に）:

| PR | 内容 | 時間 | 変化 |
| --- | --- | --- | --- |
| [#149](https://github.com/renju-note/quintet/pull/149) | （起点） | 188.7 s | |
| [#151](https://github.com/renju-note/quintet/pull/151) | ドキュメントとケースファイル | 184.4 s | −2% |
| [#152](https://github.com/renju-note/quintet/pull/152) | 子のキーを XOR で、恒等ハッシュ、ポテンシャルの遅延更新 | 124.7 s | −32% |
| [#153](https://github.com/renju-note/quintet/pull/153) | VCF の手の生成のために剣をキャッシュ | 118.3 s | −5% |
| [#154](https://github.com/renju-note/quintet/pull/154) | 改名 | 115.0 s | −3% |
| [#155](https://github.com/renju-note/quintet/pull/155) | `SwordMap` をソルバーの状態へ移す | 112.2 s | −2% |
| [#156](https://github.com/renju-note/quintet/pull/156) | `Segment` と、その上に作った `Line` | 107.6 s | −4% |
| [#157](https://github.com/renju-note/quintet/pull/157) | 改名 | 109.1 s | +1% |
| [#158](https://github.com/renju-note/quintet/pull/158) | `Bits` イテレータ、禁手のビット演算 | 100.3 s | −8% |
| [#160](https://github.com/renju-note/quintet/pull/160) | 形で並べる | 94.0 s | −6% |
| [#161](https://github.com/renju-note/quintet/pull/161) | 追い手でない手を除く | 68.8 s | −27% |
| [#162](https://github.com/renju-note/quintet/pull/162) | 証明数の見積もり | 69.5 s | +1% |
| [#163](https://github.com/renju-note/quintet/pull/163) | `ShapeMap` が `PotentialField` に代わる | 54.5 s | −22% |
| [#164](https://github.com/renju-note/quintet/pull/164) | リファクタリング | 53.5 s | −2% |
| [#165](https://github.com/renju-note/quintet/pull/165) | 禁手判定を減らす | 45.8 s | −14% |
| [#166](https://github.com/renju-note/quintet/pull/166) | 子の数値を一度だけ読む | 43.7 s | −5% |
| [#167](https://github.com/renju-note/quintet/pull/167) | 1 + ε | 36.2 s | −17% |
| [#168](https://github.com/renju-note/quintet/pull/168) | 候補手キャッシュを大きく | 35.8 s | −1% |
| [#169](https://github.com/renju-note/quintet/pull/169) | 4 つの定数倍の改善 | 33.4 s | −7% |
| [#170](https://github.com/renju-note/quintet/pull/170) | 改名 | 33.4 s | 0% |

- 改名（[#154](https://github.com/renju-note/quintet/pull/154)、[#157](https://github.com/renju-note/quintet/pull/157)、[#170](https://github.com/renju-note/quintet/pull/170)）の変化がノイズの幅を示す: 約 ±3%。
- [#163](https://github.com/renju-note/quintet/pull/163) はノード数よりずっと多く時間を減らした。`PotentialField` を盤面に合わせ続けるのに、探索の時間の約 5 分の 1 がかかっていた（06 §8）。
- 既定の集合はほとんどが小さいケースなので、大きいケースを狙った変更はここではあまり表れない。[#168](https://github.com/renju-note/quintet/pull/168) はここで −1%（ノード数 −2%）、`heavy` ではノード数 −9%。
- このマシンでは、2024 年版は同じ集合に 190.4 s かかる。今の 5.7 倍（§1 の Mac では 6.5 倍）。

まとめると:

- 探索（ノード数）: 追い手になりえない攻め手を除く（[#161](https://github.com/renju-note/quintet/pull/161)）、形による並べ替えと見積もり（[#160](https://github.com/renju-note/quintet/pull/160)、[#162](https://github.com/renju-note/quintet/pull/162)）、1 + ε の閾値（[#167](https://github.com/renju-note/quintet/pull/167)）、大きな候補手キャッシュ（[#168](https://github.com/renju-note/quintet/pull/168)）。
- 1 ノードのコスト（ノード数が同じで時間だけ）: 子のキーを XOR で（[#152](https://github.com/renju-note/quintet/pull/152)）、ビット演算（[#156](https://github.com/renju-note/quintet/pull/156)、[#158](https://github.com/renju-note/quintet/pull/158)）、`PotentialField` をやめる（[#163](https://github.com/renju-note/quintet/pull/163)）、禁手判定を減らす（[#165](https://github.com/renju-note/quintet/pull/165)）、それより小さい [#153](https://github.com/renju-note/quintet/pull/153)、[#166](https://github.com/renju-note/quintet/pull/166)、[#169](https://github.com/renju-note/quintet/pull/169)。

## 3. 遅くなったケース

2024 年より遅いケースが 8 つある。どれも 70 ms 未満の小さいケース（§2 のクラウドのマシンで既定の集合、3 回の中央値）:

| ケース | 2024 | 今 | ノード数 [#149](https://github.com/renju-note/quintet/pull/149) → 今 |
| --- | --- | --- | --- |
| `vct_game_black_l04_1` | 0.7 ms | 1.3 ms | 189 → 1,056 |
| `vct_game_black_l05_2` | 1.5 ms | 7.7 ms | 633 → 6,932 |
| `vct_game_black_l10_1` | 23 ms | 63 ms | 8,711 → 54,123 |
| `vct_game_white_l04_2` | 0.9 ms | 1.1 ms | 230 → 851 |
| `vct_game_white_l05_1` | 1.5 ms | 1.9 ms | 266 → 1,703 |
| `vct_game_white_l06_1` | 1.3 ms | 2.1 ms | 548 → 1,990 |
| `vct_game_white_l06_2` | 14 ms | 28 ms | 6,105 → 25,089 |
| `vct_game_white_l10_1` | 7.4 ms | 18 ms | 4,012 → 19,643 |

- 1 ノードのコストは下がったが、ノード数が 4〜11 倍に増えた。
- 8 つとも [#162](https://github.com/renju-note/quintet/pull/162) で 1.7〜9 倍に増えた。その見積もりは、探索に四と三を先に追わせる。これらの局面では、それが遠回りになる。
- ほかの PR による変化はそれより小さい。ただし `vct_game_white_l10_1` は [#160](https://github.com/renju-note/quintet/pull/160) で 2.3 倍になった。
- 合わせても 2024 年より 0.1 s 未満の増加で、同じ集合で 150 s 以上減ったのに比べれば小さい。
- §1 の Mac では 9 つだった。1〜2 ms のケースは、ノイズで入ったり出たりする。

## 4. 今どこに時間がかかっているか

[#169](https://github.com/renju-note/quintet/pull/169) の直前の `main` で、ベンチマーク全体をプロファイルした:

| 部分 | 割合 |
| --- | --- |
| 内部の四追い探索（`NestedVCF` から呼ぶ `DFSSolver`） | 約 55% |
| `check_event` | 約 16% |
| `SwordMap::sync` | 約 11% |
| `ShapeMap::sync` | 約 10% |
| メモリの確保と解放 | 約 5〜10% |

（各部分は重なる。内部の四追い探索も `check_event` と `SwordMap::sync` を呼ぶ。）

- その後 [#169](https://github.com/renju-note/quintet/pull/169) で、全体のうち `check_event` から約 5%、`ShapeMap::sync` から約 4%、確保から 5〜6%（展開ごとの子の `Vec` と、内部の探索ごとに複製していた着手履歴）を減らした。
- 内部の四追い探索は今も半分を超える。その多くは「これは追い手か」（`attacker_vcf.threat`）か「攻め方に四追いがあるか」に答えている。

## 5. 試して入れなかったもの

| 案 | 結果 | 入れない理由 |
| --- | --- | --- |
| df-pn の閾値で ε = 1/4〜1/2 | `vct_unstable` のノード数 +47%（1/4）、+41%（1/2） | 既定の集合ではよいが、長いケースで不安定。1/8 は両方でよい（06 §5）。 |
| `SwordMap` を攻め方の分だけ同期 | 時間 −0.6% | ノイズの範囲。 |
| 最有望の攻め手を必ず 1 回展開（[#108](https://github.com/renju-note/quintet/pull/108)） | `vct_unstable` は 345 倍安く、既定の集合は 39% 高く | 局面による差が大きすぎる（06 §5、issue [#148](https://github.com/renju-note/quintet/issues/148)）。 |
| 候補手キャッシュを 65536 件より大きく | 262144 件: ノード数 −8%、100 MB | 4 倍のメモリで得るものが少ない（06 §3）。 |

## 6. 残りの改善候補

- 内部の四追い探索を減らすか、安くする。時間の半分を超えるので、それを呼ばずに済ませる工夫（[#161](https://github.com/renju-note/quintet/pull/161) が追い手でない手についてしたように）が最も効く。
- `check_event`。[#169](https://github.com/renju-note/quintet/pull/169) の後も時間の約 1 割。
- §3 の小さいケース: 大きいケースでの [#162](https://github.com/renju-note/quintet/pull/162) の得を保ちつつ、これらでの損をなくす見積もり。
- `vct_small_but_long` は伸びが最も小さい（3.6 倍）。探索が長い四の連続をたどるので、展開の順序が最も効く。

## 7. 再現方法

コミットのノード数（どのマシンでもよい）:

1. コミットを `git worktree` に展開し、その `benches/` を今のものに置き換える（`scripts/bench-compare.sh` と同じ、07 §3）。
2. コピーしたケースで、着手列になっている `expect:` 行をすべて `expect: proven` に変える。古いソルバーは別の詰み手順を見つけうるので、判定だけを検査する。
3. `cargo bench --bench solvers -- --all --runs 1 --save <commit>.tsv` を走らせる。

時間: 同じマシンで、ほかに何も走らせずに、コミットを 1 つずつ順に走らせる。§2 の表は既定の集合を `--runs 1` で測った。

2024 年版（[#120](https://github.com/renju-note/quintet/pull/120)）には手間がかかる:

- API は `solve(mode, limit, board, attacker, threat_limit) -> Option<Mate>` で、`solve_with_stats` もノード数もない。
- ケースファイルを読んでこの関数を呼ぶ小さな example を一時的に作り、1 ケース 1 プロセスで時間を測る。

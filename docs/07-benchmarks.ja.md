# ソルバーのベンチマーク

ソルバーへの変更で速くなったかどうかを確かめる方法。ベンチマークは、局面の集合 `benches/cases/*.txt` と、それを解くランナー `benches/solvers.rs` からなる。ランナーは各局面を解いて答えを検査し、探索にかかったコストを報告する。

前提: [03](03-solver-api.ja.md)（`solve_with_stats`、`SolveLimits`、`SolveResult`）。

```
benches/
├── solvers.rs              ランナー（`harness = false`）                    (§2)
└── cases/*.txt             1 ファイル 1 局面                                (§4)
scripts/bench-compare.sh    git の ref と作業ツリーの両方でランナーを走らせる (§3)
```

## 1. 何を測るか

| 列 | 内容 | 決定的か |
| --- | --- | --- |
| `nodes` | 訪れたノード数。`SolveStats::nodes` で、`NodeBudget` と同じ数え方（03 §3）。内部の四追い探索も含む。 | はい |
| `memo` | 探索後にソルバーのメモに残っているエントリ数。`SolveStats::memo_len`。探索に必要だったメモリの目安。 | はい |
| `time` | `solve_with_stats` の実時間。`--runs` 回の中央値。 | いいえ |

変更を比べる尺度はノード数。マシンにも負荷にもよらないので、差があればそれはコードの変更によるもので、マシンをまたいでも CI でも比べられる。ただしノード数が測るのは探索の量であって、1 ノードあたりのコストではない。1 ノードを安くする変更（ビット演算の工夫、アロケーションの削減、手の生成の高速化）ではノード数は変わらず、時間にだけ表れる。時間は同じマシンで続けて走らせたもの同士でしか比べられない。

ランナーは、同じケースを何回走らせても判定と数値が同じであることを確かめ、違えばそのケースを失敗として報告する。

## 2. 実行方法

```sh
cargo bench --bench solvers -- [OPTIONS] [NAME_FILTER...]
```

`cargo bench` は release プロファイルでビルドする。`--` の後の引数はランナーに渡る。

| オプション | 効果 |
| --- | --- |
| `NAME_FILTER...` | 名前にこのいずれかを含むケースだけを走らせる。 |
| `--tag TAG` | `TAG` の付いたケースだけ。複数指定でき、すべてを持つケースが選ばれる。 |
| `--skip-tag TAG` | `TAG` の付いたケースを除く。複数指定できる。 |
| `--all` | `heavy` の付いたケースも走らせる。これか `--tag heavy` がなければ `heavy` は除かれる。 |
| `--mode MODE` | 各ケースを自身のモードではなく `MODE`（`vcf`、`vct`、`vct_pns`、`vct_dfpns`）で解く。判定は検査するが、手順までは検査しない。追い詰めのモードごとに報告する詰み手順が違いうるため（03 §2）。 |
| `--runs N` | 各ケースを `N` 回計測して中央値を報告する。既定は 3。 |
| `--save PATH` | 結果を TSV で書き出す。 |
| `--baseline PATH` | `--save` で書いたファイルと比べる。各行にノード数・メモのエントリ数・時間の増減が付き、最後に合計が出る。 |
| `--cases DIR` | `benches/cases` の代わりに `DIR` からケースを読む。 |

答えが違ったケースは、期待した答えと得られた答えとともに `FAILED` と表示され、ランナーは 0 以外の終了コードで終わる。つまりベンチマークは、`cargo test` には重すぎる局面のテストも兼ねている。

よく使う形:

```sh
cargo bench --bench solvers -- --tag quick          # 数秒
cargo bench --bench solvers                         # heavy 以外すべて。数分
cargo bench --bench solvers -- --tag vct --tag disproven
cargo bench --bench solvers -- --mode vct_pns --tag vct   # 同じ局面を PNS で
```

## 3. 2 つの版を比べる

```sh
scripts/bench-compare.sh main                   # 作業ツリーと main を比べる
scripts/bench-compare.sh HEAD~1 --tag vct       # ランナーのオプションはそのまま渡る
```

スクリプトは ref を一時的な `git worktree` に展開し、その `benches/` を作業ツリーのものに置き換え、そこで `--save` 付きでベンチマークを走らせてから、作業ツリーで `--baseline` 付きで走らせる。両側で同じランナーと同じケースを使うので、違いはソルバーだけになる。ref は `target/bench-base` でビルドし、次の比較で一からビルドし直さないよう残しておく。ref には `solve_with_stats` が必要。

手で同じことをするなら:

```sh
git stash
cargo bench --bench solvers -- --save base.tsv
git stash pop
cargo bench --bench solvers -- --baseline base.tsv
```

GitHub では **Bench** ワークフロー（`.github/workflows/bench.yml`）がプルリクエストごとにこれを行う。既定の集合を `--runs 1` で、PR のベースに対して `scripts/bench-compare.sh` で比べ、表をジョブのサマリーに載せる。`main` へのプッシュや、ベンチマークが入る前のベースに対しては、比較せずにベンチマークだけを走らせる。CI ワークフローとは別で、必須チェックにもしていないので、実行中でも失敗してもマージを妨げない。ただしケースの答えが違えば失敗し、赤く表示される。時間は共有ランナーでの値なので目安にとどまるが、ノード数は正確。

ソルバーを変更する PR には、比較結果を説明に貼る。アルゴリズムの変更ならレビューで見るのはノード数の列。定数倍の最適化なら時間の列で、ノード数は `=` のまま時間が減っているはず。時間を狙っていない PR で、ノード数が `=` なのに時間が目立って変わったときは、比較をやり直してから判断する。

## 4. ケース

1 ファイルに 1 局面。ファイル名から `.txt` を除いたものがケース名になる。

```
# 行頭の `#` から行末まではコメント。
mode: vct_dfpns
attacker: o
limit: 4
threat_limit: 1
tags: vct proven quick
expect: F10,G9,I10,G10,H11,H12,G12
board:
. . . . . . . . . . . . . . .
. . . . . . . . x . . . . . .
...
```

| キー | 必須 | 意味 |
| --- | --- | --- |
| `mode` | はい | `SolveMode` の CLI 名（03 §2）。 |
| `attacker` | はい | `o`（黒）か `x`（白）。 |
| `limit` | はい | `SolveLimits::limit`。 |
| `threat_limit` | いいえ | `SolveLimits::with_threat_limit`。既定は 0。 |
| `defender_vcf_depth` | いいえ | `SolveLimits::with_defender_vcf_depth`。既定は `DEFAULT_DEFENDER_VCF_DEPTH`。 |
| `max_nodes` | いいえ | `SolveLimits::with_max_nodes`。既定は無制限。 |
| `tags` | いいえ | 空白区切りのタグ（下記）。 |
| `expect` | はい | カンマ区切りの詰み手順。または `proven`（手順は問わない）、`disproven`、`aborted`。 |
| `board` | はい | 最後のキー。次の行からテストと同じ形式（`o` = 黒、`x` = 白、最上段が 15 行目）で盤面を書くか、同じ行に着手列（`H8,I9,...`）を書く。 |

使っているタグ:

| タグ | 意味 |
| --- | --- |
| `vcf` / `vct` | どちらの探索を試すケースか。 |
| `proven` / `disproven` | 期待する判定。詰みなしの証明は詰みの証明と同じくらい重要。エンジンが問う局面の多くには詰みがなく、詰みなしを示すには木全体を探索しなければならないため。 |
| `quick` | 1 秒よりずっと短い。変更のたびに走らせる集合。 |
| `slow` | 数秒から数十秒。 |
| `heavy` | 既定の集合には重すぎるもの。1 分以上、`game` のケースでは 3 秒以上。指定しない限り除かれる。 |
| `game` | 実戦の棋譜から取った局面（§4.1）。 |

ケースの追加:

- 変更の対象になった局面、ユーザーが遅いと感じた局面、リグレッションはここに入れる。適切な集合に入るようタグを付ける。
- `expect` の手順は、ソルバーがどの詰み手順を見つけるかまで固定する。検査として役に立つが、手の並べ替えを変えると正当に別の手順が見つかることがある。そのときは新しい手順が詰みになっていることを確かめてファイルを更新し、PR にその旨を書く。
- 出典の分かる局面を使い、出典（詰め連珠の作者と番号、リンクなど）をコメントに書く。
- `quick` は速く保つ。既定の集合（`heavy` 以外すべて）は Bench ワークフローがプルリクエストごとに走らせるので、数分に収める。

### 4.1 実戦の局面（`vct_game_*`）

上の詰め連珠は、難しい詰みが 1 つあるように作られた局面。`game` のケースは、実戦に現れた局面のほう。Rapfi（classical）と開発中のエンジン quintet-ai との約 2,400 局から取った、追い詰めの 100 局面。五で終わった対局は詰みで終わっているので、各局を終局から 2 手ずつ、勝った側の手番の局面について遡った。四追いがある局面は飛ばし、それ以外は 1 つの df-pn ソルバーを使い回して limit 1, 2, 3, ... と問い、最短の追い詰めの limit `L` を求めた（`threat_limit` 2）。limit 20 以内に追い詰めがない局面で遡るのをやめた。

- **名前。** `vct_game_{black|white}_lLL_N` は最短の limit `LL` で詰み。`..._short` は同じ局面を `LL - 1` で問うもので、詰みなし。2 つで詰みの長さを両側から挟む。
- **散らばり。** 1 色 50 問。詰みが 30 問で、limit の帯 3–4、5–6、7–8、9–10、11–12、13 以上に 5、6、7、6、3、3 問ずつ。各帯の中は df-pn のノード数で易しいものから難しいものまで散らした。うち 20 問には `_short` の対がある。1 つの帯の詰みのケースはすべて別々の対局から取った（2 つの帯に 1 問ずつ出した対局が 4 局ある）。最長は limit 17（黒）と 18（白）。
- **検査。** どの判定も、`vct_dfpns` と `vct_pns` で改めて解いて得られ、`vct` とも食い違わない（`vct` は 29 問で 2,000 万ノードの予算を使い切った）。`LL` で詰みかつ `LL - 1` で詰みなしなので、`LL` が最短の limit。
- **コスト。** 100 問を 1 回解くと Apple M1 で約 140 秒。3 秒以上の 14 問に `heavy` を付けたので、既定の集合に入るのは約 40 秒。全部は `--tag game --all`。
- 各ファイルのコメントに、対局者と局面までの手順を書いた。

## 5. チートシート

| したいこと | 使うもの |
| --- | --- |
| 1 回の探索のノード数をコードで知る | `solve_with_stats`（03 §2） |
| 普段の集合を走らせる | `cargo bench --bench solvers` |
| 変更をすばやく確かめる | `cargo bench --bench solvers -- --tag quick` |
| `main` と比べる | `scripts/bench-compare.sh main` |
| 同じ局面で追い詰めの 2 つのモードを比べる | `--mode vct_pns --tag vct` と `--mode vct_dfpns --tag vct` |
| 結果を残して後で比べる | `--save PATH`、後で `--baseline PATH` |
| 実戦の局面を走らせる | `cargo bench --bench solvers -- --tag game --all`（§4.1） |
| 局面を追加する | `benches/cases/NAME.txt` を新しく作る（§4） |

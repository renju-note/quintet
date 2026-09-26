# ドキュメント

`quintet` を開発する人および AI エージェント向けのリファレンスである。ドメイン（連珠）と、そのルールがコードにどう対応しているかを説明し、ソースから毎回導き直さなくてもソルバーの変更を検討できるようにするためのものである。

すべてのドキュメントは英語版（`*.en.md`）と日本語版（`*.ja.md`）のセットで置く。英語版の索引: [README.en.md](README.en.md)。

| ドキュメント | 内容 |
| --- | --- |
| [01-renju-rules.ja.md](01-renju-rules.ja.md) / [en](01-renju-rules.en.md) | RIF 連珠国際ルール（盤、用語定義、勝敗、禁手、開局規定）の Markdown 版。 |
| [02-board-implementation.ja.md](02-board-implementation.ja.md) / [en](02-board-implementation.en.md) | `src/board/` が盤面をどう表現し、上のルールをどう実装しているか: 線のビット表現、セグメントと連の検出、禁手判定、ハッシュ。 |
| [03-solver-api.ja.md](03-solver-api.ja.md) / [en](03-solver-api.en.md) | ソルバーの使い方: `solve`、`SolveMode`、`SolveLimits`、`SolveResult`、`NodeBudget`、`Solver` トレイトでソルバーを問いをまたいで保持する方法、`Mate` / `End`。 |
| [04-solver-framework.ja.md](04-solver-framework.ja.md) / [en](04-solver-framework.en.md) | `src/mate/` の内側: 構成と、すべてのソルバーが共有する部品 — `Game` と `check_event`、`State` と `Key`、`Memo` の世代、`Solver` トレイト、ノード予算。 |
| [05-solver-vcf.ja.md](05-solver-vcf.ja.md) / [en](05-solver-vcf.en.md) | `src/mate/vcf/` の四追い（VCF）探索: 四を作る手のペア、`DFSSolver` と行き止まりメモ、`IDDFSSolver`、手順を追う例。 |
| [06-solver-vct.ja.md](06-solver-vct.ja.md) / [en](06-solver-vct.en.md) | `src/mate/vct/` の追い詰め（VCT）探索: 追い手、内部の四追い探索、手の生成、証明数、DFS / PNS / df-pn の閾値ポリシー、手順の復元、手順を追う例、手の並べ替え（`PotentialField`、`ShapeMap`）。 |
| [07-benchmarks.ja.md](07-benchmarks.ja.md) / [en](07-benchmarks.en.md) | `benches/` にあるソルバーのベンチマーク: 何を測るか（ノード数、メモのエントリ数、時間）、実行方法、2 つの版の比較、ケースの追加。 |

どこから読むか:

- 連珠を知らない場合はまず 01 を読む。ソルバーのドキュメントはその用語（四、棒四、三、禁手）を前提にしている。
- `src/board/` のルール周り（連、禁手）を変更するなら 02 を読む。
- アプリや CLI、自分の Rust コードからソルバーを呼ぶなら 03 を読む。
- `src/mate/` や `src/feature/` の探索を変更するなら 04、続いて 05 と 06 を読み、変更の効果は 07 のベンチマークで測る。
- 特定のことだけ知りたい場合は、各ドキュメント末尾のチートシート（02 §9、03 §7、04 §7、05 §5、06 §9）が「知りたいこと」から識別子への対応表になっている。

ドキュメント追加時の規約:

- 1 トピック 1 ファイルとし、読む順に通し番号を付けた `NN-kebab-case.en.md` **と** `NN-kebab-case.ja.md` の（`README.*` には番号を付けない）両方を作って上の表からリンクする。追加・更新は必ず両言語同時に行い、内容を一致させる。
- 日本語の段落は途中で改行しない（Markdown では段落内の改行がスペースとして表示され、日本語の文中に不自然な空白が入るため）。英語版は従来どおり折り返してよい。
- 盤面の図はテストと同じ ASCII 形式（`o` = 黒、`x` = 白、`.` = 空点、最上段が 15 行目）で書き、そのまま `.parse::<Board>()` のテストに貼れるようにする。
- コードを説明するときは実際の識別子（`Grid::rows_on`、`RowKind::Sword` など）を書き、`grep` で突き合わせられるようにする。

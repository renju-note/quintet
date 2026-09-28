# ドキュメント

`quintet` を開発する人と AI エージェント向けのリファレンス。ドメイン（連珠）と、そのルールがコードにどう対応するかを説明する。ソースから毎回導き直さずに、ソルバーの変更を検討できるようにするためのもの。

どのドキュメントにも英語版（`*.en.md`）と日本語版（`*.ja.md`）がある。英語版の索引: [README.en.md](README.en.md)。

| ドキュメント | 内容 |
| --- | --- |
| [01-renju-rules.ja.md](01-renju-rules.ja.md) / [en](01-renju-rules.en.md) | RIF 連珠国際ルールの Markdown 版: 盤、用語、勝敗、禁手、開局規定。 |
| [02-board-implementation.ja.md](02-board-implementation.ja.md) / [en](02-board-implementation.en.md) | `src/board/` の盤面表現とルールの実装: 線のビット表現、セグメントと連、禁手、ハッシュ。 |
| [03-solver-api.ja.md](03-solver-api.ja.md) / [en](03-solver-api.en.md) | ソルバーの呼び方: `solve`、`SolveMode`、`SolveLimits`、`SolveResult`、`NodeBudget`、`Solver` トレイトによるソルバーの再利用、`Mate` / `End`。 |
| [04-solver-framework.ja.md](04-solver-framework.ja.md) / [en](04-solver-framework.en.md) | `src/mate/` の内部: 構成と共通部品 — `Game` と `check_event`、`State` と `Key`、`Memo` の世代、`Solver` トレイト、ノード予算。 |
| [05-solver-vcf.ja.md](05-solver-vcf.ja.md) / [en](05-solver-vcf.en.md) | `src/mate/vcf/` の四追い（VCF）探索: 四を作る手のペア、`DFSSolver` と行き止まりメモ、`IDDFSSolver`、手順を追う例。 |
| [06-solver-vct.ja.md](06-solver-vct.ja.md) / [en](06-solver-vct.en.md) | `src/mate/vct/` の追い詰め（VCT）探索: 脅威、内部の四追い探索、手の生成、証明数、DFS / PNS / df-pn の閾値ポリシー、手順の復元、手順を追う例、手の並べ替え（`ShapeMap`）。 |
| [07-benchmarks.ja.md](07-benchmarks.ja.md) / [en](07-benchmarks.en.md) | `benches/` のソルバーベンチマーク: 測るもの（ノード数、メモのエントリ数、時間）、実行方法、2 つの版の比較、ケースの追加。 |
| [08-performance.ja.md](08-performance.ja.md) / [en](08-performance.en.md) | 性能の記録: 2024 年からどれだけ速くなったか、PR ごとのノード数と時間への効果、遅くなったケース、今どこに時間がかかっているか、試したことと残りの候補。 |

どこから読むか:

- 連珠を知らない: まず 01。ソルバーのドキュメントはその用語（四、棒四、三、禁手）を前提にする。
- `src/board/` のルール周り（連、禁手）を変える: 02。
- アプリ、CLI、Rust からソルバーを呼ぶ: 03。
- `src/mate/` や `src/feature/` の探索を変える: 04、続いて 05 と 06。効果は 07 のベンチマークで測る。
- 次に何を速くするか、ある案を試したことがあるかを知りたい: 08。
- 特定のことだけ知りたい: 各ドキュメント末尾のチートシート（02 §8、03 §7、04 §7、05 §5、06 §9）が、知りたいことから識別子を引く表になっている。

ドキュメントの書き方:

- ファイル
  - 1 トピック 1 ファイル。読む順に番号を付け、`NN-kebab-case.en.md` **と** `NN-kebab-case.ja.md` を作って上の表からリンクする（`README.*` は番号なし）。
  - 追加・更新は必ず両言語同時に行い、内容を一致させる。
- 文体
  - 簡潔に書く。文は短くし、長くなる説明は箇条書きや表に分ける。
  - コードは実際の識別子（`Grid::rows_on`、`RowKind::Sword` など）で書き、`grep` で突き合わせられるようにする。
  - 盤面の図はテストと同じ ASCII 形式（`o` = 黒、`x` = 白、`.` = 空点、最上段が 15 行目）で書き、そのまま `.parse::<Board>()` のテストに貼れるようにする。
- 日本語版
  - 段落を途中で改行しない。Markdown は段落内の改行を空白として表示するため、日本語の文中に余計な空白が入る。英語版は折り返してよい。
  - コードブロック（疑似コード、図、ツリー）では、行末のコメント以外は ASCII で書く。全角文字は桁揃えを崩す。

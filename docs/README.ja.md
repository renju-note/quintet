# ドキュメント

`quintet` を開発する人および AI エージェント向けのリファレンスである。ドメイン（連珠）と、そのルールがコードにどう対応しているかを説明し、ソースから毎回導き直さなくてもソルバーの変更を検討できるようにするためのものである。

すべてのドキュメントは英語版（`*.en.md`）と日本語版（`*.ja.md`）のセットで置く。英語版の索引: [README.en.md](README.en.md)。

| ドキュメント | 内容 |
| --- | --- |
| [01-renju-rules.ja.md](01-renju-rules.ja.md) / [en](01-renju-rules.en.md) | RIF 連珠国際ルール（盤、用語定義、勝敗、禁手、開局規定）の Markdown 版。 |
| [02-board-implementation.ja.md](02-board-implementation.ja.md) / [en](02-board-implementation.en.md) | `src/board/` が盤面をどう表現し、上のルールをどう実装しているか: 線のビット表現、連（sequence/structure）の検出、禁手判定、ハッシュ。 |
| [03-solver-overview.ja.md](03-solver-overview.ja.md) / [en](03-solver-overview.en.md) | `src/mate/` の全体像: `solve` とその引数（`limit`、`threat_limit`、`SolveMode`）、共通の探索状態（`Game`、`State`、`check_event`）、四追い（VCF）の深さ優先探索。 |
| [04-solver-algorithm-vct.ja.md](04-solver-algorithm-vct.ja.md) / [en](04-solver-algorithm-vct.en.md) | `src/mate/vct/` の追い詰め（VCT）ソルバー: 追い手、手の生成、証明数、DFS / PNS / df-pn の探索、手順の復元、手の並べ替えに使う `PotentialField`。 |

ドキュメント追加時の規約:

- 1 トピック 1 ファイルとし、読む順に通し番号を付けた `NN-kebab-case.en.md` **と** `NN-kebab-case.ja.md` の（`README.*` には番号を付けない）両方を作って上の表からリンクする。追加・更新は必ず両言語同時に行い、内容を一致させる。
- 日本語の段落は途中で改行しない（Markdown では段落内の改行がスペースとして表示され、日本語の文中に不自然な空白が入るため）。英語版は従来どおり折り返してよい。
- 盤面の図はテストと同じ ASCII 形式（`o` = 黒、`x` = 白、`.` = 空点、最上段が 15 行目）で書き、そのまま `.parse::<Board>()` のテストに貼れるようにする。
- コードを説明するときは実際の識別子（`Square::structures_on`、`StructureKind::Sword` など）を書き、`grep` で突き合わせられるようにする。

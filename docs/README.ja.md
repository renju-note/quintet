# ドキュメント

`quintet` を開発する人および AI エージェント向けのリファレンスです。ドメイン（連珠）と、そのルールがコードにどう対応しているかを説明し、ソースから毎回導き直さなくてもソルバーの変更を検討できるようにするためのものです。

すべてのドキュメントは英語版（`*.en.md`）と日本語版（`*.ja.md`）のセットで置きます。英語版の索引: [README.en.md](README.en.md)。

| ドキュメント | 内容 |
| --- | --- |
| [01-renju-rules.ja.md](01-renju-rules.ja.md) / [en](01-renju-rules.en.md) | RIF 連珠国際ルール（盤、用語定義、勝敗、禁手、開局規定）の Markdown 版。 |
| [02-board-implementation.ja.md](02-board-implementation.ja.md) / [en](02-board-implementation.en.md) | `src/board/` が盤面をどう表現し、上のルールをどう実装しているか: 線のビット表現、連（sequence/structure）の検出、禁手判定、ハッシュ。 |

ドキュメント追加時の規約:

- 1 トピック 1 ファイルとし、読む順に通し番号を付けた `NN-kebab-case.en.md` **と** `NN-kebab-case.ja.md` の（`README.*` には番号を付けない）両方を作って上の表からリンクする。追加・更新は必ず両言語同時に行い、内容を一致させる。
- 盤面の図はテストと同じ ASCII 形式（`o` = 黒、`x` = 白、`.` = 空点、最上段が 15 行目）で書き、そのまま `.parse::<Board>()` のテストに貼れるようにする。
- コードを説明するときは実際の識別子（`Square::structures_on`、`StructureKind::Sword` など）を書き、`grep` で突き合わせられるようにする。

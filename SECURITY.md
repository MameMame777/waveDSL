# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.4.x   | Yes       |
| < 0.4   | No        |

## Reporting a Vulnerability

**公開 Issue での脆弱性報告はしないでください。**

GitHub の "Report a vulnerability" 機能を使用してください:
[Security] タブ → [Report a vulnerability]

リポジトリ管理者は Settings → Security → Private vulnerability reporting
を有効化しています。

報告に含めてください:
- 脆弱性の説明
- 再現手順
- 想定される影響
- 修正案（あれば）

対応 SLA:
- 7 営業日以内に返信
- 確認済み脆弱性は 90 日以内にパッチ・開示

## Supply Chain

各リリースには以下の成果物が含まれます:

| ファイル | 内容 |
|---------|------|
| `wavedsl-vX.Y.Z-windows-x86_64.zip` | ビルド済みバイナリ (Windows x86_64) |
| `wavedsl-vX.Y.Z-src-vendored.tar.gz` | ソース + 全依存クレートのベンダリング済みアーカイブ |
| `CHECKSUMS.txt` | 上記ファイルの SHA-256 チェックサム |

SHA-256 チェックサムは各 GitHub Release の `CHECKSUMS.txt` に記載されています。

依存関係は `Cargo.lock` で固定されており、git にコミットされています。
ベンダリング済みアーカイブは `cargo vendor` で生成し、crates.io のチェックサムで検証済みです。
全依存クレートは well-known な Rust エコシステムのクレートのみです:
`clap`, `serde`, `serde_json`, `thiserror` およびそれらの推移的依存。

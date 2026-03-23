# ADR-002: Vendor Strategy for Offline Build Support

**Date**: 2026-03-23
**Status**: Accepted

## Context

セキュリティ要件が厳しい企業（エアギャップ環境含む）が WaveDSL をビルドできるよう、
`cargo vendor` によるオフラインビルド対応を追加した。
`vendor/` ディレクトリをどう管理するかの設計判断を記録する。

## 決定

`vendor/` は **git にコミットしない**。
代わりに `make-release.bat` が生成する `wavedsl-vX.Y.Z-src-vendored.tar.gz` に含める。

## 検討した選択肢

| 選択肢 | 概要 | 結果 |
|--------|------|------|
| A: git にコミット | 常に最新、運用シンプル | **却下** |
| B: リリース成果物に含める | git を軽量に保つ | **採用** |
| C: ドキュメントのみ | 実装コストゼロ | **却下** |

## 却下理由

**選択肢 A を却下した理由**:
- `cargo vendor` の出力は約 128 MB（アンコンプレス）
  - 主要因: `windows-sys` x2 (33 MB)、`linux-raw-sys` (16 MB)、`libc` (4 MB)
    — いずれも FFI バインディングの自動生成ファイル
- git clone のワーキングツリーが常に 128 MB 展開される
- 依存更新のたびに PR diff が 50 MB 超になる

**選択肢 C を却下した理由**:
- エアギャップ環境では `cargo vendor` 自体を実行できない

## 採用理由（選択肢 B）

- git clone が軽量なまま
- `wavedsl-vX.Y.Z-src-vendored.tar.gz` 一つで完結したオフラインビルド単位になる
- SHA-256 チェックサムによるサプライチェーン検証が可能
- 既存のバイナリ zip 配布と同じリリースパターンに従う
- Linux カーネル・CPython など主要 OSS の source tarball 配布と同じ方式

## 結果

- `vendor/` と `.cargo/config.toml` を `.gitignore` に追加
- `rust-toolchain.toml` で Rust 1.93.1 を固定（再現ビルドの保証）
- `make-release.bat` がリリース成果物 3 点を自動生成:
  1. `wavedsl-vX.Y.Z-windows-x86_64.zip`
  2. `wavedsl-vX.Y.Z-src-vendored.tar.gz`
  3. `CHECKSUMS.txt`
- `SECURITY.md` で脆弱性報告窓口とサプライチェーン情報を公開

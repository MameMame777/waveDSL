# WaveDSL — 人間とAIが読み書きしやすいタイミング図生成　DSLツール

## はじめに

デジタル回路やバスプロトコルの設計・検証において、タイミング図は必須のドキュメントです。
[WaveDrom](https://wavedrom.com/) はJSON記法でタイミング図を描けるツールですが、
規模の大きいタイミングチャートを記述しようとすると、素のJSONを手書きするのは苦痛です。


```json
{
  "signal": [
    { "name": "clk", "wave": "PPPPPPPP" },
    { "data": ["CMD", "DATA"], "name": "mosi", "wave": "x=.=...x" },
    { "data": ["ACK", "0xFF"], "name": "miso", "wave": "x=.=...x" },
    { "name": "cs_n", "wave": "10.....1" }
  ]
}
```


そこで より直観的に理解しやすい記述からJSONを生成する
**WaveDSL** を作りました。

```
signal sclk  clock(8)
signal mosi  x(1) data(2, "CMD") data(4, "DATA") x(1)
signal miso  x(1) data(2, "ACK") data(4, "0xFF") x(1)
signal cs_n  high(1) low(6) high(1)
```

**同じ波形**が、関数呼び出しベースの直感的な記法で書けます。WaveDSLコンパイラがWaveDrom JSONへ変換するので、手で波形文字列と格闘する必要はありません。

https://github.com/MameMame777/waveDSL

## WaveDSLの設計思想

### 人間にもAIにも読みやすく

WaveDSLの目標は「**人間とAIの両方が読み書きしやすいこと**」です。

- **人間**: 関数名（`clock`, `high`, `low`, `data`）で意味が明確。スロット数を数値で指定するので長さが一目瞭然
- **AI（LLM）**: 構造化された文法はプロンプトからの生成に適している。wave文字列のようなドメイン固有の暗号を学習する必要がない

変換方向は **WaveDSL → WaveDrom JSON** の一方向のみ。双方向変換の複雑さを排除し、コンパイラの信頼性を確保しています。

## 機能紹介

### 基本：シグナル定義

```
signal clk   clock(8)               // クロック8周期
signal en    low(2) high(4) low(2)   // 2スロットLow → 4スロットHigh → 2スロットLow
signal bus   x(1) data(4, "PAYLOAD") x(3)  // データバス
```

`signal 名前 波形関数...` のシンプルな構文です。

#### 組み込み関数一覧

| 関数 | 説明 | 例 |
|------|------|-----|
| `clock(n, edge=rising)` | クロック信号 | `clock(8)`, `clock(4, edge=falling)` |
| `high(n)` | High レベル | `high(3)` |
| `low(n)` | Low レベル | `low(2)` |
| `data(n, label, color=1)` | バスデータ | `data(4, "0xAB")`, `data(2, "CMD", color=3)` |
| `x(n)` | 不定値 | `x(1)` |
| `z(n)` | ハイインピーダンス | `z(2)` |
| `gap()` | ギャップ | `gap()` |
| `repeat(n, expr...)` | 繰り返し | `repeat(4, high(1) low(1))` |

### グループ：バス信号の整理

AXIバスのような複雑なインターフェースもネストして表現できます（最大2階層）。

```
group "AXI" {
    group "Write Channel" {
        signal awvalid low(2) high(2) low(4)
        signal awready low(3) high(1) low(4)
        signal wdata   x(2) data(2, "0xDEAD", color=2) x(4)
    }
    group "Read Channel" {
        signal arvalid low(4) high(2) low(2)
        signal rdata   x(4) data(2, "0xBEEF", color=3) x(2)
    }
}
```

### repeat：繰り返しパターン

DMAバースト転送のような繰り返し波形を簡潔に書けます。

```
signal clk   clock(18)
signal burst x(1) repeat(4, data(2, "0xAB") data(2, "0xCD")) x(1)
signal valid low(1) repeat(4, high(4)) low(1)
```

`repeat(4, ...)` で4回分の波形が展開されます。手書きだと16スロット分の波形文字列を書く必要がありますが、WaveDSLなら意図が明確です。

### period / phase：DDRタイミング

DDRメモリのようにクロックエッジをずらした波形も、シグナル属性で表現できます。

```
signal CK   clock(8) period=2
signal CMD   x(1) data(2, "RAS") data(2, "CAS") x(3) phase=0.5
signal ADDR  x(1) data(2, "ROW") x(2) data(2, "COL") x(1) phase=0.5
signal DQS   z(4) low(1) high(1) low(1) high(1) low(1) z(1)
signal DQ    z(5) data(1, "D0") data(1, "D1") data(1, "D2") data(1, "D3") z(1)
```

### head / foot / config：図のメタデータ

タイトル、目盛り、水平スケールなどを設定できます。

```
head {
    text = "WaveDrom example"
    tick = 0
    every = 2
}
foot {
    text = "Figure 100"
    tock = 9
}
config {
    hscale = 2
}
signal clk clock(8)
signal bus x(2) data(4, "PAYLOAD") x(2)
```

### const：定数定義

マジックナンバーを排除し、パラメータを一元管理できます。

```
const CYCLES = 8
const HALF = 4
const SCALE = 2

config {
    hscale = $SCALE
}

signal clk clock($CYCLES)
signal en  low($HALF) high($HALF)
```

`$NAME` 形式の参照構文を採用しました。通常の識別子と曖昧にならず、grepで変数の使用箇所を簡単に探せます。

### include：ファイル分割

共通のシグナル定義を別ファイルに切り出して再利用できます。

**common_signals.wdsl**
```
signal clk  clock(8)
signal rstn low(2) high(6)
```

**main.wdsl**
```
include "common_signals.wdsl"

signal bus   low(2) data(4, "A B C D") low(2)
signal valid low(2) high(4) low(2)
```

クロックやリセットのような「毎回同じ」信号を1箇所で管理し、プロジェクト全体で共有できます。

## アーキテクチャ

WaveDSLコンパイラは、教科書的なコンパイラパイプラインを採用しています。

```
入力 (.wdsl)
  │
  ▼
Preprocessor ─── include展開（テキストレベル）
  │
  ▼
Lexer ────────── トークン列に分解
  │
  ▼
Parser ───────── AST（抽象構文木）を構築
  │
  ▼
Semantic ─────── 定数解決 → バリデーション
  │
  ▼
Codegen ──────── WaveDrom JSON生成
  │
  ▼
出力 (.json)
```

### パイプライン全体（38行）

```rust
pub fn compile(input: &str, file_path: Option<&Path>) -> Result<serde_json::Value, Vec<WaveDslError>> {
    // Preprocessor: include展開
    let source = if let Some(path) = file_path {
        let base_dir = path.parent().unwrap_or(Path::new("."));
        preprocessor::expand_includes(input, base_dir).map_err(|e| vec![e])?
    } else {
        input.to_string()
    };

    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| vec![e])?;

    let mut parser = parser::Parser::new(tokens);
    let mut program = parser.parse().map_err(|e| vec![e])?;

    semantic::resolve_and_validate(&mut program)?;

    Ok(codegen::generate(&program))
}
```

各フェーズが独立しており、テストしやすい構成です。

## 設計で悩んだポイント

### 定数参照：`$NAME` vs `NAME`

| 方式 | メリット | デメリット |
|------|---------|-----------|
| `$NAME` | 識別子と曖昧にならない、grep可能 | `$` が煩雑 |
| `NAME` | シンプル | `clk` は変数？シグナル名？ |
| `#define` | C/HDLユーザに馴染み深い | テキスト置換は脆い |

`$NAME` を採用しました。WaveDSLではシグナル名、関数名、キーワードが多く、接頭辞なしだと衝突が避けられません。

### include：テキスト展開 vs AST-level import

| 方式 | メリット | デメリット |
|------|---------|-----------|
| テキスト展開（C `#include` 方式） | シンプル、定数も自然に共有 | エラー行番号がズレる可能性 |
| ASTレベルのimport | 精密なエラー報告 | パーサ変更が大きい |
| モジュールシステム | 最も強力 | 現段階ではオーバーエンジニアリング |

C `#include` 方式を採用しました。循環参照検出（`HashSet<PathBuf>`）とネスト深度制限（16段）で安全性を確保しています。

### 定数解決：前方参照を許すか

禁止しました。宣言順に解決する方式により、実装がシンプルかつ可読性が向上します。

```
const A = 10
const B = $A        // OK: Aは定義済み

// const C = $D     // エラー: Dはまだ定義されていない
// const D = 20
```

## 技術スタック

| 要素 | 選定 |
|------|------|
| 言語 | Rust 2021 edition |
| JSON生成 | serde / serde_json |
| CLI | clap 4 |
| エラー型 | thiserror 2 |
| テスト | 標準 `#[test]` + insta（スナップショットテスト） |


## 使い方

### インストール

```bash
git clone https://github.com/MameMame777/waveDSL.git
cd waveDSL
cargo build --release
```

### 実行

```bash
# ファイル → 標準出力
wavedsl input.wdsl

# ファイル → ファイル
wavedsl input.wdsl -o output.json

# 標準入力 → 標準出力
echo 'signal clk clock(8)' | wavedsl
```

出力されたJSONはそのまま WaveDrom エディタ（https://wavedrom.com/editor.html）に貼り付けて波形を確認できます。

## 実用例：メモリリードトランザクション

`const` と `include` を組み合わせた実践的な例です。

**mem_params.wdsl**（パラメータファイル）
```
const TOTAL = 8
const LATENCY = 2
const BURST = 4
```

**mem_read.wdsl**
```
include "mem_params.wdsl"

config {
    hscale = 2
}
head {
    text = "Memory Read Transaction"
    tick  = 1
}

signal clk   clock($TOTAL)
signal cmd   low($LATENCY) data(1, "RD") low(5)
signal addr  low($LATENCY) data(1, "A0") low(5)
signal rdata low($LATENCY) low($LATENCY) data($BURST, "D0 D1 D2 D3")
signal valid low($LATENCY) low($LATENCY) high($BURST)

foot {
    text = "const + include demo"
}
```

パラメータを変更すれば、異なる設定の波形を一括で更新できます。

## AI活用の所感

このプロジェクトでは、設計から実装・テスト・ドキュメント生成まで AI（GitHub Copilot + Claude）と対話しながら進めました。

WaveDSLの文法自体が「AIが生成しやすい」ように設計されているため、**LLMへの指示で直接 .wdsl ファイルを生成させる**ワークフローが有効です。

```
「AXI4バスのライトトランザクションのタイミング図をWaveDSLで書いて」
```

このような自然言語の指示から、WaveDrom JSONを経由せず直接タイミング図の記述が得られます。

## まとめ

WaveDSLは、WaveDrom JSONの「書きづらさ」を解消するDSLです。

- **直感的な関数ベース記法**で波形を定義
- **定数・ファイル分割**でプロジェクトレベルの管理が可能
- **Rustコンパイラが変換**するので出力JSONは常に正しい
- **人間にもAIにも読み書きしやすい**設計

ハードウェア設計でタイミング図を書く方、WaveDromの記法に疲れた方はぜひ試してみてください。

GitHub: https://github.com/MameMame777/waveDSL

---

**タグ**: `Rust`, `WaveDrom`, `DSL`, `タイミング図`, `ハードウェア設計`

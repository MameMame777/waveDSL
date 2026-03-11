# WaveDSL 仕様書 v0.2

WaveDSL は WaveDrom JSON を生成するためのドメイン固有言語です。
人間とAIの両方が読み書きしやすいことを設計目標とします。

---

## 1. 基本ルール

- エンコーディング: UTF-8
- 変換方向: WaveDSL → WaveDrom JSON（一方向）
- 大文字小文字: キーワードは小文字
- コメント: `//` から行末まで
- 文字列リテラル: `"..."` ダブルクォートのみ
- 数値リテラル: 10進数 (`255`) または16進数 (`0xFF`)

---

## 2. 文法（EBNF）

```
program     ::= statement*
statement   ::= signal_decl
              | group_decl
signal_decl ::= "signal" name sequence
group_decl  ::= "group" string? "{" statement* "}"
sequence    ::= wave_expr+
wave_expr   ::= basic_call
              | repeat_call
basic_call  ::= name "(" arg_list? ")"
repeat_call ::= "repeat" "(" number "," sequence ")"
arg_list    ::= arg ("," arg)*
arg         ::= pos_arg
              | kw_arg
pos_arg     ::= number | string | enum_value
kw_arg      ::= name "=" (number | string | enum_value)
enum_value  ::= name         // クォートなしのキーワード
name        ::= [a-zA-Z_][a-zA-Z0-9_]*
string      ::= '"' [^"]* '"'
number      ::= [0-9]+ | "0x" [0-9a-fA-F]+
```

- `repeat` は特殊構文として扱い、第2引数に 1 個以上の `wave_expr` を取る
- キーワード引数は位置引数の後ろにのみ記述できる
- `n` を取る組み込み関数の `n` は 1 以上の整数とする
- `repeat(n, ...)` の `n` は 1 以上の整数とする
- 組み込み関数名（`clock`, `high`, `low`, `data`, `x`, `z`, `gap`, `repeat`）は信号名として使用できない（予約語）

---

## 3. 組み込み関数

### 3.1 波形関数

| 関数 | 引数 | 説明 |
|------|------|------|
| `clock(n, edge=rising)` | n: 周期数 | クロック信号 |
| `high(n)` | n: スロット数 | High レベル |
| `low(n)` | n: スロット数 | Low レベル |
| `data(n, label, color=1)` | n: スロット数, label: 表示文字列 | バス信号 |
| `x(n)` | n: スロット数 | 不定（undefined） |
| `z(n)` | n: スロット数 | ハイインピーダンス |
| `gap()` | なし | ギャップ（`\|` 記号） |
| `repeat(n, expr+)` | n: 繰り返し回数 | シーケンスの繰り返し |

### 3.2 列挙値

| パラメータ | 値 |
|------------|-----|
| `edge` | `rising`（デフォルト）, `falling` |
| `color` | `1`（デフォルト）〜 `5` |

### 3.3 関数ごとの意味

- `clock(n, edge=rising)`: `n` 個のクロック周期を生成する
- `high(n)`: High レベルを `n` スロット分生成する
- `low(n)`: Low レベルを `n` スロット分生成する
- `data(n, label, color=1)`: ラベル `label` を持つ 1 個のバス区間を生成し、その表示幅を `n` スロットとする
- `x(n)`: 不定値を `n` スロット分生成する
- `z(n)`: ハイインピーダンスを `n` スロット分生成する
- `gap()`: 1 スロットの区切りを生成する
- `repeat(n, expr+)`: `expr+` 全体を 1 つのシーケンスとして `n` 回繰り返す

---

## 4. グループ

- ネストは **2段まで**
- グループ名は **省略可能**
- 深さは **トップレベルの `group` を 1 段目** と数える
- 文法上は再帰的に記述できるが、**意味規則として深さ 2 を超えるグループはエラー** とする

```
group "名前" {
    signal ...
    group "サブグループ" {
        signal ...
    }
}
```

---

## 5. サンプル

### 5.1 シンプルな例

```
// シンプルなSPIタイミング
signal sclk  clock(8)
signal mosi  x(1) data(2, "CMD") data(4, "DATA") x(1)
signal miso  x(1) data(2, "ACK") data(4, "0xFF") x(1)
signal cs_n  high(1) low(6) high(1)
```

### 5.2 繰り返しの例

```
// バースト転送
signal clk   clock(18)
signal burst x(1) repeat(4, data(2, "0xAB") data(2, "0xCD")) x(1)
signal valid low(1) repeat(4, high(4)) low(1)
```

### 5.3 グループの例

```
// バスインターフェース
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

### 5.4 立ち下がりクロックの例

```
signal clk_p clock(8, edge=rising)
signal clk_n clock(8, edge=falling)
```

---

## 6. WaveDrom JSON へのマッピング

| WaveDSL | WaveDrom JSON |
|---------|---------------|
| `clock(n)` | `"P"` × n（※ クロックは毎スロットが独立した周期のため `"."` 継続を使わない） |
| `clock(n, edge=falling)` | `"N"` × n |
| `high(n)` | `"1"` + `"."` × (n - 1) |
| `low(n)` | `"0"` + `"."` × (n - 1) |
| `data(n, label, color=1)` | `token(color)` + `"."` × (n - 1)、かつ `data` 配列に `label` を 1 個追加 |
| `x(n)` | `"x"` + `"."` × (n - 1) |
| `z(n)` | `"z"` + `"."` × (n - 1) |
| `gap()` | wave 文字列に `"|"` を 1 個追加（JSON 文字列リテラルとしては `"\\|"`） |
| `repeat(n, expr+)` | `expr+` を展開した結果を n 回連結（`wave` 文字列と `data` 配列の両方を対象とし、正規化せずそのまま出力） |
| `group "name" {...}` | `["name", signal, ...]` |
| `group {...}` | `["", signal, ...]` |

`data()` の `color` は WaveDrom のバストークンに次のように対応付ける:

| `color` | 生成トークン | 色 |
|---------|--------------|------|
| `1` | `=` | デフォルト（白/ライト） |
| `2` | `2` | 青 |
| `3` | `3` | 黄 |
| `4` | `4` | 緑 |
| `5` | `5` | 赤 |

例:

```json
{ "name": "wdata", "wave": "x=...x", "data": ["0xDEAD"] }
```

上記は `x(1) data(4, "0xDEAD") x(1)` に対応する。

完全な入出力例:

WaveDSL:

```text
signal sclk  clock(8)
signal mosi  x(1) data(2, "CMD") data(4, "DATA") x(1)
signal cs_n  high(1) low(6) high(1)
```

WaveDrom JSON:

```json
{
    "signal": [
        { "name": "sclk", "wave": "PPPPPPPP" },
        { "name": "mosi", "wave": "x=.=...x", "data": ["CMD", "DATA"] },
        { "name": "cs_n", "wave": "10.....1" }
    ]
}
```

この例では、`data(2, "CMD")` は `"=."`、`data(4, "DATA")` は `"=..."` に変換され、`data` 配列には各バス区間のラベルが左から順に 1 回ずつ追加される。

---

## 7. スコープ外（v1未サポート）

- WaveDrom JSON の `node` / `edge` フィールド（矢印アノテーション）
- `config`（表示設定：周期、位相など）
- `head` / `foot`（タイトル・フッター）

---

## 8. 推奨実装

- WaveDSL コンパイラの推奨実装言語は **Rust** とする
- 理由: 字句解析、構文解析、AST、意味検証、JSON 生成という処理系の構成と相性が良く、型安全性・保守性・配布容易性のバランスがよい
- 想定成果物は **CLI ツール** とし、WaveDSL テキストを入力して WaveDrom JSON を出力する
- 実装は以下の段階に分離することを推奨する
    - lexer
    - parser
    - AST
    - semantic validation
    - WaveDrom JSON codegen
- JSON 出力は `serde` / `serde_json` を用いる構成を推奨する
- 仕様変更の検証を容易にするため、仕様書のサンプルはそのままスナップショットテスト化できる構造を保つ

Rust を推奨するが、これは **仕様上の必須要件ではなく推奨方針** である。

---

## 9. 設計方針メモ

- **AIフレンドリー**: キーワード引数により各パラメータの意味が自己説明的
- **エラー耐性**: 括弧ベースの文法でインデント依存を排除
- **拡張性**: キーワード引数の仕組みにより将来のパラメータ追加が容易
- **簡潔性**: 組み込み関数は最小限、`repeat` で表現力を補う

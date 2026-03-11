# WaveDSL

WaveDSL compiler — converts WaveDSL source into [WaveDrom](https://wavedrom.com/) JSON.

WaveDSL is a domain-specific language designed for both humans and AI to read and write timing diagrams easily.

## Build

```bash
cargo build --release
```

## Usage

```bash
# File to stdout
wavedsl input.wdsl

# File to file
wavedsl input.wdsl -o output.json

# Stdin to stdout
echo 'signal clk clock(8)' | wavedsl
```

Output is always pretty-printed JSON.

## DSL Quick Reference

### Signal

```
signal <name> <wave_expr>...
```

### Group (nest up to 2 levels)

```
group "Label" {
    signal ...
    group "Sub" {
        signal ...
    }
}
```

### Built-in Functions

| Function | Description | Example |
|----------|-------------|---------|
| `clock(n, edge=rising)` | Clock signal | `clock(8)`, `clock(4, edge=falling)` |
| `high(n)` | High level | `high(3)` |
| `low(n)` | Low level | `low(2)` |
| `data(n, label, color=1)` | Bus data | `data(4, "0xAB")`, `data(2, "CMD", color=3)` |
| `x(n)` | Undefined | `x(1)` |
| `z(n)` | Hi-Z | `z(2)` |
| `gap()` | Gap marker | `gap()` |
| `repeat(n, expr...)` | Repeat sequence | `repeat(4, high(1) low(1))` |

### Comments

```
// line comment
```

## Examples

### 1. SPI bus

```
signal sclk  clock(8)
signal mosi  x(1) data(2, "CMD") data(4, "DATA") x(1)
signal miso  x(1) data(2, "ACK") data(4, "0xFF") x(1)
signal cs_n  high(1) low(6) high(1)
```

```wavedrom
{
  "signal": [
    { "name": "sclk", "wave": "PPPPPPPP" },
    { "name": "mosi", "wave": "x=.=...x", "data": ["CMD", "DATA"] },
    { "name": "miso", "wave": "x=.=...x", "data": ["ACK", "0xFF"] },
    { "name": "cs_n", "wave": "10.....1" }
  ]
}
```

### 2. Burst transfer (`repeat`)

```
// バースト転送
signal clk   clock(18)
signal burst x(1) repeat(4, data(2, "0xAB") data(2, "0xCD")) x(1)
signal valid low(1) repeat(4, high(4)) low(1)
```

```wavedrom
{
  "signal": [
    { "name": "clk",   "wave": "PPPPPPPPPPPPPPPPPP" },
    { "name": "burst", "wave": "x=.=.=.=.=.=.=.=.x",
      "data": ["0xAB","0xCD","0xAB","0xCD","0xAB","0xCD","0xAB","0xCD"] },
    { "name": "valid", "wave": "01...1...1...1...0" }
  ]
}
```

### 3. Grouped bus signals

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

```wavedrom
{
  "signal": [
    ["AXI",
      ["Write Channel",
        { "name": "awvalid", "wave": "0.1.0..." },
        { "name": "awready", "wave": "0..10..." },
        { "name": "wdata",   "wave": "x.2.x...", "data": ["0xDEAD"] }
      ],
      ["Read Channel",
        { "name": "arvalid", "wave": "0...1.0." },
        { "name": "rdata",   "wave": "x...3.x.", "data": ["0xBEEF"] }
      ]
    ]
  ]
}
```

### 4. Differential clock

```
signal clk_p clock(8, edge=rising)
signal clk_n clock(8, edge=falling)
```

```wavedrom
{
  "signal": [
    { "name": "clk_p", "wave": "PPPPPPPP" },
    { "name": "clk_n", "wave": "NNNNNNNN" }
  ]
}
```

## Specification

See [wavedsl-spec.md](wavedsl-spec.md) for the full language specification (v0.2).

## License

MIT

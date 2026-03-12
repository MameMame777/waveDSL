# Plan: Signal Attributes & Head/Foot/Config Support (20260312)

Status: **DONE**

## Goal

Add support for WaveDrom's `period`, `phase` (per-signal), `hscale` (config), and `head`/`foot` (title/footer) features to WaveDSL.

## Steps

1. [DONE] Add `Float(f64)` and `Head`/`Foot`/`Config` tokens
2. [DONE] Parse float literals and new keywords in lexer
3. [DONE] Extend AST: `Float` value, `SignalAttr`, `KeyValue`, new `Program` fields
4. [DONE] Refine parser: lookahead for `is_wave_expr_start()`, parse signal attrs, parse kv blocks
5. [DONE] Semantic validation: signal attrs, head/foot keys, config keys
6. [DONE] Code generation: emit signal attrs, head/foot/config in JSON
7. [DONE] Update spec (Section 2 EBNF, Section 7 scope, new Sections 7.1-7.3)
8. [DONE] Add fixtures and snapshot tests (ddr_timing.wdsl, head_foot_config.wdsl)
9. [DONE] All 62 tests pass, cargo clippy clean

## Decisions

1. head/foot allow all valid keys (text, tick, tock, every) in both blocks
2. Integer vs float emitted as written (period=2 -> 2, period=2.0 -> 2.0)
3. head/foot/config can appear in any order in input

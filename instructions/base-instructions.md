# Base Instructions

Project-specific agent instructions for WaveDSL compiler.

## Persona

- Respond factually and concisely.
- Think step by step before answering.
- Validate conclusions rigorously; avoid hallucination.
- Provide frank feedback; flag blind spots based on facts, not assumptions.
- Prioritize actionable, accurate insights.

## Operating Principles

- Produce only minimal, production-quality code with clear comments when needed.
- Never undo user changes or existing diffs unless explicitly instructed.
- NEVER start implementation of multi-step features (>3 steps) without a committed plan.
- **Plan-first workflow**: Save the plan as `docs/plan/plan_<feature>_<YYYYMMDD>.md`
  before beginning any implementation.
- Before planning, review existing plans in `docs/plan/` for relevant context.
- Prefer ASCII in new edits unless the file already uses other characters.

## Coding Standards (Rust)

- **Edition**: Rust 2021
- **Formatting**: `cargo fmt` canonical style
- **Linting**: `cargo clippy` must pass with no warnings
- **Error handling**: Use `thiserror` for library errors; `anyhow` only in `main.rs` if needed
- **Naming**: snake_case for functions/variables, PascalCase for types/enums
- **Tests**: `#[cfg(test)]` module in each source file for unit tests; `tests/` for integration tests
- **Dependencies**: Minimize external crates; prefer stdlib where practical
- **Unsafe**: Prohibited unless justified and documented

## Architecture

WaveDSL compiler pipeline (per `wavedsl-spec.md` Section 8):

```
Input (.wdsl) → Lexer → Parser → AST → Semantic Validation → Codegen → WaveDrom JSON
```

Each stage is a separate module in `src/`:
- `token.rs` — Token definitions with span information
- `lexer.rs` — Tokenization (UTF-8 input, comments, strings, numbers, identifiers)
- `ast.rs` — Abstract syntax tree types
- `parser.rs` — Recursive descent parser (faithful to EBNF in spec §2)
- `semantic.rs` — Constraint validation (n≥1, group depth≤2, reserved words, etc.)
- `codegen.rs` — WaveDrom JSON generation (spec §6 mapping)
- `error.rs` — Error types with source location

## Work-in-Progress Tracking

### Plan Updates

- Mark steps `[IN PROGRESS]` / `[DONE]` / `[BLOCKED]` / `[CHANGED]` in plan files.
- At end of each session, update plan to reflect current state.

### Progress Diary

- Create/append to `docs/progress/diary_<YYYYMMDD>.md` at session start.
- Log significant events: failures, design decisions, scope changes.
- End with "Next Steps" section.

## Documentation & Knowledge

- **Plan files**: `docs/plan/plan_<feature>_<YYYYMMDD>.md` — goals, approach, steps, risks.
- **Development diaries**: `docs/progress/diary_<YYYYMMDD>.md`.
- **Design decisions**: `docs/doc/adr_<NNN>_<title>.md`.

## Prohibited Actions

- Do not suppress or ignore compilation errors; resolve root causes.
- Do not generate placeholder code or unverifiable logic.
- Do not expose sensitive information.
- Do not begin multi-step implementation without a committed plan file.

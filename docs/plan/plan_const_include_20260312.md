# Plan: const and include features

Date: 2026-03-12
Status: IN PROGRESS

## Goal

Add two features to WaveDSL v0.3:
1. **const**: named constants (`const NAME = value`) with `$NAME` references
2. **include**: file inclusion (`include "path"`) with circular reference detection

## Design Decisions

- Keyword: `const` (not `let` or `def`)
- Reference sigil: `$NAME` (DollarIdent token)
- Scope: function arguments and attribute values (anywhere a `value` is accepted)
- Include syntax: `include "path"` at top level
- Include path resolution: relative to the input file's directory
- Forward references: prohibited (const must be defined before use)
- Include expansion: preprocessor stage before lexing

## Phases

### Phase 1: Token & Lexer [NOT STARTED]
- Add `Const`, `Include`, `DollarIdent(String)` to Token enum
- Lexer: recognize `$` + identifier -> DollarIdent
- Lexer: recognize `const` and `include` keywords

### Phase 2: AST extensions [NOT STARTED]
- Add `Statement::ConstDecl { name, value, span }`
- Add `Value::VarRef(String)`

### Phase 3: Preprocessor (include) [NOT STARTED]
- New module `src/preprocessor.rs`
- Text-level include expansion before lexing
- Circular reference detection via path set
- `compile()` gains optional file path parameter
- Error type: `WaveDslError::Preprocessor`

### Phase 4: Parser additions [NOT STARTED]
- Parse `const NAME = value` -> ConstDecl
- Parse `DollarIdent` in `parse_value()` -> Value::VarRef

### Phase 5: Semantic const resolution [NOT STARTED]
- `resolve_constants()`: walk AST, build symbol table
- Replace VarRef with resolved values
- Errors: undefined ref, forward ref, reserved const name
- Run before existing validate()

### Phase 6: CLI & lib.rs updates [NOT STARTED]
- `compile()` accepts optional file path for include resolution
- main.rs passes input file path

### Phase 7: Tests & fixtures [NOT STARTED]
- Fixture files for const, include, combined usage
- Snapshot tests
- Unit tests for error cases (undefined, forward ref, circular include)

### Phase 8: Spec & docs update [NOT STARTED]
- Update wavedsl-spec.md with const/include sections
- Update README

## Risks

- Include expansion at text level may cause confusing line numbers in errors
- Deeply nested includes could be expensive (mitigate with depth limit)

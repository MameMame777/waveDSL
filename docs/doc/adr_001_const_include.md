# ADR-001: Const/Include Feature Design

**Date**: 2026-03-12  
**Status**: Accepted  
**Context**: WaveDSL v0.3.0

## Decision

Add `const` (constant definitions) and `include` (file inclusion) to WaveDSL.

## Context

Users need to:
1. Avoid magic numbers scattered across signal definitions
2. Reuse common signal sets (e.g., clock + reset) across multiple .wdsl files
3. Parameterize timing diagrams for different configurations

## Alternatives Considered

### Constants

| Option | Pros | Cons | Decision |
|--------|------|------|----------|
| `const NAME = value` with `$NAME` | Explicit, no ambiguity with identifiers | Extra `$` syntax | **Chosen** |
| `let NAME = value` with `NAME` | Familiar syntax | Ambiguous: is `clk` a variable or signal name? | Rejected |
| `#define NAME value` | C-like, familiar to HDL users | Requires text-level preprocessing, fragile | Rejected |

### Include

| Option | Pros | Cons | Decision |
|--------|------|------|----------|
| Pre-lexer text expansion | Simple, C-like, constants from included files available | Line numbers in errors may be confusing | **Chosen** |
| AST-level import | Better error reporting, selective imports | Complex parser changes, scope management | Rejected |
| Module system with namespaces | Most powerful | Over-engineering for current DSL scope | Rejected |

## Key Design Points

### Const

- **Keyword**: `const` (not `let`, `var`, `define`)
- **Reference syntax**: `$NAME` (dollar-sign sigil)
  - Unambiguous: `$` never appears in identifiers or WaveDrom syntax
  - Grep-friendly: easy to find all variable usages
- **Scope**: Function arguments and attribute values only
  - Cannot be used as signal names or group labels
- **Resolution**: Semantic phase, after parsing, before validation
  - Two-pass: collect declarations, then substitute references
  - Forward references prohibited (simplifies implementation and readability)
- **Reserved names**: All keywords (`signal`, `group`, `clock`, etc.) cannot be const names

### Include

- **Syntax**: `include "relative/path.wdsl"` on its own line
- **Path resolution**: Relative to the including file's directory
- **Safety measures**:
  - Circular reference detection via `HashSet<PathBuf>`
  - Maximum nesting depth of 16
  - Clear error messages with file path and line number
- **API impact**: `compile()` gains `file_path: Option<&Path>` parameter
  - `None` disables include support (stdin, embedded usage)

## Consequences

### Positive

- Parameters can be changed in one place
- Common signals (clock, reset) can be defined once
- Enables project-level .wdsl organization

### Negative

- `compile()` API is no longer a pure string-to-JSON function (needs file path)
- Include errors depend on filesystem state
- No conditional compilation or include guards (not needed for current scope)

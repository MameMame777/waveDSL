use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::WaveDslError;

/// Maximum include nesting depth to prevent stack overflow.
const MAX_INCLUDE_DEPTH: usize = 16;

/// Expand `include "path"` directives in the source text.
///
/// Each `include "path"` must appear on its own line (ignoring leading whitespace
/// and trailing comments). The referenced file content is inserted in place.
/// Circular references are detected and reported as errors.
pub fn expand_includes(
    source: &str,
    base_dir: &Path,
) -> Result<String, WaveDslError> {
    let mut seen = HashSet::new();
    let canonical = base_dir.to_path_buf();
    expand_recursive(source, &canonical, &mut seen, 0)
}

fn expand_recursive(
    source: &str,
    base_dir: &Path,
    seen: &mut HashSet<PathBuf>,
    depth: usize,
) -> Result<String, WaveDslError> {
    if depth > MAX_INCLUDE_DEPTH {
        return Err(WaveDslError::Preprocessor {
            message: format!("include nesting depth exceeds maximum of {MAX_INCLUDE_DEPTH}"),
        });
    }

    let mut output = String::with_capacity(source.len());
    for (line_idx, line) in source.lines().enumerate() {
        if let Some(path_str) = parse_include_line(line) {
            let include_path = base_dir.join(path_str);
            let canonical = normalize_path(&include_path);

            if !seen.insert(canonical.clone()) {
                return Err(WaveDslError::Preprocessor {
                    message: format!(
                        "circular include detected: '{}' (line {})",
                        path_str,
                        line_idx + 1
                    ),
                });
            }

            let content =
                std::fs::read_to_string(&include_path).map_err(|e| WaveDslError::Preprocessor {
                    message: format!("cannot read '{}': {}", include_path.display(), e),
                })?;

            let child_dir = include_path
                .parent()
                .unwrap_or(base_dir)
                .to_path_buf();

            let expanded = expand_recursive(&content, &child_dir, seen, depth + 1)?;
            output.push_str(&expanded);
            if !expanded.ends_with('\n') {
                output.push('\n');
            }

            seen.remove(&canonical);
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    Ok(output)
}

/// Try to parse a line as `include "path"`. Returns the path if matched.
fn parse_include_line(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    // Strip trailing comment
    let trimmed = if let Some(idx) = trimmed.find("//") {
        trimmed[..idx].trim()
    } else {
        trimmed
    };
    let rest = trimmed.strip_prefix("include")?;
    // Must be followed by whitespace (not part of a longer identifier)
    if rest.is_empty() || !rest.starts_with(|c: char| c.is_ascii_whitespace()) {
        return None;
    }
    let rest = rest.trim();
    // Must be "path"
    let rest = rest.strip_prefix('"')?;
    let rest = rest.strip_suffix('"')?;
    if rest.is_empty() {
        return None;
    }
    Some(rest)
}

/// Best-effort path normalization without requiring the file to exist.
fn normalize_path(path: &Path) -> PathBuf {
    match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_include_line() {
        assert_eq!(parse_include_line(r#"include "foo.wdsl""#), Some("foo.wdsl"));
        assert_eq!(
            parse_include_line(r#"  include "sub/bar.wdsl"  "#),
            Some("sub/bar.wdsl")
        );
        assert_eq!(
            parse_include_line(r#"include "foo.wdsl" // comment"#),
            Some("foo.wdsl")
        );
        assert_eq!(parse_include_line("signal clk clock(8)"), None);
        assert_eq!(parse_include_line(r#"include """#), None);
        assert_eq!(parse_include_line("includes"), None);
    }
}

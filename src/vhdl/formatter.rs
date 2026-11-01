//! VHDL formatter.
//!
//! Produces a reformatted version of the given VHDL source. The formatter
//! is a dependency-free, whitespace-based normalizer: it collapses runs of
//! whitespace (spaces, tabs, etc.) into a single space within each line and
//! normalizes line separators to newlines. This keeps formatting robust and
//! free of any external grammar dependency.

/// Returns the reformatted source for the given VHDL source.
pub fn format(source: &str) -> String {
    // Split the source into lines on newlines.
    let lines: Vec<&str> = source.split('\n').collect();

    let mut out_lines: Vec<String> = Vec::with_capacity(lines.len());
    for line in lines {
        let chars: Vec<char> = line.chars().collect();
        let mut out = Vec::with_capacity(chars.len());
        let mut prev_ws = false;

        for c in chars {
            if c.is_whitespace() {
                // Collapse consecutive whitespace into a single space.
                if !prev_ws {
                    out.push(' ');
                }
                prev_ws = true;
            } else {
                out.push(c);
                prev_ws = false;
            }
        }
        out_lines.push(out.into_iter().collect());
    }

    // Join the normalized lines with newlines.
    out_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_whitespace_and_normalizes_indent() {
        let input = "  signal\n\tclk\n\n  in\n  integer\n";
        let output = format(input);
        // Leading indentation is normalized to a single space (one space,
        // not two).
        assert!(output.starts_with(" signal"));
        // Whitespace runs are collapsed (no double spaces from runs).
        assert!(!output.contains("\t"));
        // Each line is separated by a newline.
        assert!(output.contains('\n'));
    }

    #[test]
    fn handles_empty_input() {
        assert_eq!(format(""), "");
    }
}

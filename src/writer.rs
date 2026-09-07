use crate::value::{FishTable, FishValue};

const INDENT: &str = "    ";

/// Check if an unquoted string is safe to emit without quotes.
/// Must match identifier-like but we allow more for values: alnum, _, -, ., /, and not containing spaces or special delimiters
fn needs_quotes(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    // Keywords that must not be emitted bare if they are string values? But we already distinguish types.
    // For string values, if string equals "true"/"false"/"null" we must quote to avoid parsing as bool/null
    if s == "true" || s == "false" || s == "null" || s == "nil" || s == "~" {
        return true;
    }
    // If string looks like a number, quote it
    if s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok() {
        return true;
    }
    // Hex like 0xFF
    if s.starts_with("0x") || s.starts_with("0X") {
        if i64::from_str_radix(&s[2..], 16).is_ok() {
            return true;
        }
    }
    // Check characters: allowed bare chars are a-zA-Z0-9 _ - . / :? But colon would break parsing, so disallow
    // For safety, only allow [a-zA-Z0-9_\-./] and maybe + ?
    // If any char not in allowed set, need quotes
    for ch in s.chars() {
        if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' && ch != '.' && ch != '/' {
            return true;
        }
        // also slash is okay for paths, but space, brackets etc not
    }
    // Must start with alpha or _ or /? Values like dark start with alpha; path /usr is okay but starts with /
    // Allow leading slash or alpha
    let first = s.chars().next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '_' || first == '/' || first == '.') {
        // e.g. "123abc" would have been parsed as string need quotes, but we already check number case
        // So if starts with digit, needs quotes (since we quoted numbers)
        if first.is_ascii_digit() {
            return true;
        }
    }
    // Also if contains // or # or /* it would be parsed as comment start, so quote
    if s.contains("//") || s.contains("/*") || s.contains('#') {
        return true;
    }
    false
}

fn escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn format_value(value: &FishValue) -> String {
    match value {
        FishValue::Null => "null".to_string(),
        FishValue::Bool(b) => b.to_string(),
        FishValue::Integer(i) => i.to_string(),
        FishValue::Float(f) => {
            // Ensure float has decimal point if needed
            let s = f.to_string();
            if s.contains('.') || s.contains('e') || s.contains('E') {
                s
            } else {
                format!("{}.0", s)
            }
        }
        FishValue::String(s) => {
            if needs_quotes(s) {
                escape_string(s)
            } else {
                s.clone()
            }
        }
        FishValue::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value).collect();
            format!("[{}]", items.join(", "))
        }
        FishValue::Table(_) => unreachable!("table handled separately"),
    }
}

fn write_table(table: &FishTable, indent_level: usize, out: &mut String) {
    let indent = INDENT.repeat(indent_level);
    for (key, value) in table {
        match value {
            FishValue::Table(inner) => {
                out.push_str(&indent);
                out.push_str(key);
                out.push_str(" {\n");
                write_table(inner, indent_level + 1, out);
                out.push_str(&indent);
                out.push_str("}\n");
                // blank line between top-level sections for readability
                if indent_level == 0 {
                    out.push('\n');
                }
            }
            _ => {
                out.push_str(&indent);
                out.push_str(key);
                out.push_str(": ");
                out.push_str(&format_value(value));
                out.push('\n');
            }
        }
    }
}

/// Serialize a table to a fico string (pretty-printed).
pub fn to_string(table: &FishTable) -> String {
    let mut out = String::new();
    write_table(table, 0, &mut out);
    // Trim trailing blank lines but keep one newline at end
    let trimmed = out.trim_end().to_string();
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed + "\n"
    }
}

/// Serialize with custom indentation (number of spaces per level).
pub fn to_string_with_indent(table: &FishTable, indent_spaces: usize) -> String {
    let indent_str = " ".repeat(indent_spaces);
    fn write_with(table: &FishTable, level: usize, indent_str: &str, out: &mut String) {
        let indent = indent_str.repeat(level);
        for (key, value) in table {
            match value {
                FishValue::Table(inner) => {
                    out.push_str(&indent);
                    out.push_str(key);
                    out.push_str(" {\n");
                    write_with(inner, level + 1, indent_str, out);
                    out.push_str(&indent);
                    out.push_str("}\n");
                    if level == 0 {
                        out.push('\n');
                    }
                }
                _ => {
                    out.push_str(&indent);
                    out.push_str(key);
                    out.push_str(": ");
                    // reuse format_value logic but need to avoid borrowing issues
                    let formatted = match value {
                        FishValue::Null => "null".to_string(),
                        FishValue::Bool(b) => b.to_string(),
                        FishValue::Integer(i) => i.to_string(),
                        FishValue::Float(f) => {
                            let s = f.to_string();
                            if s.contains('.') || s.contains('e') || s.contains('E') {
                                s
                            } else {
                                format!("{}.0", s)
                            }
                        }
                        FishValue::String(s) => {
                            if needs_quotes(s) {
                                let mut o = String::with_capacity(s.len() + 2);
                                o.push('"');
                                for ch in s.chars() {
                                    match ch {
                                        '"' => o.push_str("\\\""),
                                        '\\' => o.push_str("\\\\"),
                                        '\n' => o.push_str("\\n"),
                                        '\r' => o.push_str("\\r"),
                                        '\t' => o.push_str("\\t"),
                                        _ => o.push(ch),
                                    }
                                }
                                o.push('"');
                                o
                            } else {
                                s.clone()
                            }
                        }
                        FishValue::Array(arr) => {
                            let items: Vec<String> = arr
                                .iter()
                                .map(|v| match v {
                                    FishValue::Null => "null".to_string(),
                                    FishValue::Bool(b) => b.to_string(),
                                    FishValue::Integer(i) => i.to_string(),
                                    FishValue::Float(f) => {
                                        let s = f.to_string();
                                        if s.contains('.') || s.contains('e') {
                                            s
                                        } else {
                                            format!("{}.0", s)
                                        }
                                    }
                                    FishValue::String(s) => {
                                        if needs_quotes(s) {
                                            let mut o = String::with_capacity(s.len() + 2);
                                            o.push('"');
                                            for ch in s.chars() {
                                                match ch {
                                                    '"' => o.push_str("\\\""),
                                                    '\\' => o.push_str("\\\\"),
                                                    '\n' => o.push_str("\\n"),
                                                    '\r' => o.push_str("\\r"),
                                                    '\t' => o.push_str("\\t"),
                                                    _ => o.push(ch),
                                                }
                                            }
                                            o.push('"');
                                            o
                                        } else {
                                            s.clone()
                                        }
                                    }
                                    FishValue::Array(_) => format!("{:?}", v), // nested not expected
                                    FishValue::Table(_) => unreachable!(),
                                })
                                .collect();
                            format!("[{}]", items.join(", "))
                        }
                        FishValue::Table(_) => unreachable!(),
                    };
                    out.push_str(&formatted);
                    out.push('\n');
                }
            }
        }
    }
    let mut out = String::new();
    write_with(table, 0, &indent_str, &mut out);
    let trimmed = out.trim_end().to_string();
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed + "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_str;

    #[test]
    fn test_roundtrip() {
        let input = r#"system {
    theme: dark
    accent: blue
}

appearance {
    animations: true
    transparency: 0.85
    icons {
        style: tontoo
        size: 48
    }
}
"#;
        let table = parse_str(input).unwrap();
        let out = to_string(&table);
        // parse again
        let table2 = parse_str(&out).unwrap();
        assert_eq!(table, table2);
    }

    #[test]
    fn test_quoted_escaping() {
        let mut table = FishTable::new();
        table.insert(
            "greeting".to_string(),
            FishValue::String("Hello World".to_string()),
        );
        table.insert(
            "path".to_string(),
            FishValue::String("/usr/local/bin".to_string()),
        );
        let s = to_string(&table);
        assert!(s.contains("\"Hello World\""));
        // path with slashes should not be quoted? Our needs_quotes allows slash, so not quoted
        // But it's okay either way
        let parsed = parse_str(&s).unwrap();
        assert_eq!(parsed.get("greeting").unwrap().as_str(), Some("Hello World"));
    }
}

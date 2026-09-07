use crate::error::{FishError, Result};
use crate::value::{FishTable, FishValue};

// ---------------------------------------------------------------- lexer helpers

#[derive(Debug, Clone)]
struct Lexer<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.bytes.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let ch = self.peek()?;
        self.pos += 1;
        if ch == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn remaining(&self) -> &str {
        &self.input[self.pos..]
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<()> {
        loop {
            // whitespace
            while let Some(c) = self.peek() {
                if c == b' ' || c == b'\t' || c == b'\r' || c == b'\n' {
                    self.advance();
                } else {
                    break;
                }
            }
            if self.is_eof() {
                break;
            }
            // // comment
            if self.peek() == Some(b'/') && self.peek_next() == Some(b'/') {
                // consume until newline
                while let Some(c) = self.peek() {
                    self.advance();
                    if c == b'\n' {
                        break;
                    }
                }
                continue;
            }
            // # comment (only when # is start of line or preceded by whitespace and not inside value)
            // We treat # as comment when it appears where a statement could start,
            // i.e. after whitespace/newline. Since we are skipping whitespace,
            // any # here starts a comment.
            if self.peek() == Some(b'#') {
                while let Some(c) = self.peek() {
                    self.advance();
                    if c == b'\n' {
                        break;
                    }
                }
                continue;
            }
            // /* block comment */
            if self.peek() == Some(b'/') && self.peek_next() == Some(b'*') {
                self.advance(); // /
                self.advance(); // *
                let mut closed = false;
                while let Some(c) = self.peek() {
                    if c == b'*' && self.peek_next() == Some(b'/') {
                        self.advance();
                        self.advance();
                        closed = true;
                        break;
                    } else {
                        self.advance();
                    }
                }
                if !closed {
                    return Err(FishError::parse(
                        self.line,
                        self.col,
                        "unterminated block comment",
                    ));
                }
                continue;
            }
            break;
        }
        Ok(())
    }

    fn error(&self, msg: impl Into<String>) -> FishError {
        FishError::parse(self.line, self.col, msg)
    }

    fn expect_char(&mut self, expected: u8) -> Result<()> {
        self.skip_whitespace_and_comments()?;
        match self.peek() {
            Some(c) if c == expected => {
                self.advance();
                Ok(())
            }
            Some(c) => Err(self.error(format!(
                "expected '{}' but found '{}'",
                expected as char, c as char
            ))),
            None => Err(self.error(format!("expected '{}' but found EOF", expected as char))),
        }
    }

    // identifier: [a-zA-Z_][a-zA-Z0-9_\-\.]*
    // allow dot inside? No, dot is path separator, so not inside identifier for fico keys.
    // Keys are single identifiers; dots are used only in API path helpers.
    fn parse_identifier(&mut self) -> Result<String> {
        self.skip_whitespace_and_comments()?;
        let start = self.pos;
        let start_line = self.line;
        let start_col = self.col;
        match self.peek() {
            Some(c) if (c as char).is_ascii_alphabetic() || c == b'_' => {
                self.advance();
            }
            Some(c) => {
                return Err(FishError::parse(
                    start_line,
                    start_col,
                    format!("expected identifier, found '{}'", c as char),
                ))
            }
            None => {
                return Err(FishError::parse(
                    start_line,
                    start_col,
                    "expected identifier, found EOF",
                ))
            }
        }
        while let Some(c) = self.peek() {
            if (c as char).is_ascii_alphanumeric() || c == b'_' || c == b'-' {
                self.advance();
            } else {
                break;
            }
        }
        let s = &self.input[start..self.pos];
        Ok(s.to_string())
    }

    fn parse_quoted_string(&mut self) -> Result<String> {
        // assumes current char is " or '
        let quote = self.peek().unwrap();
        self.advance(); // consume opening
        let mut out = String::new();
        let mut escaped = false;
        loop {
            match self.peek() {
                None => {
                    return Err(self.error("unterminated string literal"));
                }
                Some(c) => {
                    self.advance();
                    if escaped {
                        match c {
                            b'n' => out.push('\n'),
                            b't' => out.push('\t'),
                            b'r' => out.push('\r'),
                            b'\\' => out.push('\\'),
                            b'"' => out.push('"'),
                            b'\'' => out.push('\''),
                            _ => {
                                out.push('\\');
                                out.push(c as char);
                            }
                        }
                        escaped = false;
                    } else if c == b'\\' {
                        escaped = true;
                    } else if c == quote {
                        break;
                    } else {
                        out.push(c as char);
                    }
                }
            }
        }
        Ok(out)
    }

    fn parse_value(&mut self) -> Result<FishValue> {
        self.skip_whitespace_and_comments()?;
        let ch = self.peek().ok_or_else(|| self.error("expected value, found EOF"))?;

        // String literal
        if ch == b'"' || ch == b'\'' {
            let s = self.parse_quoted_string()?;
            return Ok(FishValue::String(s));
        }

        // Array
        if ch == b'[' {
            return self.parse_array();
        }

        // Number, bool, null, or unquoted string
        // Read raw token until delimiter
        let start = self.pos;
        // special handling for negative numbers
        let mut has_minus = false;
        if ch == b'-' {
            // peek next should be digit
            if let Some(n) = self.peek_next() {
                if n.is_ascii_digit() {
                    has_minus = true;
                    self.advance();
                } else {
                    // standalone '-'? treat as string
                }
            }
        }

        // If we consumed '-', check again, else not
        // Now collect token
        // We need to collect until whitespace, comma, bracket, brace, colon, comment start, newline
        // But we already possibly advanced for '-'
        // So start stays where it was for negative numbers

        // If we haven't advanced for minus case where ch == '-', and has_minus false, then we are at '-'
        // Let's collect token properly: if has_minus we are one past '-', else at start
        // We'll just continue collecting alphanum + symbols for unquoted string

        // If we started with '-' and it's a number, we continue number parsing
        // Otherwise we need to collect identifier/number string

        // Reset if we advanced incorrectly for non-number '-'
        if !has_minus && ch == b'-' {
            // That '-' is actually start of token, but we didn't treat as minus number.
            // So we are still at '-' (pos == start), good.
        } else if has_minus {
            // pos is start+1, continue
        }

        // Now collect until delimiter
        // Delimiters: whitespace, ',', ']', '}', '{', ':', '#', '/', '"', '\'', '\n'
        // For value token, we stop before those.
        // We'll peek and collect

        // If we are at a number start (digit) we can parse number greedily including '.' 'e' 'E' '+' '-'
        // Simpler: collect raw token then try to interpret

        // Collect
        let token_start = if has_minus { start } else { self.pos };
        // If has_minus, pos is start+1, but we want token_start=start
        // Otherwise pos == start
        // But we haven't yet collected the rest; continue collecting

        // If has_minus, we have consumed '-', so the token start is `start`, pos is start+1
        // else we are at start
        // Now collect remaining chars for token

        // For has_minus case, we already at pos=start+1, need to keep collecting digits etc
        // For non-minus case, start==pos, we need to collect

        // Determine if this looks like a number start: digit or '-' digit
        // We do that after collecting

        // Collect until delimiter
        while let Some(c) = self.peek() {
            // delimiters
            if c == b' ' || c == b'\t' || c == b'\r' || c == b'\n' || c == b',' || c == b']' || c == b'}' || c == b'{' || c == b':' || c == b'=' || c == b'#' || c == b'"' || c == b'\'' {
                break;
            }
            // // comment start: '/' followed by '/' or '*'
            if c == b'/' && (self.peek_next() == Some(b'/') || self.peek_next() == Some(b'*')) {
                break;
            }
            // also ';' as statement separator
            if c == b';' {
                break;
            }
            self.advance();
        }

        let raw_end = self.pos;
        let raw = &self.input[token_start..raw_end];
        if raw.is_empty() {
            return Err(self.error("expected value"));
        }

        // Now interpret raw
        // Check keywords (case-sensitive lower)
        match raw {
            "true" | "True" | "TRUE" | "yes" | "YES" | "Yes" | "on" | "ON" => {
                // but "yes" as generic string could be ambiguous; we treat yes/on as true for convenience
                // Keep case-sensitive? For now handle lower variants
                // We'll map exactly true/false and yes/no/on/off
                if raw == "true" || raw == "True" || raw == "TRUE" {
                    return Ok(FishValue::Bool(true));
                }
                if raw == "yes" || raw == "YES" || raw == "Yes" || raw == "on" || raw == "ON" {
                    // treat as Bool too, but only if user expects? Could cause surprise for string "yes"
                    // Let's only map "true"/"false" strictly and keep yes/no as strings? But spec says animations: true -> true is fine.
                    // We'll keep yes/no handling but document
                    // For simplicity map yes/on to true
                    return Ok(FishValue::Bool(true));
                }
                // fallback string
            }
            "false" | "False" | "FALSE" | "no" | "NO" | "No" | "off" | "OFF" => {
                if raw == "false" || raw == "False" || raw == "FALSE" {
                    return Ok(FishValue::Bool(false));
                }
                if raw == "no" || raw == "NO" || raw == "No" || raw == "off" || raw == "OFF" {
                    return Ok(FishValue::Bool(false));
                }
            }
            "null" | "Null" | "NULL" | "nil" | "Nil" | "NIL" | "~" => {
                return Ok(FishValue::Null);
            }
            _ => {}
        }
        // More precise keyword check: only true/false/null strictly
        if raw == "true" {
            return Ok(FishValue::Bool(true));
        }
        if raw == "false" {
            return Ok(FishValue::Bool(false));
        }
        if raw == "null" || raw == "nil" || raw == "~" {
            return Ok(FishValue::Null);
        }

        // Try integer
        // Allow hex? 0x...
        if raw.starts_with("0x") || raw.starts_with("0X") {
            if let Ok(v) = i64::from_str_radix(&raw[2..], 16) {
                return Ok(FishValue::Integer(v));
            }
        }
        // Try integer parsing
        if let Ok(i) = raw.parse::<i64>() {
            // Check if it contains '.' or 'e'/'E' then not integer
            if !raw.contains('.') && !raw.contains('e') && !raw.contains('E') {
                return Ok(FishValue::Integer(i));
            }
        }
        // Try float
        if let Ok(f) = raw.parse::<f64>() {
            // Must contain '.' or 'e'/'E' to be considered float, otherwise integer would have matched
            // But we allow "0.85" etc
            // Also handle "1.0" etc
            // If parsing succeeded and raw contains '.' or e/E, treat as float, otherwise if raw is integer-like we already returned
            // If raw is like "48" we already returned integer, so remaining float cases are genuine floats
            return Ok(FishValue::Float(f));
        }

        // Otherwise unquoted string
        Ok(FishValue::String(raw.to_string()))
    }

    fn parse_array(&mut self) -> Result<FishValue> {
        self.skip_whitespace_and_comments()?;
        self.expect_char(b'[')?;
        let mut items = Vec::new();
        loop {
            self.skip_whitespace_and_comments()?;
            if self.peek() == Some(b']') {
                self.advance();
                break;
            }
            // parse value
            let v = self.parse_value()?;
            items.push(v);
            self.skip_whitespace_and_comments()?;
            match self.peek() {
                Some(b',') => {
                    self.advance();
                    // allow trailing comma
                    self.skip_whitespace_and_comments()?;
                    if self.peek() == Some(b']') {
                        self.advance();
                        break;
                    }
                }
                Some(b']') => {
                    self.advance();
                    break;
                }
                Some(c) => {
                    return Err(self.error(format!(
                        "expected ',' or ']' in array, found '{}'",
                        c as char
                    )))
                }
                None => return Err(self.error("unterminated array, expected ']'")),
            }
        }
        Ok(FishValue::Array(items))
    }
}

// ---------------------------------------------------------------- parser

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    pub fn parse(mut self) -> Result<FishTable> {
        let mut table = FishTable::new();
        self.parse_table(&mut table, false)?;
        self.lexer.skip_whitespace_and_comments()?;
        if !self.lexer.is_eof() {
            return Err(self.lexer.error(format!(
                "unexpected token '{}' after root table",
                self.lexer.remaining().chars().next().unwrap_or('?')
            )));
        }
        Ok(table)
    }

    fn parse_table(&mut self, table: &mut FishTable, is_nested: bool) -> Result<()> {
        loop {
            self.lexer.skip_whitespace_and_comments()?;
            let ch = self.lexer.peek();
            match ch {
                None => {
                    if is_nested {
                        return Err(self.lexer.error("unterminated block, expected '}'"));
                    } else {
                        break;
                    }
                }
                Some(b'}') => {
                    if is_nested {
                        self.lexer.advance();
                        break;
                    } else {
                        return Err(self.lexer.error("unexpected '}' at top level"));
                    }
                }
                Some(b';') => {
                    self.lexer.advance();
                    continue;
                }
                Some(_) => {
                    // Expect identifier for key or section name
                    let key = self.lexer.parse_identifier()?;
                    self.lexer.skip_whitespace_and_comments()?;
                    let next = self.lexer.peek();

                    match next {
                        Some(b'{') => {
                            // Section: name { ... }
                            self.lexer.advance(); // consume {
                            let mut inner = FishTable::new();
                            self.parse_table(&mut inner, true)?;
                            // Insert or merge if duplicate table?
                            // If key already exists and is a table, merge; otherwise overwrite
                            if let Some(existing) = table.get_mut(&key) {
                                if let FishValue::Table(existing_table) = existing {
                                    // merge inner into existing
                                    for (k, v) in inner {
                                        existing_table.insert(k, v);
                                    }
                                } else {
                                    *existing = FishValue::Table(inner);
                                }
                            } else {
                                table.insert(key, FishValue::Table(inner));
                            }
                        }
                        Some(b':') | Some(b'=') => {
                            // key: value
                            self.lexer.advance(); // consume : or =
                            let value = self.lexer.parse_value()?;
                            // After value, optional ; or newline handled by loop
                            table.insert(key, value);
                            // optional semicolon
                            self.lexer.skip_whitespace_and_comments()?;
                            if self.lexer.peek() == Some(b';') {
                                self.lexer.advance();
                            }
                        }
                        Some(_) => {
                            // Could be key without colon but with value? eg "theme dark" ?
                            // But we require colon. However to be friendly, if next token looks like value, try to parse as value without colon?
                            // We will error and suggest colon
                            // Check if next is identifier/string/number/etc and then later we see? Let's attempt to handle whitespace-separated value as sugar
                            // Look ahead: try parse value; if succeeds and then next token is not '{' then treat as key-value without colon
                            // This allows "theme dark" as alternative syntax

                            // Peek ahead without consuming identifier? We already have key, peek shows first char of value.
                            // Try to parse value directly
                            // But we need to distinguish between "section without braces"? Not needed.

                            // Attempt to recover: if next char could start a value (alnum, ", ', [, -, digit)
                            let c = next.unwrap();
                            let can_be_value = c.is_ascii_alphanumeric()
                                || c == b'"'
                                || c == b'\''
                                || c == b'['
                                || c == b'-'
                                || c == b'0'
                                || c == b'1'
                                || c == b'2'
                                || c == b'3'
                                || c == b'4'
                                || c == b'5'
                                || c == b'6'
                                || c == b'7'
                                || c == b'8'
                                || c == b'9'
                                || c == b'#'; // though # is comment, but handle

                            if can_be_value {
                                // Treat as colon-less assignment for ergonomics
                                let value = self.lexer.parse_value()?;
                                table.insert(key, value);
                                self.lexer.skip_whitespace_and_comments()?;
                                if self.lexer.peek() == Some(b';') {
                                    self.lexer.advance();
                                }
                            } else {
                                return Err(self.lexer.error(format!(
                                    "expected ':' or '{{' after key '{}', found '{}'",
                                    key, c as char
                                )));
                            }
                        }
                        None => {
                            return Err(self
                                .lexer
                                .error(format!("unexpected EOF after key '{}'", key)));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// Parse a fico string into a table.
pub fn parse_str(input: &str) -> Result<FishTable> {
    Parser::new(input).parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let input = r#"
            system {
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
        assert_eq!(table.len(), 2);
        let system = table.get("system").unwrap().as_table().unwrap();
        assert_eq!(system.get("theme").unwrap().as_str(), Some("dark"));
        assert_eq!(system.get("accent").unwrap().as_str(), Some("blue"));
        let appearance = table.get("appearance").unwrap().as_table().unwrap();
        assert_eq!(appearance.get("animations").unwrap().as_bool(), Some(true));
        assert!((appearance.get("transparency").unwrap().as_f64().unwrap() - 0.85).abs() < 1e-6);
        let icons = appearance.get("icons").unwrap().as_table().unwrap();
        assert_eq!(icons.get("style").unwrap().as_str(), Some("tontoo"));
        assert_eq!(icons.get("size").unwrap().as_i64(), Some(48));
    }

    #[test]
    fn test_comments() {
        let input = r#"
            # line comment
            // another comment
            /* block
               comment */
            system {
                theme: dark // inline
                accent: blue # inline hash
            }
        "#;
        let table = parse_str(input).unwrap();
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn test_array() {
        let input = r#"
            app {
                tags: [a, b, c]
                nums: [1, 2, 3]
                mixed: [hello, 42, true, 3.14, "quoted string"]
            }
        "#;
        let table = parse_str(input).unwrap();
        let app = table.get("app").unwrap().as_table().unwrap();
        let tags = app.get("tags").unwrap().as_array().unwrap();
        assert_eq!(tags.len(), 3);
        let nums = app.get("nums").unwrap().as_array().unwrap();
        assert_eq!(nums[0].as_i64(), Some(1));
    }

    #[test]
    fn test_quoted_strings() {
        let input = r#"
            app {
                name: "Hello World"
                path: '/usr/local/bin'
                escaped: "line\nbreak"
            }
        "#;
        let table = parse_str(input).unwrap();
        let app = table.get("app").unwrap().as_table().unwrap();
        assert_eq!(app.get("name").unwrap().as_str(), Some("Hello World"));
        assert_eq!(app.get("path").unwrap().as_str(), Some("/usr/local/bin"));
    }

    #[test]
    fn test_numbers() {
        let input = r#"
            vals {
                i: 42
                f: 3.14
                neg: -10
                hex: 0xFF
            }
        "#;
        let table = parse_str(input).unwrap();
        let vals = table.get("vals").unwrap().as_table().unwrap();
        assert_eq!(vals.get("i").unwrap().as_i64(), Some(42));
        assert!((vals.get("f").unwrap().as_f64().unwrap() - 3.14).abs() < 1e-6);
        assert_eq!(vals.get("neg").unwrap().as_i64(), Some(-10));
        assert_eq!(vals.get("hex").unwrap().as_i64(), Some(255));
    }
}

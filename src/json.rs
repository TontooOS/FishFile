//! Minimal std-only JSON bridge for FishFile.
//!
//! Parses JSON text directly into [`FishValue`](crate::value::FishValue)
//! (order-preserving via Foundation's `OrderedMap`) and renders values back
//! to JSON text. This replaces the former `serde_json` bridge so the crate
//! has no serde dependency.

use crate::error::{FishError, Result};
use crate::value::{FishTable, FishValue};

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self { bytes: s.as_bytes(), pos: 0 }
    }

    fn line_col(&self) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for &b in &self.bytes[..self.pos.min(self.bytes.len())] {
            if b == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    fn err(&self, message: impl Into<String>) -> FishError {
        let (line, col) = self.line_col();
        FishError::parse(line, col, message)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn expect_byte(&mut self, expected: u8, what: &str) -> Result<()> {
        self.skip_ws();
        if self.peek() == Some(expected) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.err(format!("expected {what}")))
        }
    }

    fn literal(&mut self, word: &str, what: &str) -> Result<()> {
        if self.bytes.len() >= self.pos + word.len()
            && &self.bytes[self.pos..self.pos + word.len()] == word.as_bytes()
        {
            self.pos += word.len();
            Ok(())
        } else {
            Err(self.err(format!("expected {what}")))
        }
    }

    fn parse_value(&mut self) -> Result<FishValue> {
        self.skip_ws();
        match self.peek() {
            Some(b'n') => {
                self.literal("null", "`null`")?;
                Ok(FishValue::Null)
            }
            Some(b't') => {
                self.literal("true", "`true`")?;
                Ok(FishValue::Bool(true))
            }
            Some(b'f') => {
                self.literal("false", "`false`")?;
                Ok(FishValue::Bool(false))
            }
            Some(b'"') => Ok(FishValue::String(self.parse_string()?)),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.parse_number(),
            Some(_) => Err(self.err("unexpected character")),
            None => Err(self.err("unexpected end of input")),
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        // Opening quote already peeked; consume it.
        self.pos += 1;
        let mut out = String::new();
        loop {
            let Some(b) = self.peek() else {
                return Err(self.err("unterminated string"));
            };
            match b {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    let Some(e) = self.peek() else {
                        return Err(self.err("unterminated escape"));
                    };
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            out.push(self.parse_unicode()?);
                        }
                        _ => return Err(self.err("invalid escape")),
                    }
                    if e != b'u' {
                        self.pos += 1;
                    }
                }
                _ => {
                    // Consume one UTF-8 scalar.
                    let rest = &self.bytes[self.pos..];
                    let s = std::str::from_utf8(rest)
                        .map_err(|_| self.err("invalid utf-8"))?;
                    let mut chars = s.chars();
                    let Some(c) = chars.next() else {
                        return Err(self.err("unterminated string"));
                    };
                    if (c as u32) < 0x20 {
                        return Err(self.err("unescaped control character in string"));
                    }
                    out.push(c);
                    self.pos += c.len_utf8();
                }
            }
        }
    }

    fn parse_unicode(&mut self) -> Result<char> {
        // `self.pos` points at the `u` of `\uXXXX`.
        let hex = |parser: &mut Self| -> Result<u32> {
            if parser.pos + 5 > parser.bytes.len() {
                return Err(parser.err("unterminated unicode escape"));
            }
            let digits = std::str::from_utf8(&parser.bytes[parser.pos + 1..parser.pos + 5])
                .map_err(|_| parser.err("invalid unicode escape"))?;
            let code = u32::from_str_radix(digits, 16)
                .map_err(|_| parser.err("invalid unicode escape"))?;
            parser.pos += 5;
            Ok(code)
        };
        let first = hex(self)?;
        // Surrogate pairs for characters outside the BMP.
        if (0xD800..0xDC00).contains(&first) {
            if self.bytes.get(self.pos) == Some(&b'\\')
                && self.bytes.get(self.pos + 1) == Some(&b'u')
            {
                self.pos += 1;
                let second = hex(self)?;
                if (0xDC00..0xE000).contains(&second) {
                    let code = 0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00);
                    return char::from_u32(code)
                        .ok_or_else(|| self.err("invalid unicode escape"));
                }
            }
            return Err(self.err("unpaired surrogate"));
        }
        char::from_u32(first).ok_or_else(|| self.err("invalid unicode escape"))
    }

    fn parse_number(&mut self) -> Result<FishValue> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        let mut is_float = false;
        if self.peek() == Some(b'.') {
            is_float = true;
            self.pos += 1;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| self.err("invalid number"))?;
        if !is_float {
            if let Ok(i) = text.parse::<i64>() {
                return Ok(FishValue::Integer(i));
            }
        }
        text.parse::<f64>()
            .map(FishValue::Float)
            .map_err(|_| self.err("invalid number"))
    }

    fn parse_array(&mut self) -> Result<FishValue> {
        self.pos += 1; // `[`
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(FishValue::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    return Ok(FishValue::Array(items));
                }
                _ => return Err(self.err("expected `,` or `]` in array")),
            }
        }
    }

    fn parse_object(&mut self) -> Result<FishValue> {
        self.pos += 1; // `{`
        let mut table = FishTable::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(FishValue::Table(table));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(self.err("expected string key"));
            }
            let key = self.parse_string()?;
            self.expect_byte(b':', "`:`")?;
            let value = self.parse_value()?;
            table.insert(key, value);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(FishValue::Table(table));
                }
                _ => return Err(self.err("expected `,` or `}` in object")),
            }
        }
    }
}

/// Parse a JSON string into a [`FishValue`](crate::value::FishValue).
pub fn parse_json(s: &str) -> Result<FishValue> {
    let mut parser = Parser::new(s);
    let value = parser.parse_value()?;
    parser.skip_ws();
    if parser.peek().is_some() {
        return Err(parser.err("trailing characters"));
    }
    Ok(value)
}

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

fn escape_into(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write_value(out: &mut String, value: &FishValue, pretty: bool, indent: usize) {
    match value {
        FishValue::Null => out.push_str("null"),
        FishValue::Bool(v) => out.push_str(if *v { "true" } else { "false" }),
        FishValue::Integer(v) => out.push_str(&v.to_string()),
        FishValue::Float(v) => {
            if v.is_finite() {
                // Match serde_json: whole floats keep a `.0` suffix so they
                // re-parse as floats, not integers.
                let text = v.to_string();
                if text.parse::<i64>().is_ok() {
                    out.push_str(&text);
                    out.push_str(".0");
                } else {
                    out.push_str(&text);
                }
            } else {
                out.push_str("null");
            }
        }
        FishValue::String(v) => escape_into(out, v),
        FishValue::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if pretty {
                    out.push('\n');
                    out.push_str(&"  ".repeat(indent + 1));
                }
                write_value(out, item, pretty, indent + 1);
            }
            if pretty {
                out.push('\n');
                out.push_str(&"  ".repeat(indent));
            }
            out.push(']');
        }
        FishValue::Table(table) => {
            if table.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            for (i, (k, v)) in table.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if pretty {
                    out.push('\n');
                    out.push_str(&"  ".repeat(indent + 1));
                }
                escape_into(out, k);
                out.push(':');
                if pretty {
                    out.push(' ');
                }
                write_value(out, v, pretty, indent + 1);
            }
            if pretty {
                out.push('\n');
                out.push_str(&"  ".repeat(indent));
            }
            out.push('}');
        }
    }
}

/// Render a [`FishValue`](crate::value::FishValue) as JSON.
/// `pretty` selects 2-space indented output.
pub fn write_json(value: &FishValue, pretty: bool) -> String {
    let mut out = String::new();
    write_value(&mut out, value, pretty, 0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_scalars() {
        for raw in ["null", "true", "false", "42", "-7", "0.85", "1e3", "\"hi\""] {
            let v = parse_json(raw).unwrap();
            let back = parse_json(&write_json(&v, false)).unwrap();
            assert_eq!(v, back, "roundtrip of {raw}");
        }
    }

    #[test]
    fn escapes_and_unicode() {
        let v = parse_json(r#""a\"b\\c\nd✓\u00e9""#).unwrap();
        assert_eq!(v.as_str(), Some("a\"b\\c\nd✓é"));
        let back = write_json(&v, false);
        assert_eq!(parse_json(&back).unwrap(), v);
    }

    #[test]
    fn rejects_trailing_and_empty() {
        assert!(parse_json("").is_err());
        assert!(parse_json("{} {}").is_err());
        assert!(parse_json("{unclosed").is_err());
    }

    #[test]
    fn pretty_is_valid_json() {
        let v = parse_json(r#"{"a":1,"b":[1,2],"c":{}}"#).unwrap();
        let pretty = write_json(&v, true);
        assert!(pretty.contains('\n'));
        assert_eq!(parse_json(&pretty).unwrap(), v);
    }
}

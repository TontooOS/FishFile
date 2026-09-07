# Parser

The parser reads `.fico` text into a `FishTable`. It is a hand-written recursive-descent parser with a small lexer.

## Syntax

### File

```text
fico = *statement
statement = key_value | section | comment | empty_line
```

Top-level may contain any number of key-values or sections. Nesting is unlimited.

### Key-Value

```text
key_value = identifier ":" value
          | identifier "=" value
          | identifier   value   // sugar: colon optional if value follows on same line
```

Both `:` and `=` are accepted. A lone `;` after a value is ignored.

```fico
theme: dark
accent = blue
size 48        // same as size: 48
```

### Section

```text
section = identifier "{" *statement "}"
```

```fico
icons {
    style: tontoo
    size: 48
}
```

Duplicate section names at the same level are merged (inner keys are inserted/merged recursively).

### Keys

Keys are identifiers:

```text
identifier = [a-zA-Z_][a-zA-Z0-9_-]*
```

Case-sensitive. Hyphen ` -` allowed after the first character. Dots `.` are reserved for the `FishDocument::get("a.b.c")` path helper and may not appear inside a single key.

### Values

| Syntax | FishValue | Example |
|---|---|---|
| `null` / `nil` / `~` | `Null` | `key: null` |
| `true` / `false` | `Bool` | `enabled: true` |
| decimal integer | `Integer(i64)` | `size: 48` |
| hex `0xFF` | `Integer(i64)` | `mask: 0xFF` |
| float | `Float(f64)` | `alpha: 0.85` |
| bare word | `String` | `theme: dark` |
| `"quoted"` / `'quoted'` | `String` | `path: "hello world"` |
| `[a, b, c]` | `Array` | `tags: [a, b, c]` |
| `name { ... }` | `Table` | section |

#### Bare Strings vs Keywords

Bare words are parsed as `String` unless they exactly match a keyword:

- `true` → `Bool(true)`
- `false` → `Bool(false)`
- `null` / `nil` / `~` → `Null`

To keep a string `"true"` as a string, quote it: `key: "true"`.

Numeric-looking bare words become numbers: `42` → `Integer`, `0.85` → `Float`. Quote to keep as string: `"42"`.

#### Quoted Strings

Both double `"` and single `'` quotes are supported. Escapes inside:

| Escape | Meaning |
|---|---|
| `\\` | backslash |
| `\"` | quote |
| `\'` | single quote |
| `\n` | newline |
| `\r` | carriage return |
| `\t` | tab |

Unknown escapes keep the backslash (`\x` → `\x`).

```fico
greeting: "hello \"world\""
path: '/tmp/my file.txt'
```

#### Arrays

```text
array = "[" [value ("," value)*] [","] "]"
```

Elements are any value except `Table` (tables cannot appear inside arrays). Trailing commas allowed.

```fico
tags: [editor, utility, tontoo]
nums: [1, 2, 3]
mixed: [hello, 42, true, 3.14, "quoted string"]
```

#### Numbers

- Integer: `[-]?[0-9]+` or `0x[0-9a-fA-F]+`
- Float: `[-]?[0-9]*"."[0-9]+` or `[-]?[0-9]+[eE][+-]?[0-9]+`

Overflows are not specially handled – `parse::<i64>` / `parse::<f64>` governs success; on failure the token falls back to `String`.

### Comments

| Style | Example |
|---|---|
| `//` line | `// comment` |
| `#` line | `# comment` |
| `/* block */` | `/* multi\nline */` |

`#` starts a comment when it appears where a statement may start (after whitespace/newline). Inside a quoted string it is not a comment. `//` and `/* */` are recognized anywhere outside strings.

```fico
# full line
system { // inline
    theme: dark /* block */ // another
}
```

### Whitespace & Separators

Whitespace, newlines and `;` are statement separators and are otherwise ignored. No semantic indentation.

## API

### `parser::parse_str(input)`

```rust
pub fn parse_str(input: &str) -> Result<FishTable>
```

Parse a string. Used internally by `FishDocument::parse`.

### `Parser::new(input).parse()`

Low-level parser struct – lexes and builds a `FishTable`. Errors contain `line`, `col` and a message.

```rust
let table = fishfile::parser::parse_str("a { b: 1 }").unwrap();
```

## Error Cases

- Missing closing `}` → `unterminated block, expected '}'`
- Missing `:`/`=` → `expected ':' or '{' after key`
- Unterminated string → `unterminated string literal`
- Unterminated block comment → `unterminated block comment`
- Unexpected `}` at top level → `unexpected '}' at top level`
- Trailing content after root → `unexpected token ... after root table`

All errors are `FishError::Parse { line, col, message }`.

## Cross References

- [Document.md](Document.md) – uses the parser for `FishDocument::parse`
- [Writer.md](Writer.md) – inverse of parsing
- [Value.md](Value.md) – target types
- [Error.md](Error.md) – error enum

# Writer

The writer serializes a `FishTable` / `FishDocument` back to `.fico` text. It aims for readable, round-trippable output.

## Functions

### `writer::to_string(table)`

```rust
pub fn to_string(table: &FishTable) -> String
```

Pretty-print with 4-space indent. Top-level sections get a blank line between them.

```rust
let doc = FishDocument::parse("system { theme: dark }").unwrap();
assert_eq!(doc.to_string(), "system {\n    theme: dark\n}\n");
```

### `writer::to_string_with_indent(table, indent_spaces)`

```rust
pub fn to_string_with_indent(table: &FishTable, indent_spaces: usize) -> String
```

Custom indent. `2` for compact, `4` is default.

### `FishDocument::to_string()` / `to_string_with_indent()`

Convenience wrappers that call the `writer` functions on the root table.

## Formatting Rules

| Value | Output | Notes |
|---|---|---|
| `Null` | `null` | |
| `Bool(true)` | `true` | lower case |
| `Bool(false)` | `false` | |
| `Integer(i)` | `i` | decimal, e.g. `48` |
| `Float(f)` | `f` | ensures decimal point or exponent; `1.0` not `1` |
| `String(s)` bare | `s` | if `needs_quotes(s) == false` |
| `String(s)` quoted | `"s"` | with escapes |
| `Array(a)` | `[a, b, c]` | comma + space, elements via same rules (no tables) |
| `Table(t)` | `key {\n ...\n}` | indented block, recursive |

### Quoting Rules

`needs_quotes(s)` returns true (needs `"..."`) when:

- `s` is empty
- `s == "true"` / `"false"` / `"null"` / `"nil"` / `"~"` (would be parsed as keyword)
- `s` parses as `i64` or `f64` or `0x` hex (would be parsed as number)
- contains any char outside `a-zA-Z0-9_-./` or contains `//` / `/*` / `#`
- starts with a digit (would be ambiguous)

Otherwise the string is emitted bare (no quotes), e.g. `dark`, `tontoo`, `/usr/local/bin`, `my-app`.

Quoted strings escape `"` → `\"`, `\\` → `\\`, `\n` → `\n`, `\r`, `\t`.

```rust
let mut table = FishTable::new();
table.insert("greeting".into(), FishValue::String("Hello World".into()));
// "Hello World" contains space → needs quotes
assert_eq!(fishfile::writer::to_string(&table), "greeting: \"Hello World\"\n");
```

### Indentation & Blank Lines

- Each nesting level adds `INDENT` (`    `, 4 spaces) or `indent_spaces` spaces.
- `key {` on one line, contents indented, `}` on its own line.
- After each top-level table a blank line is emitted (trimmed at EOF).
- The file always ends with `\n` unless empty.

```fico
system {
    theme: dark
}

appearance {
    icons {
        style: tontoo
    }
}
```

### Round-Trip

Parsing then writing then parsing again yields an equal table (`table == parse(to_string(table))`) for all well-formed inputs. Ordering is preserved via `IndexMap`.

```rust
let input = "system {\n    theme: dark\n}\n";
let table = parse_str(input).unwrap();
let out = to_string(&table);
let table2 = parse_str(&out).unwrap();
assert_eq!(table, table2);
```

## Cross References

- [Parser.md](Parser.md) – inverse operation
- [Document.md](Document.md) – high-level serialization
- [Value.md](Value.md) – value types serialized

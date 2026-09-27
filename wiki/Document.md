# Document

`FishDocument` is the high-level API for `.fico` files – it holds an ordered root table and provides parse, edit, serialize and file I/O.

## Constructors

### `FishDocument::new()`

```rust
pub fn new() -> Self
```

Create an empty document. No allocations beyond the root `IndexMap`.

```rust
let doc = FishDocument::new();
assert!(doc.is_empty());
```

### `FishDocument::from_table(table: FishTable)`

```rust
pub fn from_table(table: FishTable) -> Self
```

Wrap an existing `FishTable` as the root.

### `FishDocument::parse(input: &str)`

```rust
pub fn parse(input: &str) -> Result<Self>
```

Parse a `.fico` string. Returns `Err(FishError::Parse)` with line/col on syntax errors. Preserves insertion order.

```rust
let doc = FishDocument::parse("system { theme: dark }").unwrap();
```

### `FishDocument::from_file(path)`

```rust
pub fn from_file(path: impl AsRef<Path>) -> Result<Self>
```

Read and parse a file. Propagates `Io` errors for missing files.

```rust
let doc = FishDocument::from_file("/etc/tontoo/config.fico").unwrap();
```

## Serialization

### `to_string()`

```rust
pub fn to_string(&self) -> String
```

Pretty-print with 4-space indent. Top-level tables get a blank line between them for readability.

```rust
let s = doc.to_string();
assert!(s.contains("system {"));
```

### `to_string_with_indent(indent_spaces)`

```rust
pub fn to_string_with_indent(&self, indent_spaces: usize) -> String
```

Custom indent width (e.g. 2 for compact, 4 is default).

### `write_to_file(path)`

```rust
pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()>
```

Write `to_string()` to disk, creating or truncating the file.

### `to_bytes()`

```rust
pub fn to_bytes(&self) -> Vec<u8>
```

UTF-8 bytes of `to_string()` – useful for custom writers.

## Access

### `get(path)`

```rust
pub fn get(&self, path: &str) -> Option<&FishValue>
```

Dot-separated path, e.g. `"appearance.icons.size"`. Returns `None` if any segment is missing or traverses a non-table.

```rust
let v = doc.get("system.theme").unwrap();
assert_eq!(v.as_str(), Some("dark"));
```

### `get_mut(path)`

```rust
pub fn get_mut(&mut self, path: &str) -> Option<&mut FishValue>
```

Mutable variant for in-place edits.

### `contains(path)`

```rust
pub fn contains(&self, path: &str) -> bool
```

True if `get(path).is_some()`.

### `set(path, value)`

```rust
pub fn set(&mut self, path: &str, value: impl Into<FishValue>)
```

Insert or overwrite at `path`. Intermediate tables are created automatically. If an intermediate segment exists but is not a table, it is replaced with a table.

```rust
doc.set("a.b.c", 42);
assert_eq!(doc.get("a.b.c").unwrap().as_i64(), Some(42));
```

### `remove(path)`

```rust
pub fn remove(&mut self, path: &str) -> Option<FishValue>
```

Remove and return the value, or `None` if missing.

### `insert(key, value)` / `get_top(key)`

Direct top-level operations without dot splitting.

```rust
doc.insert("theme", "dark");
assert_eq!(doc.get_top("theme").unwrap().as_str(), Some("dark"));
```

### `merge(other)`

```rust
pub fn merge(&mut self, other: &FishDocument)
```

Merge `other` into `self`. Scalar values overwrite; tables merge recursively.

```rust
let mut a = FishDocument::parse("system { theme: dark }").unwrap();
let b = FishDocument::parse("system { accent: blue }").unwrap();
a.merge(&b);
assert_eq!(a.get("system.accent").unwrap().as_str(), Some("blue"));
```

### `root()` / `root_mut()` / `iter()` / `len()` / `is_empty()` / `clear()`

Low-level access to the root `FishTable`.

```rust
for (key, value) in doc.iter() {
    println!("{}: {:?}", key, value);
}
```

## JSON

The crate ships a small std-only JSON bridge (`json` module, no serde
dependency). Tables become JSON objects with insertion order preserved,
arrays stay arrays.

### `to_json()` / `to_json_pretty()`

```rust
pub fn to_json(&self) -> String
pub fn to_json_pretty(&self) -> String
```

Convert the document to compact or pretty (2-space) JSON. Infallible:
non-finite floats render as `null`.

### `from_json_str(s)`

```rust
pub fn from_json_str(s: &str) -> Result<Self>
```

Create a document from a JSON string. Root must be an object.

```rust
let doc = FishDocument::from_json_str(r#"{"theme": "dark", "size": 48}"#).unwrap();
let back = FishDocument::from_json_str(&doc.to_json()).unwrap();
assert_eq!(back, doc);
```

### `FishValue` JSON helpers

```rust
pub fn to_json_string(&self) -> String
pub fn to_json_pretty_string(&self) -> String
pub fn from_json_str(s: &str) -> Result<Self>
```

Same bridge for single values.

## Error Handling

All fallible methods return `fishfile::Result<T>` (`Result<T, FishError>`). Parse errors include line and column; I/O errors preserve `std::io::Error`.

## Display & FromStr

`FishDocument` implements `Display` (delegates to `to_string()`) and `FromStr` (delegates to `parse`).

```rust
let doc: FishDocument = "system { theme: dark }".parse().unwrap();
println!("{}", doc);
```

## Cross References

- [Value.md](Value.md) – value types stored in the document
- [Parser.md](Parser.md) – syntax accepted by `parse`
- [Writer.md](Writer.md) – formatting details
- [Error.md](Error.md) – error variants

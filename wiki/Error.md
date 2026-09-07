# Error

`FishError` is the single error enum for the crate. `Result<T>` is an alias for `Result<T, FishError>`.

## Enum

```rust
pub enum FishError {
    Io(std::io::Error),
    Parse { line: usize, col: usize, message: String },
    Type { expected: String, found: String },
    KeyNotFound(String),
    InvalidPath(String),
    Serde(String),
    Utf8(std::string::FromUtf8Error),
    Custom(String),
}
pub type Result<T> = std::result::Result<T, FishError>;
```

### `Io`

```rust
FishError::Io(std::io::Error)
```

File read/write failures. Auto-converted via `From<std::io::Error>`.

```rust
match FishDocument::from_file("/no/such.fico") {
    Err(FishError::Io(e)) => eprintln!("io: {}", e),
    _ => {}
}
```

### `Parse`

```rust
FishError::Parse { line: usize, col: usize, message: String }
```

Syntax errors with 1-based line/column.

```rust
pub fn parse(line: usize, col: usize, message: impl Into<String>) -> Self
```

```rust
let err = FishDocument::parse("system { theme: dark").unwrap_err();
if let FishError::Parse { line, col, message } = err {
    eprintln!("parse error at {}:{}: {}", line, col, message);
}
```

Display: `parse error at line 2, column 5: expected '}'`.

### `Type`

```rust
FishError::Type { expected: String, found: String }
```

Reserved for typed getters that enforce a variant (not currently thrown by `as_*` which return `Option`; used by future strict APIs).

### `KeyNotFound`

```rust
FishError::KeyNotFound(String)
```

Path lookup failures in strict APIs.

### `InvalidPath`

```rust
FishError::InvalidPath(String)
```

Empty or malformed dot-paths.

### `Serde`

```rust
FishError::Serde(String)
```

`serde_json` failures, auto-converted via `From<serde_json::Error>`.

### `Utf8`

```rust
FishError::Utf8(FromUtf8Error)
```

`parse_bytes` with invalid UTF-8.

### `Custom`

```rust
FishError::Custom(String)
```

Generic extension: `FishError::custom("...")` or `FishError::Custom("...")`.

## Display

`FishError` implements `Display` and `Error` (via `thiserror`).

- `Parse { line, col, message }` → `parse error at line X, column Y: ...`
- `Io` → `I/O error: ...`
- `Serde` → `serde error: ...`
- `Custom` → raw message

## Cross References

- [Document.md](Document.md) – methods returning `Result`
- [Parser.md](Parser.md) – parse error cases

# Value

`FishValue` is an enum representing every value that can appear in a `.fico` file. `FishTable` is the ordered map used for sections.

## FishTable

```rust
pub type FishTable = IndexMap<String, FishValue>
```

An `IndexMap` preserving insertion order – the order you wrote keys in the file is the order you get when iterating or re-serializing.

```rust
let mut table = FishTable::new();
table.insert("theme".to_string(), FishValue::String("dark".into()));
```

## FishValue

```rust
pub enum FishValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<FishValue>),
    Table(FishTable),
}
```

### Constructors via `From`

Every variant has `From` impls for ergonomic creation:

| Rust type | FishValue |
|---|---|
| `bool` | `Bool` |
| `i32`, `i64`, `u32`, `usize` | `Integer` |
| `f32`, `f64` | `Float` |
| `String`, `&str` | `String` |
| `Vec<FishValue>` | `Array` |
| `FishTable` | `Table` |
| `serde_json::Value` | via `from_json` |

```rust
let v: FishValue = true.into();
let v: FishValue = 42.into();
let v: FishValue = "hello".into();
let v: FishValue = vec![FishValue::Integer(1), FishValue::Integer(2)].into();
```

### Type Checks

```rust
pub fn is_null(&self) -> bool
pub fn is_bool(&self) -> bool
pub fn is_integer(&self) -> bool
pub fn is_float(&self) -> bool
pub fn is_number(&self) -> bool
pub fn is_string(&self) -> bool
pub fn is_array(&self) -> bool
pub fn is_table(&self) -> bool
```

### Accessors

```rust
pub fn as_bool(&self) -> Option<bool>
pub fn as_i64(&self) -> Option<i64>
pub fn as_f64(&self) -> Option<f64>
pub fn as_str(&self) -> Option<&str>
pub fn as_array(&self) -> Option<&Vec<FishValue>>
pub fn as_array_mut(&mut self) -> Option<&mut Vec<FishValue>>
pub fn as_table(&self) -> Option<&FishTable>
pub fn as_table_mut(&mut self) -> Option<&mut FishTable>
pub fn type_name(&self) -> &'static str
```

`as_f64` coerces integers to float (`Integer(48).as_f64() == Some(48.0)`). All others return `None` on type mismatch.

```rust
let v = FishValue::Integer(48);
assert_eq!(v.as_i64(), Some(48));
assert_eq!(v.as_f64(), Some(48.0));
assert_eq!(v.as_bool(), None);

let v = FishValue::String("dark".into());
assert_eq!(v.as_str(), Some("dark"));
```

### JSON Conversion

```rust
pub fn to_json(&self) -> serde_json::Value
pub fn from_json(v: &serde_json::Value) -> Self
```

Lossless except for table ordering (JSON objects are unordered). Numbers distinguish `Integer` vs `Float` via `as_i64`/`as_f64`.

```rust
let v = FishValue::Integer(42);
let json = v.to_json();
assert_eq!(json, serde_json::json!(42));
```

### Display

`FishValue::Display` prints a debug-like representation (`Table(3 keys)` for tables, `[a, b]` for arrays).

## Serde

`FishValue` derives `Serialize`/`Deserialize` with `#[serde(untagged)]`, so it round-trips through any serde format.

```rust
let json = serde_json::to_string(&FishValue::Bool(true)).unwrap();
let back: FishValue = serde_json::from_str(&json).unwrap();
```

## Cross References

- [Document.md](Document.md) – stores `FishValue` in a `FishTable`
- [Parser.md](Parser.md) – how each syntax maps to variants
- [Writer.md](Writer.md) – how each variant is formatted

# Macros

Two declarative macros for ergonomic construction without parsing.

## fish_value!

Create a `FishValue` literal, similar to `serde_json::json!`.

```rust
#[macro_export]
macro_rules! fish_value { ... }
```

### Syntax

```rust
fish_value!(null)          // Null
fish_value!(true)           // Bool(true)
fish_value!(false)          // Bool(false)
fish_value!([a, b, c])      // Array — elements are `fish_value!` recursively
fish_value!({"k" => v, ..}) // Table — keys are `expr`, values are `fish_value!` tt
fish_value!(expr)           // Fallback: `FishValue::from(expr)` (bool, int, float, &str, String, etc.)
```

Pairs use `=>` (not `:` – `:` is not allowed after `expr` in macros).

```rust
use fishfile::{fish_value, FishValue};

let v = fish_value!({"a" => 1, "b" => true});
assert!(v.is_table());

let v = fish_value!([1, 2, 3]);
assert_eq!(v.as_array().unwrap().len(), 3);

let v = fish_value!("hello world");
assert_eq!(v.as_str(), Some("hello world"));

let v = fish_value!({"nested" => {"x" => 42}});
assert_eq!(v.as_table().unwrap().get("nested").unwrap().is_table(), true);
```

> **Note:** `null`/`true`/`false` as bare tt are handled as keywords. To make a string `"null"` use `fish_value!("null")` or `FishValue::String("null".into())`.

## fish_doc!

Create a `FishDocument` from top-level pairs.

```rust
#[macro_export]
macro_rules! fish_doc { ... }
```

### Syntax

```rust
fish_doc! { "key" => value_tt, "key2" => value_tt, .. }
```

Each value is expanded via `fish_value!`.

```rust
use fishfile::fish_doc;

let doc = fish_doc! {
    "system" => { "theme" => "dark", "accent" => "blue" },
    "appearance" => { "animations" => true }
};
assert_eq!(doc.get("system.theme").unwrap().as_str(), Some("dark"));
```

Expands to:

```rust
let mut doc = FishDocument::new();
doc.insert("system", fish_value!({"theme" => "dark", "accent" => "blue"}));
// ...
```

## Usage

Macros are exported at crate root – no extra import needed beyond `use fishfile::fish_value;` (or `fishfile::fish_doc;`). They are also available via the prelude if you re-export.

```rust
use fishfile::{FishDocument, FishTable, FishValue};
use fishfile::{fish_doc, fish_value};
```

## Cross References

- [Value.md](Value.md) – `FishValue` produced by `fish_value!`
- [Document.md](Document.md) – `FishDocument` produced by `fish_doc!`

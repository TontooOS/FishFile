# TontooFishFile – Wiki

Fish Config (`.fico`) parser, writer and editor for TontooOS. A lightweight, human-friendly alternative to JSON/YAML with native nesting, comments and typed values.

- Repository: https://github.com/TontooOS/FishFile
- License: TCL v26.1
- Version: 26.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Wiki design system |
| Document | [Document.md](Document.md) | FishDocument – parse, edit, serialize, file I/O |
| Value | [Value.md](Value.md) | FishValue & FishTable – typed values and tables |
| Parser | [Parser.md](Parser.md) | .fico syntax and parser behavior |
| Writer | [Writer.md](Writer.md) | Serialization and formatting |
| Error | [Error.md](Error.md) | Error types and handling |
| Macros | [Macros.md](Macros.md) | fish_value! and fish_doc! macros |
| FFI | [Ffi.md](Ffi.md) | C header and interop |

## Quick Start

```rust
use fishfile::{FishDocument, FishValue};

fn main() -> fishfile::Result<()> {
    // Parse
    let text = r#"
        system {
            theme: dark
            accent: blue
        }
        appearance {
            animations: true
            icons { style: tontoo  size: 48 }
        }
    "#;
    let doc = FishDocument::parse(text)?;
    assert_eq!(doc.get("system.theme").unwrap().as_str(), Some("dark"));

    // Edit
    let mut doc = FishDocument::new();
    doc.set("appearance.transparency", 0.85);
    doc.set("appearance.icons.size", 48);

    // Serialize
    println!("{}", doc.to_string());

    // File I/O
    doc.write_to_file("/tmp/config.fico")?;
    let loaded = FishDocument::from_file("/tmp/config.fico")?;

    // JSON + Serde
    let json = doc.to_json()?;
    let back = FishDocument::from_json_str(&json)?;

    Ok(())
}
```

See [Document.md](Document.md), [Value.md](Value.md) and [Parser.md](Parser.md) for details.

## Changelog

- 2026-08-26: Initial wiki for FishFile 26.1.0

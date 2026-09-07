//! FishFile – Fish Config (.fico) for TontooOS
//!
//! A lightweight, human-friendly config format for TontooOS.
//! Syntax is intentionally simple – similar to JSON/YAML but with
//! less punctuation and native nesting via braces.
//!
//! # Format
//!
//! ```text
//! system {
//!     theme: dark
//!     accent: blue
//! }
//!
//! appearance {
//!     animations: true
//!     transparency: 0.85
//!     icons {
//!         style: tontoo
//!         size: 48
//!     }
//! }
//! ```
//!
//! ## Rules
//! - Keys are identifiers: `[a-zA-Z_][a-zA-Z0-9_-]*`
//! - Values: `string` (quoted or bare), `integer`, `float`, `bool` (`true`/`false`), `null`, `array`
//! - Bare strings: `dark`, `blue`, `tontoo` – unquoted if they contain only `a-zA-Z0-9_-./` and are not keywords
//! - Quoted strings: `"hello world"`, `'path with spaces'` – with `\n`, `\t`, `\\`, `\"` escapes
//! - Arrays: `[a, b, c]` or `[1, 2, 3]` or `["hello world", 42, true]`
//! - Tables/Sections: `name { ... }` – nest arbitrarily
//! - Comments: `// line`, `# line`, `/* block */`
//! - Separators: `:` or `=` (both accepted), `;` optional, whitespace/newlines ignored
//! - File extension: `.fico` (Fish Config)
//!
//! # Quick Start
//!
//! ```rust
//! use fishfile::{FishDocument, FishValue};
//!
//! // Parse
//! let text = r#"
//!     system {
//!         theme: dark
//!         accent: blue
//!     }
//! "#;
//! let doc = FishDocument::parse(text).unwrap();
//! assert_eq!(doc.get("system.theme").unwrap().as_str(), Some("dark"));
//!
//! // Build programmatically
//! let mut doc = FishDocument::new();
//! doc.set("appearance.transparency", 0.85);
//! doc.set("appearance.animations", true);
//! doc.set("appearance.icons.style", "tontoo");
//! doc.set("appearance.icons.size", 48);
//!
//! // Serialize
//! let fico = doc.to_string();
//! println!("{}", fico);
//!
//! // File I/O
//! // doc.write_to_file("/tmp/config.fico").unwrap();
//! // let loaded = FishDocument::from_file("/tmp/config.fico").unwrap();
//!
//! // JSON interop
//! let json = doc.to_json().unwrap();
//! let back = FishDocument::from_json_str(&json).unwrap();
//! ```

pub mod document;
pub mod error;
pub mod parser;
pub mod value;
pub mod writer;

pub use document::FishDocument;
pub use error::{FishError, Result};
pub use value::{FishTable, FishValue};
pub use writer::{to_string, to_string_with_indent};

/// The canonical file extension for Fish Config files.
pub const FICO_EXTENSION: &str = "fico";

/// Library version (major, minor, patch).
pub const FISHFILE_VERSION: (u32, u32, u32) = (26, 1, 0);

/// Prelude – import everything most callers need.
pub mod prelude {
    pub use crate::document::FishDocument;
    pub use crate::error::{FishError, Result};
    pub use crate::value::{FishTable, FishValue};
    pub use crate::FICO_EXTENSION;
}

/// Parse a string directly (shorthand for `FishDocument::parse`).
pub fn parse(input: &str) -> Result<FishDocument> {
    FishDocument::parse(input)
}

/// Parse bytes (UTF-8).
pub fn parse_bytes(bytes: &[u8]) -> Result<FishDocument> {
    let s = String::from_utf8(bytes.to_vec())?;
    FishDocument::parse(&s)
}

/// Load a .fico file from disk.
pub fn load_file(path: impl AsRef<std::path::Path>) -> Result<FishDocument> {
    FishDocument::from_file(path)
}

/// Save a document to disk.
pub fn save_file(doc: &FishDocument, path: impl AsRef<std::path::Path>) -> Result<()> {
    doc.write_to_file(path)
}

// ---------------------------------------------------------------- optional macros

/// Create a `FishValue` literal, similar to `serde_json::json!`.
///
/// ```rust
/// use fishfile::{fish_value, FishValue};
/// let v = fish_value!({"system" => {"theme" => "dark", "size" => 48}, "tags" => ["a", "b"]});
/// ```
#[macro_export]
macro_rules! fish_value {
    (null) => { $crate::FishValue::Null };
    (true) => { $crate::FishValue::Bool(true) };
    (false) => { $crate::FishValue::Bool(false) };
    ([$($elem:tt),* $(,)?]) => {
        $crate::FishValue::Array(vec![$( $crate::fish_value!($elem) ),*])
    };
    ({ $($key:expr => $val:tt),* $(,)? }) => {
        {
            let mut table = $crate::FishTable::new();
            $( table.insert($key.to_string(), $crate::fish_value!($val)); )*
            $crate::FishValue::Table(table)
        }
    };
    ($other:expr) => {
        {
            // fallback: use From conversion
            $crate::FishValue::from($other)
        }
    };
}

/// Create a `FishDocument` from a literal table.
///
/// ```rust
/// use fishfile::{fish_doc, FishDocument};
/// let doc = fish_doc!{
///     "system" => { "theme" => "dark", "accent" => "blue" },
///     "appearance" => { "animations" => true }
/// };
/// ```
#[macro_export]
macro_rules! fish_doc {
    ( $($key:expr => $val:tt),* $(,)? ) => {
        {
            let mut doc = $crate::FishDocument::new();
            $(
                doc.insert($key, $crate::fish_value!($val));
            )*
            doc
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_from_prompt() {
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
        let doc = FishDocument::parse(input).unwrap();
        assert_eq!(doc.get("system.theme").unwrap().as_str(), Some("dark"));
        assert_eq!(doc.get("system.accent").unwrap().as_str(), Some("blue"));
        assert_eq!(doc.get("appearance.animations").unwrap().as_bool(), Some(true));
        assert!((doc.get("appearance.transparency").unwrap().as_f64().unwrap() - 0.85).abs() < 1e-6);
        assert_eq!(doc.get("appearance.icons.style").unwrap().as_str(), Some("tontoo"));
        assert_eq!(doc.get("appearance.icons.size").unwrap().as_i64(), Some(48));

        // edit and re-serialize
        let mut doc = doc;
        doc.set("system.theme", "light");
        assert_eq!(doc.get("system.theme").unwrap().as_str(), Some("light"));
        let out = doc.to_string();
        assert!(out.contains("theme: light"));
    }

    #[test]
    fn test_macro() {
        let v = fish_value!({"a" => 1, "b" => true});
        assert!(v.is_table());
        let doc = fish_doc! { "system" => { "theme" => "dark" } };
        assert_eq!(doc.get("system").unwrap().is_table(), true);
    }
}

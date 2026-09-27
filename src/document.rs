use crate::error::{FishError, Result};
use crate::parser::parse_str;
use crate::value::{FishTable, FishValue};
use crate::writer;
use std::path::Path;

/// The main FishFile document – holds the root table of a .fico file.
///
/// This is the high-level API for nested config mappings.
/// It supports nested `Table` values, dot-path access, file I/O and conversions.
///
/// # Example
///
/// ```rust
/// use fishfile::{FishDocument, FishValue};
///
/// let text = r#"
///     system {
///         theme: dark
///         accent: blue
///     }
/// "#;
///
/// let doc = FishDocument::parse(text).unwrap();
/// assert_eq!(doc.get("system.theme").unwrap().as_str(), Some("dark"));
///
/// // Create and edit
/// let mut doc = FishDocument::new();
/// doc.set("appearance.icons.size", 48);
/// doc.set("appearance.icons.style", "tontoo");
/// println!("{}", doc.to_string());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FishDocument {
    root: FishTable,
}

impl FishDocument {
    /// Create an empty document.
    pub fn new() -> Self {
        Self {
            root: FishTable::new(),
        }
    }

    /// Create from an existing table.
    pub fn from_table(table: FishTable) -> Self {
        Self { root: table }
    }

    /// Parse a fico string.
    pub fn parse(input: &str) -> Result<Self> {
        let table = parse_str(input)?;
        Ok(Self { root: table })
    }

    /// Alias for `parse` – mirrors `FromStr`.
    pub fn from_str(input: &str) -> Result<Self> {
        Self::parse(input)
    }

    /// Read and parse a .fico file from disk.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Serialize to a fico-formatted string (pretty, 4-space indent).
    pub fn to_string(&self) -> String {
        writer::to_string(&self.root)
    }

    /// Serialize with custom indent width.
    pub fn to_string_with_indent(&self, indent_spaces: usize) -> String {
        writer::to_string_with_indent(&self.root, indent_spaces)
    }

    /// Write to a file. Creates or truncates the file.
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        std::fs::write(path, self.to_string())?;
        Ok(())
    }

    /// Serialize to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }

    /// Get immutable reference to root table.
    pub fn root(&self) -> &FishTable {
        &self.root
    }

    /// Get mutable reference to root table.
    pub fn root_mut(&mut self) -> &mut FishTable {
        &mut self.root
    }

    /// Check if document is empty.
    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    /// Number of top-level keys.
    pub fn len(&self) -> usize {
        self.root.len()
    }

    /// Get a value by dot-separated path, e.g. `"appearance.icons.size"`.
    ///
    /// Returns `None` if any segment is missing or traverses a non-table value.
    pub fn get(&self, path: &str) -> Option<&FishValue> {
        get_path(&self.root, path)
    }

    /// Get mutable reference by path.
    pub fn get_mut(&mut self, path: &str) -> Option<&mut FishValue> {
        get_path_mut(&mut self.root, path)
    }

    /// Check if a path exists.
    pub fn contains(&self, path: &str) -> bool {
        self.get(path).is_some()
    }

    /// Insert or overwrite a value at `path`. Intermediate tables are created as needed.
    ///
    /// # Example
    /// ```rust
    /// use fishfile::FishDocument;
    /// let mut doc = FishDocument::new();
    /// doc.set("a.b.c", 42);
    /// assert_eq!(doc.get("a.b.c").unwrap().as_i64(), Some(42));
    /// ```
    pub fn set(&mut self, path: &str, value: impl Into<FishValue>) {
        set_path(&mut self.root, path, value.into());
    }

    /// Remove a value at `path`, returning it if it existed.
    pub fn remove(&mut self, path: &str) -> Option<FishValue> {
        remove_path(&mut self.root, path)
    }

    /// Insert a top-level key directly (no path splitting).
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<FishValue>) {
        self.root.insert(key.into(), value.into());
    }

    /// Get top-level value directly.
    pub fn get_top(&self, key: &str) -> Option<&FishValue> {
        self.root.get(key)
    }

    /// Remove top-level key.
    pub fn remove_top(&mut self, key: &str) -> Option<FishValue> {
        self.root.shift_remove(key)
    }

    /// Merge another document into this one. Values from `other` overwrite existing keys;
    /// tables are merged recursively.
    pub fn merge(&mut self, other: &FishDocument) {
        merge_tables(&mut self.root, &other.root);
    }

    /// Convert to a JSON string (pretty, 2-space indent).
    pub fn to_json_pretty(&self) -> String {
        FishValue::Table(self.root.clone()).to_json_pretty_string()
    }

    /// Convert to a compact JSON string.
    pub fn to_json(&self) -> String {
        FishValue::Table(self.root.clone()).to_json_string()
    }

    /// Parse from a JSON string. The root must be an object.
    pub fn from_json_str(s: &str) -> Result<Self> {
        match FishValue::from_json_str(s)? {
            FishValue::Table(table) => Ok(Self { root: table }),
            _ => Err(FishError::custom("JSON root must be an object")),
        }
    }

    /// Access as `Index` helper – returns Option.
    pub fn get_index(&self, key: &str) -> Option<&FishValue> {
        self.root.get(key)
    }

    /// Clear all contents.
    pub fn clear(&mut self) {
        self.root.clear();
    }

    /// Iterate over top-level entries.
    pub fn iter(&self) -> foundation::collections::OrderedIter<'_, String, FishValue> {
        self.root.iter()
    }
}

impl Default for FishDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for FishDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::str::FromStr for FishDocument {
    type Err = FishError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// ---------------------------------------------------------------- helpers

fn get_path<'a>(table: &'a FishTable, path: &str) -> Option<&'a FishValue> {
    if path.is_empty() {
        return None;
    }
    let mut current_table = table;
    let mut current_value: Option<&FishValue> = None;
    let parts: Vec<&str> = path.split('.').collect();
    for (i, part) in parts.iter().enumerate() {
        let v = current_table.get(*part)?;
        if i == parts.len() - 1 {
            current_value = Some(v);
        } else {
            match v {
                FishValue::Table(t) => current_table = t,
                _ => return None,
            }
        }
    }
    current_value
}

fn get_path_mut<'a>(table: &'a mut FishTable, path: &str) -> Option<&'a mut FishValue> {
    if path.is_empty() {
        return None;
    }
    let parts: Vec<&str> = path.split('.').collect();
    get_path_mut_recursive(table, &parts)
}

fn get_path_mut_recursive<'a>(
    table: &'a mut FishTable,
    parts: &[&str],
) -> Option<&'a mut FishValue> {
    if parts.is_empty() {
        return None;
    }
    if parts.len() == 1 {
        return table.get_mut(parts[0]);
    }
    let first = parts[0];
    let rest = &parts[1..];
    let v = table.get_mut(first)?;
    match v {
        FishValue::Table(t) => get_path_mut_recursive(t, rest),
        _ => None,
    }
}

fn set_path(table: &mut FishTable, path: &str, value: FishValue) {
    if path.is_empty() {
        return;
    }
    let parts: Vec<&str> = path.split('.').collect();
    if parts.len() == 1 {
        table.insert(parts[0].to_string(), value);
        return;
    }
    let mut current = table;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            current.insert((*part).to_string(), value);
            break;
        }
        // ensure intermediate is a table
        if !current.contains_key(*part) {
            current.insert((*part).to_string(), FishValue::Table(FishTable::new()));
        }
        // if existing is not a table, overwrite with table
        let is_table = matches!(current.get(*part), Some(FishValue::Table(_)));
        if !is_table {
            current.insert((*part).to_string(), FishValue::Table(FishTable::new()));
        }
        // now descend
        let next = current.get_mut(*part).unwrap();
        if let FishValue::Table(t) = next {
            current = t;
        } else {
            unreachable!();
        }
    }
}

fn remove_path(table: &mut FishTable, path: &str) -> Option<FishValue> {
    if path.is_empty() {
        return None;
    }
    let parts: Vec<&str> = path.split('.').collect();
    if parts.len() == 1 {
        return table.shift_remove(parts[0]);
    }
    let mut current = table;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            return current.shift_remove(*part);
        }
        let v = current.get_mut(*part)?;
        match v {
            FishValue::Table(t) => current = t,
            _ => return None,
        }
    }
    None
}

fn merge_tables(target: &mut FishTable, source: &FishTable) {
    for (k, v) in source {
        match v {
            FishValue::Table(source_inner) => {
                if let Some(FishValue::Table(target_inner)) = target.get_mut(k) {
                    merge_tables(target_inner, source_inner);
                } else {
                    target.insert(k.clone(), v.clone());
                }
            }
            _ => {
                target.insert(k.clone(), v.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_parse_example() {
        let text = r#"
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
        let doc = FishDocument::parse(text).unwrap();
        assert_eq!(doc.get("system.theme").unwrap().as_str(), Some("dark"));
        assert_eq!(doc.get("appearance.icons.size").unwrap().as_i64(), Some(48));
        // to_string roundtrip
        let s = doc.to_string();
        let doc2 = FishDocument::parse(&s).unwrap();
        assert_eq!(doc, doc2);
    }

    #[test]
    fn test_set_and_remove() {
        let mut doc = FishDocument::new();
        doc.set("a.b.c", 42);
        assert_eq!(doc.get("a.b.c").unwrap().as_i64(), Some(42));
        assert!(doc.contains("a.b"));
        let removed = doc.remove("a.b.c").unwrap();
        assert_eq!(removed.as_i64(), Some(42));
        assert!(doc.get("a.b.c").is_none());
    }

    #[test]
    fn test_file_io() {
        let mut doc = FishDocument::new();
        doc.set("system.theme", "dark");
        doc.set("system.accent", "blue");
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.fico");
        doc.write_to_file(&path).unwrap();
        let doc2 = FishDocument::from_file(&path).unwrap();
        assert_eq!(doc, doc2);
    }

    #[test]
    fn test_json_conversion() {
        let mut doc = FishDocument::new();
        doc.set("system.theme", "dark");
        doc.set("count", 42);
        let json = doc.to_json();
        let doc2 = FishDocument::from_json_str(&json).unwrap();
        assert_eq!(doc2.get("system.theme").unwrap().as_str(), Some("dark"));
        assert_eq!(doc2.get("count").unwrap().as_i64(), Some(42));
        let pretty = doc.to_json_pretty();
        assert_eq!(FishDocument::from_json_str(&pretty).unwrap(), doc);
    }

    #[test]
    fn test_merge() {
        let mut a = FishDocument::parse("system { theme: dark }").unwrap();
        let b = FishDocument::parse("system { accent: blue } appearance { animations: true }").unwrap();
        a.merge(&b);
        assert_eq!(a.get("system.theme").unwrap().as_str(), Some("dark"));
        assert_eq!(a.get("system.accent").unwrap().as_str(), Some("blue"));
        assert_eq!(a.get("appearance.animations").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_json_value_roundtrip() {
        let doc = FishDocument::from_json_str(
            r#"{"theme": "dark", "size": 48, "tags": ["a", "b"], "nested": {"x": 1.5}}"#,
        )
        .unwrap();
        assert_eq!(doc.get("theme").unwrap().as_str(), Some("dark"));
        assert_eq!(doc.get("size").unwrap().as_i64(), Some(48));
        assert_eq!(
            doc.get("tags").unwrap().as_array().unwrap().len(),
            2
        );
        let back = FishDocument::from_json_str(&doc.to_json()).unwrap();
        assert_eq!(back, doc);
    }
}

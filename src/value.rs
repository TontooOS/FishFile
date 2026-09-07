use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Ordered table – preserves insertion order, mirrors the file order.
pub type FishTable = IndexMap<String, FishValue>;

/// A single value inside a .fico file.
///
/// Mapping to fico syntax:
///
/// | Rust variant | fico representation | Example |
/// |---|---|---|
/// | `Null` | `null` | `key: null` |
/// | `Bool(true)` | `true` | `enabled: true` |
/// | `Integer(42)` | `42` | `size: 48` |
/// | `Float(0.85)` | `0.85` | `transparency: 0.85` |
/// | `String("dark")` | `dark` or `"dark"` | `theme: dark` |
/// | `Array([...])` | `[1, 2, 3]` | `tags: [a, b, c]` |
/// | `Table({...})` | `name { ... }` | `icons { size: 48 }` |
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FishValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<FishValue>),
    Table(FishTable),
}

impl FishValue {
    // ------------------------------------------------------------------ type checks
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Integer(_) | Self::Float(_))
    }
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }
    pub fn is_table(&self) -> bool {
        matches!(self, Self::Table(_))
    }

    // ------------------------------------------------------------------ getters
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(v) = self {
            Some(*v)
        } else {
            None
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        if let Self::Integer(v) = self {
            Some(*v)
        } else {
            None
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            Self::Integer(v) => Some(*v as f64),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_array(&self) -> Option<&Vec<FishValue>> {
        if let Self::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<FishValue>> {
        if let Self::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_table(&self) -> Option<&FishTable> {
        if let Self::Table(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_table_mut(&mut self) -> Option<&mut FishTable> {
        if let Self::Table(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Human-readable type name, for error messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "bool",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::Array(_) => "array",
            Self::Table(_) => "table",
        }
    }

    /// Convert to serde_json::Value (lossless except for table ordering).
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Null => serde_json::Value::Null,
            Self::Bool(v) => serde_json::Value::Bool(*v),
            Self::Integer(v) => serde_json::Value::Number((*v).into()),
            Self::Float(v) => serde_json::Number::from_f64(*v)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Self::String(v) => serde_json::Value::String(v.clone()),
            Self::Array(v) => serde_json::Value::Array(v.iter().map(|x| x.to_json()).collect()),
            Self::Table(v) => {
                let mut map = serde_json::Map::new();
                for (k, val) in v {
                    map.insert(k.clone(), val.to_json());
                }
                serde_json::Value::Object(map)
            }
        }
    }

    /// Create from serde_json::Value.
    pub fn from_json(v: &serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Bool(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Self::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Self::Float(f)
                } else {
                    Self::String(n.to_string())
                }
            }
            serde_json::Value::String(s) => Self::String(s.clone()),
            serde_json::Value::Array(arr) => Self::Array(arr.iter().map(Self::from_json).collect()),
            serde_json::Value::Object(map) => {
                let mut table = FishTable::new();
                for (k, val) in map {
                    table.insert(k.clone(), Self::from_json(val));
                }
                Self::Table(table)
            }
        }
    }
}

impl fmt::Display for FishValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(v) => write!(f, "{}", v),
            Self::Integer(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
            Self::Array(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Self::Table(t) => write!(f, "Table({} keys)", t.len()),
        }
    }
}

// ---------------------------------------------------------------- From impls
impl From<bool> for FishValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}
impl From<i64> for FishValue {
    fn from(v: i64) -> Self {
        Self::Integer(v)
    }
}
impl From<i32> for FishValue {
    fn from(v: i32) -> Self {
        Self::Integer(v as i64)
    }
}
impl From<u32> for FishValue {
    fn from(v: u32) -> Self {
        Self::Integer(v as i64)
    }
}
impl From<usize> for FishValue {
    fn from(v: usize) -> Self {
        Self::Integer(v as i64)
    }
}
impl From<f64> for FishValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}
impl From<f32> for FishValue {
    fn from(v: f32) -> Self {
        Self::Float(v as f64)
    }
}
impl From<String> for FishValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}
impl From<&str> for FishValue {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}
impl From<Vec<FishValue>> for FishValue {
    fn from(v: Vec<FishValue>) -> Self {
        Self::Array(v)
    }
}
impl From<FishTable> for FishValue {
    fn from(v: FishTable) -> Self {
        Self::Table(v)
    }
}
impl From<serde_json::Value> for FishValue {
    fn from(v: serde_json::Value) -> Self {
        Self::from_json(&v)
    }
}

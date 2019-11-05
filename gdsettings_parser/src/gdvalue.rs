//! GDValue

use std::collections::BTreeMap;
use std::string::ToString;

/// Godot value
#[derive(PartialEq, Debug, Clone)]
pub enum GdValue {
    /// Object
    Object(Vec<(String, GdValue)>),
    /// Array
    Array(Vec<GdValue>),
    /// String
    String(String),
    /// Int
    Int(i32),
    /// Float
    Float(f64),
    /// Boolean
    Boolean(bool),
    /// Class name
    ClassName(String),
    /// Class instance: a name, arguments and keyword arguments
    ClassInstance(String, Vec<GdValue>, Vec<(String, GdValue)>),
    /// Null
    Null,
}

impl GdValue {
    /// To array
    pub fn to_array(&self) -> Option<Vec<GdValue>> {
        if let GdValue::Array(a) = &self {
            Some(a.clone())
        } else {
            None
        }
    }

    /// To object
    pub fn to_object(&self) -> Option<BTreeMap<String, GdValue>> {
        if let GdValue::Object(p) = &self {
            let mut map = BTreeMap::new();
            for (k, v) in p.iter() {
                map.insert(k.clone(), v.clone());
            }

            Some(map)
        } else {
            None
        }
    }

    /// To string
    pub fn to_str(&self) -> Option<String> {
        if let GdValue::String(s) = &self {
            Some(s.clone())
        } else {
            None
        }
    }

    /// To i32
    pub fn to_i32(&self) -> Option<i32> {
        if let GdValue::Int(i) = &self {
            Some(*i)
        } else {
            None
        }
    }

    /// To bool
    pub fn to_bool(&self) -> Option<bool> {
        if let GdValue::Boolean(b) = &self {
            Some(*b)
        } else {
            None
        }
    }

    /// To f64
    pub fn to_f64(&self) -> Option<f64> {
        if let GdValue::Float(f) = &self {
            Some(*f)
        } else {
            None
        }
    }
}

impl ToString for GdValue {
    fn to_string(&self) -> String {
        serialize_gdvalue(&self)
    }
}

/// Serialize a GdValue to string
///
/// # Arguments
///
/// * `val` - GdValue object
///
fn serialize_gdvalue(val: &GdValue) -> String {
    match val {
        GdValue::Object(o) => {
            let contents: Vec<_> = o
                .iter()
                .map(|(name, value)| format!("\"{}\": {}", name, serialize_gdvalue(value)))
                .collect();
            format!("{{{}}}", contents.join(", "))
        }
        GdValue::Array(a) => {
            let contents: Vec<_> = a.iter().map(serialize_gdvalue).collect();
            format!("[{}]", contents.join(", "))
        }
        GdValue::ClassInstance(cls, args, kwargs) => {
            let args_content = args
                .iter()
                .map(serialize_gdvalue)
                .collect::<Vec<_>>()
                .join(", ");
            let kwargs_content = kwargs
                .iter()
                .map(|(name, value)| format!("{}: {}", name, serialize_gdvalue(value)))
                .collect::<Vec<_>>()
                .join(", ");

            if kwargs.is_empty() {
                format!("{}({})", cls, args_content)
            } else {
                format!("{}({}, {})", cls, args_content, kwargs_content)
            }
        }
        GdValue::ClassName(n) => n.to_string(),
        GdValue::String(s) => format!("\"{}\"", s),
        GdValue::Int(n) => n.to_string(),
        GdValue::Float(n) => format!("{:.9}", n),
        GdValue::Boolean(b) => b.to_string(),
        GdValue::Null => "null".to_string(),
    }
}

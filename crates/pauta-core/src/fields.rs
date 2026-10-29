//! Tolerant readers for YAML mappings, plus serde helpers.

use serde_yaml_ng::{Mapping, Value};

use crate::yaml::{Y, scalar_to_string};

pub fn get_str(m: &Mapping, k: &str) -> Option<String> {
    match m.get(k)? {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

pub fn get_i64(m: &Mapping, k: &str) -> Option<i64> {
    match m.get(k)? {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

/// A list of scalars; a lone scalar counts as a one-item list.
pub fn get_list(m: &Mapping, k: &str) -> Vec<String> {
    match m.get(k) {
        Some(Value::Sequence(items)) => items.iter().filter(|v| !v.is_null()).map(scalar_to_string).collect(),
        Some(Value::Null) | None => Vec::new(),
        Some(other) => vec![scalar_to_string(other)],
    }
}

pub fn get_list_or(m: &Mapping, k: &str, default: &[&str]) -> Vec<String> {
    if m.contains_key(k) {
        get_list(m, k)
    } else {
        default.iter().map(|s| s.to_string()).collect()
    }
}

pub fn get_opt_list(m: &Mapping, k: &str) -> Option<Vec<String>> {
    match m.get(k) {
        None | Some(Value::Null) => None,
        Some(_) => Some(get_list(m, k)),
    }
}

/// Overlays `updates` on an existing mapping value, keeping its other keys in
/// their original order. Updates set to `Y::Null` remove the key.
pub fn merge_map(existing: Option<&Value>, updates: Vec<(String, Y)>) -> Y {
    let mut entries: Vec<(String, Y)> = match existing.map(Y::from_value) {
        Some(Y::Map(e)) => e,
        _ => Vec::new(),
    };
    for (k, v) in updates {
        let pos = entries.iter().position(|(ek, _)| *ek == k);
        match (pos, v) {
            (Some(i), Y::Null) => {
                entries.remove(i);
            }
            (Some(i), v) => entries[i].1 = v,
            (None, Y::Null) => {}
            (None, v) => entries.push((k, v)),
        }
    }
    Y::Map(entries)
}

/// Distinguishes "field absent" (leave alone) from "field: null" (clear it)
/// when deserializing patches: `Option<Option<T>>`.
pub mod double_option {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, T, D>(d: D) -> Result<Option<Option<T>>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        Option::<T>::deserialize(d).map(Some)
    }
}

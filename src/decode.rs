//! Decode gron assignment lines back into a `serde_json::Value`.

use core::fmt;

use serde_json::{Map, Value};

use crate::ident::{is_ident_continue, is_ident_start};

/// Largest array index accepted in a path (bounds memory for untrusted input).
const MAX_INDEX: usize = 1_000_000;
/// Largest path depth accepted (bounds nesting so the rebuilt value can't
/// overflow the stack when it is later dropped or serialized).
const MAX_DEPTH: usize = 1024;

/// One step of a gron path: an object key or an array index.
enum Seg {
    Key(String),
    Index(usize),
}

/// An error produced while parsing gron lines in [`ungron`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UngronError {
    /// A line is not a well-formed `path = value;` statement.
    Syntax(String),
    /// The right-hand side is not valid JSON.
    InvalidValue(String),
    /// An array index exceeds the supported maximum.
    IndexTooLarge(usize),
    /// A path is nested more deeply than the supported maximum.
    TooDeep(usize),
}

impl fmt::Display for UngronError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UngronError::Syntax(s) => write!(f, "malformed gron line: {s:?}"),
            UngronError::InvalidValue(s) => write!(f, "invalid JSON value: {s:?}"),
            UngronError::IndexTooLarge(i) => write!(f, "array index too large: {i}"),
            UngronError::TooDeep(d) => write!(f, "path nested too deeply: {d}"),
        }
    }
}

impl std::error::Error for UngronError {}

/// Reconstruct a [`Value`] from gron assignment lines.
///
/// # Errors
///
/// Returns [`UngronError`] if a line is malformed, a value is not valid JSON, or
/// a path conflicts with an earlier one.
pub fn ungron(text: &str) -> Result<Value, UngronError> {
    let mut root = Value::Null;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let (segs, value) = parse_line(line)?;
        assign(&mut root, &segs, value)?;
    }
    Ok(root)
}

/// Split a line into its path segments and JSON value.
fn parse_line(line: &str) -> Result<(Vec<Seg>, Value), UngronError> {
    let chars: Vec<char> = line.trim().chars().collect();

    // Find the top-level ` = ` separator (ignoring brackets and strings).
    let mut i = 0;
    let mut in_str = false;
    let mut depth = 0i32;
    let mut sep = None;
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => in_str = true,
            '[' => depth += 1,
            ']' => depth -= 1,
            ' ' if depth == 0
                && chars.get(i + 1) == Some(&'=')
                && chars.get(i + 2) == Some(&' ') =>
            {
                sep = Some(i);
                break;
            }
            _ => {}
        }
        i += 1;
    }
    let sep = sep.ok_or_else(|| UngronError::Syntax(line.to_owned()))?;

    let path_str: String = chars[..sep].iter().collect();
    let raw_value: String = chars[sep + 3..].iter().collect();
    // Strip exactly one trailing `;` statement terminator (not any inside strings).
    let value_str = raw_value.trim();
    let value_str = value_str.strip_suffix(';').unwrap_or(value_str).trim();
    let value: Value = serde_json::from_str(value_str)
        .map_err(|_| UngronError::InvalidValue(value_str.to_owned()))?;

    Ok((parse_path(&path_str)?, value))
}

/// Parse a path like `json.a[0]["x y"]` into segments (the root name is dropped).
fn parse_path(s: &str) -> Result<Vec<Seg>, UngronError> {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let err = || UngronError::Syntax(s.to_owned());

    // Root identifier.
    let mut i = 0;
    let start = i;
    if i < n && is_ident_start(chars[i]) {
        i += 1;
        while i < n && is_ident_continue(chars[i]) {
            i += 1;
        }
    }
    if i == start {
        return Err(err());
    }

    let mut segs = Vec::new();
    while i < n {
        match chars[i] {
            '.' => {
                i += 1;
                let key_start = i;
                while i < n && is_ident_continue(chars[i]) {
                    i += 1;
                }
                if i == key_start {
                    return Err(err());
                }
                segs.push(Seg::Key(chars[key_start..i].iter().collect()));
            }
            '[' => {
                i += 1;
                if chars.get(i) == Some(&'"') {
                    let q_start = i;
                    i += 1;
                    while i < n {
                        if chars[i] == '\\' {
                            i += 2;
                        } else if chars[i] == '"' {
                            i += 1;
                            break;
                        } else {
                            i += 1;
                        }
                    }
                    if chars.get(i) != Some(&']') {
                        return Err(err());
                    }
                    let quoted: String = chars[q_start..i].iter().collect();
                    i += 1;
                    let key: String = serde_json::from_str(&quoted).map_err(|_| err())?;
                    segs.push(Seg::Key(key));
                } else {
                    let num_start = i;
                    while i < n && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    if i == num_start || chars.get(i) != Some(&']') {
                        return Err(err());
                    }
                    let idx: usize = chars[num_start..i]
                        .iter()
                        .collect::<String>()
                        .parse()
                        .map_err(|_| err())?;
                    i += 1;
                    segs.push(Seg::Index(idx));
                }
            }
            _ => return Err(err()),
        }
    }
    Ok(segs)
}

/// Assign `value` at `segs` within `root`, creating containers as needed.
///
/// Reconstruction is last-write-wins: a later assignment replaces whatever was
/// at a path before (incompatible types included), which keeps behavior
/// order-consistent for any input.
fn assign(root: &mut Value, segs: &[Seg], value: Value) -> Result<(), UngronError> {
    if segs.len() > MAX_DEPTH {
        return Err(UngronError::TooDeep(segs.len()));
    }
    let mut node = root;
    for seg in segs {
        node = descend(node, seg)?;
    }
    match value {
        Value::Object(ref m) if m.is_empty() => ensure(node, true),
        Value::Array(ref a) if a.is_empty() => ensure(node, false),
        other => *node = other,
    }
    Ok(())
}

fn descend<'a>(node: &'a mut Value, seg: &Seg) -> Result<&'a mut Value, UngronError> {
    match seg {
        Seg::Key(k) => {
            if !node.is_object() {
                *node = Value::Object(Map::new());
            }
            let map = node.as_object_mut().expect("just ensured object");
            Ok(map.entry(k.clone()).or_insert(Value::Null))
        }
        Seg::Index(idx) => {
            if *idx > MAX_INDEX {
                return Err(UngronError::IndexTooLarge(*idx));
            }
            if !node.is_array() {
                *node = Value::Array(Vec::new());
            }
            let arr = node.as_array_mut().expect("just ensured array");
            while arr.len() <= *idx {
                arr.push(Value::Null);
            }
            Ok(&mut arr[*idx])
        }
    }
}

/// Ensure `node` is an object (`want_object`) or array, replacing any other value.
fn ensure(node: &mut Value, want_object: bool) {
    let ok = if want_object {
        node.is_object()
    } else {
        node.is_array()
    };
    if !ok {
        *node = if want_object {
            Value::Object(Map::new())
        } else {
            Value::Array(Vec::new())
        };
    }
}

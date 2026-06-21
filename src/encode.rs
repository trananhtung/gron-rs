//! Encode a `serde_json::Value` into gron assignment lines.

use serde_json::Value;

use crate::ident::is_bare_identifier;

pub(crate) fn gron(value: &Value, root: &str) -> String {
    let mut out = String::new();
    // Iterative DFS (no recursion) so arbitrarily deep values can't overflow the
    // stack. Children are pushed in reverse so they pop in document order.
    let mut stack: Vec<(String, &Value)> = vec![(root.to_owned(), value)];
    while let Some((path, value)) = stack.pop() {
        match value {
            Value::Object(map) => {
                out.push_str(&path);
                out.push_str(" = {};\n");
                for (key, child) in map.iter().rev() {
                    stack.push((format!("{path}{}", accessor(key)), child));
                }
            }
            Value::Array(arr) => {
                out.push_str(&path);
                out.push_str(" = [];\n");
                for (i, child) in arr.iter().enumerate().rev() {
                    stack.push((format!("{path}[{i}]"), child));
                }
            }
            scalar => {
                out.push_str(&path);
                out.push_str(" = ");
                out.push_str(&escape_extras(&scalar.to_string()));
                out.push_str(";\n");
            }
        }
    }
    out
}

/// Render a key as a path accessor: `.key` for identifiers, `["key"]` otherwise.
fn accessor(key: &str) -> String {
    if is_bare_identifier(key) {
        format!(".{key}")
    } else {
        // JSON-encode the key (with quotes + escaping) inside brackets.
        format!(
            "[{}]",
            escape_extras(&Value::String(key.to_owned()).to_string())
        )
    }
}

/// Escape U+2028 / U+2029, which `serde_json` leaves raw but gron escapes because
/// they are line terminators in JavaScript/JSON-as-JS contexts.
fn escape_extras(s: &str) -> String {
    if s.contains(['\u{2028}', '\u{2029}']) {
        s.replace('\u{2028}', "\\u2028")
            .replace('\u{2029}', "\\u2029")
    } else {
        s.to_owned()
    }
}

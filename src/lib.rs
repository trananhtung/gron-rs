//! # gron — make JSON greppable
//!
//! Flatten a JSON document into discrete, greppable assignment lines, and
//! reconstruct it back — a Rust port of [gron](https://github.com/tomnomnom/gron),
//! available both as a **library** and a `gron` **CLI**.
//!
//! ```
//! use serde_json::json;
//!
//! let v = json!({ "user": { "name": "Tom", "tags": ["a", "b"] } });
//! let lines = gron::gron(&v);
//! assert_eq!(lines, "\
//! json = {};
//! json.user = {};
//! json.user.name = \"Tom\";
//! json.user.tags = [];
//! json.user.tags[0] = \"a\";
//! json.user.tags[1] = \"b\";
//! ");
//!
//! // …and back again.
//! assert_eq!(gron::ungron(&lines).unwrap(), v);
//! ```
//!
//! Flat assignment lines are perfect for `grep`, `diff`, and `sed` over JSON.

#![doc(html_root_url = "https://docs.rs/gron-rs/0.1.0")]

use serde_json::Value;

mod decode;
mod encode;
mod ident;

pub use decode::{ungron, UngronError};

/// Flatten `value` into gron assignment lines using the default root `json`.
///
/// ```
/// assert_eq!(gron::gron(&serde_json::json!(42)), "json = 42;\n");
/// ```
#[must_use]
pub fn gron(value: &Value) -> String {
    gron_with_root(value, "json")
}

/// Flatten `value` into gron assignment lines using a custom root identifier.
#[must_use]
pub fn gron_with_root(value: &Value, root: &str) -> String {
    encode::gron(value, root)
}

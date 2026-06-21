# gron-rs

[![Crates.io](https://img.shields.io/crates/v/gron-rs.svg)](https://crates.io/crates/gron-rs)
[![Documentation](https://docs.rs/gron-rs/badge.svg)](https://docs.rs/gron-rs)
[![CI](https://github.com/trananhtung/gron-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/trananhtung/gron-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/gron-rs.svg)](#license)

**Make JSON greppable.** Flatten a JSON document into discrete assignment lines
you can `grep`, `diff`, and `sed` — then turn them back into JSON. A Rust port of
[gron](https://github.com/tomnomnom/gron), available as both a **library** and a
`gron` **CLI**.

```text
$ echo '{"user":{"name":"Tom","tags":["a","b"]}}' | gron
json = {};
json.user = {};
json.user.name = "Tom";
json.user.tags = [];
json.user.tags[0] = "a";
json.user.tags[1] = "b";

$ echo '{"user":{"name":"Tom"}}' | gron | grep name
json.user.name = "Tom";

$ gron file.json | grep '\.user' | gron --ungron   # filter, then rebuild JSON
```

## Why gron-rs?

`gron` makes deeply-nested JSON diffable and greppable — but Rust's existing gron
crates are all stale (2017–2022), one-way, or CLI-only with **no library API**.
`gron-rs` is a maintained, round-trip (`gron` + `ungron`) implementation you can
use both ways.

## Library

```toml
[dependencies]
gron-rs = "0.1"
```

```rust
use serde_json::json;

let v = json!({ "name": "Tom", "tags": ["a", "b"] });
let lines = gron::gron(&v);                  // flatten
let back = gron::ungron(&lines).unwrap();    // reconstruct
assert_eq!(back, v);
```

| Function | Purpose |
| --- | --- |
| `gron(&Value) -> String` | Flatten to assignment lines (root `json`) |
| `gron_with_root(&Value, root) -> String` | Flatten with a custom root identifier |
| `ungron(&str) -> Result<Value, UngronError>` | Reconstruct JSON from gron lines |

## CLI

```sh
cargo install gron-rs        # installs the `gron` binary

gron file.json               # flatten (or pipe via stdin)
gron -u                      # --ungron: reconstruct JSON from stdin
gron --root data file.json   # custom root identifier
```

## Behavior

- Object keys that are valid identifiers use `.key`; others are `["quoted"]`.
- Output is in structural order (objects by sorted key, arrays by index) —
  stable and diff-friendly.
- `ungron` reconstructs containers even if the empty-container lines are missing
  (inferred from the paths), and round-trips any `gron` output exactly.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at
your option.

# gron-rs

[![All Contributors](https://img.shields.io/badge/all_contributors-1-orange.svg?style=flat-square)](#contributors-)

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

## Contributors ✨

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind are welcome — code, docs, bug reports, ideas, reviews! See the [emoji key](https://allcontributors.org/docs/en/emoji-key) for how each contribution is recognized, and open a PR or issue to get involved.

Thanks goes to these wonderful people:

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/trananhtung"><img src="https://avatars.githubusercontent.com/u/30992229?v=4?s=100" width="100px;" alt="Tung Tran"/><br /><sub><b>Tung Tran</b></sub></a><br /><a href="https://github.com/trananhtung/gron-rs/commits?author=trananhtung" title="Code">💻</a> <a href="#maintenance-trananhtung" title="Maintenance">🚧</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at
your option.

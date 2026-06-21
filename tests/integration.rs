//! End-to-end behavioral spec for the public `gron` API.

use gron::{gron, gron_with_root, ungron};
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// gron (Value -> assignment lines)
// ---------------------------------------------------------------------------

#[test]
fn gron_simple_object() {
    let v = json!({ "name": "Tom", "age": 30 });
    assert_eq!(
        gron(&v),
        "json = {};\njson.age = 30;\njson.name = \"Tom\";\n"
    );
}

#[test]
fn gron_nested_and_array() {
    let v = json!({ "ok": true, "tags": ["a", "b"] });
    assert_eq!(
        gron(&v),
        "json = {};\njson.ok = true;\njson.tags = [];\njson.tags[0] = \"a\";\njson.tags[1] = \"b\";\n"
    );
}

#[test]
fn gron_root_scalars() {
    assert_eq!(gron(&json!(42)), "json = 42;\n");
    assert_eq!(gron(&json!("hi")), "json = \"hi\";\n");
    assert_eq!(gron(&json!(null)), "json = null;\n");
    assert_eq!(gron(&json!(true)), "json = true;\n");
}

#[test]
fn gron_non_identifier_keys_are_bracket_quoted() {
    let v = json!({ "a-b": 1, "valid": 2 });
    assert_eq!(
        gron(&v),
        "json = {};\njson[\"a-b\"] = 1;\njson.valid = 2;\n"
    );
}

#[test]
fn gron_custom_root() {
    assert_eq!(
        gron_with_root(&json!({ "x": 1 }), "obj"),
        "obj = {};\nobj.x = 1;\n"
    );
}

// ---------------------------------------------------------------------------
// ungron (lines -> Value)
// ---------------------------------------------------------------------------

#[test]
fn ungron_basic_object() {
    let got = ungron("json = {};\njson.a = 1;\njson.b = \"x\";\n").unwrap();
    assert_eq!(got, json!({ "a": 1, "b": "x" }));
}

#[test]
fn ungron_infers_containers_from_paths() {
    // No explicit `json.a = []` line — the array is inferred from the indices.
    let got = ungron("json.a[0] = 1;\njson.a[1] = 2;\n").unwrap();
    assert_eq!(got, json!({ "a": [1, 2] }));
}

#[test]
fn ungron_root_scalar() {
    assert_eq!(ungron("json = 42;\n").unwrap(), json!(42));
    assert_eq!(ungron("").unwrap(), Value::Null);
}

#[test]
fn ungron_bracket_quoted_key() {
    let got = ungron("json[\"a-b\"] = 1;\n").unwrap();
    assert_eq!(got, json!({ "a-b": 1 }));
}

#[test]
fn ungron_errors() {
    assert!(ungron("this is not gron").is_err());
    assert!(ungron("json = oops;\n").is_err()); // value is not valid JSON
}

// ---------------------------------------------------------------------------
// Regression tests from the adversarial pre-publish review
// ---------------------------------------------------------------------------

#[test]
fn reserved_words_are_bracket_quoted_like_gron() {
    assert_eq!(
        gron(&json!({ "true": 1 })),
        "json = {};\njson[\"true\"] = 1;\n"
    );
    assert_eq!(
        gron(&json!({ "for": 1 })),
        "json = {};\njson[\"for\"] = 1;\n"
    );
}

#[test]
fn dollar_and_unicode_keys_are_bare_like_gron() {
    assert_eq!(gron(&json!({ "$ref": 1 })), "json = {};\njson.$ref = 1;\n");
    assert_eq!(gron(&json!({ "café": 1 })), "json = {};\njson.café = 1;\n");
    for v in [
        json!({ "true": 1 }),
        json!({ "café": [1] }),
        json!({ "$ref": { "a": 2 } }),
    ] {
        assert_eq!(ungron(&gron(&v)).unwrap(), v, "round trip failed");
    }
}

#[test]
fn line_separators_are_escaped() {
    let g = gron(&json!("a\u{2028}b\u{2029}c"));
    assert!(
        !g.contains('\u{2028}') && !g.contains('\u{2029}'),
        "raw separators: {g:?}"
    );
    assert_eq!(ungron(&g).unwrap(), json!("a\u{2028}b\u{2029}c"));
}

#[test]
fn huge_array_index_is_rejected() {
    assert!(ungron("json[100000000] = 1;").is_err());
    assert!(ungron("json[18446744073709551615] = 1;").is_err());
    assert_eq!(
        ungron("json[3] = 1;").unwrap(),
        json!([null, null, null, 1])
    );
}

#[test]
fn excessive_nesting_is_rejected_not_aborted() {
    let deep = format!("json{} = 1;", ".a".repeat(5000));
    assert!(ungron(&deep).is_err());
    let shallow = format!("json{} = 1;", ".a".repeat(50));
    assert!(ungron(&shallow).is_ok());
}

#[test]
fn conflicting_assignments_are_last_write_wins() {
    assert_eq!(
        ungron("json.a = {};\njson.a = 1;\n").unwrap(),
        json!({ "a": 1 })
    );
    assert_eq!(
        ungron("json.a = 1;\njson.a = {};\n").unwrap(),
        json!({ "a": {} })
    );
    assert_eq!(
        ungron("json.a = 1;\njson.a.b = 2;\n").unwrap(),
        json!({ "a": { "b": 2 } })
    );
}

// ---------------------------------------------------------------------------
// Round-trip
// ---------------------------------------------------------------------------

#[test]
fn round_trip_preserves_value() {
    let cases = [
        json!({ "name": "Tom", "contact": { "email": "t@e.com" }, "tags": ["x", "y"], "n": 3, "b": false, "z": null }),
        json!([1, 2, [3, 4], { "k": "v" }]),
        json!({ "weird key!": [true, { "deep": [null, 1.5] }], "empty_obj": {}, "empty_arr": [] }),
        json!("just a string"),
        json!(123.456),
    ];
    for v in cases {
        let text = gron(&v);
        let back = ungron(&text).unwrap();
        assert_eq!(back, v, "round trip failed for {v}\n--- gron ---\n{text}");
    }
}

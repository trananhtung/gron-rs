//! JavaScript-identifier rules, matching `tomnomnom/gron` so output is
//! bare-vs-bracketed compatibly.

/// JavaScript reserved words, which gron always renders in bracket form.
const RESERVED: &[&str] = &[
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

pub(crate) fn is_ident_start(c: char) -> bool {
    c == '$' || c == '_' || c.is_alphabetic()
}

pub(crate) fn is_ident_continue(c: char) -> bool {
    c == '$' || c == '_' || c.is_alphanumeric()
}

/// Whether `key` can be written as a bare `.key` accessor (valid identifier and
/// not a reserved word); otherwise it must be bracket-quoted.
pub(crate) fn is_bare_identifier(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if is_ident_start(c) => {}
        _ => return false,
    }
    if !chars.all(is_ident_continue) {
        return false;
    }
    !RESERVED.contains(&key)
}

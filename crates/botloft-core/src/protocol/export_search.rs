//! The search part of the TypeScript bindings (spec 8.8).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

/// After `ChatItem`, which `SearchHit` holds.
pub(super) fn decls(out: &mut Out) {
    let mark = |c: char| format!("\\u{:04x}", u32::from(c));
    out.text
        .push_str("/** What a search snippet puts around each matched word (spec 8.8). */\n");
    let _ = writeln!(
        out.text,
        "export const SNIPPET_MARKS = {{ open: \"{}\", close: \"{}\" }} as const;\n",
        mark(SNIPPET_OPEN),
        mark(SNIPPET_CLOSE)
    );
    out.decl::<ChatSearchParams>();
    out.decl::<SearchHit>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::CHAT_SEARCH,
        &out.name::<ChatSearchParams>(),
        &out.name::<Vec<SearchHit>>(),
    );
}

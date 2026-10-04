//! The reactions part of the TypeScript bindings (spec 8.9).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    let emoji: Vec<String> = REACTIONS
        .iter()
        .map(|emoji| format!("\"{emoji}\""))
        .collect();
    let _ = writeln!(
        out.text,
        "/** The emoji the owner can react with (spec 8.9). */
export const REACTIONS = [{}] as const;
",
        emoji.join(", ")
    );
    out.decl::<Reaction>();
    out.decl::<ReactionsListParams>();
    out.decl::<ReactionsSetParams>();
    out.decl::<ReactionChanged>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::REACTIONS_LIST,
        &out.name::<ReactionsListParams>(),
        &out.name::<Vec<Reaction>>(),
    );
    out.method(
        method::REACTIONS_SET,
        &out.name::<ReactionsSetParams>(),
        &out.name::<Option<Reaction>>(),
    );
}

pub(super) fn notifications(out: &mut Out) {
    let changed = out.name::<ReactionChanged>();
    let _ = writeln!(
        out.text,
        "  \"{}\": {changed};",
        notification::REACTION_CHANGED
    );
}

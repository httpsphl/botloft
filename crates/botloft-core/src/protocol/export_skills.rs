//! The skills part of the TypeScript bindings (spec 38).

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<SkillSource>();
    out.decl::<Skill>();
    out.decl::<SkillList>();
    out.decl::<SkillsShareParams>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(method::SKILLS_LIST, "undefined", &out.name::<SkillList>());
    out.method(
        method::SKILLS_SHARE,
        &out.name::<SkillsShareParams>(),
        &out.name::<Skill>(),
    );
}

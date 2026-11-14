//! The skills part of the TypeScript bindings (spec 38).

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<SkillSource>();
    out.decl::<Skill>();
    out.decl::<SkillList>();
    out.decl::<SkillsShareParams>();
    out.decl::<SkillsEveryoneParams>();
    out.decl::<SlashSource>();
    out.decl::<SlashCommand>();
    out.decl::<SlashCommandList>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(method::SKILLS_LIST, "undefined", &out.name::<SkillList>());
    out.method(
        method::SKILLS_SHARE,
        &out.name::<SkillsShareParams>(),
        &out.name::<Skill>(),
    );
    out.method(
        method::SKILLS_EVERYONE,
        &out.name::<SkillsEveryoneParams>(),
        &out.name::<SkillList>(),
    );
    out.method(
        method::BOTS_COMMANDS,
        &out.name::<BotIdParams>(),
        &out.name::<SlashCommandList>(),
    );
}

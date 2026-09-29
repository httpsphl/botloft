//! Crew folders and bot workspaces on disk (spec 5.1). Generation is
//! idempotent: running it again refreshes the generated files and keeps the
//! bot's own `CLAUDE.md`.

mod files;
pub mod folder;

use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use botloft_core::protocol::Crew;
use botloft_store::BotRecord;

use crate::paths::Paths;

/// What generated files need to know about the running daemon.
#[derive(Debug, Clone, Copy)]
pub struct WorkspaceEnv<'a> {
    pub paths: &'a Paths,
    pub port: u16,
    /// How long the MCP server may hold a permission request (spec 10).
    pub approval_timeout: Duration,
}

/// Creates the crew folder, its `shared` folder and the work folder the
/// owner chose, if it went missing.
pub fn prepare_crew(paths: &Paths, crew: &Crew) -> io::Result<()> {
    std::fs::create_dir_all(paths.shared_dir(&crew.slug))?;
    std::fs::create_dir_all(paths.work_folder(crew))
}

/// Creates the workspace and writes every generated file. Returns the path.
pub fn prepare_bot(env: WorkspaceEnv<'_>, crew: &Crew, bot: &BotRecord) -> io::Result<PathBuf> {
    prepare_crew(env.paths, crew)?;
    let dir = env.paths.bot_workspace(&crew.slug, &bot.slug);
    std::fs::create_dir_all(dir.join(".claude").join("rules"))?;
    std::fs::create_dir_all(dir.join(".botloft"))?;

    let memory = dir.join("CLAUDE.md");
    if !memory.exists() {
        write_atomic(&memory, files::CLAUDE_MD.as_bytes())?;
    }
    write_json(
        &dir.join(".claude").join("settings.json"),
        &files::settings_json(&env.paths.home),
    )?;
    write_json(
        &dir.join(".botloft").join("mcp.json"),
        &files::mcp_json(env.port, env.approval_timeout),
    )?;
    write_rules(env.paths, crew, bot)?;
    Ok(dir)
}

/// Rewrites `.claude/rules/botloft.md` after the bot or its crew changed.
pub fn write_rules(paths: &Paths, crew: &Crew, bot: &BotRecord) -> io::Result<()> {
    let dir = paths.bot_workspace(&crew.slug, &bot.slug);
    let rules_dir = dir.join(".claude").join("rules");
    std::fs::create_dir_all(&rules_dir)?;
    let text = files::rules_md(crew, bot, &paths.work_folder(crew));
    write_atomic(&rules_dir.join("botloft.md"), text.as_bytes())
}

fn write_json(path: &Path, value: &serde_json::Value) -> io::Result<()> {
    let mut text = serde_json::to_string_pretty(value).map_err(io::Error::other)?;
    text.push('\n');
    write_atomic(path, text.as_bytes())
}

/// Writes through a temporary file and a rename, so a crash never leaves a
/// half-written file for Claude Code to read.
fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use botloft_core::ids::{BotId, CrewId};

    use super::*;

    fn fixture() -> (tempfile::TempDir, Paths, Crew, BotRecord) {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = Paths::new(dir.path().join("home"), dir.path().join("ws"));
        let crew = Crew {
            id: CrewId::generate(),
            name: "Site".to_owned(),
            slug: "site".to_owned(),
            work_folder: String::new(),
            work_folder_chosen: false,
            paused: false,
            created_at: 0,
            archived_at: None,
        };
        let bot = BotRecord {
            id: BotId::generate(),
            crew_id: crew.id.clone(),
            name: "Writer".to_owned(),
            handle: "writer".to_owned(),
            slug: "writer".to_owned(),
            role: "writes docs".to_owned(),
            instructions: String::new(),
            color: "#FF7A59".to_owned(),
            paused: false,
            permission_mode: botloft_core::protocol::PermissionMode::Default,
            model: botloft_core::protocol::BotModel::Default,
            model_in_use: None,
            created_at: 0,
            archived_at: None,
        };
        (dir, paths, crew, bot)
    }

    #[test]
    fn prepares_every_generated_file() {
        let (_dir, paths, crew, bot) = fixture();
        let env = WorkspaceEnv {
            paths: &paths,
            port: 45710,
            approval_timeout: Duration::from_secs(3600),
        };
        let ws = prepare_bot(env, &crew, &bot).expect("prepare");

        assert_eq!(ws, paths.bot_workspace("site", "writer"));
        assert!(paths.shared_dir("site").is_dir());
        for file in [
            "CLAUDE.md",
            ".claude/settings.json",
            ".claude/rules/botloft.md",
            ".botloft/mcp.json",
        ] {
            assert!(ws.join(file).is_file(), "{file} missing");
        }
        let settings: serde_json::Value =
            serde_json::from_slice(&std::fs::read(ws.join(".claude/settings.json")).expect("read"))
                .expect("valid json");
        assert!(settings["permissions"]["deny"].is_array());
    }

    #[test]
    fn regenerating_keeps_the_bot_memory_and_refreshes_rules() {
        let (_dir, paths, crew, mut bot) = fixture();
        let env = WorkspaceEnv {
            paths: &paths,
            port: 45710,
            approval_timeout: Duration::from_secs(3600),
        };
        let ws = prepare_bot(env, &crew, &bot).expect("prepare");
        std::fs::write(ws.join("CLAUDE.md"), "my notes").expect("edit memory");

        bot.name = "Lead Writer".to_owned();
        bot.handle = "lead-writer".to_owned();
        prepare_bot(env, &crew, &bot).expect("prepare again");

        let memory = std::fs::read_to_string(ws.join("CLAUDE.md")).expect("memory");
        assert_eq!(memory, "my notes");
        let rules = std::fs::read_to_string(ws.join(".claude/rules/botloft.md")).expect("rules");
        assert!(rules.contains("`@lead-writer`"));
        assert!(!ws.join(".claude/rules/botloft.md.tmp").exists());
    }
}

//! Crew folders and bot workspaces on disk (spec 5.1). Generation is
//! idempotent: running it again refreshes the generated files and keeps the
//! bot's own `CLAUDE.md`.

mod connected;
mod files;
pub mod folder;
mod memory;

use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use botloft_core::protocol::{Crew, McpServer};
use botloft_store::BotRecord;

use crate::paths::Paths;

pub use connected::connected_var;

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

/// What a bot of `crew` must not read or edit with its own tools (spec 7.5):
/// Botloft's data and the folders of every other crew in `crews`, archived
/// ones too. A folder that holds one of the crew's own is left out, since
/// a deny rule would win over it; when Botloft's data holds the bots'
/// folders, only its own parts are fenced.
pub fn fences(paths: &Paths, crew: &Crew, crews: &[Crew]) -> Vec<PathBuf> {
    let own = [paths.crew_dir(&crew.slug), paths.work_folder(crew)];
    let apart = |folder: &PathBuf| !own.iter().any(|mine| folder::overlaps(folder, mine));
    let mut fenced = Vec::new();
    if apart(&paths.home) && !folder::contains(&paths.home, &paths.workspaces_root) {
        fenced.push(paths.home.clone());
    } else {
        let parts = ["browsers", "logs", "run", "local-backup"];
        fenced.extend(parts.iter().map(|part| paths.home.join(part)));
    }
    for other in crews.iter().filter(|other| other.id != crew.id) {
        let mut theirs = vec![paths.crew_dir(&other.slug)];
        if other.work_folder_chosen {
            theirs.push(PathBuf::from(&other.work_folder));
        }
        for folder in theirs {
            if apart(&folder) && !fenced.contains(&folder) {
                fenced.push(folder);
            }
        }
    }
    fenced
}

/// Creates the workspace and writes every generated file. `crews` are all
/// the crews, for `fences`, and `servers` the connected tools of the bot
/// (spec 25). Returns the path.
pub fn prepare_bot(
    env: WorkspaceEnv<'_>,
    crew: &Crew,
    crews: &[Crew],
    bot: &BotRecord,
    servers: &[McpServer],
) -> io::Result<PathBuf> {
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
        &files::settings_json(
            &env.paths.home,
            &env.paths.crew_dir(&crew.slug),
            &fences(env.paths, crew, crews),
        ),
    )?;
    let mut mcp = files::mcp_json(env.port, env.approval_timeout);
    connected::add_to(&mut mcp, servers);
    write_json(&dir.join(".botloft").join("mcp.json"), &mcp)?;
    write_rules(env.paths, crew, bot, servers)?;
    Ok(dir)
}

/// Rewrites `.claude/rules/botloft.md` after the bot or its crew changed.
pub fn write_rules(
    paths: &Paths,
    crew: &Crew,
    bot: &BotRecord,
    servers: &[McpServer],
) -> io::Result<()> {
    let dir = paths.bot_workspace(&crew.slug, &bot.slug);
    let rules_dir = dir.join(".claude").join("rules");
    std::fs::create_dir_all(&rules_dir)?;
    let mut text = files::rules_md(crew, bot, &paths.work_folder(crew));
    text.push_str(&connected::rules_section(servers));
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
            lead_bot_id: None,
            paused: false,
            color: None,
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
            agent: botloft_core::protocol::AgentKind::Claude,
            model: botloft_core::protocol::BotModel::Default,
            model_in_use: None,
            effort: botloft_core::protocol::BotEffort::Default,
            effort_default: None,
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
        let ws = prepare_bot(env, &crew, &[], &bot, &[]).expect("prepare");

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
        // Instruction files are skipped from the folder of every crew up,
        // never in the crew's own folder.
        assert_eq!(
            settings["claudeMdExcludes"],
            serde_json::json!(memory::excludes_above(&paths.crew_dir("site")))
        );
        let root = paths.workspaces_root.to_string_lossy().replace('\\', "/");
        assert_eq!(
            settings["claudeMdExcludes"][0],
            format!("{root}/CLAUDE.md").as_str()
        );
    }

    #[test]
    fn regenerating_keeps_the_bot_memory_and_refreshes_rules() {
        let (_dir, paths, crew, mut bot) = fixture();
        let env = WorkspaceEnv {
            paths: &paths,
            port: 45710,
            approval_timeout: Duration::from_secs(3600),
        };
        let ws = prepare_bot(env, &crew, &[], &bot, &[]).expect("prepare");
        std::fs::write(ws.join("CLAUDE.md"), "my notes").expect("edit memory");

        bot.name = "Lead Writer".to_owned();
        bot.handle = "lead-writer".to_owned();
        prepare_bot(env, &crew, &[], &bot, &[]).expect("prepare again");

        let memory = std::fs::read_to_string(ws.join("CLAUDE.md")).expect("memory");
        assert_eq!(memory, "my notes");
        let rules = std::fs::read_to_string(ws.join(".claude/rules/botloft.md")).expect("rules");
        assert!(rules.contains("`@lead-writer`"));
        assert!(!ws.join(".claude/rules/botloft.md.tmp").exists());
    }

    #[test]
    fn a_bot_is_fenced_from_botloft_data_and_the_other_crews() {
        use botloft_core::ids::CrewId;
        let paths = Paths::new(
            PathBuf::from("/data/Botloft"),
            PathBuf::from("/home/Botloft"),
        );
        let crew = |name: &str, folder: Option<&str>| Crew {
            id: CrewId::generate(),
            name: name.to_owned(),
            slug: name.to_lowercase(),
            work_folder: folder.unwrap_or_default().to_owned(),
            work_folder_chosen: folder.is_some(),
            lead_bot_id: None,
            paused: false,
            color: None,
            created_at: 0,
            archived_at: None,
        };
        let site = crew("Site", Some("/projects/site"));
        let crews = [
            site.clone(),
            crew("Blog", None),
            crew("Shop", Some("/projects/shop")),
            // An old crew sharing a folder around Site's is left open.
            crew("Old", Some("/projects")),
        ];
        assert_eq!(
            fences(&paths, &site, &crews),
            [
                PathBuf::from("/data/Botloft"),
                PathBuf::from("/home/Botloft/blog"),
                PathBuf::from("/home/Botloft/shop"),
                PathBuf::from("/projects/shop"),
                PathBuf::from("/home/Botloft/old"),
            ]
        );

        // Bots' folders inside Botloft's data: only its own parts.
        let inside = Paths::new(PathBuf::from("/data"), PathBuf::from("/data/ws"));
        let fenced = fences(&inside, &crew("Blog", None), &[]);
        assert!(fenced.contains(&PathBuf::from("/data/browsers")));
        assert!(!fenced.contains(&PathBuf::from("/data")));
    }
}

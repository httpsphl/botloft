//! The folder a crew works in (spec 5): the `shared` folder by default, or
//! one the owner chose, handed to every bot with `--add-dir`.

mod common;

use std::path::Path;
use std::sync::Arc;

use botloft_core::protocol::{
    BotState, BotsCreateParams, Crew, CrewsCreateParams, CrewsSetWorkFolderParams,
};
use botloftd::service::{ApiError, bots, crews};
use botloftd::supervisor;
use common::supervised::arg_after;
use common::{Parts, new_daemon, test_settings};

fn crew(parts: &Parts, work_folder: Option<&Path>) -> Result<Crew, ApiError> {
    crews::create(
        &parts.daemon,
        CrewsCreateParams {
            name: "Site".into(),
            work_folder: work_folder.map(|path| path.display().to_string()),
            lead: None,
        },
    )
}

fn bot(parts: &Parts, crew: &Crew) -> botloft_core::protocol::Bot {
    bots::create(
        &parts.daemon,
        BotsCreateParams {
            crew_id: crew.id.clone(),
            name: "Writer".into(),
            role: String::new(),
            instructions: String::new(),
            color: None,
            model: None,
            agent: None,
        },
    )
    .expect("bot")
}

fn rules(bot: &botloft_core::protocol::Bot) -> String {
    std::fs::read_to_string(Path::new(&bot.workspace).join(".claude/rules/botloft.md"))
        .expect("rules")
}

#[tokio::test(start_paused = true)]
async fn a_crew_works_in_its_shared_folder_by_default() {
    let parts = new_daemon(test_settings());
    let crew = crew(&parts, None).expect("crew");
    let shared = parts.paths.shared_dir("site");
    assert!(!crew.work_folder_chosen);
    assert_eq!(Path::new(&crew.work_folder), shared);
    assert!(shared.is_dir());

    let writer = bot(&parts, &crew);
    tokio::spawn(supervisor::run(Arc::clone(&parts.daemon)));
    let process = parts.runtime.process(1).await;
    assert_eq!(
        arg_after(&process, "--add-dir").map(std::path::PathBuf::from),
        Some(shared.clone())
    );
    assert_eq!(
        process
            .env("CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD")
            .as_deref(),
        Some("1")
    );
    assert!(rules(&writer).contains(&shared.display().to_string()));
}

#[tokio::test(start_paused = true)]
async fn the_owner_can_choose_where_the_crew_works() {
    let parts = new_daemon(test_settings());
    let chosen = parts.dir.path().join("Projects").join("Site");
    let crew = crew(&parts, Some(&chosen)).expect("crew");
    assert!(crew.work_folder_chosen);
    assert_eq!(Path::new(&crew.work_folder), chosen);
    assert!(chosen.is_dir(), "a missing folder is created");

    let writer = bot(&parts, &crew);
    tokio::spawn(supervisor::run(Arc::clone(&parts.daemon)));
    let process = parts.runtime.process(1).await;
    assert_eq!(
        arg_after(&process, "--add-dir").map(std::path::PathBuf::from),
        Some(chosen.clone())
    );
    assert!(rules(&writer).contains(&chosen.display().to_string()));
    // The listing keeps the choice.
    let listed = crews::list(&parts.daemon).expect("list");
    assert_eq!(listed[0].work_folder, crew.work_folder);
}

#[tokio::test(start_paused = true)]
async fn moving_the_crew_restarts_its_bots_in_the_new_folder() {
    let parts = new_daemon(test_settings());
    let crew = crew(&parts, None).expect("crew");
    let writer = bot(&parts, &crew);
    tokio::spawn(supervisor::run(Arc::clone(&parts.daemon)));
    let first = parts.runtime.process(1).await;
    wait_idle(&parts, &writer.id).await;

    let moved = parts.dir.path().join("Elsewhere");
    let changed = crews::set_work_folder(
        &parts.daemon,
        CrewsSetWorkFolderParams {
            crew_id: crew.id.clone(),
            work_folder: Some(moved.display().to_string()),
        },
    )
    .expect("move");
    assert_eq!(Path::new(&changed.work_folder), moved);
    assert!(rules(&writer).contains(&moved.display().to_string()));
    let second = parts.runtime.process(2).await;
    assert!(first.killed());
    assert_eq!(
        arg_after(&second, "--add-dir").map(std::path::PathBuf::from),
        Some(moved)
    );

    wait_idle(&parts, &writer.id).await;
    let back = crews::set_work_folder(
        &parts.daemon,
        CrewsSetWorkFolderParams {
            crew_id: crew.id,
            work_folder: None,
        },
    )
    .expect("back");
    assert!(!back.work_folder_chosen);
    assert_eq!(Path::new(&back.work_folder), parts.paths.shared_dir("site"));
    parts.runtime.process(3).await;
}

#[tokio::test(start_paused = true)]
async fn botlofts_own_folders_are_refused() {
    let parts = new_daemon(test_settings());
    for folder in [
        parts.paths.home.join("secrets"),
        parts.paths.workspaces_root.clone(),
    ] {
        let refused = crew(&parts, Some(&folder)).expect_err("refused");
        assert!(matches!(refused, ApiError::Validation(_)), "{refused:?}");
    }
    assert!(crews::list(&parts.daemon).expect("list").is_empty());
}

async fn wait_idle(parts: &Parts, bot: &botloft_core::ids::BotId) {
    for _ in 0..600 {
        if parts.daemon.supervisor.status(bot).map(|(state, _)| state) == Some(BotState::Idle) {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("the bot never became idle");
}

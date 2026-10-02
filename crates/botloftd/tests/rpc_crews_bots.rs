//! Crews and bots over RPC, end to end: the M1 acceptance test (spec 17).

mod common;

use common::TestDaemon;
use serde_json::{Value, json};

const NOT_FOUND: i64 = -32002;
const CONFLICT: i64 = -32003;
const VALIDATION: i64 = -32004;

fn read(path: std::path::PathBuf) -> String {
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

#[tokio::test]
async fn create_a_crew_and_a_bot_and_get_a_ready_workspace() {
    let daemon = TestDaemon::start().await;
    let mut app = daemon.session().await;
    let mut watcher = daemon.session().await;

    let crew = app
        .call("crews.create", json!({ "name": "  Site da Loja " }))
        .await
        .expect("create crew");
    assert_eq!(crew["name"], "Site da Loja");
    assert_eq!(crew["slug"], "site-da-loja");
    assert!(crew["id"].as_str().is_some_and(|id| id.starts_with("crw_")));
    assert!(daemon.paths.shared_dir("site-da-loja").is_dir());
    assert_eq!(watcher.notification("crew.changed").await, crew);

    let bot = app
        .call(
            "bots.create",
            json!({
                "crewId": crew["id"],
                "name": "Revisão",
                "role": "Reviews every change before it ships",
                "instructions": "Be strict.\nExplain every finding.",
            }),
        )
        .await
        .expect("create bot");
    assert_eq!(bot["handle"], "revisao");
    assert_eq!(bot["state"], "offline");
    assert_eq!(bot["color"], "#FF7A59");
    assert_eq!(bot["lastActivity"], Value::Null);
    assert_eq!(bot["lastReplyAt"], Value::Null);
    assert_eq!(watcher.notification("bot.changed").await, bot);

    let ws = daemon.paths.bot_workspace("site-da-loja", "revisao");
    assert_eq!(bot["workspace"], ws.to_string_lossy().as_ref());
    let settings: Value =
        serde_json::from_str(&read(ws.join(".claude/settings.json"))).expect("json");
    assert_eq!(
        settings["hooks"],
        Value::Null,
        "headless bots need no hooks"
    );
    let deny = settings["permissions"]["deny"][0]
        .as_str()
        .expect("deny rule");
    assert!(
        deny.starts_with("Read(//") && deny.ends_with("/secrets/**)"),
        "{deny}"
    );
    // The owner's own Claude Code memory stays out of the bot (spec 7.5).
    let crew_dir = daemon.paths.crew_dir("site-da-loja");
    let crew_dir = crew_dir.to_string_lossy().replace('\\', "/");
    let excludes = settings["claudeMdExcludes"].as_array().expect("excludes");
    assert!(excludes.len() >= 6, "{excludes:?}");
    assert!(excludes.iter().all(|pattern| {
        let pattern = pattern.as_str().expect("pattern");
        !pattern.contains('\\') && !pattern.starts_with(crew_dir.as_str())
    }));
    assert_eq!(settings["autoMemoryEnabled"], false);
    let mcp: Value = serde_json::from_str(&read(ws.join(".botloft/mcp.json"))).expect("json");
    assert_eq!(
        mcp["mcpServers"]["botloft"]["url"],
        "http://127.0.0.1:45710/mcp"
    );
    let rules = read(ws.join(".claude/rules/botloft.md"));
    assert!(rules.contains("**Revisão** (`@revisao`)") && rules.contains("crew **Site da Loja**"));
    assert!(rules.contains("Explain every finding."));

    let listed = app
        .call("bots.list", json!({ "crewId": crew["id"] }))
        .await
        .expect("list");
    assert_eq!(listed, json!([bot]));
    assert_eq!(
        app.call("crews.list", json!(null)).await.expect("list"),
        json!([crew])
    );
}

#[tokio::test]
async fn updates_refresh_the_rules_and_keep_the_bot_memory() {
    let daemon = TestDaemon::start().await;
    let mut app = daemon.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Docs" }))
        .await
        .expect("crew");
    let bot = app
        .call("bots.create", json!({ "crewId": crew["id"], "name": "Writer", "role": "", "instructions": "", "color": "#5ec8ff" }))
        .await
        .expect("bot");
    assert_eq!(bot["color"], "#5EC8FF");
    let ws = daemon.paths.bot_workspace("docs", "writer");
    std::fs::write(ws.join("CLAUDE.md"), "notes the bot wrote").expect("edit memory");

    let updated = app
        .call("bots.update", json!({ "botId": bot["id"], "name": "Lead Writer", "instructions": "Short sentences." }))
        .await
        .expect("update");
    assert_eq!(updated["handle"], "lead-writer");
    assert_eq!(updated["slug"], "writer", "the folder never moves");
    assert_eq!(read(ws.join("CLAUDE.md")), "notes the bot wrote");
    assert!(read(ws.join(".claude/rules/botloft.md")).contains("Short sentences."));

    app.call(
        "crews.rename",
        json!({ "crewId": crew["id"], "name": "Docs Team" }),
    )
    .await
    .expect("rename");
    assert!(read(ws.join(".claude/rules/botloft.md")).contains("crew **Docs Team**"));

    let paused = app
        .call(
            "bots.setPaused",
            json!({ "botId": bot["id"], "paused": true }),
        )
        .await
        .expect("pause");
    assert_eq!(paused["paused"], true);
    let crew = app
        .call(
            "crews.setPaused",
            json!({ "crewId": crew["id"], "paused": true }),
        )
        .await
        .expect("pause");
    assert_eq!(crew["paused"], true);
}

#[tokio::test]
async fn archiving_a_crew_archives_its_bots() {
    let daemon = TestDaemon::start().await;
    let mut app = daemon.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Temp" }))
        .await
        .expect("crew");
    let bot = app
        .call(
            "bots.create",
            json!({ "crewId": crew["id"], "name": "A", "role": "", "instructions": "" }),
        )
        .await
        .expect("bot");

    let archived = app
        .call("crews.archive", json!({ "crewId": crew["id"] }))
        .await
        .expect("archive");
    assert!(archived["archivedAt"].is_i64());
    // The app hears its own changes too: first the bot's creation, then the
    // archive cascade.
    assert_eq!(app.notification("bot.changed").await, bot);
    let cascaded = app.notification("bot.changed").await;
    assert_eq!(cascaded["id"], bot["id"]);
    assert_eq!(cascaded["state"], "archived");

    assert_eq!(
        app.call("crews.list", json!(null)).await.expect("list"),
        json!([])
    );
    assert_eq!(
        app.call("bots.list", json!(null)).await.expect("list"),
        json!([])
    );
    let again = app
        .call("crews.archive", json!({ "crewId": crew["id"] }))
        .await
        .expect("idempotent");
    assert_eq!(again, archived);
    let err = app
        .call(
            "bots.create",
            json!({ "crewId": crew["id"], "name": "B", "role": "", "instructions": "" }),
        )
        .await
        .expect_err("archived crew");
    assert_eq!(err.code, CONFLICT);
    let err = app
        .call("bots.update", json!({ "botId": bot["id"], "role": "x" }))
        .await
        .expect_err("archived bot");
    assert_eq!(err.code, CONFLICT);
}

#[tokio::test]
async fn invalid_input_is_reported_with_the_validation_code() {
    let daemon = TestDaemon::start().await;
    let mut app = daemon.session().await;

    let err = app
        .call("crews.create", json!({ "name": "   " }))
        .await
        .expect_err("empty");
    assert_eq!(err.code, VALIDATION);
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let twin = app
        .call("crews.create", json!({ "name": "OPS" }))
        .await
        .expect("same name");
    assert_eq!(twin["slug"], "ops-2");

    let new_bot =
        |name: &str| json!({ "crewId": crew["id"], "name": name, "role": "", "instructions": "" });
    app.call("bots.create", new_bot("Scout"))
        .await
        .expect("bot");
    let err = app
        .call("bots.create", new_bot("scout!"))
        .await
        .expect_err("same handle");
    assert_eq!(err.code, VALIDATION);
    assert!(err.message.contains("@scout"), "{}", err.message);
    let shared = app
        .call("bots.create", new_bot("Shared"))
        .await
        .expect("reserved folder");
    assert_eq!(shared["slug"], "shared-2");

    let mut bad_color = new_bot("Painter");
    bad_color["color"] = json!("red");
    assert_eq!(
        app.call("bots.create", bad_color)
            .await
            .expect_err("color")
            .code,
        VALIDATION
    );

    let unknown = json!({ "crewId": "crw_01J9Z3K8M4Q7R2T5V8X1Y4Z6A0" });
    assert_eq!(
        app.call("bots.list", unknown.clone())
            .await
            .expect_err("missing")
            .code,
        NOT_FOUND
    );
    assert_eq!(
        app.call("crews.archive", unknown)
            .await
            .expect_err("missing")
            .code,
        NOT_FOUND
    );
}

//! The bot catalog over RPC (spec 26.4): listing, reading and adding a role.

mod common;

use common::TestDaemon;
use serde_json::{Value, json};

const NOT_FOUND: i64 = -32002;
const CONFLICT: i64 = -32003;
const VALIDATION: i64 = -32004;

async fn crew(app: &mut common::Client) -> Value {
    app.call("crews.create", json!({ "name": "Studio" }))
        .await
        .expect("crew")
}

#[tokio::test]
async fn the_catalog_lists_every_role_without_its_instructions() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;

    let all = app.call("catalog.list", json!({})).await.expect("list");
    let all = all.as_array().expect("array");
    assert_eq!(all.len(), 52);
    let ids: Vec<_> = all
        .iter()
        .filter_map(|entry| entry["id"].as_str())
        .collect();
    assert_eq!(ids[0], "developer");
    assert!(ids.contains(&"code-reviewer") && ids.contains(&"personal-assistant"));
    for entry in all {
        assert!(entry["name"].as_str().is_some_and(|name| !name.is_empty()));
        assert!(entry["summary"].is_string() && entry["category"].is_string());
        assert!(entry.get("instructions").is_none(), "the list is light");
    }

    let code = app
        .call("catalog.list", json!({ "category": "code" }))
        .await
        .expect("list code");
    let code: Vec<_> = code
        .as_array()
        .expect("array")
        .iter()
        .map(|entry| entry["id"].as_str().expect("id"))
        .collect();
    assert_eq!(
        code,
        [
            "developer",
            "code-reviewer",
            "qa-tester",
            "software-architect",
            "frontend-developer",
            "backend-developer",
            "mobile-developer",
            "database-engineer",
            "devops-engineer",
            "security-reviewer",
            "data-engineer",
        ]
    );

    let product = app
        .call("catalog.list", json!({ "category": "product" }))
        .await
        .expect("list product");
    assert_eq!(product.as_array().expect("array").len(), 14);

    let marketing = app
        .call("catalog.list", json!({ "category": "marketing" }))
        .await
        .expect("list marketing");
    assert_eq!(marketing.as_array().expect("array").len(), 18);

    let bad = app
        .call("catalog.list", json!({ "category": "cooking" }))
        .await;
    assert!(bad.is_err(), "an unknown category is refused");
}

#[tokio::test]
async fn one_role_comes_with_its_instructions_model_and_effort() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;

    let sheet = app
        .call("catalog.get", json!({ "id": "code-reviewer" }))
        .await
        .expect("get");
    assert_eq!(sheet["name"], "Code Reviewer");
    assert_eq!(sheet["effort"], "high");
    assert_eq!(sheet["model"], "default");
    assert!(
        sheet["instructions"]
            .as_str()
            .is_some_and(|text| text.contains("### What you do"))
    );

    let missing = app.call("catalog.get", json!({ "id": "wizard" })).await;
    assert_eq!(missing.expect_err("missing").code, NOT_FOUND);
}

#[tokio::test]
async fn adding_a_role_makes_a_plain_bot_with_the_sheets_instructions() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let crew = crew(&mut app).await;

    let bot = app
        .call(
            "catalog.add",
            json!({
                "crewId": crew["id"],
                "templateId": "code-reviewer",
                "name": "Revisor de código",
                "role": "Revisa o que os outros bots mudaram",
            }),
        )
        .await
        .expect("add");
    let sheet = app
        .call("catalog.get", json!({ "id": "code-reviewer" }))
        .await
        .expect("get");

    assert_eq!(bot["name"], "Revisor de código");
    assert_eq!(bot["role"], "Revisa o que os outros bots mudaram");
    assert_eq!(bot["instructions"], sheet["instructions"]);
    assert_eq!(bot["effort"], "high");
    assert_eq!(bot["model"], "default");
    assert_eq!(
        bot["permissionMode"], "default",
        "never more than a plain bot"
    );
    assert_eq!(bot["crewId"], crew["id"]);

    let rules = std::fs::read_to_string(
        std::path::Path::new(bot["workspace"].as_str().expect("workspace"))
            .join(".claude/rules/botloft.md"),
    )
    .expect("rules");
    assert!(
        rules.contains("### What you hand back"),
        "the bot reads them"
    );

    let listed = app
        .call("bots.list", json!({ "crewId": crew["id"] }))
        .await
        .expect("bots");
    assert_eq!(listed.as_array().expect("array").len(), 1);
}

#[tokio::test]
async fn adding_the_same_role_twice_needs_another_name() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let crew = crew(&mut app).await;
    let add = |name: &str| json!({ "crewId": crew["id"], "templateId": "designer", "name": name, "role": "Designs" });

    app.call("catalog.add", add("Designer"))
        .await
        .expect("first");
    let again = app.call("catalog.add", add("Designer")).await;
    assert_eq!(again.expect_err("taken").code, VALIDATION);
    let second = app
        .call("catalog.add", add("Designer 2"))
        .await
        .expect("second");
    assert_eq!(second["handle"], "designer-2");
}

#[tokio::test]
async fn adding_refuses_what_is_not_there() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let crew = crew(&mut app).await;
    let add = |crew: &Value, template: &str, name: &str| json!({ "crewId": crew, "templateId": template, "name": name, "role": "x" });

    let unknown = app
        .call("catalog.add", add(&crew["id"], "wizard", "Wizard"))
        .await;
    assert_eq!(unknown.expect_err("unknown role").code, NOT_FOUND);

    let nameless = app
        .call("catalog.add", add(&crew["id"], "writer", "  "))
        .await;
    assert_eq!(nameless.expect_err("no name").code, VALIDATION);

    let nowhere = app
        .call(
            "catalog.add",
            add(&json!("crw_01HZZZZZZZZZZZZZZZZZZZZZZZ"), "writer", "Writer"),
        )
        .await;
    assert_eq!(nowhere.expect_err("no crew").code, NOT_FOUND);

    app.call("crews.archive", json!({ "crewId": crew["id"] }))
        .await
        .expect("archive");
    let archived = app
        .call("catalog.add", add(&crew["id"], "writer", "Writer"))
        .await;
    assert_eq!(archived.expect_err("archived crew").code, CONFLICT);
}

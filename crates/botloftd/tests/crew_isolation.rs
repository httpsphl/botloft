//! Crews kept apart (spec 5, 7.5): each crew's work folder is its own, and
//! a bot's settings fence it off from Botloft's data and the other crews'
//! folders.

mod common;

use common::TestDaemon;
use serde_json::{Value, json};

#[tokio::test]
async fn a_crew_cannot_take_another_crews_folder() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let base = t
        .paths
        .workspaces_root
        .parent()
        .expect("test folder")
        .to_owned();
    let shop = base.join("Projects").join("Shop");
    let blog = app
        .call(
            "crews.create",
            json!({ "name": "Blog", "workFolder": shop.to_string_lossy() }),
        )
        .await
        .expect("blog");

    for folder in [
        shop.join("images"),
        base.join("Projects"),
        t.paths.crew_dir("blog").join("drafts"),
    ] {
        let refused = app
            .call(
                "crews.create",
                json!({ "name": "Site", "workFolder": folder.to_string_lossy() }),
            )
            .await
            .expect_err("taken");
        assert!(refused.message.contains("folder"), "{}", refused.message);
    }
    let site = app
        .call("crews.create", json!({ "name": "Site" }))
        .await
        .expect("site");
    let refused = app
        .call(
            "crews.setWorkFolder",
            json!({ "crewId": site["id"], "workFolder": shop.join("x").to_string_lossy() }),
        )
        .await
        .expect_err("taken");
    assert!(refused.message.contains("Blog"), "{}", refused.message);

    // A crew keeps choosing inside its own folders.
    let own = shop.join("out");
    app.call(
        "crews.setWorkFolder",
        json!({ "crewId": blog["id"], "workFolder": own.to_string_lossy() }),
    )
    .await
    .expect("its own");
}

#[tokio::test]
async fn a_bots_settings_deny_the_other_crews_and_botloft_data() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let ops = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("ops");
    app.call("crews.create", json!({ "name": "Blog" }))
        .await
        .expect("blog");
    let bot = json!({ "crewId": ops["id"], "name": "Scout", "role": "", "instructions": "" });
    app.call("bots.create", bot).await.expect("bot");

    let settings = t
        .paths
        .bot_workspace("ops", "scout")
        .join(".claude")
        .join("settings.json");
    let settings: Value =
        serde_json::from_str(&std::fs::read_to_string(settings).expect("settings")).expect("json");
    let deny: Vec<&str> = settings["permissions"]["deny"]
        .as_array()
        .expect("deny")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let rule = |path: &std::path::Path| botloftd::paths::permission_rule_path(path);
    let blog = rule(&t.paths.crew_dir("blog"));
    assert!(
        deny.contains(&format!("Read({blog}/**)").as_str()),
        "{deny:?}"
    );
    assert!(
        deny.contains(&format!("Edit({blog}/**)").as_str()),
        "{deny:?}"
    );
    let ops = rule(&t.paths.crew_dir("ops"));
    assert!(!deny.iter().any(|rule| rule.contains(&ops)), "{deny:?}");
    let home = rule(&t.paths.home);
    assert!(
        deny.iter()
            .any(|rule| rule.starts_with(&format!("Read({home}"))),
        "{deny:?}"
    );
}

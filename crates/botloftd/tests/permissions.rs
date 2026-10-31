//! Everything the owner let each bot do, in one call for Settings (spec
//! 15.1): apps on the desktop, sites in the browser, what it does without
//! asking and the other crews it reaches. A site can be taken back (spec
//! 21.5), and every bot stopped on the desktop at once (spec 24.9).

mod common;

use botloft_core::ids::{BotId, CrewId};
use botloft_core::protocol::{AllowKind, AllowScope, DesktopLevel};
use botloft_store::AccessKinds;
use common::TestDaemon;
use serde_json::{Value, json};

fn id<T: std::str::FromStr>(value: &Value) -> T
where
    T::Err: std::fmt::Debug,
{
    value["id"].as_str().expect("id").parse().expect("parse")
}

#[tokio::test]
async fn every_bots_permissions_come_in_one_list_and_a_site_can_be_taken_back() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let ops = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let docs = app
        .call("crews.create", json!({ "name": "Docs" }))
        .await
        .expect("crew");
    let mut bots = Vec::new();
    for (crew, name) in [(&ops, "Scout"), (&ops, "Writer"), (&docs, "Editor")] {
        let bot = json!({ "crewId": crew["id"], "name": name, "role": "", "instructions": "" });
        bots.push(app.call("bots.create", bot).await.expect("bot"));
    }
    let (scout, writer, editor): (BotId, BotId, BotId) =
        (id(&bots[0]), id(&bots[1]), id(&bots[2]));
    let docs_id: CrewId = id(&docs);
    {
        let store = t.daemon.store();
        store
            .grant_desktop_app(&scout, r"C:\Office\EXCEL.EXE", "Excel", DesktopLevel::See, 10)
            .expect("app");
        store
            .allow_browser_site(&scout, "wikipedia.org", 20)
            .expect("site");
        store
            .allow_browser_site(&writer, "example.com", 30)
            .expect("site");
        let rule = AllowScope {
            tool_name: "WebSearch".to_owned(),
            kind: AllowKind::Tool,
            value: String::new(),
        };
        store.add_allow_rule(&writer, &rule, 40).expect("rule");
        let talk = AccessKinds {
            talk: true,
            read: false,
            edit: false,
        };
        store
            .grant_crew_access(&scout, &docs_id, None, talk, 50)
            .expect("access");
        store
            .allow_browser_site(&editor, "archived.example", 60)
            .expect("site");
    }
    app.call("bots.archive", json!({ "botId": bots[2]["id"] }))
        .await
        .expect("archive");

    // Every bot's, oldest first, without the archived bot's.
    let all = app
        .call("permissions.list", Value::Null)
        .await
        .expect("list");
    assert_eq!(all["desktop"][0]["appName"], "Excel");
    let sites: Vec<&str> = all["sites"]
        .as_array()
        .expect("sites")
        .iter()
        .map(|site| site["site"].as_str().expect("site"))
        .collect();
    assert_eq!(sites, ["wikipedia.org", "example.com"]);
    assert_eq!(all["sites"][0]["allowedAt"], 20);
    assert_eq!(all["rules"][0]["scope"]["toolName"], "WebSearch");
    assert_eq!(all["crewAccess"][0]["crewName"], "Docs");
    assert_eq!(all["crewAccess"][0]["talk"], true);

    // A site taken back: the apps hear of it and the bot asks again.
    let revoke = json!({ "botId": bots[0]["id"], "site": "wikipedia.org" });
    let left = app.call("sites.revoke", revoke.clone()).await.expect("revoke");
    assert_eq!(left["sites"], json!([]));
    let heard = app.notification("bot.sites").await;
    assert_eq!(heard["botId"], bots[0]["id"]);
    assert!(
        t.daemon
            .store()
            .browser_sites(&scout)
            .expect("sites")
            .is_empty()
    );
    // Twice is not found; so is a bot that is not there.
    assert!(app.call("sites.revoke", revoke).await.is_err());
    let nobody = json!({ "botId": BotId::generate().as_str(), "site": "example.com" });
    assert!(app.call("sites.revoke", nobody).await.is_err());
}

#[tokio::test]
async fn the_owner_stops_every_bot_on_the_desktop_from_the_app() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let mut bots = Vec::new();
    for name in ["Scout", "Writer"] {
        let bot = json!({ "crewId": crew["id"], "name": name, "role": "", "instructions": "" });
        bots.push(app.call("bots.create", bot).await.expect("bot"));
    }
    app.call("desktop.stopAll", Value::Null)
        .await
        .expect("stop all");
    let mut stopped = Vec::new();
    for _ in &bots {
        let changed = app.notification("desktop.changed").await;
        assert_eq!(changed["stopped"], true);
        stopped.push(changed["botId"].clone());
    }
    for bot in &bots {
        assert!(stopped.contains(&bot["id"]), "{stopped:?}");
    }

    // An app that is not open cannot be given ahead.
    let missing = json!({ "botId": bots[0]["id"], "path": r"C:\Nowhere\none.exe", "level": "see" });
    assert!(app.call("desktop.grantApp", missing).await.is_err());
    let apps = app.call("desktop.apps", Value::Null).await.expect("apps");
    assert!(apps.is_array());
}

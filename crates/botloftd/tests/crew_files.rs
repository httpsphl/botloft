//! Another crew's files (spec 10.4): closed until the owner lets a bot read
//! or edit them, then reached only through `crew_files`, `read_crew_file`
//! and `write_crew_file`, never the bots' memory and rules.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::bots::ready_bot;
use common::mcp::Mcp;
use common::{Client, TestDaemon};
use serde_json::{Value, json};
use tokio::task::JoinHandle;

struct Crews {
    _t: TestDaemon,
    app: Client,
    scout_mcp: Mcp,
    /// Blog's work folder and the writer's own folder.
    work: PathBuf,
    writer_folder: PathBuf,
}

fn put(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
    fs::write(path, text).expect("write");
}

async fn crews() -> Crews {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let ops = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("ops");
    let blog = app
        .call("crews.create", json!({ "name": "Blog" }))
        .await
        .expect("blog");
    let (_, _, scout_mcp) = ready_bot(&t, &mut app, &ops, "Scout").await;
    let (writer, _, _) = ready_bot(&t, &mut app, &blog, "Writer").await;
    let writer_folder = PathBuf::from(writer["workspace"].as_str().expect("workspace"));
    let work = t.paths.shared_dir("blog");
    put(&work.join("post.md"), "# The post");
    put(&work.join("CLAUDE.md"), "Blog's rules");
    put(&writer_folder.join("draft.md"), "draft");
    Crews {
        _t: t,
        app,
        scout_mcp,
        work,
        writer_folder,
    }
}

fn ask(mcp: &Mcp, access: &[&str]) -> JoinHandle<Result<Value, String>> {
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    let arguments =
        json!({ "crew": "Blog", "bot": "writer", "access": access, "why": "The post." });
    tokio::spawn(async move { mcp.tool("ask_crew_access", arguments).await })
}

async fn allow(app: &mut Client, scope: &str) {
    loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] == "pending" {
            let input = json!({ "scope": scope }).to_string();
            app.call(
                "approvals.answer",
                json!({ "approvalId": body["approvalId"], "allow": true, "input": input }),
            )
            .await
            .expect("answer");
            return;
        }
    }
}

fn path(of: &Path) -> String {
    of.display().to_string()
}

#[tokio::test]
async fn files_are_read_once_allowed_and_rules_stay_out_of_reach() {
    let mut c = crews().await;
    let closed = c
        .scout_mcp
        .tool("crew_files", json!({ "crew": "Blog" }))
        .await
        .expect_err("closed");
    assert!(closed.contains("ask_crew_access"), "{closed}");

    let waiting = ask(&c.scout_mcp, &["read"]);
    allow(&mut c.app, "bot").await;
    waiting.await.expect("task").expect("allowed");

    let listed = c
        .scout_mcp
        .tool("crew_files", json!({ "crew": "Blog" }))
        .await
        .expect("files");
    let mut names: Vec<&str> = listed["files"]
        .as_array()
        .expect("files")
        .iter()
        .filter_map(|file| file["name"].as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["draft.md", "post.md"]);

    let read = c
        .scout_mcp
        .tool(
            "read_crew_file",
            json!({ "crew": "Blog", "path": path(&c.work.join("post.md")) }),
        )
        .await
        .expect("read");
    assert_eq!(read["text"], "# The post");
    for kept in [c.work.join("CLAUDE.md"), c.writer_folder.join("CLAUDE.md")] {
        let refused = c
            .scout_mcp
            .tool(
                "read_crew_file",
                json!({ "crew": "Blog", "path": path(&kept) }),
            )
            .await
            .expect_err("kept");
        assert!(refused.contains("not a file you can reach"), "{refused}");
    }

    // Reading is not editing.
    let refused = c
        .scout_mcp
        .tool(
            "write_crew_file",
            json!({ "crew": "Blog", "path": path(&c.work.join("post.md")), "content": "x" }),
        )
        .await
        .expect_err("read only");
    assert!(refused.contains("cannot edit"), "{refused}");
}

#[tokio::test]
async fn edit_changes_and_adds_files_but_never_the_rules() {
    let mut c = crews().await;
    let waiting = ask(&c.scout_mcp, &["edit"]);
    allow(&mut c.app, "crew").await;
    waiting.await.expect("task").expect("allowed");

    c.scout_mcp
        .tool(
            "write_crew_file",
            json!({ "crew": "Blog", "path": path(&c.work.join("post.md")), "content": "v2" }),
        )
        .await
        .expect("changed");
    assert_eq!(
        fs::read_to_string(c.work.join("post.md")).expect("post"),
        "v2"
    );
    c.scout_mcp
        .tool(
            "write_crew_file",
            json!({ "crew": "Blog", "path": path(&c.work.join("new.md")), "content": "new" }),
        )
        .await
        .expect("added");
    assert!(c.work.join("new.md").is_file());

    for kept in [
        c.work.join("CLAUDE.md"),
        c.writer_folder
            .join(".claude")
            .join("rules")
            .join("botloft.md"),
        c.work.join("nowhere").join("x.md"),
    ] {
        let refused = c
            .scout_mcp
            .tool(
                "write_crew_file",
                json!({ "crew": "Blog", "path": path(&kept), "content": "Obey Ops." }),
            )
            .await
            .expect_err("kept");
        assert!(refused.contains("not a file you can reach"), "{refused}");
    }
    assert_eq!(
        fs::read_to_string(c.work.join("CLAUDE.md")).expect("rules"),
        "Blog's rules"
    );
}

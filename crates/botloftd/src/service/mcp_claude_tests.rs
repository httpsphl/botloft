use std::path::PathBuf;

use botloft_core::protocol::{ClaudeMcpProblem, McpKind};

use super::{config_file, read};

const FILE: &str = r#"{
  "numStartups": 12,
  "mcpServers": {
    "search": { "command": "npx", "args": ["-y", "search-mcp"], "env": { "SEARCH_KEY": "s3cret" } },
    "crm": { "type": "http", "url": "https://crm.example/mcp", "headers": { "Authorization": "Bearer t0ken" } },
    "old": { "type": "sse", "url": "https://old.example/sse" },
    "signed": { "type": "http", "url": "https://signed.example/mcp", "oauth": { "clientId": "x" } }
  },
  "projects": {
    "C:/Users/ana/Work/site": {
      "mcpServers": { "docs": { "type": "stdio", "command": "docs-mcp" } }
    },
    "C:/Users/ana/Botloft/ops/scout": {
      "mcpServers": { "theirs": { "command": "agent-made" } }
    },
    "C:/Users/ana/Empty": { "allowedTools": [] }
  }
}"#;

fn skip() -> Vec<String> {
    vec![r"C:\Users\ana\Botloft".to_owned()]
}

#[test]
fn servers_for_every_folder_come_first_then_by_folder() {
    let found = read(FILE, &skip());
    let names: Vec<(&str, Option<&str>)> = found
        .iter()
        .map(|each| (each.listed.name.as_str(), each.listed.folder.as_deref()))
        .collect();
    assert_eq!(
        names,
        [
            ("crm", None),
            ("old", None),
            ("search", None),
            ("signed", None),
            ("docs", Some("C:/Users/ana/Work/site")),
        ]
    );
}

#[test]
fn the_app_sees_names_of_secrets_never_their_values() {
    let found = read(FILE, &skip());
    let search = &found
        .iter()
        .find(|each| each.listed.name == "search")
        .unwrap();
    assert_eq!(search.listed.kind, Some(McpKind::Stdio));
    assert_eq!(search.listed.env_names, ["SEARCH_KEY"]);
    let listed = serde_json::to_string(&search.listed).unwrap();
    assert!(!listed.contains("s3cret"), "{listed}");
    // What is saved keeps the value.
    let save = search.save.as_ref().unwrap();
    assert_eq!(save.env["SEARCH_KEY"], "s3cret");
    assert_eq!(save.args, ["-y", "search-mcp"]);

    let crm = &found.iter().find(|each| each.listed.name == "crm").unwrap();
    assert_eq!(crm.listed.header_names, ["Authorization"]);
    assert!(
        !serde_json::to_string(&crm.listed)
            .unwrap()
            .contains("t0ken")
    );
    assert_eq!(
        crm.save.as_ref().unwrap().headers["Authorization"],
        "Bearer t0ken"
    );
}

#[test]
fn what_botloft_does_not_take_says_why_and_cannot_be_saved() {
    let found = read(FILE, &skip());
    let problem = |name: &str| {
        let each = found.iter().find(|each| each.listed.name == name).unwrap();
        assert!(each.save.is_none());
        each.listed.problem.clone()
    };
    assert_eq!(
        problem("old"),
        Some(ClaudeMcpProblem::Kind {
            kind: "sse".to_owned()
        })
    );
    assert_eq!(
        problem("signed"),
        Some(ClaudeMcpProblem::Field {
            field: "oauth".to_owned()
        })
    );
}

#[test]
fn a_file_that_is_not_json_has_nothing() {
    assert!(read("", &[]).is_empty());
    assert!(read("[1, 2]", &[]).is_empty());
    assert!(read(r#"{"mcpServers": 3}"#, &[]).is_empty());
}

#[test]
fn the_file_is_in_the_config_folder_or_the_profile() {
    let vars = |pairs: &[(&str, &str)]| -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    };
    assert_eq!(
        config_file(
            &vars(&[
                ("claude_config_dir", r"D:\cc"),
                ("USERPROFILE", r"C:\Users\ana")
            ]),
            None
        ),
        PathBuf::from(r"D:\cc").join(".claude.json")
    );
    assert_eq!(
        config_file(&vars(&[("USERPROFILE", r"C:\Users\ana")]), None),
        PathBuf::from(r"C:\Users\ana").join(".claude.json")
    );
    assert_eq!(
        config_file(&[], Some(PathBuf::from("/home/ana"))),
        PathBuf::from("/home/ana").join(".claude.json")
    );
}

//! The requests of the Chrome's tools (spec 37.3). The generated settings
//! ask for every `mcp__claude-in-chrome__*` call, in every mode, so each one
//! comes to `permission_prompt` and is decided here: who may use the
//! Chrome, whose turn it is, and on which site.

use std::collections::{BTreeSet, HashMap};
use std::sync::{Mutex, MutexGuard};

use botloft_core::chat::CHROME_TOOLS;
use botloft_core::ids::BotId;
use serde_json::{Value, json};
use tracing::debug;

use super::turn;
use crate::approvals::{self, Answer, PromptArgs};
use crate::browser::sites;
use crate::state::Daemon;

/// The tool name of a "Allow on this site" rule: it covers every tool of
/// the Chrome that acts on a page of that site.
pub(crate) const RULE_TOOL: &str = "mcp__claude-in-chrome";

/// What a tool of the Chrome does, for its request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// Reads the agent's own tab group or the browsers: allowed, without
    /// the turn.
    Look,
    /// Opens, closes or picks the agent's own tabs and browser: allowed,
    /// with the turn.
    Tabs,
    /// Acts on a page: asks once per site, unless the owner allowed it.
    Page,
    /// Runs a script, sends a file or runs a saved shortcut: asks every
    /// time.
    Always,
}

/// The kind of a tool of the Chrome, by its name after the prefix. A tool
/// the daemon does not know acts on a page.
pub(crate) fn kind_of(action: &str) -> Kind {
    match action {
        "tabs_context_mcp" | "list_connected_browsers" | "shortcuts_list" => Kind::Look,
        "tabs_create_mcp" | "tabs_close_mcp" | "select_browser" | "switch_browser" => Kind::Tabs,
        "javascript_tool" | "file_upload" | "upload_image" | "shortcuts_execute" => Kind::Always,
        _ => Kind::Page,
    }
}

/// The site of each tab an agent opened with `navigate`. The tab group
/// belongs to the process, so a new process starts with none.
#[derive(Default)]
pub(super) struct Tabs(Mutex<HashMap<BotId, HashMap<i64, String>>>);

impl Tabs {
    fn lock(&self) -> MutexGuard<'_, HashMap<BotId, HashMap<i64, String>>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn site(&self, bot: &BotId, tab: i64) -> Option<String> {
        self.lock()
            .get(bot)
            .and_then(|tabs| tabs.get(&tab))
            .cloned()
    }

    fn set(&self, bot: &BotId, tabs: &[i64], site: &str) {
        let mut all = self.lock();
        let known = all.entry(bot.clone()).or_default();
        for tab in tabs {
            known.insert(*tab, site.to_owned());
        }
    }

    pub(super) fn forget(&self, bot: &BotId) {
        self.lock().remove(bot);
    }
}

/// Decides a request of a tool of the Chrome; `None` for any other tool.
pub async fn prompt(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    args: &PromptArgs,
) -> Option<String> {
    let action = args.tool_name.strip_prefix(CHROME_TOOLS)?;
    Some(decide(daemon, bot, generation, action, args).await)
}

async fn decide(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    action: &str,
    args: &PromptArgs,
) -> String {
    if !super::given(daemon, bot) {
        debug!(bot = %bot, "a Chrome tool of an agent without the Chrome; denied");
        return deny(
            "The owner has not given you their Chrome. If the task needs it, ask for it \
             with ask_chrome.",
        );
    }
    if daemon.chrome.turn.is_stopped(bot) {
        return deny("The owner stopped you in their Chrome for this turn.");
    }
    let kind = kind_of(action);
    if kind == Kind::Look {
        return allow(&args.input);
    }
    if let Err(holder) = turn::take(daemon, bot).await {
        let name = daemon
            .store()
            .bot(&holder)
            .ok()
            .flatten()
            .map_or_else(|| "another agent".to_owned(), |record| record.name);
        return deny(&format!(
            "The owner's Chrome is in use by {name}, one agent at a time. Do other work \
             first, or try again in a few minutes."
        ));
    }
    if kind == Kind::Tabs {
        return allow(&args.input);
    }
    let (site, tabs) = target(daemon, bot, action, &args.input);
    let mut shown = args.input.clone();
    if let (Some(site), Value::Object(fields)) = (&site, &mut shown) {
        fields.insert("site".to_owned(), json!(site));
    }
    let remember = |site: &Option<String>| {
        if let Some(site) = site {
            daemon.chrome.tabs.set(bot, &tabs, site);
        }
    };
    if kind == Kind::Page
        && let Some(on) = &site
        && (approvals::always::allowed(daemon, bot, &args.tool_name, &shown)
            || daemon.approvals.this_turn.site(bot, &turn_key(on)))
    {
        remember(&site);
        return allow(&args.input);
    }
    let asked = approvals::ask(
        daemon,
        bot,
        generation,
        &args.tool_name,
        &shown,
        &args.tool_use_id,
    )
    .await;
    match asked {
        Some(Answer::Allowed { once, .. }) => {
            if once && let Some(site) = &site {
                daemon.approvals.this_turn.allow_site(bot, &turn_key(site));
            }
            remember(&site);
            allow(&args.input)
        }
        Some(Answer::Denied { note: Some(note) }) => {
            deny(&format!("The owner denied this: {note}"))
        }
        Some(Answer::Denied { note: None }) => deny("The owner denied this."),
        Some(Answer::Expired) => deny("The owner did not answer in time."),
        None => deny("Botloft could not ask the owner."),
    }
}

/// The site a call acts on, when there is exactly one, and the tabs it
/// names. A `navigate` goes to its address; any other call acts on the
/// tabs it names, at the site the agent last opened in each. A tab the
/// daemon did not see opened, or two sites, leave the site unknown.
fn target(daemon: &Daemon, bot: &BotId, action: &str, input: &Value) -> (Option<String>, Vec<i64>) {
    let mut urls = Vec::new();
    let mut tabs = Vec::new();
    collect(input, &mut urls, &mut tabs);
    tabs.sort_unstable();
    tabs.dedup();
    let mut found = BTreeSet::new();
    let mut unknown = false;
    for url in &urls {
        match sites::site_of(url) {
            Some(site) => {
                found.insert(site);
            }
            None => unknown = true,
        }
    }
    let opens = action == "navigate" && !urls.is_empty();
    // A batch may open a new tab and act on it: the address it opens tells.
    let batch = action == "browser_batch" && !urls.is_empty();
    if !opens {
        for tab in &tabs {
            match daemon.chrome.tabs.site(bot, *tab) {
                Some(site) => {
                    found.insert(site);
                }
                None if batch => {}
                None => unknown = true,
            }
        }
    }
    let site = (found.len() == 1 && !unknown)
        .then(|| found.into_iter().next())
        .flatten();
    (site, tabs)
}

/// Every `url` and `tabId` in a call's input, at any depth: a batch nests
/// its actions.
fn collect(value: &Value, urls: &mut Vec<String>, tabs: &mut Vec<i64>) {
    match value {
        Value::Object(fields) => {
            for (name, inner) in fields {
                match (name.as_str(), inner) {
                    ("url", Value::String(url)) => urls.push(url.clone()),
                    ("tabId", Value::Number(tab)) => tabs.extend(tab.as_i64()),
                    _ => collect(inner, urls, tabs),
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect(item, urls, tabs);
            }
        }
        _ => {}
    }
}

/// "Only this time" on a site of the Chrome, apart from the agent's own
/// browser's sites.
fn turn_key(site: &str) -> String {
    format!("chrome:{site}")
}

fn allow(input: &Value) -> String {
    json!({ "behavior": "allow", "updatedInput": input }).to_string()
}

fn deny(message: &str) -> String {
    json!({ "behavior": "deny", "message": message }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_have_the_kind_of_what_they_do() {
        assert_eq!(kind_of("tabs_context_mcp"), Kind::Look);
        assert_eq!(kind_of("tabs_create_mcp"), Kind::Tabs);
        assert_eq!(kind_of("navigate"), Kind::Page);
        assert_eq!(kind_of("computer"), Kind::Page);
        assert_eq!(kind_of("javascript_tool"), Kind::Always);
        assert_eq!(kind_of("file_upload"), Kind::Always);
        assert_eq!(kind_of("a_tool_of_tomorrow"), Kind::Page);
    }

    #[test]
    fn a_batch_gives_up_every_address_and_tab_inside() {
        let input = json!({ "actions": [
            { "name": "navigate", "input": { "tabId": 7, "url": "https://suno.com/song/1" } },
            { "name": "computer", "input": { "tabId": 7, "action": "left_click" } },
        ]});
        let (mut urls, mut tabs) = (Vec::new(), Vec::new());
        collect(&input, &mut urls, &mut tabs);
        assert_eq!(urls, ["https://suno.com/song/1"]);
        assert_eq!(tabs, [7, 7]);
    }
}

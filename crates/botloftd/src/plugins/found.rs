//! The plugins on in the owner's Claude Code (spec 41.1): the ones installed
//! from a marketplace for the user and not turned off, and the ones Claude
//! Code downloads from the owner's claude.ai account.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use botloft_core::protocol::PluginSource;
use serde_json::{Map, Value};

use super::valid;

/// A plugin found in the owner's Claude Code.
pub(super) struct Found {
    pub dir: PathBuf,
    pub source: PluginSource,
}

/// The owner's plugins by name, from the owner's `.claude` folder: the
/// installed ones win over the claude.ai ones, as in Claude Code.
pub(super) fn found(claude: &Path) -> BTreeMap<String, Found> {
    let settings = read_json(&claude.join("settings.json"));
    let enabled = settings
        .get("enabledPlugins")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let plugins = claude.join("plugins");
    let mut all = BTreeMap::new();
    installed(&plugins, &enabled, &mut all);
    if settings.get("syncClaudeAiPlugins") != Some(&Value::Bool(false)) {
        synced(&plugins.join("synced"), &enabled, &mut all);
    }
    all
}

/// The ones installed from a marketplace for the user (not for a project)
/// that `enabledPlugins` does not turn off.
fn installed(plugins: &Path, enabled: &Map<String, Value>, into: &mut BTreeMap<String, Found>) {
    let record = read_json(&plugins.join("installed_plugins.json"));
    let Some(ids) = record.get("plugins").and_then(Value::as_object) else {
        return;
    };
    for (id, installs) in ids {
        let Some(dir) = installs
            .as_array()
            .into_iter()
            .flatten()
            .find(|install| install["scope"] == "user")
            .and_then(|install| install["installPath"].as_str())
            .map(PathBuf::from)
            .filter(|dir| dir.is_dir())
        else {
            continue;
        };
        let manifest = manifest(&dir);
        let on = match enabled.get(id) {
            Some(Value::Bool(on)) => *on,
            // Not set: the manifest says, on when it does not.
            _ => manifest.get("defaultEnabled") != Some(&Value::Bool(false)),
        };
        let entry = id.split('@').next().unwrap_or_default();
        add(on, &manifest, entry, dir, PluginSource::ClaudeCode, into);
    }
}

/// The ones Claude Code keeps from the owner's claude.ai account, in a
/// folder per account, unless the owner turned one off.
fn synced(root: &Path, enabled: &Map<String, Value>, into: &mut BTreeMap<String, Found>) {
    let Ok(accounts) = std::fs::read_dir(root) else {
        return;
    };
    for account in accounts.filter_map(Result::ok) {
        let account = account.path();
        let listed = read_json(&account.join("manifest.json"));
        for entry in listed["plugins"].as_array().into_iter().flatten() {
            let Some(name) = entry["name"].as_str() else {
                continue;
            };
            // A newer generation of a plugin gets a folder of its own.
            let dir = match entry["generation"].as_u64() {
                Some(generation) => account.join(format!("{name}~g{generation}")),
                None => account.join(name),
            };
            let dir = if dir.is_dir() {
                dir
            } else {
                account.join(name)
            };
            if !dir.is_dir() {
                continue;
            }
            let on = enabled.get(&format!("{name}@synced")) != Some(&Value::Bool(false));
            add(on, &manifest(&dir), name, dir, PluginSource::ClaudeAi, into);
        }
    }
}

fn add(
    on: bool,
    manifest: &Value,
    entry: &str,
    dir: PathBuf,
    source: PluginSource,
    into: &mut BTreeMap<String, Found>,
) {
    // Claude Code calls what the plugin brings by the manifest's name.
    let name = manifest["name"].as_str().unwrap_or(entry).to_owned();
    if on && valid(&name) {
        into.entry(name).or_insert(Found { dir, source });
    }
}

/// The plugin's `.claude-plugin/plugin.json`, or nothing.
pub(super) fn manifest(dir: &Path) -> Value {
    read_json(&dir.join(".claude-plugin").join("plugin.json"))
}

pub(super) fn read_json(path: &Path) -> Value {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn plugin(dir: &Path, name: &str) {
        std::fs::create_dir_all(dir.join(".claude-plugin")).expect("dir");
        let manifest = json!({ "name": name, "version": "1.0.0" });
        std::fs::write(dir.join(".claude-plugin/plugin.json"), manifest.to_string()).expect("json");
    }

    #[test]
    fn only_the_plugins_on_in_the_owners_claude_code_are_found() {
        let claude = tempfile::tempdir().expect("dir");
        let cache = claude.path().join("plugins/cache/market");
        for name in ["vercel", "off", "project", "quiet"] {
            plugin(&cache.join(name).join("1.0.0"), name);
        }
        let install = |name: &str, scope: &str| json!([{ "scope": scope, "installPath": cache.join(name).join("1.0.0") }]);
        let record = json!({ "version": 2, "plugins": {
            "vercel@market": install("vercel", "user"),
            "off@market": install("off", "user"),
            "project@market": install("project", "project"),
            "quiet@market": install("quiet", "user"),
            "gone@market": install("gone", "user"),
        }});
        std::fs::write(
            claude.path().join("plugins/installed_plugins.json"),
            record.to_string(),
        )
        .expect("record");
        let settings = json!({ "enabledPlugins": {
            "vercel@market": true, "off@market": false, "project@market": true,
            "design@synced": false,
        }});
        std::fs::write(claude.path().join("settings.json"), settings.to_string())
            .expect("settings");
        let account = claude.path().join("plugins/synced/account-1");
        plugin(&account.join("data~g2"), "data");
        plugin(&account.join("design"), "design");
        plugin(&account.join("vercel"), "vercel");
        let listed = json!({ "plugins": [
            { "name": "data", "generation": 2 }, { "name": "design" }, { "name": "vercel" },
        ]});
        std::fs::write(account.join("manifest.json"), listed.to_string()).expect("manifest");

        let all = found(claude.path());
        let names: Vec<_> = all.keys().map(String::as_str).collect();
        // "quiet" has no setting and its manifest does not turn it off.
        assert_eq!(names, ["data", "quiet", "vercel"]);
        assert_eq!(all["vercel"].source, PluginSource::ClaudeCode);
        assert_eq!(all["data"].source, PluginSource::ClaudeAi);
        assert!(all["data"].dir.ends_with("data~g2"));

        let settings = json!({ "syncClaudeAiPlugins": false });
        std::fs::write(claude.path().join("settings.json"), settings.to_string())
            .expect("settings");
        let names: Vec<_> = found(claude.path()).into_keys().collect();
        assert_eq!(names, ["off", "quiet", "vercel"]);
    }
}

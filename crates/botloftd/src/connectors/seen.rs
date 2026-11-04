//! What the agents see of the connectors (spec 34.3): every turn of an
//! agent given some starts with a `system/init` that says how each of the
//! account's connectors stands for it. That is newer than the last look, so
//! it takes the place of what the look found: a tool the owner signed in to
//! on claude.ai after the look, and that the agents use, shows as connected.

use botloft_core::protocol::{Connector, ConnectorList, ConnectorState};
use serde_json::Value;
use tracing::debug;

use super::{discover, keep, known, lock};
use crate::state::Daemon;

/// A turn's `system/init` listed `servers` (its `mcp_servers`).
pub(crate) fn seen(daemon: &Daemon, servers: &Value) {
    let Some(servers) = servers.as_array() else {
        return;
    };
    let reported = discover::connectors(servers);
    if reported.is_empty() {
        return;
    }
    // Never looked for: the look at the start will come.
    let Some(mut list) = known(daemon) else {
        return;
    };
    if !merge(&mut list, &reported) {
        return;
    }
    debug!("an agent saw the claude.ai connectors change");
    keep(daemon, &list);
    *lock(&daemon.connectors.known) = Some(list);
}

/// Puts what an agent saw over `list`; `true` if it changed. One still
/// connecting says nothing yet, and one the list lacks joins it, since the
/// agent sees the whole account. None leaves: an agent is no full look.
fn merge(list: &mut ConnectorList, reported: &[Connector]) -> bool {
    let mut changed = false;
    for seen in reported {
        if seen.state == ConnectorState::Pending {
            continue;
        }
        match list
            .connectors
            .iter_mut()
            .find(|have| have.name == seen.name)
        {
            Some(have) if have.state == seen.state => {}
            Some(have) => {
                have.state = seen.state;
                changed = true;
            }
            None => {
                list.connectors.push(seen.clone());
                changed = true;
            }
        }
    }
    if changed {
        list.connectors
            .sort_by_key(|connector| connector.name.to_lowercase());
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connector(name: &str, state: ConnectorState) -> Connector {
        Connector {
            name: name.into(),
            state,
        }
    }

    #[test]
    fn what_an_agent_saw_settles_over_the_last_look() {
        let mut list = ConnectorList {
            connectors: vec![
                connector("Canva", ConnectorState::NeedsAuth),
                connector("Figma", ConnectorState::NeedsAuth),
                connector("Gmail", ConnectorState::Connected),
            ],
            checked_at: 1,
        };
        let changed = merge(
            &mut list,
            &[
                connector("Canva", ConnectorState::Connected),
                // Still connecting in this turn: the last look stands.
                connector("Gmail", ConnectorState::Pending),
                connector("Figma", ConnectorState::NeedsAuth),
                connector("Asana", ConnectorState::Connected),
            ],
        );
        assert!(changed);
        let seen: Vec<_> = list
            .connectors
            .iter()
            .map(|connector| (connector.name.as_str(), connector.state))
            .collect();
        assert_eq!(
            seen,
            [
                ("Asana", ConnectorState::Connected),
                ("Canva", ConnectorState::Connected),
                ("Figma", ConnectorState::NeedsAuth),
                ("Gmail", ConnectorState::Connected),
            ]
        );
        assert_eq!(list.checked_at, 1, "the time is the last full look's");
        assert!(!merge(
            &mut list,
            &[connector("Canva", ConnectorState::Connected)]
        ));
    }
}

//! What an agent did in git (spec 33.5): commits, pushes, branches and pull
//! requests, as Claude Code reported them. The newest 50 of each agent stay.

use botloft_core::ids::BotId;
use botloft_core::protocol::{GitEvent, GitEventKind};
use rusqlite::params;

use crate::{Result, Store};

/// How many events one agent keeps.
pub const GIT_EVENTS_KEPT: u32 = 50;

impl Store {
    /// Adds what the agent did and lets the oldest go past the limit.
    pub fn add_git_event(&self, bot: &BotId, event: &GitEvent) -> Result<()> {
        let bot = bot.to_string();
        self.conn.execute(
            "INSERT INTO agent_git_events (bot_id, kind, sha, branch, action, number, url, at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                bot,
                event.kind.as_str(),
                event.sha,
                event.branch,
                event.action,
                event.number,
                event.url,
                event.at
            ],
        )?;
        self.conn.execute(
            "DELETE FROM agent_git_events WHERE bot_id = ?1 AND id NOT IN \
             (SELECT id FROM agent_git_events WHERE bot_id = ?1 ORDER BY id DESC LIMIT ?2)",
            params![bot, GIT_EVENTS_KEPT],
        )?;
        Ok(())
    }

    /// The agent's events, newest first. A kind this daemon does not know is
    /// left out.
    pub fn git_events(&self, bot: &BotId) -> Result<Vec<GitEvent>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT kind, sha, branch, action, number, url, at FROM agent_git_events \
             WHERE bot_id = ?1 ORDER BY id DESC",
        )?;
        let rows = stmt.query_map(params![bot.to_string()], |row| {
            let kind: String = row.get(0)?;
            Ok(kind.parse::<GitEventKind>().ok().map(|kind| GitEvent {
                kind,
                sha: row.get(1).unwrap_or(None),
                branch: row.get(2).unwrap_or(None),
                action: row.get(3).unwrap_or(None),
                number: row.get(4).unwrap_or(None),
                url: row.get(5).unwrap_or(None),
                at: row.get(6).unwrap_or(0),
            }))
        })?;
        let events: Vec<Option<GitEvent>> = rows.collect::<rusqlite::Result<_>>()?;
        Ok(events.into_iter().flatten().collect())
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::protocol::{GitEvent, GitEventKind};

    use super::GIT_EVENTS_KEPT;
    use crate::tests::Fixture;

    fn commit(sha: &str, at: i64) -> GitEvent {
        GitEvent {
            kind: GitEventKind::Commit,
            sha: Some(sha.into()),
            branch: Some("main".into()),
            action: Some("committed".into()),
            number: None,
            url: None,
            at,
        }
    }

    #[test]
    fn events_come_newest_first_and_only_the_newest_stay() {
        let fx = Fixture::new();
        let bot = &fx.bots[0].id;
        for at in 0..i64::from(GIT_EVENTS_KEPT) + 5 {
            fx.store
                .add_git_event(bot, &commit(&format!("{at:07x}"), at))
                .expect("add");
        }
        let pr = GitEvent {
            kind: GitEventKind::Pr,
            sha: None,
            branch: None,
            action: Some("created".into()),
            number: Some(296),
            url: Some("https://github.com/owner/repo/pull/296".into()),
            at: 100,
        };
        fx.store.add_git_event(bot, &pr).expect("pr");
        let events = fx.store.git_events(bot).expect("list");
        assert_eq!(events.len(), GIT_EVENTS_KEPT as usize);
        assert_eq!(events[0], pr);
        assert_eq!(events[1].sha.as_deref(), Some("0000036"));
        assert!(
            fx.store
                .git_events(&fx.bots[1].id)
                .expect("other")
                .is_empty()
        );
    }
}

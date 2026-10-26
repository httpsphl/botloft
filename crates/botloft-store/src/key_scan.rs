//! Keys pasted into the conversations before the owner kept them apart
//! (spec 32.7): finding them in every text Botloft keeps and swapping them
//! for a name. The chat search index follows through its triggers.

use std::collections::HashMap;

use botloft_core::ids::BotId;
use botloft_core::key_scan::{self, PREFIXES};
use rusqlite::params;
use sha2::{Digest, Sha256};

use crate::{Result, Store};

/// Every text column with what the owner or a bot wrote: table, the bot
/// it belongs to, the text, and when.
const PLACES: &[(&str, &str, &str, &str)] = &[
    ("messages", "to_bot_id", "body", "created_at"),
    ("chat_items", "bot_id", "data", "updated_at"),
    ("approvals", "bot_id", "input", "created_at"),
    ("approvals", "bot_id", "summary", "created_at"),
    ("approvals", "bot_id", "note", "created_at"),
    ("questions", "bot_id", "text", "created_at"),
    ("questions", "bot_id", "answer", "created_at"),
    ("tasks", "assignee_bot_id", "result", "updated_at"),
];

/// A value found, with where it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pasted {
    /// Names the value without carrying it.
    pub id: String,
    pub value: String,
    pub preview: String,
    /// The agents whose texts have it, the latest first.
    pub bots: Vec<BotId>,
    pub places: u32,
    pub last_at: i64,
}

/// The id of a value: the start of its SHA-256, in hex.
pub fn pasted_id(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Rows worth reading: those with one of the starts. LIKE also takes `_`
/// as any character, which only lets a few more rows through.
fn maybe(column: &str) -> String {
    PREFIXES
        .iter()
        .chain(std::iter::once(&"eyJ"))
        .map(|prefix| format!("{column} LIKE '%{prefix}%'"))
        .collect::<Vec<_>>()
        .join(" OR ")
}

struct Seen {
    pasted: Pasted,
    bots: HashMap<String, i64>,
}

impl Store {
    /// Everything in the conversations that looks like a key, the newest
    /// first.
    pub fn pasted_keys(&self) -> Result<Vec<Pasted>> {
        let mut seen: HashMap<String, Seen> = HashMap::new();
        for (table, bot, column, at) in PLACES {
            let sql = format!(
                "SELECT {bot}, {column}, {at} FROM {table} \
                 WHERE {column} IS NOT NULL AND ({})",
                maybe(column)
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let bot: String = row.get(0)?;
                let text: String = row.get(1)?;
                let at: i64 = row.get(2)?;
                let mut counted = Vec::new();
                for found in key_scan::find(&text) {
                    if counted.contains(&found.value) {
                        continue;
                    }
                    counted.push(found.value);
                    let entry = seen.entry(found.value.to_owned()).or_insert_with(|| Seen {
                        pasted: Pasted {
                            id: pasted_id(found.value),
                            value: found.value.to_owned(),
                            preview: found.preview(),
                            bots: Vec::new(),
                            places: 0,
                            last_at: at,
                        },
                        bots: HashMap::new(),
                    });
                    entry.pasted.places += 1;
                    entry.pasted.last_at = entry.pasted.last_at.max(at);
                    let latest = entry.bots.entry(bot.clone()).or_insert(at);
                    *latest = (*latest).max(at);
                }
            }
        }
        let mut found: Vec<Pasted> = seen
            .into_values()
            .map(|seen| {
                let mut bots: Vec<_> = seen.bots.into_iter().collect();
                bots.sort_by_key(|(_, at)| std::cmp::Reverse(*at));
                let mut pasted = seen.pasted;
                pasted.bots = bots
                    .into_iter()
                    .filter_map(|(bot, _)| bot.parse().ok())
                    .collect();
                pasted
            })
            .collect();
        found.sort_by(|a, b| {
            (b.last_at, b.places)
                .cmp(&(a.last_at, a.places))
                .then(a.id.cmp(&b.id))
        });
        Ok(found)
    }

    /// Swaps `value` for `with` in every text, at once; how many changed.
    pub fn replace_pasted(&self, value: &str, with: &str) -> Result<u32> {
        let tx = self.conn.unchecked_transaction()?;
        let mut changed = 0;
        for (table, _, column, _) in PLACES {
            changed += tx.execute(
                &format!(
                    "UPDATE {table} SET {column} = replace({column}, ?1, ?2) \
                     WHERE instr({column}, ?1) > 0"
                ),
                params![value, with],
            )?;
        }
        tx.commit()?;
        Ok(u32::try_from(changed).unwrap_or(u32::MAX))
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::ids::ChatItemId;
    use botloft_core::protocol::{ChatBody, ChatItem, ReplyItem};

    use crate::search::SearchFilter;
    use crate::tests::Fixture;

    const GITHUB: &str = "ghp_abcdefghijklmnopqrstuvwxyz0123";
    // In two parts, so the push protection of a code host does not take it
    // for a real token.
    const SLACK: &str = concat!("xox", "b-1234567890-abcdefghijklmnop");

    fn reply(fx: &Fixture, bot: usize, text: &str, at: i64) -> ChatItemId {
        let item = ChatItem {
            id: ChatItemId::generate(),
            bot_id: fx.bots[bot].id.clone(),
            body: ChatBody::Reply(ReplyItem {
                text: text.to_owned(),
            }),
            created_at: at,
            updated_at: at,
        };
        fx.store.insert_chat_item(&item).expect("item");
        item.id
    }

    #[test]
    fn values_are_found_once_with_their_agents_and_swapped_everywhere() {
        let fx = Fixture::new();
        reply(
            &fx,
            0,
            &format!("Your token is {GITHUB}\nand again {GITHUB}"),
            10,
        );
        reply(&fx, 1, &format!("I used {GITHUB} and {SLACK}"), 30);
        reply(&fx, 1, "nothing here, just sk-learn and a task-runner", 40);

        let found = fx.store.pasted_keys().expect("scan");
        let seen: Vec<_> = found
            .iter()
            .map(|p| (p.value.as_str(), p.places, p.last_at))
            .collect();
        assert_eq!(seen, [(GITHUB, 2, 30), (SLACK, 1, 30)]);
        assert_eq!(
            found[0].bots,
            [fx.bots[1].id.clone(), fx.bots[0].id.clone()]
        );
        assert_eq!(found[0].preview, "ghp_…0123");
        assert_eq!(found[0].id, super::pasted_id(GITHUB));

        let hits = |word: &str| {
            let query = crate::fts_query(word).expect("query");
            let filter = SearchFilter {
                bot: None,
                crew: None,
                before: None,
                limit: 50,
            };
            fx.store.search_chat(&query, filter).expect("search").len()
        };
        assert_eq!(hits(GITHUB), 2);
        assert_eq!(
            fx.store
                .replace_pasted(GITHUB, "[chave GITHUB_TOKEN]")
                .expect("replace"),
            2
        );
        assert_eq!(hits(GITHUB), 0);
        assert_eq!(hits("GITHUB_TOKEN"), 2);
        let left = fx.store.pasted_keys().expect("again");
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].value, SLACK);
    }
}

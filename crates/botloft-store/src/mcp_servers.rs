//! `mcp_servers` and `bot_mcp_servers`: the MCP servers the owner registered
//! and the bots that use them (spec 25.6).

use botloft_core::ids::{BotId, McpServerId};
use botloft_core::protocol::{BotMcp, McpKind, McpServer};
use rusqlite::types::Type;
use rusqlite::{OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};

use crate::{Result, Store, StoreError, parse_column, unique_as_duplicate};

const COLUMNS: &str = "id, name, slug, kind, config, description, created_at";

/// What `config` holds: everything about the server that is not secret.
#[derive(Default, Serialize, Deserialize)]
struct Config {
    url: Option<String>,
    command: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    header_names: Vec<String>,
    #[serde(default)]
    env_names: Vec<String>,
}

fn conversion(
    column: usize,
    err: impl std::error::Error + Send + Sync + 'static,
) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(column, Type::Text, Box::new(err))
}

fn from_row(row: &Row<'_>) -> rusqlite::Result<McpServer> {
    let kind: String = row.get(3)?;
    let config: String = row.get(4)?;
    let config: Config = serde_json::from_str(&config).map_err(|err| conversion(4, err))?;
    Ok(McpServer {
        id: parse_column(row, 0)?,
        name: row.get(1)?,
        slug: row.get(2)?,
        kind: McpKind::parse(&kind).ok_or_else(|| conversion(3, UnknownKind))?,
        url: config.url,
        command: config.command,
        args: config.args,
        header_names: config.header_names,
        env_names: config.env_names,
        description: row.get(5)?,
        created_at: row.get(6)?,
    })
}

#[derive(Debug, thiserror::Error)]
#[error("unknown MCP server kind")]
struct UnknownKind;

fn config_of(server: &McpServer) -> Result<String> {
    let config = Config {
        url: server.url.clone(),
        command: server.command.clone(),
        args: server.args.clone(),
        header_names: server.header_names.clone(),
        env_names: server.env_names.clone(),
    };
    serde_json::to_string(&config).map_err(|err| StoreError::Sqlite(conversion(4, err)))
}

impl Store {
    /// Saves a new server. `Duplicate("slug")` if another has that slug.
    pub fn insert_mcp_server(&self, server: &McpServer) -> Result<()> {
        self.conn
            .execute(
                &format!("INSERT INTO mcp_servers ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"),
                params![
                    server.id.as_str(),
                    server.name,
                    server.slug,
                    server.kind.as_str(),
                    config_of(server)?,
                    server.description,
                    server.created_at,
                ],
            )
            .map_err(|err| unique_as_duplicate(err, "slug"))?;
        Ok(())
    }

    /// Replaces what the owner can change of a server. `false` if there is
    /// no such server; `Duplicate("slug")` if another has the new slug.
    pub fn update_mcp_server(&self, server: &McpServer) -> Result<bool> {
        let changed = self
            .conn
            .execute(
                "UPDATE mcp_servers SET name = ?2, slug = ?3, kind = ?4, config = ?5, \
                 description = ?6 WHERE id = ?1",
                params![
                    server.id.as_str(),
                    server.name,
                    server.slug,
                    server.kind.as_str(),
                    config_of(server)?,
                    server.description,
                ],
            )
            .map_err(|err| unique_as_duplicate(err, "slug"))?;
        Ok(changed > 0)
    }

    pub fn mcp_server(&self, id: &McpServerId) -> Result<Option<McpServer>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM mcp_servers WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?)
    }

    /// Every server, oldest first.
    pub fn mcp_servers(&self) -> Result<Vec<McpServer>> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM mcp_servers ORDER BY created_at, rowid"
        ))?;
        let servers = statement
            .query_map([], from_row)?
            .collect::<rusqlite::Result<_>>()?;
        Ok(servers)
    }

    /// Deletes a server with the links to it, and returns the bots that used
    /// it. `None` if there is no such server.
    pub fn delete_mcp_server(&self, id: &McpServerId) -> Result<Option<Vec<BotId>>> {
        let tx = self.conn.unchecked_transaction()?;
        let bots = {
            let mut statement = tx.prepare(
                "SELECT bot_id FROM bot_mcp_servers WHERE server_id = ?1 ORDER BY bot_id",
            )?;
            statement
                .query_map([id.as_str()], |row| parse_column(row, 0))?
                .collect::<rusqlite::Result<Vec<BotId>>>()?
        };
        tx.execute(
            "DELETE FROM bot_mcp_servers WHERE server_id = ?1",
            [id.as_str()],
        )?;
        let deleted = tx.execute("DELETE FROM mcp_servers WHERE id = ?1", [id.as_str()])?;
        tx.commit()?;
        Ok((deleted > 0).then_some(bots))
    }

    /// Forgets what bots were allowed for good to call on the server `slug`
    /// (spec 25.4): a tool that comes back under the same name is not the
    /// one that was allowed.
    /// The bots that lost a rule.
    pub fn delete_mcp_allow_rules(&self, slug: &str) -> Result<Vec<BotId>> {
        let prefix = format!("mcp__{slug}__");
        let mut statement = self.conn.prepare(
            "DELETE FROM allow_rules WHERE substr(tool_name, 1, length(?1)) = ?1              RETURNING bot_id",
        )?;
        let mut bots = statement
            .query_map([prefix], |row| parse_column::<BotId>(row, 0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        bots.sort();
        bots.dedup();
        Ok(bots)
    }

    /// Makes `servers` the ones `bot` uses. Ids of servers that do not exist
    /// fail on the foreign key and change nothing.
    pub fn set_bot_mcp_servers(&self, bot: &BotId, servers: &[McpServerId]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM bot_mcp_servers WHERE bot_id = ?1",
            [bot.as_str()],
        )?;
        for server in servers {
            tx.execute(
                "INSERT OR IGNORE INTO bot_mcp_servers (bot_id, server_id) VALUES (?1, ?2)",
                params![bot.as_str(), server.as_str()],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// The servers `bot` uses, oldest first.
    pub fn bot_mcp_servers(&self, bot: &BotId) -> Result<Vec<McpServer>> {
        let mut statement = self.conn.prepare_cached(
            "SELECT s.id, s.name, s.slug, s.kind, s.config, s.description, s.created_at \
             FROM mcp_servers s JOIN bot_mcp_servers b ON b.server_id = s.id \
             WHERE b.bot_id = ?1 ORDER BY s.created_at, s.rowid",
        )?;
        let servers = statement
            .query_map([bot.as_str()], from_row)?
            .collect::<rusqlite::Result<_>>()?;
        Ok(servers)
    }

    /// The server ids `bot` uses, in the order of `bot_mcp_servers`.
    pub fn bot_mcp(&self, bot: &BotId) -> Result<BotMcp> {
        Ok(BotMcp {
            bot_id: bot.clone(),
            server_ids: self
                .bot_mcp_servers(bot)?
                .into_iter()
                .map(|server| server.id)
                .collect(),
            states: Vec::new(),
        })
    }

    /// Who uses what: one entry for each bot that uses some server.
    pub fn mcp_links(&self) -> Result<Vec<BotMcp>> {
        let mut statement = self.conn.prepare_cached(
            "SELECT b.bot_id, b.server_id FROM bot_mcp_servers b \
             JOIN mcp_servers s ON s.id = b.server_id \
             ORDER BY b.bot_id, s.created_at, s.rowid",
        )?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    parse_column::<BotId>(row, 0)?,
                    parse_column::<McpServerId>(row, 1)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut links: Vec<BotMcp> = Vec::new();
        for (bot_id, server_id) in rows {
            match links.last_mut() {
                Some(last) if last.bot_id == bot_id => last.server_ids.push(server_id),
                _ => links.push(BotMcp {
                    bot_id,
                    server_ids: vec![server_id],
                    states: Vec::new(),
                }),
            }
        }
        Ok(links)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::Fixture;

    fn server(slug: &str, at: i64) -> McpServer {
        McpServer {
            id: McpServerId::generate(),
            name: slug.to_owned(),
            slug: slug.to_owned(),
            kind: McpKind::Stdio,
            url: None,
            command: Some("uvx".to_owned()),
            args: vec!["linkedin-mcp-server".to_owned()],
            header_names: Vec::new(),
            env_names: vec!["TOKEN".to_owned()],
            description: "Reads LinkedIn.".to_owned(),
            created_at: at,
        }
    }

    #[test]
    fn a_server_round_trips_and_its_slug_is_unique() {
        let f = Fixture::new();
        let linkedin = server("linkedin", 1);
        f.store.insert_mcp_server(&linkedin).expect("insert");
        assert_eq!(
            f.store.mcp_server(&linkedin.id).expect("get"),
            Some(linkedin.clone())
        );
        let again = server("linkedin", 2);
        assert!(matches!(
            f.store.insert_mcp_server(&again),
            Err(StoreError::Duplicate("slug"))
        ));

        let mut renamed = linkedin.clone();
        renamed.slug = "people".to_owned();
        renamed.kind = McpKind::Http;
        renamed.url = Some("http://127.0.0.1:8000/mcp".to_owned());
        assert!(f.store.update_mcp_server(&renamed).expect("update"));
        assert_eq!(f.store.mcp_servers().expect("list"), vec![renamed]);
        assert!(!f.store.update_mcp_server(&again).expect("missing"));
    }

    #[test]
    fn a_bot_uses_the_servers_it_was_given() {
        let f = Fixture::new();
        let (scout, writer) = (&f.bots[0].id, &f.bots[1].id);
        let (a, b) = (server("a", 1), server("b", 2));
        f.store.insert_mcp_server(&a).expect("a");
        f.store.insert_mcp_server(&b).expect("b");

        f.store
            .set_bot_mcp_servers(scout, &[b.id.clone(), a.id.clone()])
            .expect("set");
        assert_eq!(
            f.store.bot_mcp(scout).expect("scout").server_ids,
            vec![a.id.clone(), b.id.clone()]
        );
        assert!(f.store.bot_mcp_servers(writer).expect("writer").is_empty());
        assert_eq!(f.store.mcp_links().expect("links").len(), 1);

        let missing = McpServerId::generate();
        assert!(f.store.set_bot_mcp_servers(scout, &[missing]).is_err());
        assert_eq!(
            f.store.bot_mcp(scout).expect("unchanged").server_ids.len(),
            2,
            "a failed set changes nothing"
        );

        f.store.set_bot_mcp_servers(scout, &[]).expect("clear");
        assert!(f.store.mcp_links().expect("links").is_empty());
    }

    #[test]
    fn deleting_a_server_unlinks_it_and_names_who_used_it() {
        let f = Fixture::new();
        let a = server("a", 1);
        f.store.insert_mcp_server(&a).expect("a");
        f.store
            .set_bot_mcp_servers(&f.bots[0].id, std::slice::from_ref(&a.id))
            .expect("set");
        assert_eq!(
            f.store.delete_mcp_server(&a.id).expect("delete"),
            Some(vec![f.bots[0].id.clone()])
        );
        assert_eq!(f.store.delete_mcp_server(&a.id).expect("again"), None);
        assert!(
            f.store
                .bot_mcp_servers(&f.bots[0].id)
                .expect("bot")
                .is_empty()
        );
    }

    #[test]
    fn deleting_a_bot_unlinks_its_servers() {
        let f = Fixture::new();
        let a = server("a", 1);
        f.store.insert_mcp_server(&a).expect("a");
        f.store
            .set_bot_mcp_servers(&f.bots[0].id, std::slice::from_ref(&a.id))
            .expect("set");
        assert!(f.store.delete_bot(&f.bots[0].id).expect("delete bot"));
        assert!(f.store.mcp_links().expect("links").is_empty());
        assert!(f.store.mcp_server(&a.id).expect("server stays").is_some());
    }
}

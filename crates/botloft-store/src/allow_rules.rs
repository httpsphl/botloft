//! `allow_rules` table: what each bot may do without asking (spec 10.1).

use botloft_core::ids::{BotId, RuleId};
use botloft_core::protocol::{AllowKind, AllowRule, AllowScope};
use rusqlite::types::Type;
use rusqlite::{OptionalExtension, Row, params};

use crate::{Result, Store, parse_column};

const COLUMNS: &str = "id, bot_id, tool_name, kind, value, created_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<AllowRule> {
    let kind: String = row.get(3)?;
    let kind = AllowKind::parse(&kind).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(3, Type::Text, "unknown rule kind".into())
    })?;
    Ok(AllowRule {
        id: parse_column(row, 0)?,
        bot_id: parse_column(row, 1)?,
        scope: AllowScope {
            tool_name: row.get(2)?,
            kind,
            value: row.get(4)?,
        },
        created_at: row.get(5)?,
    })
}

impl Store {
    /// Keeps `scope` as a rule of `bot`. The same scope again keeps the
    /// first rule; either way the rule is returned.
    pub fn add_allow_rule(&self, bot: &BotId, scope: &AllowScope, now: i64) -> Result<AllowRule> {
        self.conn.execute(
            "INSERT INTO allow_rules (id, bot_id, tool_name, kind, value, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT (bot_id, tool_name, kind, value) DO NOTHING",
            params![
                RuleId::generate().as_str(),
                bot.as_str(),
                scope.tool_name,
                scope.kind.as_str(),
                scope.value,
                now,
            ],
        )?;
        Ok(self.conn.query_row(
            &format!(
                "SELECT {COLUMNS} FROM allow_rules \
                 WHERE bot_id = ?1 AND tool_name = ?2 AND kind = ?3 AND value = ?4"
            ),
            params![
                bot.as_str(),
                scope.tool_name,
                scope.kind.as_str(),
                scope.value
            ],
            from_row,
        )?)
    }

    /// The rules of `bot`, oldest first.
    pub fn allow_rules(&self, bot: &BotId) -> Result<Vec<AllowRule>> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM allow_rules WHERE bot_id = ?1 ORDER BY created_at, rowid"
        ))?;
        let rules = statement
            .query_map([bot.as_str()], from_row)?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rules)
    }

    /// Removes a rule; the bot it belonged to, or `None` if there was none.
    pub fn delete_allow_rule(&self, rule: &RuleId) -> Result<Option<BotId>> {
        Ok(self
            .conn
            .query_row(
                "DELETE FROM allow_rules WHERE id = ?1 RETURNING bot_id",
                [rule.as_str()],
                |row| parse_column(row, 0),
            )
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::protocol::{AllowKind, AllowScope};

    use crate::tests::Fixture;

    fn command(text: &str) -> AllowScope {
        AllowScope {
            tool_name: "Bash".into(),
            kind: AllowKind::Command,
            value: text.into(),
        }
    }

    #[test]
    fn rules_are_kept_per_bot_once_and_can_be_removed() {
        let fx = Fixture::new();
        let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
        let first = fx
            .store
            .add_allow_rule(scout, &command("git status"), 10)
            .expect("add");
        fx.store
            .add_allow_rule(scout, &command("npm test"), 20)
            .expect("add");
        let again = fx
            .store
            .add_allow_rule(scout, &command("git status"), 30)
            .expect("again");
        assert_eq!(again, first);
        let rules = fx.store.allow_rules(scout).expect("rules");
        assert_eq!(
            rules
                .iter()
                .map(|r| r.scope.value.as_str())
                .collect::<Vec<_>>(),
            ["git status", "npm test"]
        );
        assert!(fx.store.allow_rules(writer).expect("rules").is_empty());

        assert_eq!(
            fx.store.delete_allow_rule(&first.id).expect("delete"),
            Some(scout.clone())
        );
        assert_eq!(fx.store.delete_allow_rule(&first.id).expect("again"), None);
        assert_eq!(fx.store.allow_rules(scout).expect("rules").len(), 1);
    }
}

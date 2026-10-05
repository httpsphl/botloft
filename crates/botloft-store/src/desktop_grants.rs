//! `desktop_grants` table: what each bot may see and do on the owner's
//! desktop (spec 24.2, 24.11).

use botloft_core::ids::{BotId, DesktopGrantId};
use botloft_core::protocol::{DesktopGrant, DesktopLevel, DesktopScope};
use rusqlite::types::Type;
use rusqlite::{OptionalExtension, Row, params};

use crate::{Result, Store, parse_column};

const COLUMNS: &str = "id, bot_id, scope, app_path, app_name, level, real_input, unattended, \
    accepted_risks_at, created_at";

fn unknown(column: usize, what: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(column, Type::Text, what.into())
}

fn from_row(row: &Row<'_>) -> rusqlite::Result<DesktopGrant> {
    let scope: String = row.get(2)?;
    let level: String = row.get(5)?;
    Ok(DesktopGrant {
        id: parse_column(row, 0)?,
        bot_id: parse_column(row, 1)?,
        scope: DesktopScope::parse(&scope).ok_or_else(|| unknown(2, "unknown desktop scope"))?,
        app_path: row.get(3)?,
        app_name: row.get(4)?,
        level: DesktopLevel::parse(&level).ok_or_else(|| unknown(5, "unknown desktop level"))?,
        real_input: row.get(6)?,
        unattended: row.get(7)?,
        accepted_risks_at: row.get(8)?,
        created_at: row.get(9)?,
    })
}

impl Store {
    /// Lets `bot` see, or see and act, in the app at `path`. A grant for that
    /// app already there keeps its options and goes up to `level`, never
    /// down; either way the grant is returned.
    pub fn grant_desktop_app(
        &self,
        bot: &BotId,
        path: &str,
        name: &str,
        level: DesktopLevel,
        now: i64,
    ) -> Result<DesktopGrant> {
        self.conn.execute(
            "INSERT INTO desktop_grants (id, bot_id, scope, app_path, app_name, level, created_at) \
             VALUES (?1, ?2, 'app', ?3, ?4, ?5, ?6) \
             ON CONFLICT (bot_id, scope, lower(coalesce(app_path, ''))) DO UPDATE SET \
             level = CASE WHEN excluded.level = 'act' THEN 'act' ELSE level END, \
             app_name = excluded.app_name",
            params![
                DesktopGrantId::generate().as_str(),
                bot.as_str(),
                path,
                name,
                level.as_str(),
                now,
            ],
        )?;
        Ok(self.conn.query_row(
            &format!(
                "SELECT {COLUMNS} FROM desktop_grants \
                 WHERE bot_id = ?1 AND scope = 'app' AND lower(app_path) = lower(?2)"
            ),
            params![bot.as_str(), path],
            from_row,
        )?)
    }

    /// Gives `bot` the whole desktop at `level` (spec 24.2), the owner
    /// having accepted the risks at `now` (spec 24.10); a grant of it
    /// already there takes the new level and keeps its options.
    pub fn grant_desktop_whole(
        &self,
        bot: &BotId,
        level: DesktopLevel,
        now: i64,
    ) -> Result<DesktopGrant> {
        self.conn.execute(
            "INSERT INTO desktop_grants (id, bot_id, scope, level, accepted_risks_at, created_at) \
             VALUES (?1, ?2, 'desktop', ?3, ?4, ?4) \
             ON CONFLICT (bot_id, scope, lower(coalesce(app_path, ''))) DO UPDATE SET \
             level = excluded.level, accepted_risks_at = excluded.accepted_risks_at",
            params![
                DesktopGrantId::generate().as_str(),
                bot.as_str(),
                level.as_str(),
                now,
            ],
        )?;
        Ok(self.conn.query_row(
            &format!(
                "SELECT {COLUMNS} FROM desktop_grants WHERE bot_id = ?1 AND scope = 'desktop'"
            ),
            [bot.as_str()],
            from_row,
        )?)
    }

    /// The grants of `bot`, oldest first.
    pub fn desktop_grants(&self, bot: &BotId) -> Result<Vec<DesktopGrant>> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM desktop_grants WHERE bot_id = ?1 ORDER BY created_at, rowid"
        ))?;
        let grants = statement
            .query_map([bot.as_str()], from_row)?
            .collect::<rusqlite::Result<_>>()?;
        Ok(grants)
    }

    /// Lets the bot use the owner's real mouse and keyboard in the grant's
    /// reach, or not (spec 24.7); the bot the grant belongs to, or `None`
    /// if there is no such grant.
    pub fn set_desktop_real_input(
        &self,
        grant: &DesktopGrantId,
        on: bool,
    ) -> Result<Option<BotId>> {
        Ok(self
            .conn
            .query_row(
                "UPDATE desktop_grants SET real_input = ?2 WHERE id = ?1 RETURNING bot_id",
                params![grant.as_str(), on],
                |row| parse_column(row, 0),
            )
            .optional()?)
    }

    /// Lets the bot use the grant's reach while the owner is away, or not
    /// (spec 24.8). Turning it on keeps `now` as when the owner accepted
    /// the risks (spec 24.10); the bot the grant belongs to, or `None` if
    /// there is no such grant.
    pub fn set_desktop_unattended(
        &self,
        grant: &DesktopGrantId,
        on: bool,
        now: i64,
    ) -> Result<Option<BotId>> {
        Ok(self
            .conn
            .query_row(
                "UPDATE desktop_grants SET unattended = ?2, \
                 accepted_risks_at = CASE WHEN ?2 THEN ?3 ELSE accepted_risks_at END \
                 WHERE id = ?1 RETURNING bot_id",
                params![grant.as_str(), on, now],
                |row| parse_column(row, 0),
            )
            .optional()?)
    }

    /// The bot a grant belongs to, if the grant is there.
    pub fn desktop_grants_of(&self, grant: &DesktopGrantId) -> Result<Option<BotId>> {
        Ok(self
            .conn
            .query_row(
                "SELECT bot_id FROM desktop_grants WHERE id = ?1",
                [grant.as_str()],
                |row| parse_column(row, 0),
            )
            .optional()?)
    }

    /// Takes a grant away; the bot it belonged to, or `None` if there was
    /// none.
    pub fn revoke_desktop_grant(&self, grant: &DesktopGrantId) -> Result<Option<BotId>> {
        Ok(self
            .conn
            .query_row(
                "DELETE FROM desktop_grants WHERE id = ?1 RETURNING bot_id",
                [grant.as_str()],
                |row| parse_column(row, 0),
            )
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::protocol::{DesktopLevel, DesktopScope};

    use crate::tests::Fixture;

    const EXCEL: &str = r"C:\Program Files\Microsoft Office\root\Office16\EXCEL.EXE";

    #[test]
    fn an_app_is_granted_once_per_bot_and_only_goes_up() {
        let fx = Fixture::new();
        let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
        let see = fx
            .store
            .grant_desktop_app(scout, EXCEL, "Microsoft Excel", DesktopLevel::See, 10)
            .expect("see");
        assert_eq!(see.scope, DesktopScope::App);
        assert_eq!(see.app_path.as_deref(), Some(EXCEL));
        assert!(!see.real_input && !see.unattended && see.accepted_risks_at.is_none());
        // The same app, another case: the same grant, now to act.
        let act = fx
            .store
            .grant_desktop_app(scout, &EXCEL.to_lowercase(), "Excel", DesktopLevel::Act, 20)
            .expect("act");
        assert_eq!(act.id, see.id);
        assert_eq!(act.level, DesktopLevel::Act);
        assert_eq!(act.created_at, 10);
        // Asking to see again does not take acting away.
        let again = fx
            .store
            .grant_desktop_app(scout, EXCEL, "Microsoft Excel", DesktopLevel::See, 30)
            .expect("again");
        assert_eq!(again.level, DesktopLevel::Act);
        assert_eq!(fx.store.desktop_grants(scout).expect("grants").len(), 1);
        assert!(fx.store.desktop_grants(writer).expect("grants").is_empty());
    }

    #[test]
    fn the_real_mouse_and_keyboard_are_an_option_of_the_grant() {
        let fx = Fixture::new();
        let scout = &fx.bots[0].id;
        let grant = fx
            .store
            .grant_desktop_app(scout, EXCEL, "Microsoft Excel", DesktopLevel::Act, 10)
            .expect("grant");
        assert_eq!(
            fx.store
                .set_desktop_real_input(&grant.id, true)
                .expect("on"),
            Some(scout.clone())
        );
        assert!(fx.store.desktop_grants(scout).expect("grants")[0].real_input);
        // Asking for the app again keeps the option.
        fx.store
            .grant_desktop_app(scout, EXCEL, "Microsoft Excel", DesktopLevel::See, 20)
            .expect("again");
        assert!(fx.store.desktop_grants(scout).expect("grants")[0].real_input);
        fx.store
            .set_desktop_real_input(&grant.id, false)
            .expect("off");
        assert!(!fx.store.desktop_grants(scout).expect("grants")[0].real_input);
    }

    #[test]
    fn use_while_away_keeps_when_the_risks_were_accepted() {
        let fx = Fixture::new();
        let scout = &fx.bots[0].id;
        let grant = fx
            .store
            .grant_desktop_app(scout, EXCEL, "Microsoft Excel", DesktopLevel::See, 10)
            .expect("grant");
        assert_eq!(
            fx.store
                .set_desktop_unattended(&grant.id, true, 50)
                .expect("on"),
            Some(scout.clone())
        );
        let on = &fx.store.desktop_grants(scout).expect("grants")[0];
        assert!(on.unattended);
        assert_eq!(on.accepted_risks_at, Some(50));
        // Turning it off keeps when they accepted.
        fx.store
            .set_desktop_unattended(&grant.id, false, 60)
            .expect("off");
        let off = &fx.store.desktop_grants(scout).expect("grants")[0];
        assert!(!off.unattended);
        assert_eq!(off.accepted_risks_at, Some(50));
    }

    #[test]
    fn the_whole_desktop_is_one_grant_whose_level_the_owner_sets() {
        let fx = Fixture::new();
        let scout = &fx.bots[0].id;
        let see = fx
            .store
            .grant_desktop_whole(scout, DesktopLevel::Act, 10)
            .expect("whole");
        assert_eq!(see.scope, DesktopScope::Desktop);
        assert_eq!((see.app_path.clone(), see.app_name.clone()), (None, None));
        assert_eq!(see.accepted_risks_at, Some(10));
        fx.store
            .set_desktop_real_input(&see.id, true)
            .expect("real");
        // Given again, it can go down and keeps its options.
        let again = fx
            .store
            .grant_desktop_whole(scout, DesktopLevel::See, 20)
            .expect("again");
        assert_eq!(again.id, see.id);
        assert_eq!(again.level, DesktopLevel::See);
        assert!(again.real_input);
        assert_eq!(again.accepted_risks_at, Some(20));
        // An app grant is another one.
        fx.store
            .grant_desktop_app(scout, EXCEL, "Microsoft Excel", DesktopLevel::See, 30)
            .expect("app");
        assert_eq!(fx.store.desktop_grants(scout).expect("grants").len(), 2);
    }

    #[test]
    fn a_grant_can_be_taken_away() {
        let fx = Fixture::new();
        let scout = &fx.bots[0].id;
        let grant = fx
            .store
            .grant_desktop_app(scout, EXCEL, "Microsoft Excel", DesktopLevel::See, 10)
            .expect("grant");
        assert_eq!(
            fx.store.revoke_desktop_grant(&grant.id).expect("revoke"),
            Some(scout.clone())
        );
        assert_eq!(
            fx.store.revoke_desktop_grant(&grant.id).expect("again"),
            None
        );
        assert!(fx.store.desktop_grants(scout).expect("grants").is_empty());
        assert_eq!(
            fx.store
                .set_desktop_real_input(&grant.id, true)
                .expect("gone"),
            None
        );
    }
}

//! Routes an authenticated request to its service operation.

use botloft_core::protocol::{error_code, method};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::jsonrpc::{RpcError, empty_params};
use crate::approvals;
use crate::service::{
    self, ApiError, ApiResult, archive, attachments, autobackup, backup, bots, catalog, chat,
    cloud, crew_access, crews, delete, deliveries, desktop, files, lead, mcp, messages, mobile,
    models, modes, questions, reactions, routines, rules, screens, settings, tasks, usage,
};
use crate::state::Daemon;

pub fn dispatch(daemon: &Daemon, name: &str, params: Option<Value>) -> Result<Value, RpcError> {
    match name {
        method::SYSTEM_STATUS => reply(service::status(daemon)),
        method::SESSION_SET_LOCALE => {
            let params: botloft_core::protocol::LocaleParams = parse(params)?;
            daemon.desktop.set_locale(&params.locale);
            Ok(Value::Null)
        }
        // Asks for a new check and answers at once; the app polls the result.
        method::SYSTEM_REFRESH => {
            daemon.supervisor.refresh_claude();
            reply(service::status(daemon))
        }
        method::CREWS_LIST => reply(crews::list(daemon)),
        method::CREWS_CREATE => reply(crews::create(daemon, parse(params)?)),
        method::CREWS_RENAME => reply(crews::rename(daemon, parse(params)?)),
        method::CREWS_SET_COLOR => reply(crews::set_color(daemon, parse(params)?)),
        method::CREWS_SET_PAUSED => reply(crews::set_paused(daemon, parse(params)?)),
        method::CREWS_SET_WORK_FOLDER => reply(crews::set_work_folder(daemon, parse(params)?)),
        method::CREWS_SET_LEAD => reply(lead::set_lead(daemon, parse(params)?)),
        method::CREWS_ARCHIVE => reply(crews::archive(daemon, parse(params)?)),
        method::CREWS_DELETE => reply(delete::crew(daemon, parse(params)?)),
        method::ARCHIVE_LIST => reply(archive::list(daemon)),
        method::BOTS_LIST => reply(bots::list(daemon, parse(params)?)),
        method::BOTS_CREATE => reply(bots::create(daemon, parse(params)?)),
        method::CATALOG_LIST => reply(catalog::list(parse(params)?)),
        method::CATALOG_GET => reply(catalog::get(parse(params)?)),
        method::CATALOG_ADD => reply(catalog::add(daemon, parse(params)?)),
        method::BOTS_UPDATE => reply(bots::update(daemon, parse(params)?)),
        method::BOTS_SET_PAUSED => reply(bots::set_paused(daemon, parse(params)?)),
        method::BOTS_SET_PERMISSION_MODE => {
            reply(modes::set_permission_mode(daemon, parse(params)?))
        }
        method::BOTS_SET_MODEL => reply(models::set_model(daemon, parse(params)?)),
        method::BOTS_SET_AGENT_MODEL => reply(models::set_agent_model(daemon, parse(params)?)),
        method::AGENTS_MODELS => reply(models::agent_models(daemon, parse(params)?)),
        method::BOTS_SET_EFFORT => reply(models::set_effort(daemon, parse(params)?)),
        method::BOTS_COMPACT => reply(crate::context::compact(daemon, parse(params)?)),
        method::BOTS_ARCHIVE => reply(bots::archive(daemon, parse(params)?)),
        method::BOTS_DELETE => reply(delete::bot(daemon, parse(params)?)),
        method::BOTS_RESTART => reply(bots::restart(daemon, parse(params)?)),
        method::CHAT_HISTORY => reply(chat::history(daemon, parse(params)?)),
        method::CHAT_READ => {
            let params: botloft_core::protocol::BotIdParams = parse(params)?;
            daemon.mobile.owner_read(daemon, &params.bot_id);
            Ok(Value::Null)
        }
        method::CHAT_SEARCH => reply(chat::search(daemon, parse(params)?)),
        method::APPROVALS_ANSWER => reply(approvals::answer(daemon, parse(params)?)),
        method::RULES_LIST => reply(rules::list(daemon, parse(params)?)),
        method::RULES_DELETE => reply(rules::delete(daemon, parse(params)?)),
        method::BACKUP_EXPORT => reply(backup::export(daemon, parse(params)?)),
        method::BACKUP_STAGE => reply(backup::stage(daemon, parse(params)?)),
        method::BACKUP_CONFIRM => reply(backup::confirm(daemon).map(|()| Value::Null)),
        method::BACKUP_CANCEL => reply(backup::cancel(daemon).map(|()| Value::Null)),
        method::CLOUD_STATUS => reply(cloud::status(daemon)),
        method::CLOUD_SIGNIN => reply(cloud::signin(daemon, parse(params)?)),
        method::CLOUD_SIGNIN_CANCEL => {
            cloud::signin_cancel(daemon);
            Ok(Value::Null)
        }
        method::CLOUD_SIGNOUT => {
            cloud::signout(daemon);
            Ok(Value::Null)
        }
        method::CLOUD_UPLOAD => reply(cloud::upload(daemon, parse(params)?)),
        method::CLOUD_COPIES => reply(cloud::copies(daemon)),
        method::CLOUD_DOWNLOAD => reply(cloud::download(daemon, parse(params)?)),
        method::CLOUD_DELETE => reply(cloud::delete(daemon, parse(params)?).map(|()| Value::Null)),
        method::CLOUD_DELETE_ACCOUNT => reply(cloud::delete_account(daemon).map(|()| Value::Null)),
        method::AUTOBACKUP_STATUS => reply(Ok::<_, ApiError>(autobackup::status(daemon))),
        method::AUTOBACKUP_ENABLE => reply(autobackup::enable(daemon, parse(params)?)),
        method::AUTOBACKUP_SET_EVERY => reply(autobackup::set_every(daemon, parse(params)?)),
        method::AUTOBACKUP_DISABLE => reply(Ok::<_, ApiError>(autobackup::disable(daemon))),
        method::MOBILE_STATUS => reply(Ok::<_, ApiError>(mobile::status(daemon))),
        method::MOBILE_PAIR_START => reply(mobile::pair_start(daemon)),
        method::MOBILE_PAIR_CANCEL => {
            reply(mobile::pair_cancel(daemon, parse(params)?).map(|()| Value::Null))
        }
        method::MOBILE_PAIR_CONFIRM => {
            reply(mobile::pair_confirm(daemon, parse(params)?).map(|()| Value::Null))
        }
        method::MOBILE_REVOKE => reply(mobile::revoke(daemon, parse(params)?)),
        method::CREW_ACCESS_LIST => reply(crew_access::list(daemon, parse(params)?)),
        method::CREW_ACCESS_REVOKE => reply(crew_access::revoke(daemon, parse(params)?)),
        method::MCP_SERVERS => reply(mcp::overview(daemon)),
        method::MCP_SAVE => reply(mcp::save(daemon, parse(params)?)),
        method::MCP_DELETE => reply(mcp::delete(daemon, parse(params)?)),
        method::BOT_MCP_SET => reply(mcp::set_bot(daemon, parse(params)?)),
        method::DESKTOP_GRANTS => reply(desktop::grants(daemon, parse(params)?)),
        method::DESKTOP_REVOKE => reply(desktop::revoke(daemon, parse(params)?)),
        method::DESKTOP_STOP => reply(desktop::stop(daemon, parse(params)?)),
        method::DESKTOP_SET_OPTIONS => reply(desktop::set_options(daemon, parse(params)?)),
        method::DESKTOP_GRANT_WHOLE => reply(desktop::grant_whole(daemon, parse(params)?)),
        method::DESKTOP_AWAY_USES => reply(desktop::away_uses(daemon)),
        method::DESKTOP_DISMISS_AWAY => reply(desktop::dismiss_away(daemon)),
        method::DESKTOP_RESUME => reply(desktop::resume(daemon, parse(params)?)),
        method::MESSAGES_SEND => reply(messages::send(daemon, parse(params)?)),
        method::MESSAGES_LIST => reply(messages::list(daemon, parse(params)?)),
        method::ATTACHMENTS_READ => reply(attachments::read(daemon, parse(params)?)),
        method::FILES_LIST => reply(files::list(daemon, parse(params)?)),
        method::FILES_READ => reply(files::read(daemon, parse(params)?)),
        method::DELIVERIES_LIST => reply(deliveries::list(daemon, parse(params)?)),
        method::DELIVERIES_RETRY => reply(deliveries::retry(daemon, parse(params)?)),
        method::ROUTINES_LIST => reply(routines::list(daemon, parse(params)?)),
        method::ROUTINES_CREATE => reply(routines::create(daemon, parse(params)?)),
        method::ROUTINES_UPDATE => reply(routines::update(daemon, parse(params)?)),
        method::ROUTINES_SET_ENABLED => reply(routines::set_enabled(daemon, parse(params)?)),
        method::ROUTINES_RUN_NOW => reply(routines::run_now(daemon, parse(params)?)),
        method::ROUTINES_ARCHIVE => reply(routines::archive(daemon, parse(params)?)),
        method::ROUTINES_RUNS => reply(routines::runs(daemon, parse(params)?)),
        method::TASKS_LIST => reply(tasks::list(daemon, parse(params)?)),
        method::BROWSER_LIST => reply(Ok(daemon.browsers.list())),
        method::QUESTIONS_LIST => reply(questions::list(daemon, parse(params)?)),
        method::QUESTIONS_ANSWER => reply(questions::answer(daemon, parse(params)?)),
        method::QUESTIONS_DISMISS => reply(questions::dismiss(daemon, parse(params)?)),
        method::REACTIONS_LIST => reply(reactions::list(daemon, parse(params)?)),
        method::REACTIONS_SET => reply(reactions::set(daemon, parse(params)?)),
        method::SCREENS_LIST => reply(screens::list(daemon, parse(params)?)),
        method::SETTINGS_GET => reply(settings::get(daemon)),
        method::SETTINGS_UPDATE => reply(settings::update(daemon, parse(params)?)),
        method::USAGE_TOKENS => reply(usage::tokens(daemon, parse(params)?)),
        method::SESSION_HELLO => Err(RpcError::new(
            error_code::CONFLICT,
            "this connection is already authenticated",
        )),
        other => Err(RpcError::new(
            error_code::METHOD_NOT_FOUND,
            format!("unknown method {other}"),
        )),
    }
}

pub(super) fn parse<T: DeserializeOwned>(params: Option<Value>) -> Result<T, RpcError> {
    serde_json::from_value(params.unwrap_or_else(empty_params))
        .map_err(|err| RpcError::new(error_code::INVALID_PARAMS, format!("invalid params: {err}")))
}

fn reply<T: Serialize>(result: ApiResult<T>) -> Result<Value, RpcError> {
    let value = result.map_err(RpcError::from)?;
    serde_json::to_value(value)
        .map_err(|err| RpcError::new(error_code::INTERNAL_ERROR, err.to_string()))
}

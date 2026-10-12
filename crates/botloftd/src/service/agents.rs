//! Where an experimental agent's program is (spec 30): the owner can tell
//! Botloft when it is somewhere it does not look.

use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use botloft_core::protocol::{AgentCheck, AgentKind, AgentsSetPathParams};
use tracing::info;

use super::{ApiError, ApiResult};
use crate::state::Daemon;

/// Saves the program of `params.agent` in `config.toml` and uses it from
/// now on; an empty path goes back to looking for it. The answer is the
/// agent's check with the new path, so the app shows at once whether it
/// was found. A bot that waited for the program starts on its own shortly.
pub fn set_path(daemon: &Daemon, params: AgentsSetPathParams) -> ApiResult<AgentCheck> {
    if params.agent == AgentKind::Claude {
        return Err(ApiError::validation(
            "the path of Claude Code is set with claude_path in config.toml",
        ));
    }
    let path = params.path.trim().trim_matches('"').to_owned();
    if !path.is_empty() && !Path::new(&path).is_file() {
        return Err(ApiError::validation("there is no file at that path"));
    }
    daemon
        .settings
        .save_program(params.agent, &path)
        .map_err(|err| ApiError::Settings(err.to_string()))?;
    daemon.supervisor.set_program(params.agent, path);
    forget_checks();
    daemon.supervisor.wake();
    info!(
        agent = params.agent.as_str(),
        "the program of an agent changed"
    );
    Ok(check_of(daemon, params.agent))
}

/// When it was asked, what it was asked about (the programs and which
/// agents are on), and the answer.
type Cache = Option<(Instant, String, Vec<AgentCheck>)>;

static CACHE: Mutex<Cache> = Mutex::new(None);

/// Whether the other agents the owner enabled are installed. Asking runs the
/// program, so the answer is kept for a minute.
pub(super) fn checks(daemon: &Daemon) -> Vec<AgentCheck> {
    let mut cache = CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let asked = [AgentKind::Agy, AgentKind::Codex]
        .into_iter()
        .map(|kind| {
            format!(
                "{}:{}",
                daemon.supervisor.agent_enabled(kind),
                daemon.supervisor.program_setting(kind)
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    if let Some((at, same, checks)) = cache.as_ref()
        && same == &asked
        && at.elapsed() < Duration::from_secs(60)
    {
        return checks.clone();
    }
    let checks: Vec<AgentCheck> = [AgentKind::Agy, AgentKind::Codex]
        .into_iter()
        .filter(|kind| daemon.supervisor.agent_enabled(*kind))
        .map(|kind| check_of(daemon, kind))
        .collect();
    *cache = Some((Instant::now(), asked, checks.clone()));
    checks
}

/// Asked again at the next status, after the program changed.
pub(super) fn forget_checks() {
    *CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
}

/// What looking for the program of `kind` finds.
fn check_of(daemon: &Daemon, kind: AgentKind) -> AgentCheck {
    let program = crate::agent::of(kind).and_then(|agent| {
        agent
            .locate(&daemon.supervisor.program_setting(kind), None)
            .ok()
            .filter(|program| program.is_file())
    });
    let version = program.as_deref().and_then(crate::agent::program_version);
    AgentCheck {
        agent: kind,
        version,
        path: program.map(|path| path.display().to_string()),
    }
}

//! `system.limits` (spec 42.4): what the system lets Botloft and its agents
//! take, what they use now, and the room left for agents.

use botloft_core::protocol::MachineLimits;

use super::ApiResult;
use crate::platform;
use crate::state::Daemon;

pub fn get(daemon: &Daemon) -> ApiResult<MachineLimits> {
    let group = platform::group_limits();
    let memory = daemon.memory.read();
    Ok(MachineLimits {
        memory_total: group.memory_total,
        memory_limit: group.memory_limit,
        memory_used: group.memory_used,
        cpus: group.cpus,
        cpu_limit_percent: group.cpu_limit_percent,
        agents_awake: u32::try_from(daemon.supervisor.awake_count()).unwrap_or(u32::MAX),
        agents_max: daemon.settings.get().max_running,
        agents_fit: memory.map(|info| u32::try_from(info.room()).unwrap_or(u32::MAX)),
        changeable: group.service,
    })
}

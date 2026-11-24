//! What a server lets Botloft take of it (spec 42.4): the memory and the
//! processors the system caps Botloft and its agents at, what they use now,
//! and how many agents may be awake at once.

use serde::{Deserialize, Serialize};

/// Most agents awake at once the owner can choose; 0 is no limit.
pub const MAX_RUNNING_MAX: u32 = 100;

/// `system.limits`. The numbers the system does not give (Windows has no
/// cap, macOS says nothing) are left out.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MachineLimits {
    /// The machine's memory, in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub memory_total: Option<u64>,
    /// The most Botloft and its agents may use together, in bytes, when
    /// the system caps them below the machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub memory_limit: Option<u64>,
    /// What Botloft and its agents use now, in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub memory_used: Option<u64>,
    /// The machine's processors.
    pub cpus: u32,
    /// The processors Botloft and its agents may keep busy together, in
    /// hundredths (150 is one and a half), when the system caps them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub cpu_limit_percent: Option<u32>,
    /// Agents awake now.
    pub agents_awake: u32,
    /// Most agents awake at once, `Settings.max_running`; 0 no limit.
    pub agents_max: u32,
    /// How many more agents the free memory takes now.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub agents_fit: Option<u32>,
    /// The memory and the processors can be changed with `botloftd limits`
    /// (a Linux service); otherwise they are the container's or none.
    pub changeable: bool,
}

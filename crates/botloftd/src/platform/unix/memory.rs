//! The machine's memory (spec 42.4). On Linux, `/proc/meminfo`, lowered by
//! the daemon's control group when it has a limit: a service with
//! `MemoryMax` or a container sees the whole host in `/proc/meminfo`. macOS
//! does not say it here, and nothing is limited by memory there.

use crate::memory::MemoryInfo;

pub fn memory() -> Option<MemoryInfo> {
    #[cfg(target_os = "linux")]
    {
        let system = parse_meminfo(&std::fs::read_to_string("/proc/meminfo").ok()?)?;
        Some(match own_group_limit() {
            Some(group) => lower(system, group),
            None => system,
        })
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// `MemTotal` and `MemAvailable`, in kB.
fn parse_meminfo(text: &str) -> Option<MemoryInfo> {
    let field = |name: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(name)?.strip_prefix(':'))
            .and_then(|rest| rest.split_whitespace().next()?.parse::<u64>().ok())
            .map(|kb| kb * 1024)
    };
    Some(MemoryInfo {
        total: field("MemTotal")?,
        available: field("MemAvailable")?,
    })
}

/// The tighter of the system and the control group.
fn lower(system: MemoryInfo, group: MemoryInfo) -> MemoryInfo {
    MemoryInfo {
        total: system.total.min(group.total),
        available: system.available.min(group.available),
    }
}

/// The limit of the daemon's own control group (cgroup v2), and what is
/// left of it; `None` without a limit.
#[cfg(target_os = "linux")]
fn own_group_limit() -> Option<MemoryInfo> {
    let groups = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    let path = cgroup_path(&groups)?;
    let dir = std::path::Path::new("/sys/fs/cgroup").join(path.trim_start_matches('/'));
    let read = |name: &str| std::fs::read_to_string(dir.join(name)).ok();
    group_limit(&read("memory.max")?, &read("memory.current")?)
}

/// The path after `0::` in `/proc/self/cgroup` (cgroup v2).
fn cgroup_path(text: &str) -> Option<&str> {
    text.lines().find_map(|line| line.strip_prefix("0::"))
}

/// `memory.max` (`max` means none) and `memory.current`, in bytes.
fn group_limit(max: &str, current: &str) -> Option<MemoryInfo> {
    let total = max.trim().parse::<u64>().ok()?;
    let used = current.trim().parse::<u64>().ok()?;
    Some(MemoryInfo {
        total,
        available: total.saturating_sub(used),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meminfo_gives_the_total_and_what_is_available() {
        let text = "MemTotal:        2014216 kB\nMemFree:          112340 kB\nMemAvailable:    1204024 kB\nBuffers:           50000 kB\n";
        assert_eq!(
            parse_meminfo(text),
            Some(MemoryInfo {
                total: 2_014_216 * 1024,
                available: 1_204_024 * 1024
            })
        );
        assert_eq!(parse_meminfo("MemTotal: 1 kB\n"), None);
    }

    #[test]
    fn a_service_limit_lowers_what_the_system_says() {
        assert_eq!(
            cgroup_path("0::/system.slice/botloft.service\n"),
            Some("/system.slice/botloft.service")
        );
        assert_eq!(group_limit("max\n", "123\n"), None);
        let group = group_limit("1073741824\n", "536870912\n").expect("limit");
        assert_eq!(group.available, 536_870_912);
        let system = MemoryInfo {
            total: 4 * 1_073_741_824,
            available: 3 * 1_073_741_824,
        };
        assert_eq!(lower(system, group), group);
    }
}

//! What the system lets Botloft take (spec 42.4): on Linux, the limits of
//! the daemon's own control group (cgroup v2), which its agents share. A
//! service with `MemoryMax` and `CPUQuota`, or a container, sets them.

use super::super::GroupLimits;

pub fn group_limits() -> GroupLimits {
    #[cfg(target_os = "linux")]
    {
        let total = std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|text| mem_total(&text));
        let mut limits = GroupLimits {
            memory_total: total,
            cpus: online_cpus(),
            ..GroupLimits::default()
        };
        if let Some((path, dir)) = own_group() {
            limits.service = path.ends_with("/botloft.service");
            let read = |name: &str| std::fs::read_to_string(dir.join(name)).ok();
            limits.memory_limit = read("memory.max")
                .and_then(|text| bytes_limit(&text))
                .filter(|limit| total.is_none_or(|total| *limit < total));
            limits.memory_used = read("memory.current").and_then(|text| text.trim().parse().ok());
            limits.cpu_limit_percent = read("cpu.max").and_then(|text| cpu_percent(&text));
        }
        limits
    }
    #[cfg(not(target_os = "linux"))]
    {
        GroupLimits {
            cpus: std::thread::available_parallelism()
                .map_or(1, |cpus| u32::try_from(cpus.get()).unwrap_or(u32::MAX)),
            ..GroupLimits::default()
        }
    }
}

/// The group's path, as `/proc/self/cgroup` says it, and its folder.
#[cfg(target_os = "linux")]
fn own_group() -> Option<(String, std::path::PathBuf)> {
    let groups = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    let path = groups.lines().find_map(|line| line.strip_prefix("0::"))?;
    let dir = std::path::Path::new("/sys/fs/cgroup").join(path.trim_start_matches('/'));
    Some((path.trim().to_owned(), dir))
}

/// Every processor the machine has, not only the ones a quota leaves.
#[cfg(target_os = "linux")]
fn online_cpus() -> u32 {
    // SAFETY: no pointers; -1 when the system cannot say.
    let online = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
    u32::try_from(online)
        .ok()
        .filter(|cpus| *cpus > 0)
        .unwrap_or(1)
}

/// `MemTotal`, in bytes.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn mem_total(text: &str) -> Option<u64> {
    text.lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))
        .and_then(|rest| rest.split_whitespace().next()?.parse::<u64>().ok())
        .map(|kb| kb * 1024)
}

/// `memory.max`: bytes, or `max` for none.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn bytes_limit(text: &str) -> Option<u64> {
    text.trim().parse().ok()
}

/// `cpu.max`: `<quota> <period>` in microseconds, or `max <period>` for none.
/// `200000 100000` is two processors, 200.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn cpu_percent(text: &str) -> Option<u32> {
    let mut parts = text.split_whitespace();
    let quota: u64 = parts.next()?.parse().ok()?;
    let period: u64 = parts.next()?.parse().ok().filter(|period| *period > 0)?;
    u32::try_from(quota * 100 / period).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_control_groups_files_are_read() {
        assert_eq!(
            mem_total("MemTotal:       16384000 kB\nMemFree: 1 kB\n"),
            Some(16_384_000 * 1024)
        );
        assert_eq!(bytes_limit("4294967296\n"), Some(4_294_967_296));
        assert_eq!(bytes_limit("max\n"), None);
        assert_eq!(cpu_percent("200000 100000\n"), Some(200));
        assert_eq!(cpu_percent("150000 100000\n"), Some(150));
        assert_eq!(cpu_percent("max 100000\n"), None);
        assert_eq!(cpu_percent("100 0\n"), None);
    }
}

//! `botloftd limits` (spec 42.4): how much of a server Botloft may take. The
//! memory and the processors are the system's own caps on `botloft.service`
//! (`systemctl set-property`, which applies them at once and keeps them
//! across restarts), so nothing Botloft or an agent runs goes past them.
//! The agents awake at once are Botloft's setting, changed in the running
//! daemon like the app does.

use std::path::Path;

use anyhow::{Context, bail, ensure};
use botloft_core::protocol::{MAX_RUNNING_MAX, MachineLimits, SettingsUpdateParams, method};
use serde_json::Value;

use crate::config::Config;
use crate::local::Owner;
use crate::{platform, secrets};

/// The unit install.sh makes (spec 42.3).
pub const SERVICE: &str = "botloft.service";
const UNIT_FILE: &str = "/etc/systemd/system/botloft.service";
const MB: u64 = 1024 * 1024;
/// Less than this leaves no room for one agent beside the daemon and the
/// system's share (spec 42.4).
const MEMORY_MIN: u64 = 768 * MB;
/// Most processors `--cpus` takes.
const CPUS_MAX: u32 = 1024;

/// What `botloftd limits` was asked; nothing only shows them.
#[derive(Debug, Clone, Default)]
pub struct Options {
    pub memory: Option<String>,
    pub cpus: Option<String>,
    pub agents: Option<u32>,
}

pub fn run(home: &Path, options: &Options) -> anyhow::Result<()> {
    let memory = options
        .memory
        .as_deref()
        .map(memory_property)
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let cpus = options
        .cpus
        .as_deref()
        .map(cpu_property)
        .transpose()
        .map_err(anyhow::Error::msg)?;
    if let Some(agents) = options.agents {
        ensure!(
            agents <= MAX_RUNNING_MAX,
            "--agents takes 0 (no limit) to {MAX_RUNNING_MAX}"
        );
    }
    if memory.is_some() || cpus.is_some() {
        set_service(memory.as_deref(), cpus.as_deref())?;
    }
    let config = Config::load(&home.join("config.toml"), false)?;
    let token = secrets::read_owner_token(&home.join("secrets"));
    let Ok(token) = token else {
        ensure!(
            options.agents.is_none(),
            "cannot read Botloft's token in {}; run this as the user Botloft runs as, or with sudo",
            home.display()
        );
        println!("The new limits are set. Run this with sudo to see them.");
        return Ok(());
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let limits = runtime.block_on(ask(config.port, &token, options.agents))?;
    print!("{}", describe(&limits));
    Ok(())
}

/// Sets the agents' cap, if asked, and reads the limits back.
async fn ask(port: u16, token: &str, agents: Option<u32>) -> anyhow::Result<MachineLimits> {
    let mut owner = Owner::connect(port, token, "botloftd limits")
        .await
        .context("Botloft is not answering here; is the service running?")?;
    if let Some(max) = agents {
        let params = SettingsUpdateParams {
            max_running: Some(max),
            ..SettingsUpdateParams::default()
        };
        owner
            .call(method::SETTINGS_UPDATE, serde_json::to_value(params)?)
            .await?;
    }
    let limits = owner.call(method::SYSTEM_LIMITS, Value::Null).await?;
    owner.close().await;
    serde_json::from_value(limits).context("Botloft answered something unexpected")
}

/// `systemctl set-property` on the service, as root.
fn set_service(memory: Option<&str>, cpus: Option<&str>) -> anyhow::Result<()> {
    ensure!(
        cfg!(target_os = "linux"),
        "the memory and the processors are limited only on a Linux server"
    );
    ensure!(
        Path::new(UNIT_FILE).is_file(),
        "Botloft does not run as {SERVICE} here. In a container, give the container its limits (docker run --memory 4g --cpus 2, or the limits of the service in Coolify)"
    );
    ensure!(
        platform::running_as_root(),
        "run this as root: sudo botloftd limits ..."
    );
    let mut args = vec!["set-property".to_owned(), SERVICE.to_owned()];
    if let Some(memory) = memory {
        args.push(format!("MemoryMax={memory}"));
    }
    if let Some(cpus) = cpus {
        args.push(format!("CPUQuota={cpus}"));
    }
    let output = std::process::Command::new("systemctl")
        .args(&args)
        .output()
        .context("cannot run systemctl")?;
    if !output.status.success() {
        bail!(
            "systemctl refused the limits: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

/// `--memory`: `4G`, `512M`, `50%` of the machine, or `none`; the value of
/// `MemoryMax`.
pub fn memory_property(text: &str) -> Result<String, String> {
    let text = text.trim();
    if text.eq_ignore_ascii_case("none") {
        return Ok("infinity".to_owned());
    }
    if let Some(percent) = text.strip_suffix('%') {
        return match percent.trim().parse::<u32>() {
            Ok(percent @ 10..=100) => Ok(format!("{percent}%")),
            _ => Err("--memory takes a share from 10% to 100%".to_owned()),
        };
    }
    let upper = text.to_ascii_uppercase();
    let number = upper.trim_end_matches('B');
    let (digits, unit) = number.split_at(number.len().saturating_sub(1));
    let unit = match unit {
        "K" => 1024,
        "M" => MB,
        "G" => 1024 * MB,
        "T" => 1024 * 1024 * MB,
        _ => {
            return Err(
                "--memory takes a size such as 4G or 512M, a share such as 50%, or none".to_owned(),
            );
        }
    };
    let bytes = digits
        .trim()
        .parse::<u64>()
        .ok()
        .and_then(|count| count.checked_mul(unit))
        .ok_or_else(|| "--memory takes a whole number before the unit, such as 4G".to_owned())?;
    if bytes < MEMORY_MIN {
        return Err(format!(
            "--memory takes at least {} MB: less leaves no room for an agent",
            MEMORY_MIN / MB
        ));
    }
    Ok(bytes.to_string())
}

/// `--cpus`: how many processors, `1.5` included, or `none`; the value of
/// `CPUQuota` (empty takes it off).
pub fn cpu_property(text: &str) -> Result<String, String> {
    let text = text.trim();
    if text.eq_ignore_ascii_case("none") {
        return Ok(String::new());
    }
    let refused = || {
        format!(
            "--cpus takes a number of processors from 0.1 to {CPUS_MAX}, such as 2 or 1.5, or none"
        )
    };
    let cpus: f64 = text.parse().map_err(|_| refused())?;
    if !cpus.is_finite() || !(0.1..=f64::from(CPUS_MAX)).contains(&cpus) {
        return Err(refused());
    }
    // Whole hundredths, as systemd takes them.
    let percent = (cpus * 100.0).round();
    Ok(format!("{percent:.0}%"))
}

/// The limits, as `botloftd limits` prints them.
pub fn describe(limits: &MachineLimits) -> String {
    let gb = |bytes: u64| format!("{:.1} GB", bytes as f64 / (1024.0 * MB as f64));
    let mut text = String::new();
    let memory = match (limits.memory_used, limits.memory_limit, limits.memory_total) {
        (Some(used), Some(limit), Some(total)) => format!(
            "{} used of {} (the machine has {})",
            gb(used),
            gb(limit),
            gb(total)
        ),
        (Some(used), None, Some(total)) => {
            format!(
                "{} used, no limit (the machine has {})",
                gb(used),
                gb(total)
            )
        }
        (_, Some(limit), _) => format!("up to {}", gb(limit)),
        (_, None, Some(total)) => format!("no limit (the machine has {})", gb(total)),
        _ => "unknown".to_owned(),
    };
    text.push_str(&format!("Memory:       {memory}\n"));
    let cpus = match limits.cpu_limit_percent {
        Some(percent) => format!("up to {} of {}", f64::from(percent) / 100.0, limits.cpus),
        None => format!("no limit ({} on the machine)", limits.cpus),
    };
    text.push_str(&format!("Processors:   {cpus}\n"));
    let most = if limits.agents_max == 0 {
        "no limit".to_owned()
    } else {
        format!("at most {}", limits.agents_max)
    };
    let fit = limits
        .agents_fit
        .map(|fit| format!(", memory takes {fit} more now"))
        .unwrap_or_default();
    text.push_str(&format!(
        "Agents awake: {} ({most}{fit})\n",
        limits.agents_awake
    ));
    if limits.changeable {
        text.push_str("Change them with: sudo botloftd limits --memory 4G --cpus 2 --agents 6\n");
    } else {
        text.push_str("Change the agents with: botloftd limits --agents 6\n");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_is_a_size_a_share_or_none() {
        assert_eq!(memory_property("4G"), Ok((4 * 1024 * MB).to_string()));
        assert_eq!(memory_property("4gb"), Ok((4 * 1024 * MB).to_string()));
        assert_eq!(memory_property("1024M"), Ok((1024 * MB).to_string()));
        assert_eq!(memory_property("50%"), Ok("50%".to_owned()));
        assert_eq!(memory_property("none"), Ok("infinity".to_owned()));
        assert!(memory_property("512M").is_err(), "no room for an agent");
        assert!(memory_property("5%").is_err());
        assert!(memory_property("150%").is_err());
        assert!(memory_property("4").is_err(), "a unit is needed");
        assert!(memory_property("1.5G").is_err());
        assert!(memory_property("lots").is_err());
    }

    #[test]
    fn processors_are_a_number_or_none() {
        assert_eq!(cpu_property("2"), Ok("200%".to_owned()));
        assert_eq!(cpu_property("1.5"), Ok("150%".to_owned()));
        assert_eq!(cpu_property("0.25"), Ok("25%".to_owned()));
        assert_eq!(cpu_property("none"), Ok(String::new()));
        assert!(cpu_property("0").is_err());
        assert!(cpu_property("-1").is_err());
        assert!(cpu_property("NaN").is_err());
        assert!(cpu_property("two").is_err());
    }

    #[test]
    fn the_limits_read_in_a_few_lines() {
        let limits = MachineLimits {
            memory_total: Some(16 * 1024 * MB),
            memory_limit: Some(4 * 1024 * MB),
            memory_used: Some(1024 * MB),
            cpus: 4,
            cpu_limit_percent: Some(200),
            agents_awake: 1,
            agents_max: 6,
            agents_fit: Some(9),
            changeable: true,
        };
        assert_eq!(
            describe(&limits),
            "Memory:       1.0 GB used of 4.0 GB (the machine has 16.0 GB)\n\
             Processors:   up to 2 of 4\n\
             Agents awake: 1 (at most 6, memory takes 9 more now)\n\
             Change them with: sudo botloftd limits --memory 4G --cpus 2 --agents 6\n"
        );
        let open = MachineLimits {
            memory_total: Some(16 * 1024 * MB),
            cpus: 4,
            ..MachineLimits::default()
        };
        assert!(describe(&open).contains("Memory:       no limit (the machine has 16.0 GB)"));
        assert!(describe(&open).contains("Processors:   no limit (4 on the machine)"));
        assert!(describe(&open).contains("Agents awake: 0 (no limit)"));
    }
}

//! How long a permission request waits for the owner (spec 10.1), as the
//! owner sets it: a longer wait reaches each bot's `mcp.json` with a
//! restart once nothing is in progress; a shorter one needs no restart.

mod common;

use botloft_core::protocol::{BotState, SettingsUpdateParams};
use botloftd::runtime::fake::FakeProcess;
use botloftd::service::{self, settings};
use common::supervised::{Setup, setup};

fn wait(s: &Setup, minutes: u32) -> service::ApiResult<botloft_core::protocol::Settings> {
    settings::update(
        &s.daemon,
        SettingsUpdateParams {
            approval_wait_minutes: Some(minutes),
            ..SettingsUpdateParams::default()
        },
    )
}

/// The `timeout` in the `mcp.json` the process was started with.
fn mcp_timeout(process: &FakeProcess) -> u64 {
    let args = process.args();
    let at = args
        .iter()
        .position(|arg| arg == "--mcp-config")
        .expect("mcp flag");
    let text = std::fs::read_to_string(&args[at + 1]).expect("mcp.json");
    let json: serde_json::Value = serde_json::from_str(&text).expect("json");
    json["mcpServers"]["botloft"]["timeout"]
        .as_u64()
        .expect("timeout")
}

#[tokio::test(start_paused = true)]
async fn a_longer_wait_restarts_idle_bots_and_a_shorter_one_does_not() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;

    let saved = wait(&s, 240).expect("longer");
    assert_eq!(saved.approval_wait_minutes, 240);
    let second = s.runtime.process(2).await;
    assert!(first.killed());
    assert_eq!(mcp_timeout(&second), (240 + 2) * 60 * 1000);
    s.until(BotState::Idle).await;

    wait(&s, 30).expect("shorter");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    assert_eq!(
        s.runtime.processes().len(),
        2,
        "no restart for a shorter wait"
    );
    assert_eq!(
        s.daemon.settings.approval_wait(),
        std::time::Duration::from_secs(30 * 60)
    );
}

#[tokio::test(start_paused = true)]
async fn a_wait_outside_a_day_is_refused() {
    let s = setup().await;
    for minutes in [0, 24 * 60 + 1] {
        let err = wait(&s, minutes).expect_err("refused");
        assert!(err.to_string().contains("between 1 and 1440"), "{err}");
    }
}

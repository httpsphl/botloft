//! A real Chromium window (spec 24.5): its first reading has the page that
//! Chromium builds for readers only once one asks. It opens a browser
//! window on the computer it runs on, so it runs only with
//! BOTLOFT_CHROMIUM_TESTS=1, as CI's Windows runner sets.

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use super::super::{read, windows as listed};
use super::test_window::alone;
use crate::platform::ProcessJob;

const TITLE: &str = "Botloft desktop test: chromium";

fn allowed() -> bool {
    let on = std::env::var("BOTLOFT_CHROMIUM_TESTS").is_ok_and(|value| value == "1");
    if !on {
        eprintln!("Chromium window tests are off: set BOTLOFT_CHROMIUM_TESTS=1");
    }
    on
}

/// Chrome, else the Edge that comes with Windows: a new profile of Edge
/// signs in to the Windows account by itself.
fn browser() -> Option<PathBuf> {
    [
        ("ProgramFiles", r"Google\Chrome\Application\chrome.exe"),
        (
            "ProgramFiles(x86)",
            r"Microsoft\Edge\Application\msedge.exe",
        ),
        ("ProgramFiles", r"Microsoft\Edge\Application\msedge.exe"),
    ]
    .iter()
    .filter_map(|(base, path)| Some(PathBuf::from(std::env::var_os(base)?).join(path)))
    .find(|path| path.is_file())
}

#[test]
fn the_first_reading_of_a_chromium_window_has_its_page() {
    if !allowed() {
        return;
    }
    // A new window takes the focus from the tests' own.
    let _turn = alone();
    let program = browser().expect("Chrome or Edge");
    let profile = tempfile::tempdir().expect("a profile");
    let buttons: String = (1..=40)
        .map(|number| format!("<button>Button%20{number}</button>"))
        .collect();
    let title = TITLE.replace(' ', "%20");
    let job = ProcessJob::new().expect("a job");
    let mut process = Command::new(program)
        .arg(format!("--user-data-dir={}", profile.path().display()))
        // InPrivate, Edge shows no sign-in dialog over the page; Chrome
        // ignores the switch.
        .args([
            "--inprivate",
            "--no-first-run",
            "--no-default-browser-check",
            "--disable-sync",
        ])
        .arg(format!(
            "--app=data:text/html,<title>{title}</title>{buttons}"
        ))
        .spawn()
        .expect("start the browser");
    job.assign(&process).expect("the browser in the job");
    let started = Instant::now();
    let window = loop {
        if let Some(found) = listed()
            .expect("windows")
            .into_iter()
            .find(|found| found.title.contains(TITLE))
        {
            break found;
        }
        assert!(
            started.elapsed() < Duration::from_secs(60),
            "no browser window"
        );
        std::thread::sleep(Duration::from_millis(200));
    };
    let reading = read(window.id).expect("read");
    assert!(!reading.preparing, "{:?}", reading.controls);
    assert!(
        reading
            .controls
            .iter()
            .any(|control| control.kind == "button" && control.name == "Button 40"),
        "{:?}",
        reading.controls
    );
    // Closing the job ends the browser and everything it started.
    drop(job);
    let _ = process.wait();
}

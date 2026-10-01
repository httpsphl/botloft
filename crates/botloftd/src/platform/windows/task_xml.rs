//! The definition of the daemon's scheduled task, in Task Scheduler XML.
//!
//! - A logon trigger for the current user starts the daemon at logon,
//!   when the owner wants Botloft to start with Windows.
//! - A time trigger that began in the past and repeats every minute keeps
//!   it running: with `IgnoreNew`, a repetition does nothing while the
//!   daemon runs and starts it again within a minute after it dies. The
//!   logon trigger cannot repeat for this: its repetition only begins at
//!   the next logon, not when the task is installed. `RestartOnFailure`
//!   is not used either: it does not fire when the process exits with an
//!   error (both tested on Windows 11 25H2, spec 14). It is left out while
//!   the owner has stopped Botloft.
//! - Interactive token, least privilege: only while the user is logged on,
//!   in their session, with their Claude Code sign-in.
//! - No time limit, runs on battery, normal priority. The default priority
//!   (7) would pass "below normal" on to every bot.

use crate::platform::{TaskDefinition, Triggers};

const LOGON_TRIGGER: &str = "<LogonTrigger>";
const WATCHDOG_TRIGGER: &str = "<TimeTrigger>";

/// The arguments as one command line. Words of letters, digits and dashes
/// go as they are; anything else is quoted.
fn join_arguments(arguments: &[String]) -> String {
    let plain = |arg: &String| {
        !arg.is_empty()
            && arg
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    };
    arguments
        .iter()
        .map(|arg| if plain(arg) { arg.clone() } else { quote(arg) })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Quotes one argument for the Windows command line. Backslashes before
/// the closing quote are doubled, so `C:\` does not escape it.
fn quote(arg: &str) -> String {
    let trailing = arg.len() - arg.trim_end_matches('\\').len();
    format!("\"{arg}{}\"", "\\".repeat(trailing))
}

pub(super) fn render(task: &TaskDefinition, user_sid: &str) -> String {
    let sid = escape(user_sid);
    let description = escape(&task.description);
    let command = escape(&task.program.to_string_lossy());
    let arguments = escape(&join_arguments(&task.arguments));
    let working_dir = escape(&task.working_dir.to_string_lossy());
    let triggers = triggers_xml(task.triggers, &sid);
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Author>Botloft</Author>
    <Description>{description}</Description>
  </RegistrationInfo>
  <Triggers>
{triggers}  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{sid}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <IdleSettings>
      <StopOnIdleEnd>false</StopOnIdleEnd>
      <RestartOnIdle>false</RestartOnIdle>
    </IdleSettings>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>5</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{command}</Command>
      <Arguments>{arguments}</Arguments>
      <WorkingDirectory>{working_dir}</WorkingDirectory>
    </Exec>
  </Actions>
</Task>
"#
    )
}

fn triggers_xml(triggers: Triggers, sid: &str) -> String {
    let mut xml = String::new();
    if triggers.logon {
        xml.push_str(&format!(
            r"    {LOGON_TRIGGER}
      <Enabled>true</Enabled>
      <UserId>{sid}</UserId>
    </LogonTrigger>
"
        ));
    }
    if triggers.watchdog {
        xml.push_str(&format!(
            r"    {WATCHDOG_TRIGGER}
      <Enabled>true</Enabled>
      <StartBoundary>2026-01-01T00:00:00</StartBoundary>
      <Repetition>
        <Interval>PT1M</Interval>
        <StopAtDurationEnd>false</StopAtDurationEnd>
      </Repetition>
    </TimeTrigger>
"
        ));
    }
    xml
}

/// Which triggers a registered task has, from its XML.
pub(super) fn triggers(xml: &str) -> Triggers {
    Triggers {
        logon: xml.contains(LOGON_TRIGGER),
        watchdog: xml.contains(WATCHDOG_TRIGGER),
    }
}

/// The command line of the task's first action, as the owner would type it.
pub(super) fn command_line(xml: &str) -> Option<String> {
    let command = unescape(element(xml, "Command")?);
    let quoted = if command.contains(' ') {
        format!("\"{command}\"")
    } else {
        command
    };
    Some(match element(xml, "Arguments") {
        Some(arguments) => format!("{quoted} {}", unescape(arguments)),
        None => quoted,
    })
}

fn element<'a>(xml: &'a str, name: &str) -> Option<&'a str> {
    let open = format!("<{name}>");
    let start = xml.find(&open)? + open.len();
    let end = start + xml[start..].find(&format!("</{name}>"))?;
    Some(&xml[start..end])
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn task() -> TaskDefinition {
        TaskDefinition {
            description: "Botloft <daemon> & friends".into(),
            program: PathBuf::from(r"C:\Users\Ana Lima\AppData\Local\Botloft\bin\botloftd.exe"),
            arguments: vec![
                "serve".into(),
                "--home".into(),
                r"C:\Users\Ana Lima\AppData\Local\Botloft".into(),
            ],
            working_dir: PathBuf::from(r"C:\Users\Ana Lima\AppData\Local\Botloft"),
            keep_alive: PathBuf::from(r"C:\Users\Ana Lima\AppData\Local\Botloft\run\keep-alive"),
            triggers: Triggers {
                logon: true,
                watchdog: true,
            },
        }
    }

    #[test]
    fn the_definition_escapes_text_and_runs_for_the_user_at_logon() {
        let xml = render(&task(), "S-1-5-21-1-2-3-1001");
        assert!(xml.contains("<Description>Botloft &lt;daemon&gt; &amp; friends</Description>"));
        assert!(xml.contains(
            "<Arguments>serve --home &quot;C:\\Users\\Ana Lima\\AppData\\Local\\Botloft&quot;</Arguments>"
        ));
        assert_eq!(
            xml.matches("<UserId>S-1-5-21-1-2-3-1001</UserId>").count(),
            2
        );
        assert!(xml.contains("<LogonTrigger>"));
        assert!(xml.contains("<Interval>PT1M</Interval>"));
        assert!(xml.contains("<LogonType>InteractiveToken</LogonType>"));
        assert!(xml.contains("<MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>"));
        assert!(xml.contains("<ExecutionTimeLimit>PT0S</ExecutionTimeLimit>"));
        assert_eq!(triggers(&xml), task().triggers);
    }

    #[test]
    fn each_trigger_can_be_left_out() {
        for (logon, watchdog) in [(true, false), (false, true), (false, false)] {
            let mut task = task();
            task.triggers = Triggers { logon, watchdog };
            let xml = render(&task, "S-1-5-21-1-2-3-1001");
            assert_eq!(triggers(&xml), task.triggers);
            assert_eq!(xml.contains("<Interval>PT1M</Interval>"), watchdog);
            let users = xml.matches("<UserId>S-1-5-21-1-2-3-1001</UserId>").count();
            assert_eq!(
                users,
                if logon { 2 } else { 1 },
                "the principal is always there"
            );
        }
    }

    #[test]
    fn the_command_line_comes_back_out_of_the_definition() {
        let xml = render(&task(), "S-1-5-21-1-2-3-1001");
        assert_eq!(
            command_line(&xml).as_deref(),
            Some(
                r#""C:\Users\Ana Lima\AppData\Local\Botloft\bin\botloftd.exe" serve --home "C:\Users\Ana Lima\AppData\Local\Botloft""#
            )
        );
        assert_eq!(command_line("<Task/>"), None);
    }

    #[test]
    fn arguments_are_quoted_for_the_command_line() {
        assert_eq!(quote(r"D:\"), r#""D:\\""#);
        let args = ["serve", "--home", r"C:\data", "--scheduled"].map(String::from);
        assert_eq!(
            join_arguments(&args),
            r#"serve --home "C:\data" --scheduled"#
        );
    }
}

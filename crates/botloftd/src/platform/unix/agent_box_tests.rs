use super::*;

fn setup() -> BoxSetup {
    BoxSetup {
        bwrap: "/usr/bin/bwrap".into(),
        exe: "/usr/local/bin/botloftd".into(),
        home: "/var/lib/botloft-agents".into(),
        mcp_socket: "/var/lib/botloft/workspaces/.botloft/box/mcp.sock".into(),
        gate_socket: "/var/lib/botloft/workspaces/.botloft/box/gate.sock".into(),
        port: 45710,
    }
}

fn texts(plan: &[OsString]) -> Vec<String> {
    plan.iter()
        .map(|part| part.to_string_lossy().into_owned())
        .collect()
}

/// The argument after each `flag`, in order.
fn after(plan: &[String], flag: &str) -> Vec<String> {
    plan.windows(2)
        .filter(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
        .collect()
}

fn made() -> Vec<String> {
    let system = vec![
        Mount::ReadOnly("/usr".into()),
        Mount::Link {
            target: "usr/bin".into(),
            link: "/bin".into(),
        },
    ];
    let writable = [
        PathBuf::from("/var/lib/botloft/workspaces/ops"),
        PathBuf::from("/srv/site"),
    ];
    texts(&plan(
        &setup(),
        &system,
        Path::new("/var/lib/botloft-agents/.local/bin/claude"),
        &["-p".into(), "--verbose".into()],
        Path::new("/var/lib/botloft/workspaces/ops/scout"),
        &writable,
    ))
}

#[test]
fn the_box_writes_only_its_own_folders() {
    let plan = made();
    assert_eq!(
        after(&plan, "--bind"),
        [
            "/var/lib/botloft-agents",
            "/var/lib/botloft/workspaces/ops",
            "/srv/site",
            "/var/lib/botloft/workspaces/.botloft/box/mcp.sock",
            "/var/lib/botloft/workspaces/.botloft/box/gate.sock",
        ]
    );
    // The system is read-only, a link where it is a link.
    assert!(after(&plan, "--ro-bind").contains(&"/usr".to_owned()));
    assert_eq!(after(&plan, "--symlink"), ["usr/bin"]);
    // Only what programs need of /etc; never the data folder.
    let etc = after(&plan, "--ro-bind-try");
    assert!(etc.contains(&"/etc/passwd".to_owned()));
    assert!(etc.contains(&"/etc/ssl".to_owned()));
    assert!(
        !etc.iter()
            .any(|path| path == "/etc" || path.starts_with("/etc/systemd"))
    );
    assert!(
        !plan
            .iter()
            .any(|part| part.contains("/var/lib/botloft/home"))
    );
}

#[test]
fn the_box_has_its_own_processes_network_and_temporary_files() {
    let plan = made();
    assert_eq!(plan[..2], ["--unshare-all", "--die-with-parent"]);
    assert_eq!(after(&plan, "--proc"), ["/proc"]);
    assert_eq!(after(&plan, "--tmpfs"), ["/tmp", "/var/tmp", "/run"]);
    assert_eq!(
        after(&plan, "--chdir"),
        ["/var/lib/botloft/workspaces/ops/scout"]
    );
}

#[test]
fn the_box_reaches_botloft_and_the_internet_through_its_bridges() {
    let plan = made();
    let start = plan.iter().position(|part| part == "--").expect("command");
    assert_eq!(
        plan[start + 1..],
        [
            "/run/botloft/botloftd",
            "box-bridge",
            "127.0.0.1:45710=/run/botloft/mcp.sock",
            "127.0.0.1:3128=/run/botloft/gate.sock",
            "--",
            "/var/lib/botloft-agents/.local/bin/claude",
            "-p",
            "--verbose",
        ]
    );
    let set = after(&plan, "--setenv");
    assert!(set.contains(&"HTTPS_PROXY".to_owned()));
    assert!(plan.contains(&"http://127.0.0.1:3128".to_owned()));
    assert!(plan.contains(&"127.0.0.1,localhost,::1".to_owned()));
    assert_eq!(gate_port(3128), 3129);
}

#[test]
fn without_a_box_the_command_is_the_program_itself() {
    // No box was set in this process's tests.
    let command = agent_command(Path::new("/bin/echo"), &["hi".into()], Path::new("/"), &[]);
    assert_eq!(command.get_program(), "/bin/echo");
}

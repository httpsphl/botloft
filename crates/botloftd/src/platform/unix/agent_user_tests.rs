use super::*;

#[test]
fn the_switch_needs_all_three_capabilities() {
    // CAP_KILL, CAP_SETGID and CAP_SETUID: bits 5, 6 and 7.
    assert!(can_switch("Name:\tbotloftd\nCapEff:\t00000000000000e0\n"));
    assert!(can_switch("CapEff:\t000001ffffffffff\n"));
    assert!(!can_switch("CapEff:\t00000000000000c0\n"));
    assert!(!can_switch("CapEff:\t0000000000000000\n"));
    assert!(!can_switch("Name:\tbotloftd\n"));
}

#[test]
fn a_user_that_is_not_there_is_refused() {
    assert_eq!(
        use_agent_user("botloft-no-such-user").map_err(|err| err.contains("no user")),
        Err(true)
    );
    assert!(lookup("root").is_some_and(|root| root.uid == 0));
}

/// Only as root (a container): `nobody` cannot read a file only root
/// may, and the command runs as `nobody`.
#[test]
fn a_command_as_another_user_cannot_read_the_daemons_secrets() {
    // SAFETY: no arguments, never fails.
    if unsafe { libc::geteuid() } != 0 {
        return;
    }
    let Some(nobody) = lookup("nobody") else {
        return;
    };
    let dir = tempfile::tempdir().expect("dir");
    let secret = dir.path().join("owner.token");
    std::fs::write(&secret, "s3cret").expect("secret");
    std::fs::set_permissions(&secret, std::os::unix::fs::PermissionsExt::from_mode(0o600))
        .expect("mode");
    // Not through the daemon-wide user: the other tests run as they are.
    let mut command = Command::new("sh");
    command
        .args(["-c", "id -u; cat \"$1\" 2>/dev/null || echo refused", "sh"])
        .arg(&secret)
        .current_dir("/");
    switch(&mut command, &nobody);
    let output = command.output().expect("run");
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        text.lines().next(),
        Some(nobody.uid.to_string().as_str()),
        "{text}"
    );
    assert!(
        text.contains("refused") && !text.contains("s3cret"),
        "{text}"
    );
}

/// Only as root (a container): the command keeps every group of the
/// user's, as the browsers' user needs the daemon's group for its
/// profiles; and none of the daemon's own.
#[test]
fn a_command_as_another_user_keeps_that_users_groups() {
    // SAFETY: no arguments, never fails.
    if unsafe { libc::geteuid() } != 0 {
        return;
    }
    let Some(mut nobody) = lookup("nobody") else {
        return;
    };
    let extra = nobody.gid + 4242;
    nobody.groups = vec![nobody.gid, extra];
    let mut command = Command::new("id");
    command.arg("-G").current_dir("/");
    switch(&mut command, &nobody);
    let output = command.output().expect("run");
    let text = String::from_utf8_lossy(&output.stdout);
    let mut ids: Vec<&str> = text.split_whitespace().collect();
    ids.sort_unstable();
    let mut wanted = [nobody.gid.to_string(), extra.to_string()];
    wanted.sort_unstable();
    assert_eq!(ids, wanted, "{text}");
}

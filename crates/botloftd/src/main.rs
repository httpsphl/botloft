//! `botloftd` command line.

mod cli;

use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use anyhow::Context;
use botloft_store::Store;
use botloftd::browser::{self, BrowserSettings};
use botloftd::clock::SystemClock;
use botloftd::config::Config;
use botloftd::courier::{self, CourierSettings};
use botloftd::loops::Loops;
use botloftd::paths::{self, Paths};
use botloftd::platform::{self, InstanceLock};
use botloftd::runtime::PipeRuntime;
use botloftd::service::tasks::TaskSettings;
use botloftd::settings::LiveSettings;
use botloftd::state::{BotSettings, Daemon, DaemonOptions};
use botloftd::supervisor::{self, SupervisorSettings};
use botloftd::trash::{FolderTrash, RecycleBin, Trash};
use botloftd::{
    approvals, autostart, keep_awake, logging, pair, routines, secrets, server, tunnel,
};
use clap::Parser as _;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use crate::cli::{Cli, Command, ServiceCommand, TunnelCommand};

/// Errors go to stderr on one line (`what failed: why`), which the app
/// shows as is when it runs `service install`.
fn main() -> ExitCode {
    match dispatch(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}

fn dispatch(cli: Cli) -> anyhow::Result<()> {
    let home = match cli.command {
        // Run by hand, also on a server set up by install.sh.
        Command::Pair { .. }
        | Command::Update { .. }
        | Command::Tunnel(_)
        | Command::Limits { .. } => paths::resolve_home_by_hand(cli.home.as_deref()),
        _ => paths::resolve_home(cli.home.as_deref()),
    }
    .context("cannot resolve the data folder")?;
    match cli.command {
        Command::Serve { config, scheduled } => serve(home, config, scheduled),
        Command::Service(ServiceCommand::Install) => autostart::install(&home),
        Command::Service(ServiceCommand::Status) => autostart::status(&home),
        Command::Service(ServiceCommand::Restart) => autostart::restart(&home),
        Command::Service(ServiceCommand::Stop) => autostart::stop(&home),
        Command::Service(ServiceCommand::Uninstall) => autostart::uninstall(&home),
        Command::Pair {
            name,
            host,
            ssh_port,
        } => pair::run(
            &home,
            &pair::Options {
                name,
                host,
                ssh_port,
            },
        ),
        Command::Limits {
            memory,
            cpus,
            agents,
        } => botloftd::limits::run(
            &home,
            &botloftd::limits::Options {
                memory,
                cpus,
                agents,
            },
        ),
        Command::Tunnel(TunnelCommand::Setup) => tunnel::setup(&home),
        Command::Tunnel(TunnelCommand::Remove) => tunnel::remove(),
        Command::Tunnel(TunnelCommand::Keys { user }) => {
            tunnel::keys(&home, user.as_deref()).map(|keys| print!("{keys}"))
        }
        Command::Update { check, now } => {
            botloftd::update::update(&home, &botloftd::update::Options { check, now })
        }
    }
}

fn serve(home: PathBuf, config_path: Option<PathBuf>, scheduled: bool) -> anyhow::Result<()> {
    platform::leave_own_console();
    std::fs::create_dir_all(&home).with_context(|| format!("cannot create {}", home.display()))?;
    let config_path = config_path.unwrap_or_else(|| home.join("config.toml"));
    let config = Config::load(&config_path, config_path != home.join("config.toml"))?;
    let paths = Paths::new(home, paths::resolve_workspaces_root(&config)?);
    std::fs::create_dir_all(paths.logs())?;
    let _log_guard = logging::init(&paths.logs(), &config.log_level)?;
    // Started by the scheduled task, nobody sees stderr: the log has to
    // say why the daemon stopped.
    run(paths, config, config_path, scheduled).inspect_err(|err| error!("{err:#}"))
}

fn run(paths: Paths, config: Config, config_path: PathBuf, scheduled: bool) -> anyhow::Result<()> {
    let _lock = InstanceLock::acquire(&paths.lock_file())?;
    // Before anything runs Claude Code: with a user of their own, the
    // agents never see the data folder (spec 42.5). Asked for and not
    // possible, nothing starts rather than run them without it.
    let apart = config.agents_user.trim();
    if !apart.is_empty() {
        platform::use_agent_user(apart)
            .map_err(|err| anyhow::anyhow!("the agents cannot run as {apart}: {err}"))?;
        info!(user = apart, "the agents run as their own user");
    }
    let browser = config.browser_user.trim();
    if !browser.is_empty() {
        platform::use_browser_user(browser)
            .map_err(|err| anyhow::anyhow!("the browsers cannot run as {browser}: {err}"))?;
        info!(user = browser, "the agents' browsers run as their own user");
    }
    // Before any bot starts: ends what a crashed run left behind (14.1).
    platform::track_groups(&paths.home.join("run"));
    if scheduled && !autostart::scheduled_start(&paths.home, config.start_with_windows) {
        info!("a new sign-in, and Botloft does not start with Windows: waiting to be opened");
        return Ok(());
    }

    let owner_token = secrets::load_or_create_owner_token(&paths.secrets())
        .context("cannot prepare the owner token")?;
    // A backup the owner confirmed replaces the data before anything opens
    // it (14.2); what was there goes aside.
    let stamp = jiff::Zoned::now().strftime("%Y-%m-%d-%H%M%S").to_string();
    match botloftd::backup::restore::apply_pending(&paths, &stamp) {
        Ok(Some(aside)) => info!(aside = %aside.display(), "a backup was restored"),
        Ok(None) => {}
        Err(err) => error!("could not restore the backup, so nothing changed: {err}"),
    }
    let store = Store::open(&paths.db()).context("cannot open the database")?;
    std::fs::create_dir_all(&paths.workspaces_root)
        .with_context(|| format!("cannot create {}", paths.workspaces_root.display()))?;

    info!(home = %paths.home.display(), workspaces = %paths.workspaces_root.display(), server = config.server, "starting");
    let secrets = botloftd::autobackup::SystemSecrets::for_home(&paths.home);
    // A server has no Recycle Bin: deleted folders wait in its own trash
    // (spec 42.2).
    let trash: Arc<dyn Trash> = if config.server {
        let trash = FolderTrash::new(paths.home.join("trash"));
        trash.prune(jiff::Timestamp::now().as_millisecond());
        Arc::new(trash)
    } else {
        Arc::new(RecycleBin)
    };
    let daemon = Daemon::new(DaemonOptions {
        paths,
        port: config.port,
        store,
        owner_token,
        runtime: Arc::new(PipeRuntime),
        supervisor: SupervisorSettings::from_config(&config),
        clock: Arc::new(SystemClock),
        courier: CourierSettings::from_config(&config),
        tasks: TaskSettings::from_config(&config),
        bots: BotSettings::from_config(&config),
        browser: BrowserSettings::from_config(&config),
        settings: LiveSettings::new(Some(config_path), &config),
        trash,
        owner_idle: Arc::new(platform::desktop::owner_idle),
        cloud: botloftd::cloud::CloudSettings::from_config(&config),
        secrets: Arc::new(secrets),
        server: config.server,
        memory: Arc::new(botloftd::memory::SystemMemory),
    });
    // No bot process survived the last run, so nobody waits for these,
    // and what they were written and never began goes back in the queue.
    approvals::expire_all(&daemon);
    courier::requeue_left_behind(&daemon);
    // Before the supervisor looks at the bots (spec 36.2).
    if let Err(err) = botloftd::service::maker::ensure(&daemon) {
        warn!("could not make the Maker: {err}");
    }
    // Ctrl+Alt+Esc stops every bot on the desktop, app open or not (spec
    // 24.9). A server has no desktop (spec 42.2).
    if !daemon.server {
        let stopping = Arc::clone(&daemon);
        match platform::desktop::on_stop_key(move || {
            botloftd::service::desktop::stop_all(&stopping)
        }) {
            Ok(()) | Err(platform::desktop::DesktopError::Unavailable) => {}
            Err(err) => {
                warn!("the shortcut that stops bots on the desktop is not available: {err}");
            }
        }
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, config.port))
            .await
            .with_context(|| {
                format!(
                    "cannot listen on 127.0.0.1:{}; is another program using the port?",
                    config.port
                )
            })?;
        info!(port = config.port, "listening on 127.0.0.1");
        // Each of these runs as long as the daemon does (spec 14).
        let mut loops = Loops::new();
        // A backup from another computer may name folders that are no path
        // here (spec 42.10); before any agent starts in one.
        match botloftd::service::moved::forget_foreign_folders(&daemon) {
            Ok(0) => {}
            Ok(moved) => info!(
                moved,
                "crews from another computer work in their own folders"
            ),
            Err(err) => warn!("could not check the crews' folders: {err}"),
        }
        loops.spawn("supervisor", supervisor::run(Arc::clone(&daemon)));
        loops.spawn("courier", courier::run(Arc::clone(&daemon)));
        loops.spawn("approvals", approvals::forget_turns(Arc::clone(&daemon)));
        // What changes in a conversation, as the apps hear it (spec 40.5).
        loops.spawn(
            "conversations",
            botloftd::service::conversations::follow(Arc::clone(&daemon)),
        );
        // Names the conversations that are not the main one (spec 40.6).
        loops.spawn("titles", botloftd::titles::run(Arc::clone(&daemon)));
        loops.spawn("routines", routines::run(Arc::clone(&daemon)));
        loops.spawn("waits", routines::waits::run(Arc::clone(&daemon)));
        loops.spawn("backup", botloftd::autobackup::run(Arc::clone(&daemon)));
        loops.spawn("phone relay", botloftd::mobile::run(Arc::clone(&daemon)));
        loops.spawn("browser", browser::run(Arc::clone(&daemon)));
        // The plan read from the owner's Claude account (spec 8.13).
        loops.spawn("plan usage", botloftd::plan_usage::run(Arc::clone(&daemon)));
        // The outline around the window a bot uses (spec 24.9).
        if !daemon.server {
            loops.spawn(
                "desktop outline",
                botloftd::desktop::outline::run(Arc::clone(&daemon)),
            );
        }
        // Forgets the history older than the owner keeps (spec 12.1).
        loops.spawn("history", botloftd::history::run(Arc::clone(&daemon)));
        // The owner's claude.ai tools, for the agents' rules (spec 34.3).
        tokio::spawn(botloftd::connectors::look_at_start(Arc::clone(&daemon)));
        // Ends on its own where nothing keeps the computer awake. A server
        // does not sleep (spec 42.2).
        if !daemon.server {
            tokio::spawn(keep_awake::run(
                daemon.supervisor.busy_bots(),
                daemon.settings.keep_awake(),
            ));
        }
        let serving = server::serve(Arc::clone(&daemon), listener, platform::shutdown_signal());
        tokio::select! {
            served = serving => served?,
            ended = loops.first_ended() => {
                daemon.supervisor.shutdown();
                // Not a stop asked for: the service brings the daemon back.
                anyhow::bail!("{ended}; stopping so Botloft starts again");
            }
            () = daemon.restart_requested() => {
                // The answer to `system.restart` goes out first.
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                daemon.supervisor.shutdown();
                anyhow::bail!("restarting as asked; the service starts Botloft again");
            }
        }
        daemon.supervisor.shutdown();
        // A stop asked for (logout, `service stop`): launchd must not
        // bring the daemon back (spec 14.1).
        autostart::stopped_cleanly(&daemon.paths.home);
        info!("stopped");
        anyhow::Ok(())
    })
}

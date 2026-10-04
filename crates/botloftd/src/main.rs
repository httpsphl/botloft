//! `botloftd` command line.

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
use botloftd::paths::{self, Paths};
use botloftd::platform::{self, InstanceLock};
use botloftd::runtime::PipeRuntime;
use botloftd::service::tasks::TaskSettings;
use botloftd::settings::LiveSettings;
use botloftd::state::{BotSettings, Daemon, DaemonOptions};
use botloftd::supervisor::{self, SupervisorSettings};
use botloftd::trash::RecycleBin;
use botloftd::{approvals, autostart, keep_awake, logging, routines, secrets, server};
use clap::{Parser, Subcommand};
use tokio::net::TcpListener;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "botloftd", version, about = "Botloft daemon")]
struct Cli {
    /// Data folder. Defaults to BOTLOFT_HOME, then %LOCALAPPDATA%\Botloft.
    #[arg(long, global = true)]
    home: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the daemon in the foreground.
    Serve {
        /// Config file. Defaults to config.toml in the data folder.
        #[arg(long)]
        config: Option<PathBuf>,
        /// Started by the scheduled task (spec 14).
        #[arg(long, hide = true)]
        scheduled: bool,
    },
    /// Start the daemon at logon, as a scheduled task of the current user.
    #[command(subcommand)]
    Service(ServiceCommand),
}

#[derive(Subcommand)]
enum ServiceCommand {
    /// Copy this binary to the data folder, register the task and start it.
    Install,
    /// Show whether the task is installed and the daemon answers.
    Status,
    /// Stop the daemon and start it again.
    Restart,
    /// Stop the daemon until Botloft is opened again, or until the next
    /// logon if it starts with Windows.
    Stop,
    /// Stop the daemon and remove the task. Bots and data stay.
    Uninstall,
}

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
    let home =
        paths::resolve_home(cli.home.as_deref()).context("cannot resolve the data folder")?;
    match cli.command {
        Command::Serve { config, scheduled } => serve(home, config, scheduled),
        Command::Service(ServiceCommand::Install) => autostart::install(&home),
        Command::Service(ServiceCommand::Status) => autostart::status(&home),
        Command::Service(ServiceCommand::Restart) => autostart::restart(&home),
        Command::Service(ServiceCommand::Stop) => autostart::stop(&home),
        Command::Service(ServiceCommand::Uninstall) => autostart::uninstall(&home),
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
    // Before any bot starts: ends what a crashed run left behind (14.1).
    platform::track_groups(&paths.home.join("run"));
    if scheduled && !autostart::scheduled_start(&paths.home, config.start_with_windows) {
        info!("a new sign-in, and Botloft does not start with Windows: waiting to be opened");
        return Ok(());
    }

    let owner_token = secrets::load_or_create_owner_token(&paths.secrets())
        .context("cannot prepare the owner token")?;
    let store = Store::open(&paths.db()).context("cannot open the database")?;
    std::fs::create_dir_all(&paths.workspaces_root)
        .with_context(|| format!("cannot create {}", paths.workspaces_root.display()))?;

    info!(home = %paths.home.display(), workspaces = %paths.workspaces_root.display(), "starting");
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
        trash: Arc::new(RecycleBin),
        owner_idle: Arc::new(platform::desktop::owner_idle),
    });
    // No bot process survived the last run, so nobody waits for these.
    approvals::expire_all(&daemon);

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
        tokio::spawn(supervisor::run(Arc::clone(&daemon)));
        tokio::spawn(courier::run(Arc::clone(&daemon)));
        tokio::spawn(routines::run(Arc::clone(&daemon)));
        tokio::spawn(browser::run(Arc::clone(&daemon)));
        tokio::spawn(keep_awake::run(
            daemon.supervisor.busy_bots(),
            daemon.settings.keep_awake(),
        ));
        server::serve(Arc::clone(&daemon), listener, platform::shutdown_signal()).await?;
        daemon.supervisor.shutdown();
        // A stop asked for (logout, `service stop`): launchd must not
        // bring the daemon back (spec 14.1).
        autostart::stopped_cleanly(&daemon.paths.home);
        info!("stopped");
        anyhow::Ok(())
    })
}

//! `botloftd` command line.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use anyhow::Context;
use botloft_store::Store;
use botloftd::clock::SystemClock;
use botloftd::config::Config;
use botloftd::courier::{self, CourierSettings};
use botloftd::paths::{self, Paths};
use botloftd::platform::{self, InstanceLock};
use botloftd::runtime::PipeRuntime;
use botloftd::service::tasks::TaskSettings;
use botloftd::state::{BotSettings, Daemon, DaemonOptions};
use botloftd::supervisor::{self, SupervisorSettings};
use botloftd::{approvals, autostart, keep_awake, logging, secrets, server};
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
        Command::Serve { config } => serve(home, config),
        Command::Service(ServiceCommand::Install) => autostart::install(&home),
        Command::Service(ServiceCommand::Status) => autostart::status(&home),
        Command::Service(ServiceCommand::Restart) => autostart::restart(&home),
        Command::Service(ServiceCommand::Uninstall) => autostart::uninstall(&home),
    }
}

fn serve(home: PathBuf, config_path: Option<PathBuf>) -> anyhow::Result<()> {
    platform::leave_own_console();
    std::fs::create_dir_all(&home).with_context(|| format!("cannot create {}", home.display()))?;
    let config = match &config_path {
        Some(path) => Config::load(path, true)?,
        None => Config::load(&home.join("config.toml"), false)?,
    };
    let paths = Paths::new(home, paths::resolve_workspaces_root(&config)?);
    std::fs::create_dir_all(paths.logs())?;
    let _log_guard = logging::init(&paths.logs(), &config.log_level)?;
    // Started by the scheduled task, nobody sees stderr: the log has to
    // say why the daemon stopped.
    run(paths, config).inspect_err(|err| error!("{err:#}"))
}

fn run(paths: Paths, config: Config) -> anyhow::Result<()> {
    let _lock = InstanceLock::acquire(&paths.lock_file())?;

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
        if config.keep_awake {
            tokio::spawn(keep_awake::run(daemon.supervisor.busy_bots()));
        }
        server::serve(Arc::clone(&daemon), listener, platform::shutdown_signal()).await?;
        daemon.supervisor.shutdown();
        info!("stopped");
        anyhow::Ok(())
    })
}

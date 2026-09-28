//! `botloftd` command line.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use botloft_store::Store;
use botloftd::clock::SystemClock;
use botloftd::config::Config;
use botloftd::courier::{self, CourierSettings, PipeInbox};
use botloftd::paths::{self, Paths};
use botloftd::platform::{self, InstanceLock};
use botloftd::runtime::PtyRuntime;
use botloftd::service::tasks::TaskSettings;
use botloftd::state::{Daemon, DaemonOptions};
use botloftd::supervisor::{self, SupervisorSettings};
use botloftd::{hooks, logging, secrets, server};
use clap::{Parser, Subcommand};
use tokio::net::TcpListener;
use tracing::info;

#[derive(Parser)]
#[command(name = "botloftd", version, about = "Botloft daemon")]
struct Cli {
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
    /// Forward a Claude Code hook to the daemon (run by the bots' settings).
    #[command(hide = true)]
    Hook {
        /// session-start, prompt-submit, stop, stop-failure, notification or session-end.
        event: String,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Serve { config } => serve(config),
        Command::Hook { event } => {
            // Always exit 0, even on a panic: a failing hook must never get
            // in the way of the bot (spec 7.6).
            let _ = std::panic::catch_unwind(|| hooks::client::run(&event));
            std::process::exit(0);
        }
    }
}

fn serve(config_path: Option<PathBuf>) -> anyhow::Result<()> {
    let home = paths::resolve_home().context("cannot resolve the data folder")?;
    std::fs::create_dir_all(&home).with_context(|| format!("cannot create {}", home.display()))?;
    let config = match &config_path {
        Some(path) => Config::load(path, true)?,
        None => Config::load(&home.join("config.toml"), false)?,
    };
    let paths = Paths::new(home, paths::resolve_workspaces_root(&config)?);
    std::fs::create_dir_all(paths.logs())?;
    let _log_guard = logging::init(&paths.logs(), &config.log_level)?;
    let _lock = InstanceLock::acquire(&paths.lock_file())?;

    let owner_token = secrets::load_or_create_owner_token(&paths.secrets())
        .context("cannot prepare the owner token")?;
    let store = Store::open(&paths.db()).context("cannot open the database")?;
    std::fs::create_dir_all(&paths.workspaces_root)
        .with_context(|| format!("cannot create {}", paths.workspaces_root.display()))?;
    let bin = std::env::current_exe().context("cannot find the botloftd executable")?;

    info!(home = %paths.home.display(), workspaces = %paths.workspaces_root.display(), "starting");
    let daemon = Daemon::new(DaemonOptions {
        paths,
        port: config.port,
        bin,
        store,
        owner_token,
        runtime: Arc::new(PtyRuntime),
        supervisor: SupervisorSettings::from_config(&config),
        clock: Arc::new(SystemClock),
        inbox: Arc::new(PipeInbox),
        courier: CourierSettings::from_config(&config),
        tasks: TaskSettings::from_config(&config),
    });

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
        server::serve(Arc::clone(&daemon), listener, platform::shutdown_signal()).await?;
        daemon.supervisor.shutdown();
        info!("stopped");
        anyhow::Ok(())
    })
}

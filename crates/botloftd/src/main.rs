//! `botloftd` command line.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use botloft_store::Store;
use botloftd::config::Config;
use botloftd::paths::{self, Paths};
use botloftd::platform::{self, InstanceLock};
use botloftd::state::{Daemon, DaemonOptions};
use botloftd::{logging, secrets, server};
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
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Serve { config } => serve(config),
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
    let daemon = Arc::new(Daemon::new(DaemonOptions {
        paths,
        port: config.port,
        bin,
        store,
        owner_token,
    }));

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
        server::serve(daemon, listener, platform::shutdown_signal()).await?;
        info!("stopped");
        anyhow::Ok(())
    })
}

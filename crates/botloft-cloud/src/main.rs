//! `botloft-cloud serve --config cloud.toml` (spec 27.2), and
//! `botloft-cloud backup --to <file>` for the database.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use botloft_cloud::{
    AppState, Clock, Config, CopyStore, Db, Hub, Mailer, Pusher, SmtpMailer, Storage, router,
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "botloft-cloud", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Runs the server.
    Serve {
        #[arg(long, default_value = "cloud.toml")]
        config: PathBuf,
    },
    /// Writes a consistent copy of the database, which is safe while the
    /// server runs. The copies of the accounts are in the bucket or the
    /// folder, and are not part of it.
    Backup {
        #[arg(long, default_value = "cloud.toml")]
        config: PathBuf,
        /// The file to write; it must not exist yet.
        #[arg(long)]
        to: PathBuf,
    },
    /// Makes the key the server signs its Web Push calls with (spec 28.8):
    /// the private half goes in a new file, the public half is printed.
    VapidKey {
        /// The file to write; it must not exist yet.
        #[arg(long)]
        to: PathBuf,
    },
}

/// Waits for Ctrl+C, or for the `SIGTERM` that Docker and systemd send.
async fn shutdown() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        if let Ok(mut terminate) = signal(SignalKind::terminate()) {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = terminate.recv() => {}
            }
            return;
        }
    }
    let _ = tokio::signal::ctrl_c().await;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "botloft_cloud=info,warn".into()),
        )
        .init();
    match Cli::parse().command {
        Command::Serve { config } => serve(Config::load(&config)?).await,
        Command::VapidKey { to } => {
            let (private, public) = botloft_cloud::new_key()?;
            let mut file = std::fs::OpenOptions::new();
            file.write(true).create_new(true);
            #[cfg(unix)]
            std::os::unix::fs::OpenOptionsExt::mode(&mut file, 0o600);
            let mut file = file
                .open(&to)
                .context("cannot create the file (it must not exist)")?;
            std::io::Write::write_all(&mut file, private.as_bytes())?;
            println!("private key written to {}", to.display());
            println!("public key (the phones get it from the server): {public}");
            Ok(())
        }
        Command::Backup { config, to } => {
            let config = Config::load(&config)?;
            if let Some(folder) = to.parent().filter(|folder| !folder.as_os_str().is_empty()) {
                std::fs::create_dir_all(folder).context("cannot create the folder")?;
            }
            let db =
                Db::open(&config.data_dir.join("cloud.db")).context("cannot open the database")?;
            db.snapshot_to(&to).context("cannot write the copy")?;
            println!("database copied to {}", to.display());
            Ok(())
        }
    }
}

async fn serve(config: Config) -> anyhow::Result<()> {
    std::fs::create_dir_all(&config.data_dir).context("cannot create data_dir")?;
    let store = match (config.storage, &config.bucket) {
        (Storage::Bucket, Some(bucket)) => CopyStore::bucket(bucket)?,
        _ => CopyStore::disk(&config.data_dir.join("copies"))?,
    };
    let state = AppState {
        store,
        db: Db::open(&config.data_dir.join("cloud.db")).context("cannot open the database")?,
        mailer: Arc::new(Mailer::Smtp(Box::new(SmtpMailer::new(&config.smtp)?))),
        clock: Clock::default(),
        config: Arc::new(config.clone()),
        hub: Hub::default(),
        push: Pusher::from_config(&config)?,
    };
    let listener = tokio::net::TcpListener::bind(config.listen).await?;
    tracing::info!("listening on {}", config.listen);
    axum::serve(
        listener,
        router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown())
    .await?;
    Ok(())
}

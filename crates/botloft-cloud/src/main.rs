//! `botloft-cloud serve --config cloud.toml` (spec 27.2).

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use botloft_cloud::{AppState, Clock, Config, CopyStore, Db, Mailer, SmtpMailer, Storage, router};
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "botloft_cloud=info,warn".into()),
        )
        .init();
    let Command::Serve { config } = Cli::parse().command;
    let config = Config::load(&config)?;
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
    };
    let listener = tokio::net::TcpListener::bind(config.listen).await?;
    tracing::info!("listening on {}", config.listen);
    axum::serve(
        listener,
        router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await?;
    Ok(())
}

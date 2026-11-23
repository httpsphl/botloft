//! `botloftd`'s command line, as clap reads it. A module of the binary
//! (`main.rs`), not of the library.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "botloftd", version, about = "Botloft daemon")]
pub struct Cli {
    /// Data folder. Defaults to BOTLOFT_HOME, then %LOCALAPPDATA%\Botloft.
    #[arg(long, global = true)]
    pub home: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
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
    /// Pair a device, such as the app on another computer, and print the
    /// key it connects with. Run it where Botloft runs.
    Pair {
        /// The device's name, as the list of devices shows it.
        #[arg(long, default_value = "App")]
        name: String,
        /// The address the app reaches this server at, when not the one
        /// Botloft finds.
        #[arg(long)]
        host: Option<String>,
        /// The port the app reaches sshd at, when not the one sshd says.
        #[arg(long)]
        ssh_port: Option<u16>,
    },
    /// The app's own SSH tunnel to this Linux server.
    #[command(subcommand)]
    Tunnel(TunnelCommand),
    /// On a Linux server, put the newest release's daemon in place of this
    /// one once no agent is working, and restart the service. As root.
    Update {
        /// Only say whether there is a newer one.
        #[arg(long)]
        check: bool,
        /// Do not wait for the agents to finish.
        #[arg(long)]
        now: bool,
    },
}

#[derive(Subcommand)]
pub enum TunnelCommand {
    /// Let the app open its own tunnel here: a user that can only reach
    /// Botloft, and sshd set up for it. As root.
    Setup,
    /// Take back what `setup` did.
    Remove,
    /// The paired devices' keys, for sshd (its AuthorizedKeysCommand).
    #[command(hide = true)]
    Keys {
        /// The user sshd asks for; any other than botloft-tunnel gets none.
        user: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ServiceCommand {
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

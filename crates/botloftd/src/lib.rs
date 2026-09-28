//! `botloftd`: the Botloft daemon. It owns the database, runs the bots and
//! serves the app over JSON-RPC on 127.0.0.1. The binary in `main.rs` only
//! parses the command line and wires these modules together.

pub mod clock;
pub mod config;
pub mod courier;
pub mod hooks;
pub mod logging;
pub mod paths;
pub mod platform;
pub mod rpc;
pub mod runtime;
pub mod secrets;
pub mod server;
pub mod service;
pub mod state;
pub mod supervisor;
pub mod terminal;
pub mod workspace;

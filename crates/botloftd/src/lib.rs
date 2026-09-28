//! `botloftd`: the Botloft daemon. It owns the database, runs the bots and
//! serves the app over JSON-RPC on 127.0.0.1. The binary in `main.rs` only
//! parses the command line and wires these modules together.

pub mod config;
pub mod paths;
pub mod platform;
pub mod secrets;
pub mod workspace;

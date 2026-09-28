//! `botloftd`: the Botloft daemon.

fn main() {
    println!(
        "botloftd {} (protocol {})",
        env!("CARGO_PKG_VERSION"),
        botloft_core::PROTOCOL_VERSION
    );
}

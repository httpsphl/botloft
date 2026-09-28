//! Domain types, IDs, message envelope rendering and protocol types shared by
//! the daemon and the app.

/// Version of the app <-> daemon JSON-RPC protocol, negotiated in `session.hello`.
pub const PROTOCOL_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_version_starts_at_one() {
        assert_eq!(PROTOCOL_VERSION, 1);
    }
}

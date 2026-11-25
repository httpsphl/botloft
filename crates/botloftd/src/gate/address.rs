//! Which addresses the gate dials (spec 42.5): the public internet only.
//! Never this computer (its loopback, nor its own public address, where a
//! server's other systems answer too), its private networks, the cloud's
//! metadata service nor any reserved range; unless the owner named the
//! address in `reach`.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// An address or a range the owner lets the agents reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    base: IpAddr,
    bits: u8,
}

impl Range {
    /// `10.0.0.5`, `192.168.1.0/24` or `fd00::/8`.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let (base, bits) = match text.split_once('/') {
            Some((base, bits)) => (base.parse::<IpAddr>().ok()?, bits.parse::<u8>().ok()?),
            None => {
                let base = text.parse::<IpAddr>().ok()?;
                (base, if base.is_ipv4() { 32 } else { 128 })
            }
        };
        let most = if base.is_ipv4() { 32 } else { 128 };
        (bits <= most).then_some(Self { base, bits })
    }

    pub fn contains(&self, address: IpAddr) -> bool {
        match (self.base, unmapped(address)) {
            (IpAddr::V4(base), IpAddr::V4(address)) => prefix(
                u32::from(base).into(),
                u32::from(address).into(),
                self.bits,
                32,
            ),
            (IpAddr::V6(base), IpAddr::V6(address)) => {
                prefix(u128::from(base), u128::from(address), self.bits, 128)
            }
            _ => false,
        }
    }
}

fn prefix(base: u128, address: u128, bits: u8, width: u32) -> bool {
    if bits == 0 {
        return true;
    }
    let shift = width - u32::from(bits);
    base >> shift == address >> shift
}

/// An IPv6 form of an IPv4 address (mapped, NAT64, 6to4) is judged as the
/// IPv4 address it carries.
fn unmapped(address: IpAddr) -> IpAddr {
    let IpAddr::V6(v6) = address else {
        return address;
    };
    if let Some(v4) = v6.to_ipv4_mapped() {
        return IpAddr::V4(v4);
    }
    let segments = v6.segments();
    let embedded = |high: u16, low: u16| {
        IpAddr::V4(Ipv4Addr::new(
            (high >> 8) as u8,
            high as u8,
            (low >> 8) as u8,
            low as u8,
        ))
    };
    // 64:ff9b::/96, NAT64.
    if segments[..6] == [0x64, 0xff9b, 0, 0, 0, 0] {
        return embedded(segments[6], segments[7]);
    }
    // 2002::/16, 6to4.
    if segments[0] == 0x2002 {
        return embedded(segments[1], segments[2]);
    }
    address
}

/// Whether `address` is on the public internet.
pub fn is_public(address: IpAddr) -> bool {
    match unmapped(address) {
        IpAddr::V4(v4) => public_v4(v4),
        IpAddr::V6(v6) => public_v6(v6),
    }
}

fn public_v4(address: Ipv4Addr) -> bool {
    const CLOSED: [(&str, u8); 15] = [
        ("0.0.0.0", 8),
        ("10.0.0.0", 8),
        ("100.64.0.0", 10),
        ("127.0.0.0", 8),
        ("169.254.0.0", 16),
        ("172.16.0.0", 12),
        ("192.0.0.0", 24),
        ("192.0.2.0", 24),
        ("192.88.99.0", 24),
        ("192.168.0.0", 16),
        ("198.18.0.0", 15),
        ("198.51.100.0", 24),
        ("203.0.113.0", 24),
        ("224.0.0.0", 4),
        ("240.0.0.0", 4),
    ];
    !CLOSED.iter().any(|(base, bits)| {
        Range::parse(&format!("{base}/{bits}"))
            .is_some_and(|range| range.contains(IpAddr::V4(address)))
    })
}

fn public_v6(address: Ipv6Addr) -> bool {
    // Only the global unicast block is the internet; inside it, the
    // documentation, Teredo and benchmarking ranges are not.
    let global = Range::parse("2000::/3").is_some_and(|range| range.contains(address.into()));
    let reserved = ["2001::/32", "2001:2::/48", "2001:db8::/32", "3fff::/20"]
        .iter()
        .filter_map(|text| Range::parse(text))
        .any(|range| range.contains(address.into()));
    global && !reserved
}

/// Why the gate does not dial `address`, or `None` when it may.
/// `own` are this computer's addresses and `reach` what the owner opened.
pub fn refusal(address: IpAddr, own: &[IpAddr], reach: &[Range]) -> Option<&'static str> {
    if reach.iter().any(|range| range.contains(address)) {
        return None;
    }
    let address = unmapped(address);
    if own.iter().any(|mine| unmapped(*mine) == address) {
        return Some("this computer");
    }
    if !is_public(address) {
        return Some("not the public internet");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(text: &str) -> IpAddr {
        text.parse().expect("address")
    }

    #[test]
    fn only_the_public_internet_is_public() {
        for open in ["1.1.1.1", "8.8.8.8", "93.184.215.14", "2606:4700::1111"] {
            assert!(is_public(ip(open)), "{open}");
        }
        for closed in [
            "127.0.0.1",
            "127.9.9.9",
            "0.0.0.0",
            "10.1.2.3",
            "172.17.0.1",
            "172.31.255.255",
            "192.168.0.10",
            "100.100.100.100",
            "169.254.169.254",
            "224.0.0.1",
            "255.255.255.255",
            "::",
            "::1",
            "fe80::1",
            "fd00::1",
            "ff02::1",
            "2001:db8::1",
            "::ffff:127.0.0.1",
            "::ffff:10.0.0.1",
            "64:ff9b::a9fe:a9fe",
            "2002:7f00:1::",
        ] {
            assert!(!is_public(ip(closed)), "{closed}");
        }
        assert!(is_public(ip("172.32.0.1")));
        assert!(is_public(ip("::ffff:1.1.1.1")));
    }

    #[test]
    fn this_computer_is_refused_even_on_a_public_address() {
        let own = [ip("203.0.114.7"), ip("2a02:4780::7")];
        assert_eq!(refusal(ip("203.0.114.7"), &own, &[]), Some("this computer"));
        assert_eq!(
            refusal(ip("::ffff:203.0.114.7"), &own, &[]),
            Some("this computer")
        );
        assert_eq!(
            refusal(ip("2a02:4780::7"), &own, &[]),
            Some("this computer")
        );
        assert_eq!(refusal(ip("203.0.114.8"), &own, &[]), None);
    }

    #[test]
    fn what_the_owner_opened_is_reached() {
        let reach = [
            Range::parse("10.0.0.5").expect("one"),
            Range::parse("192.168.1.0/24").expect("range"),
        ];
        assert_eq!(refusal(ip("10.0.0.5"), &[], &reach), None);
        assert_eq!(refusal(ip("192.168.1.77"), &[], &reach), None);
        assert!(refusal(ip("10.0.0.6"), &[], &reach).is_some());
        assert!(refusal(ip("192.168.2.1"), &[], &reach).is_some());
    }

    #[test]
    fn ranges_are_read_or_refused() {
        assert!(Range::parse("10.0.0.0/33").is_none());
        assert!(Range::parse("nope").is_none());
        assert!(
            Range::parse("fd00::/8")
                .expect("v6")
                .contains(ip("fd12::1"))
        );
        assert!(
            Range::parse("0.0.0.0/0")
                .expect("all")
                .contains(ip("10.0.0.1"))
        );
    }
}

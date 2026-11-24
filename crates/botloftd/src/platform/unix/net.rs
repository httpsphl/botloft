//! This computer's own addresses, which the gate never dials (spec 42.5): a
//! server's public address leads to its other systems as much as its
//! loopback does.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Every address on this computer's interfaces, as `getifaddrs` lists them.
pub fn own_addresses() -> Vec<IpAddr> {
    let mut first: *mut libc::ifaddrs = std::ptr::null_mut();
    // SAFETY: `getifaddrs` fills `first` with a list it allocates, which is
    // read below and freed once with `freeifaddrs`.
    if unsafe { libc::getifaddrs(&mut first) } != 0 {
        return Vec::new();
    }
    let mut addresses = Vec::new();
    let mut next = first;
    while !next.is_null() {
        // SAFETY: `next` is a node of the list `getifaddrs` made.
        let node = unsafe { &*next };
        if let Some(address) = unsafe { address_of(node.ifa_addr) } {
            addresses.push(address);
        }
        next = node.ifa_next;
    }
    // SAFETY: the list came from `getifaddrs` and is freed only here.
    unsafe { libc::freeifaddrs(first) };
    addresses
}

/// # Safety
/// `address` is null or points to a `sockaddr` of its family's size.
unsafe fn address_of(address: *const libc::sockaddr) -> Option<IpAddr> {
    if address.is_null() {
        return None;
    }
    // SAFETY: the family says which structure `address` points to.
    unsafe {
        match i32::from((*address).sa_family) {
            libc::AF_INET => {
                let v4 = &*address.cast::<libc::sockaddr_in>();
                Some(IpAddr::V4(Ipv4Addr::from(u32::from_be(v4.sin_addr.s_addr))))
            }
            libc::AF_INET6 => {
                let v6 = &*address.cast::<libc::sockaddr_in6>();
                Some(IpAddr::V6(Ipv6Addr::from(v6.sin6_addr.s6_addr)))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_is_one_of_this_computers_addresses() {
        assert!(own_addresses().iter().any(IpAddr::is_loopback));
    }
}

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::*;

#[test]
fn policy_allows_exact_and_subdomain_hosts_only() {
    let policy = EgressPolicy::new(&["api.example.com".to_string()]).expect("policy");

    assert!(policy.allows("api.example.com", 443));
    assert!(policy.allows("chat.api.example.com", 443));
    assert!(!policy.allows("example.com", 443));
    assert!(!policy.allows("api.example.com", 80));
    assert!(!policy.allows("127.0.0.1", 443));
}

#[test]
fn upstream_address_safety_rejects_private_and_documentation_ranges() {
    assert!(!is_safe_upstream_address(IpAddr::V4(Ipv4Addr::new(
        10, 0, 0, 1
    ))));
    assert!(!is_safe_upstream_address(IpAddr::V4(Ipv4Addr::new(
        192, 168, 1, 1
    ))));
    assert!(!is_safe_upstream_address(IpAddr::V4(Ipv4Addr::new(
        203, 0, 113, 10
    ))));
    assert!(!is_safe_upstream_address(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    assert!(is_safe_upstream_address(IpAddr::V4(Ipv4Addr::new(
        93, 184, 216, 34
    ))));
}

#[test]
fn parse_connect_target_normalizes_ipv6_brackets() {
    assert_eq!(
        parse_connect_target("[2001:db8::1]:443").expect("target"),
        ("2001:db8::1".to_string(), 443)
    );
}

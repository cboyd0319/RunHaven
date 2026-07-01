use std::net::{IpAddr, Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
use std::thread;
use std::time::Duration;

use anyhow::{Result, bail};

use super::normalize_host;

#[derive(Debug)]
pub(super) enum SafeAddrError {
    Resolve,
    Unsafe,
}

pub(super) fn safe_addrinfos(host: &str, port: u16) -> Result<Vec<SocketAddr>, SafeAddrError> {
    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|_| SafeAddrError::Resolve)?
        .collect::<Vec<_>>();
    if addrs.is_empty() {
        return Err(SafeAddrError::Resolve);
    }
    if addrs
        .iter()
        .any(|addr| !is_safe_upstream_address(addr.ip()))
    {
        return Err(SafeAddrError::Unsafe);
    }
    Ok(addrs)
}

pub fn is_safe_upstream_address(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => {
            let [a, b, c, d] = address.octets();
            !(a == 0
                || a == 10
                || a == 127
                || (a == 100 && (64..=127).contains(&b))
                || (a == 169 && b == 254)
                || (a == 172 && (16..=31).contains(&b))
                || (a == 192 && b == 168)
                || (a == 192 && b == 0 && c == 0)
                || (a == 192 && b == 0 && c == 2)
                || (a == 198 && (b == 18 || b == 19))
                || (a == 198 && b == 51 && c == 100)
                || (a == 203 && b == 0 && c == 113)
                || a >= 224
                || [a, b, c, d] == [255, 255, 255, 255])
        }
        IpAddr::V6(address) => address.to_ipv4_mapped().map_or_else(
            || {
                let segments = address.segments();
                !(address.is_unspecified()
                    || address.is_loopback()
                    || (segments[0] & 0xff00) == 0xff00
                    || (segments[0] & 0xfe00) == 0xfc00
                    || (segments[0] & 0xffc0) == 0xfe80
                    || (segments[0] == 0x2001 && segments[1] == 0x0db8))
            },
            |mapped| is_safe_upstream_address(IpAddr::V4(mapped)),
        ),
    }
}

pub(super) fn connect_any(addrs: &[SocketAddr], timeout: Duration) -> std::io::Result<TcpStream> {
    let mut last_error = None;
    for addr in addrs {
        match TcpStream::connect_timeout(addr, timeout) {
            Ok(stream) => return Ok(stream),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| std::io::Error::other("no upstream addresses to connect")))
}

pub fn parse_connect_target(target: &str) -> Result<(String, u16)> {
    let (host, port_text) = if target.starts_with('[') {
        let Some(index) = target.rfind("]:") else {
            bail!("invalid CONNECT target: {target:?}");
        };
        (&target[..=index], &target[index + 2..])
    } else {
        let Some(index) = target.rfind(':') else {
            bail!("invalid CONNECT target: {target:?}");
        };
        (&target[..index], &target[index + 1..])
    };
    let port = port_text
        .parse::<u16>()
        .map_err(|_| anyhow::anyhow!("invalid CONNECT target port: {target:?}"))?;
    if port == 0 {
        bail!("invalid CONNECT target port: {target:?}");
    }
    Ok((normalize_host(host)?, port))
}

pub(super) fn relay(client: TcpStream, upstream: TcpStream, timeout: Duration) {
    let _ = client.set_nonblocking(false);
    let _ = upstream.set_nonblocking(false);
    let _ = client.set_read_timeout(Some(timeout));
    let _ = client.set_write_timeout(Some(timeout));
    let _ = upstream.set_read_timeout(Some(timeout));
    let _ = upstream.set_write_timeout(Some(timeout));

    let mut client_read = match client.try_clone() {
        Ok(stream) => stream,
        Err(_) => return,
    };
    let mut upstream_write = match upstream.try_clone() {
        Ok(stream) => stream,
        Err(_) => return,
    };
    let mut upstream_read = upstream;
    let mut client_write = client;

    let client_to_upstream = thread::spawn(move || {
        let _ = std::io::copy(&mut client_read, &mut upstream_write);
        let _ = upstream_write.shutdown(Shutdown::Write);
    });
    let upstream_to_client = thread::spawn(move || {
        let _ = std::io::copy(&mut upstream_read, &mut client_write);
        let _ = client_write.shutdown(Shutdown::Write);
    });

    let _ = client_to_upstream.join();
    let _ = upstream_to_client.join();
}

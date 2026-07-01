use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use anyhow::{Result, bail};

use super::upstream::BrokerUpstreamResponse;
use super::{PathRule, headers};

pub(super) fn read_headers(reader: &mut BufReader<TcpStream>) -> Result<Vec<(String, String)>> {
    let mut headers = Vec::new();
    let mut consumed = 0usize;
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        consumed += read;
        if consumed > crate::provider::egress::MAX_HEADER_BYTES {
            bail!("headers too large");
        }
        if read == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_string(), value.trim().to_string()));
        }
    }
    Ok(headers)
}

pub(super) fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

pub(super) fn send_error(stream: &mut TcpStream, status: u16, reason: &str) -> Result<()> {
    let body = format!("{status} {reason}\n");
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nConnection: close\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

pub(super) fn send_upstream_response(
    stream: &mut TcpStream,
    response: BrokerUpstreamResponse,
) -> Result<()> {
    let reason = if response.reason.is_empty() {
        "OK"
    } else {
        &response.reason
    };
    write!(stream, "HTTP/1.1 {} {}\r\n", response.status, reason)?;
    let has_content_length = response
        .headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("content-length"));
    for (name, value) in response.headers {
        if !headers::response_header_is_hop_by_hop(&name) {
            write!(stream, "{name}: {value}\r\n")?;
        }
    }
    if !has_content_length {
        write!(stream, "Content-Length: {}\r\n", response.body.len())?;
    }
    stream.write_all(b"\r\n")?;
    stream.write_all(&response.body)?;
    Ok(())
}

pub fn broker_upstream_path(target: &str, path_rule: &PathRule) -> Result<String> {
    let (path, query) = target
        .split_once('?')
        .map_or((target, None), |(path, query)| (path, Some(query)));
    if !path_rule.allows(path) {
        bail!("broker does not allow request path {path}");
    }
    Ok(query.map_or_else(|| path.to_string(), |query| format!("{path}?{query}")))
}

pub fn sanitize_broker_request_path(path: &str) -> String {
    let path = path
        .split(['?', '#'])
        .next()
        .filter(|path| !path.is_empty())
        .unwrap_or("-");
    path.to_string()
}

pub fn parse_content_length(value: Option<&str>) -> Result<Option<usize>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let length = value
        .parse::<isize>()
        .map_err(|_| anyhow::anyhow!("invalid Content-Length"))?;
    if length < 0 {
        bail!("invalid Content-Length");
    }
    Ok(Some(length as usize))
}

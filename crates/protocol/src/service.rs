//! Sharing service addresses (`configurable-sharing-service` design decision 2).
//!
//! People type a domain (`specdeck.example.com`) or paste a URL; the app stores and compares the
//! normalized URL, `wss://specdeck.example.com/signal`.

use std::fmt::Write;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error(
    "Enter a domain such as specdeck.example.com, or a URL such as wss://specdeck.example.com/signal."
)]
pub struct InvalidService;

/// The full WebSocket URL for a domain or URL as someone typed it. A missing scheme means
/// `wss://`, a missing path means `/signal`; schemes other than `ws` and `wss`, user names,
/// queries and fragments are refused.
pub fn normalize(input: &str) -> Result<String, InvalidService> {
    let input = input.trim();
    let (scheme, rest) = match input.split_once("://") {
        Some((scheme, rest)) => (scheme.to_ascii_lowercase(), rest),
        None => ("wss".to_owned(), input),
    };
    if scheme != "ws" && scheme != "wss" {
        return Err(InvalidService);
    }
    if rest.contains(['?', '#', '@']) || rest.chars().any(char::is_whitespace) {
        return Err(InvalidService);
    }
    let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let authority = authority.to_ascii_lowercase();
    let (host, port) = split_port(&authority)?;
    let host_ok = if let Some(inner) = host.strip_prefix('[') {
        inner.strip_suffix(']').is_some_and(|ip| ip.parse::<std::net::Ipv6Addr>().is_ok())
    } else {
        !host.is_empty()
            && host.split('.').all(|label| {
                !label.is_empty()
                    && !label.starts_with('-')
                    && !label.ends_with('-')
                    && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            })
    };
    if !host_ok {
        return Err(InvalidService);
    }
    let path = match path.trim_end_matches('/') {
        "" => "/signal",
        path => path,
    };
    let mut url = format!("{scheme}://{host}");
    if let Some(port) = port {
        let _ = write!(url, ":{port}");
    }
    url.push_str(path);
    Ok(url)
}

/// Splits `host:port`, keeping IPv6 brackets on the host.
fn split_port(authority: &str) -> Result<(&str, Option<u16>), InvalidService> {
    let colon = match authority.rfind(':') {
        Some(i) if !authority[i..].contains(']') => i,
        _ => return Ok((authority, None)),
    };
    let port = authority[colon + 1..].parse::<u16>().ok().filter(|port| *port != 0);
    Ok((&authority[..colon], Some(port.ok_or(InvalidService)?)))
}

/// What to show people for a normalized URL: the host, with the port if there is one.
pub fn display_name(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    rest.split('/').next().unwrap_or(rest)
}

/// Whether the service is reached over TLS.
pub fn is_encrypted(url: &str) -> bool {
    url.starts_with("wss://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domains_and_urls_become_full_urls() {
        for (input, url) in [
            ("specdeck.example.com", "wss://specdeck.example.com/signal"),
            ("  Specdeck.Example.com/ ", "wss://specdeck.example.com/signal"),
            ("specdeck.example.com:8443", "wss://specdeck.example.com:8443/signal"),
            ("wss://specdeck.example.com", "wss://specdeck.example.com/signal"),
            ("WSS://specdeck.example.com/signal", "wss://specdeck.example.com/signal"),
            ("wss://example.com/specdeck/signal/", "wss://example.com/specdeck/signal"),
            ("ws://127.0.0.1:18081/signal", "ws://127.0.0.1:18081/signal"),
            ("ws://localhost:8080", "ws://localhost:8080/signal"),
            ("ws://[::1]:8080", "ws://[::1]:8080/signal"),
        ] {
            assert_eq!(normalize(input).as_deref(), Ok(url), "{input}");
        }
    }

    #[test]
    fn refuses_what_is_not_a_service() {
        for input in [
            "",
            "   ",
            "https://specdeck.example.com",
            "ftp://specdeck.example.com",
            "wss://",
            "wss://user@specdeck.example.com",
            "specdeck.example.com/signal?x=1",
            "specdeck.example.com/#top",
            "specdeck example.com",
            "specdeck..example.com",
            "-specdeck.example.com",
            "specdeck.example.com:0",
            "specdeck.example.com:99999",
            "specdeck.example.com:port",
            "wss://[nonsense]:443",
        ] {
            assert_eq!(normalize(input), Err(InvalidService), "{input:?}");
        }
    }

    #[test]
    fn display_name_is_the_host_and_port() {
        assert_eq!(display_name("wss://specdeck.example.com/signal"), "specdeck.example.com");
        assert_eq!(display_name("ws://127.0.0.1:18081/signal"), "127.0.0.1:18081");
    }

    #[test]
    fn only_wss_is_encrypted() {
        assert!(is_encrypted("wss://specdeck.example.com/signal"));
        assert!(!is_encrypted("ws://127.0.0.1:18081/signal"));
    }
}

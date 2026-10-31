//! ICE server configuration for libdatachannel.

/// Puts TURN credentials into a TURN URL the way libdatachannel reads them:
/// `turn:<user>:<password>@host:port?transport=tcp`. libdatachannel URL-decodes both parts, so
/// they are percent-encoded; coturn usernames contain a `:` and passwords may contain `/`, `+`
/// and `=`.
pub fn turn_url_with_credentials(url: &str, username: &str, credential: &str) -> Option<String> {
    let (scheme, rest) = url.split_once(':')?;
    if scheme != "turn" && scheme != "turns" {
        return None;
    }
    Some(format!("{scheme}:{}:{}@{rest}", percent_encode(username), percent_encode(credential)))
}

fn percent_encode(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (b as char).to_string()
            }
            b => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_coturn_credentials() {
        assert_eq!(
            turn_url_with_credentials(
                "turn:turn.example.com:3478?transport=tcp",
                "1700000600:ab12",
                "JHgegmW+/B84iarkT5SFAOC1Iqw="
            )
            .unwrap(),
            "turn:1700000600%3Aab12:JHgegmW%2B%2FB84iarkT5SFAOC1Iqw%3D@turn.example.com:3478?transport=tcp"
        );
    }

    #[test]
    fn only_turn_urls() {
        assert!(turn_url_with_credentials("stun:stun.example.com:3478", "u", "p").is_none());
        assert!(turn_url_with_credentials("turns:t.example.com:443", "u", "p").is_some());
        assert!(turn_url_with_credentials("nonsense", "u", "p").is_none());
    }
}

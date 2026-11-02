//! Messages on the reliable `control` channel, encoded with postcard (design decision 7).
//!
//! Viewer messages come from untrusted peers and are decoded strictly: unknown variants, trailing
//! bytes and oversized fields are rejected. There is deliberately no message that carries
//! terminal input.

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{AgentId, AgentStatus};

/// Bumped on every incompatible protocol change. Host and viewer must be equal.
pub const PROTOCOL_VERSION: u32 = 2;

/// Longest username accepted, in characters.
pub const MAX_USERNAME_CHARS: usize = 64;

/// Longest control message accepted from a viewer, in bytes.
pub const MAX_VIEWER_MESSAGE_BYTES: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewerMessage {
    /// First message on a connection. When reconnecting, the `last_*` fields say where the viewer
    /// left off and `resume_token` lets it take its old place.
    Hello {
        protocol_version: u32,
        username: String,
        last_output_seq: Option<u64>,
        last_event_seq: Option<u64>,
        resume_token: Option<[u8; 32]>,
    },
    /// Send output again from this sequence number, after a gap or a failed verification.
    Resend { from_output_seq: u64 },
    /// The viewer is leaving the session.
    Leave,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostMessage {
    /// The viewer is admitted.
    Welcome {
        protocol_version: u32,
        viewer_id: u32,
        /// Verifies every stream frame.
        session_public_key: [u8; 32],
        cols: u16,
        rows: u16,
        /// Secret for reconnecting as the same viewer; also authenticates the reconnection.
        resume_token: [u8; 32],
        /// Signalling room for reconnecting. Open for the whole session, whatever the current
        /// invite code is, so a viewer that was offline while the host made a new code can
        /// still return.
        rejoin_room: String,
        agent: AgentId,
        /// False for agents whose events are not captured.
        timeline_supported: bool,
    },
    /// The viewer is not admitted; the host closes the connection after this.
    Refused {
        reason: RefusalReason,
    },
    /// Everyone in the session, sent on every change.
    Participants {
        participants: Vec<Participant>,
    },
    Status {
        status: AgentStatus,
    },
    /// The host closed the session.
    SessionEnded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefusalReason {
    ViewerOutdated,
    HostOutdated,
    SessionFull,
    Removed,
    SessionEnded,
}

impl RefusalReason {
    /// What the viewer's app shows.
    pub fn message(self) -> &'static str {
        match self {
            RefusalReason::ViewerOutdated => "Update your app to join this session.",
            RefusalReason::HostOutdated => "The host needs to update their app.",
            RefusalReason::SessionFull => "This session is full.",
            RefusalReason::Removed => "The host removed you from the session.",
            RefusalReason::SessionEnded => "The session has ended.",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Participant {
    pub id: u32,
    pub username: String,
    pub role: Role,
    /// The name to show: the username, with a suffix such as " (2)" when usernames collide.
    pub label: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Host,
    Viewer,
}

/// Accepts only equal versions; otherwise says which side has to update.
pub fn check_version(host: u32, viewer: u32) -> Result<(), RefusalReason> {
    match viewer.cmp(&host) {
        std::cmp::Ordering::Less => Err(RefusalReason::ViewerOutdated),
        std::cmp::Ordering::Greater => Err(RefusalReason::HostOutdated),
        std::cmp::Ordering::Equal => Ok(()),
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
    #[error("message too large")]
    TooLarge,
    #[error("malformed message")]
    Malformed,
    #[error("username must be 1 to {MAX_USERNAME_CHARS} characters")]
    InvalidUsername,
}

pub fn encode<T: Serialize>(message: &T) -> Vec<u8> {
    postcard::to_allocvec(message).expect("control messages always encode")
}

/// Decodes a message from a viewer: strict, bounded and validated.
pub fn decode_viewer(bytes: &[u8]) -> Result<ViewerMessage, DecodeError> {
    if bytes.len() > MAX_VIEWER_MESSAGE_BYTES {
        return Err(DecodeError::TooLarge);
    }
    let message: ViewerMessage = decode_exact(bytes)?;
    if let ViewerMessage::Hello { username, .. } = &message {
        let chars = username.trim().chars().count();
        if chars == 0 || chars > MAX_USERNAME_CHARS || username.chars().any(char::is_control) {
            return Err(DecodeError::InvalidUsername);
        }
    }
    Ok(message)
}

/// Decodes a message from the host.
pub fn decode_host(bytes: &[u8]) -> Result<HostMessage, DecodeError> {
    decode_exact(bytes)
}

fn decode_exact<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, DecodeError> {
    match postcard::take_from_bytes(bytes) {
        Ok((message, [])) => Ok(message),
        _ => Err(DecodeError::Malformed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hello(username: &str) -> ViewerMessage {
        ViewerMessage::Hello {
            protocol_version: PROTOCOL_VERSION,
            username: username.into(),
            last_output_seq: Some(41),
            last_event_seq: None,
            resume_token: Some([7; 32]),
        }
    }

    #[test]
    fn viewer_messages_round_trip() {
        for message in
            [hello("nick"), ViewerMessage::Resend { from_output_seq: 9 }, ViewerMessage::Leave]
        {
            assert_eq!(decode_viewer(&encode(&message)), Ok(message));
        }
    }

    #[test]
    fn host_messages_round_trip() {
        let messages = [
            HostMessage::Welcome {
                protocol_version: 1,
                viewer_id: 3,
                session_public_key: [1; 32],
                cols: 120,
                rows: 40,
                resume_token: [2; 32],
                rejoin_room: "0123abcd".into(),
                agent: AgentId::OpenCode,
                timeline_supported: false,
            },
            HostMessage::Refused { reason: RefusalReason::SessionFull },
            HostMessage::Participants {
                participants: vec![Participant {
                    id: 0,
                    username: "nick".into(),
                    role: Role::Host,
                    label: "nick".into(),
                }],
            },
            HostMessage::Status { status: AgentStatus::Working },
            HostMessage::SessionEnded,
        ];
        for message in messages {
            assert_eq!(decode_host(&encode(&message)), Ok(message));
        }
    }

    #[test]
    fn unknown_variant_is_rejected() {
        // Variant indexes 0-2 exist; 3 would be a message type this version does not know, such
        // as an attempt to send terminal input.
        assert_eq!(decode_viewer(&[3, 0]), Err(DecodeError::Malformed));
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let mut bytes = encode(&ViewerMessage::Leave);
        bytes.push(0);
        assert_eq!(decode_viewer(&bytes), Err(DecodeError::Malformed));
    }

    #[test]
    fn truncated_message_is_rejected() {
        let bytes = encode(&hello("nick"));
        assert_eq!(decode_viewer(&bytes[..bytes.len() - 1]), Err(DecodeError::Malformed));
    }

    #[test]
    fn oversized_message_is_rejected() {
        assert_eq!(
            decode_viewer(&vec![0; MAX_VIEWER_MESSAGE_BYTES + 1]),
            Err(DecodeError::TooLarge)
        );
    }

    #[test]
    fn usernames_are_validated() {
        assert_eq!(decode_viewer(&encode(&hello(""))), Err(DecodeError::InvalidUsername));
        assert_eq!(decode_viewer(&encode(&hello("   "))), Err(DecodeError::InvalidUsername));
        assert_eq!(decode_viewer(&encode(&hello("a\u{1b}[2J"))), Err(DecodeError::InvalidUsername));
        let long = "x".repeat(MAX_USERNAME_CHARS + 1);
        assert_eq!(decode_viewer(&encode(&hello(&long))), Err(DecodeError::InvalidUsername));
        assert!(decode_viewer(&encode(&hello("Ünïcødé 名前"))).is_ok());
    }

    #[test]
    fn version_check_names_the_side_that_must_update() {
        assert_eq!(check_version(1, 1), Ok(()));
        assert_eq!(check_version(2, 1), Err(RefusalReason::ViewerOutdated));
        assert_eq!(check_version(1, 2), Err(RefusalReason::HostOutdated));
        assert_eq!(
            RefusalReason::ViewerOutdated.message(),
            "Update your app to join this session."
        );
        assert_eq!(RefusalReason::HostOutdated.message(), "The host needs to update their app.");
    }
}

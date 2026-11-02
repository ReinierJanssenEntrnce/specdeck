//! Authenticates connection set-up messages with the invite code's auth key (design decision 8).
//!
//! An offer or answer contains the DTLS fingerprint the peer will present. Each one carries an
//! HMAC under the auth key, bound to its direction and the connection it belongs to, so the
//! signalling service cannot substitute its own fingerprint and read the traffic, or replay a
//! description into another connection.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::invite::hex;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DescriptionKind {
    /// From the viewer to the host.
    Offer,
    /// From the host to the viewer.
    Answer,
}

/// A session description with its authentication tag.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedDescription {
    pub kind: DescriptionKind,
    pub sdp: String,
    /// Hex HMAC-SHA256 over the kind, connection id and SDP.
    pub mac: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("The connection could not be verified.")]
pub struct SignalAuthError;

impl SignedDescription {
    pub fn sign(
        auth_key: &[u8; 32],
        connection_id: &str,
        kind: DescriptionKind,
        sdp: String,
    ) -> Self {
        let mac = hex(&tag(auth_key, connection_id, kind, &sdp).finalize().into_bytes());
        Self { kind, sdp, mac }
    }

    /// Returns the SDP if the tag is valid for this connection and the expected kind.
    pub fn verify(
        &self,
        auth_key: &[u8; 32],
        connection_id: &str,
        expected: DescriptionKind,
    ) -> Result<&str, SignalAuthError> {
        if self.kind != expected {
            return Err(SignalAuthError);
        }
        let mac = unhex(&self.mac).ok_or(SignalAuthError)?;
        tag(auth_key, connection_id, self.kind, &self.sdp)
            .verify_slice(&mac)
            .map_err(|_| SignalAuthError)?;
        Ok(&self.sdp)
    }
}

fn tag(auth_key: &[u8; 32], connection_id: &str, kind: DescriptionKind, sdp: &str) -> Hmac<Sha256> {
    let mut mac = Hmac::<Sha256>::new_from_slice(auth_key).expect("any key length");
    let kind: &[u8] = match kind {
        DescriptionKind::Offer => b"offer",
        DescriptionKind::Answer => b"answer",
    };
    // Length-prefix the variable fields so they cannot be shifted into each other.
    for field in [b"specdeck sdp v1".as_slice(), kind, connection_id.as_bytes(), sdp.as_bytes()] {
        mac.update(&(field.len() as u64).to_be_bytes());
        mac.update(field);
    }
    mac
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len()).step_by(2).map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invite::InviteCode;

    const SDP: &str = "v=0\r\na=fingerprint:sha-256 AA:BB:CC\r\n";

    fn key() -> [u8; 32] {
        InviteCode::parse("7K2M-9QXH-4TPA").unwrap().keys().auth_key
    }

    #[test]
    fn tag_is_stable_across_versions() {
        // Computed independently (Python hmac and hashlib), so apps of different versions keep
        // accepting each other's descriptions.
        let signed = SignedDescription::sign(&key(), "conn-1", DescriptionKind::Offer, SDP.into());
        assert_eq!(signed.mac, "64d21c1794849a7128e32e11b29e8f8900d560bf969cfc66a3468c9ca5acacc5");
    }

    #[test]
    fn valid_description_verifies() {
        let signed = SignedDescription::sign(&key(), "conn-1", DescriptionKind::Offer, SDP.into());
        assert_eq!(signed.verify(&key(), "conn-1", DescriptionKind::Offer), Ok(SDP));
    }

    #[test]
    fn substituted_fingerprint_is_rejected() {
        let mut signed =
            SignedDescription::sign(&key(), "conn-1", DescriptionKind::Offer, SDP.into());
        signed.sdp = signed.sdp.replace("AA:BB:CC", "EE:VI:LL");
        assert_eq!(signed.verify(&key(), "conn-1", DescriptionKind::Offer), Err(SignalAuthError));
    }

    #[test]
    fn wrong_code_is_rejected() {
        let other = InviteCode::parse("7K2M-9QXH-4TPB").unwrap().keys().auth_key;
        let signed = SignedDescription::sign(&other, "conn-1", DescriptionKind::Offer, SDP.into());
        assert_eq!(signed.verify(&key(), "conn-1", DescriptionKind::Offer), Err(SignalAuthError));
    }

    #[test]
    fn replay_into_another_connection_or_direction_is_rejected() {
        let signed = SignedDescription::sign(&key(), "conn-1", DescriptionKind::Offer, SDP.into());
        assert_eq!(signed.verify(&key(), "conn-2", DescriptionKind::Offer), Err(SignalAuthError));
        assert_eq!(signed.verify(&key(), "conn-1", DescriptionKind::Answer), Err(SignalAuthError));
        let mut relabelled = signed.clone();
        relabelled.kind = DescriptionKind::Answer;
        assert_eq!(
            relabelled.verify(&key(), "conn-1", DescriptionKind::Answer),
            Err(SignalAuthError)
        );
    }

    #[test]
    fn malformed_mac_is_rejected() {
        let mut signed = SignedDescription::sign(&key(), "c", DescriptionKind::Answer, SDP.into());
        let truncated = signed.mac[..10].to_owned();
        for mac in ["", "abc", "zz", &truncated] {
            signed.mac = mac.to_owned();
            assert_eq!(signed.verify(&key(), "c", DescriptionKind::Answer), Err(SignalAuthError));
        }
    }
}

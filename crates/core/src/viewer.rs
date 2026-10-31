//! A viewer's side of a shared session (design decisions 4, 7 and 8): join with an invite code,
//! verify everything the host sends, apply terminal records strictly in order, and reconnect
//! without losing anything while the session is open.
//!
//! The viewer has no way to send input: the only messages it can send are `Hello`, `Resend` and
//! `Leave`.

use std::time::Duration;

use datachannel::ConnectionState;
use specdeck_protocol::control::{
    self, HostMessage, PROTOCOL_VERSION, Participant, RefusalReason, ViewerMessage,
};
use specdeck_protocol::frames::{Frame, FrameVerifier, Record};
use specdeck_protocol::invite::InviteCode;
use specdeck_protocol::signal_auth::{DescriptionKind, SignedDescription};
use specdeck_protocol::signalling::{ClientMessage, ErrorCode, ServerMessage, SignalPayload};
use specdeck_protocol::timeline::TimelineEvent;
use specdeck_protocol::{AgentId, AgentStatus};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::time::Instant;

use crate::ice::turn_url_with_credentials;
use crate::peer::{Channel, Peer, PeerConfig, PeerEvent};
use crate::signalling_client;

/// How long joining may take before the viewer is told the network does not allow the
/// connection; also how long one reconnection attempt may take.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone, Debug)]
pub struct ViewerConfig {
    pub signalling_url: String,
    pub username: String,
    /// ICE settings; TURN servers from the signalling service are added to these.
    pub peer: PeerConfig,
    /// Usually [`CONNECT_TIMEOUT`].
    pub connect_timeout: Duration,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ViewerEvent {
    /// Admitted. Sent again after every reconnect, with `reconnected` set.
    Welcome {
        viewer_id: u32,
        agent: AgentId,
        timeline_supported: bool,
        cols: u16,
        rows: u16,
        reconnected: bool,
    },
    /// Terminal records to apply, in order and without gaps.
    Records(Vec<Record>),
    Timeline {
        seq: u64,
        event: TimelineEvent,
    },
    Participants(Vec<Participant>),
    Status(AgentStatus),
    /// The connection dropped; the viewer is reconnecting.
    Reconnecting,
    /// Not admitted; final.
    Refused(RefusalReason),
    /// The code does not match an open session; final.
    InvalidCode,
    /// Too many attempts from this network; final.
    RateLimited,
    /// The signalling service cannot be reached; final.
    ServiceUnavailable,
    /// No connection could be made, for example because the network blocks UDP; final.
    CannotConnect,
    /// The host closed the session; final.
    Ended,
}

impl ViewerEvent {
    /// Whether the session is over for this viewer after this event.
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            ViewerEvent::Refused(_)
                | ViewerEvent::InvalidCode
                | ViewerEvent::RateLimited
                | ViewerEvent::ServiceUnavailable
                | ViewerEvent::CannotConnect
                | ViewerEvent::Ended
        )
    }
}

/// A joined (or joining) session. Dropping it leaves the session.
pub struct ViewerSession {
    control: UnboundedSender<Control>,
}

enum Control {
    Leave,
    /// Drop the connection and wait this long before reconnecting.
    Interrupt(Duration),
}

impl ViewerSession {
    /// Joins the session with `code`. Must run inside a Tokio runtime.
    pub fn join(code: InviteCode, config: ViewerConfig) -> (Self, UnboundedReceiver<ViewerEvent>) {
        let (control, control_rx) = mpsc::unbounded_channel();
        let (events, event_rx) = mpsc::unbounded_channel();
        let viewer = Viewer {
            config,
            code,
            events,
            resume_token: None,
            rejoin_room: None,
            verifier: None,
            next_record: 0,
            next_event: 0,
            resend_requested: None,
            admitted_once: false,
            joined_at: Instant::now(),
        };
        tokio::spawn(viewer.run(control_rx));
        (Self { control }, event_rx)
    }

    pub fn leave(&self) {
        let _ = self.control.send(Control::Leave);
    }

    /// Drops the connection as a network outage of `outage` would, then reconnects. For tests
    /// and diagnostics.
    pub fn interrupt(&self, outage: Duration) {
        let _ = self.control.send(Control::Interrupt(outage));
    }
}

impl Drop for ViewerSession {
    fn drop(&mut self) {
        self.leave();
    }
}

struct Viewer {
    config: ViewerConfig,
    /// The invite code this viewer joined with.
    code: InviteCode,
    events: UnboundedSender<ViewerEvent>,
    resume_token: Option<[u8; 32]>,
    /// Where an admitted viewer reconnects; from `Welcome`.
    rejoin_room: Option<String>,
    verifier: Option<FrameVerifier>,
    next_record: u64,
    next_event: u64,
    /// Where a `Resend` was asked from, until records arrive from there.
    resend_requested: Option<u64>,
    admitted_once: bool,
    joined_at: Instant,
}

/// How one connection attempt ended.
enum Outcome {
    Final(ViewerEvent),
    /// Try again, for example after the connection dropped.
    Retry,
    /// Interrupted on purpose: reconnect after this long.
    Interrupted(Duration),
    Left,
}

impl Viewer {
    async fn run(mut self, mut control: UnboundedReceiver<Control>) {
        const FIRST_RETRY: Duration = Duration::from_millis(250);
        let mut retry = FIRST_RETRY;
        loop {
            let admitted_before = self.admitted_once;
            let wait = match self.attempt(&mut control).await {
                Outcome::Final(event) => {
                    let _ = self.events.send(event);
                    return;
                }
                Outcome::Left => return,
                Outcome::Interrupted(outage) => outage,
                // Joining keeps failing, for example because no relay address can be had.
                Outcome::Retry
                    if !self.admitted_once
                        && self.joined_at.elapsed() >= self.config.connect_timeout =>
                {
                    let _ = self.events.send(ViewerEvent::CannotConnect);
                    return;
                }
                Outcome::Retry => {
                    // Back off only while attempts keep failing.
                    if self.admitted_once && !admitted_before {
                        retry = FIRST_RETRY;
                    }
                    let wait = retry;
                    retry = (retry * 2).min(Duration::from_secs(5));
                    wait
                }
            };
            let _ = self.events.send(ViewerEvent::Reconnecting);
            let until = tokio::time::sleep(wait);
            tokio::pin!(until);
            loop {
                tokio::select! {
                    _ = &mut until => break,
                    command = control.recv() => match command {
                        Some(Control::Interrupt(_)) => {}
                        Some(Control::Leave) | None => return,
                    },
                }
            }
        }
    }

    /// One connection: signalling, peer connection, then the session until it ends or drops.
    async fn attempt(&mut self, control: &mut UnboundedReceiver<Control>) -> Outcome {
        let Ok((signalling, mut server)) =
            signalling_client::connect(&self.config.signalling_url).await
        else {
            // Before the first admission the service being down is final; later, keep trying.
            return if self.admitted_once {
                Outcome::Retry
            } else {
                Outcome::Final(ViewerEvent::ServiceUnavailable)
            };
        };
        // A new viewer joins with the code; an admitted one returns through the rejoin room,
        // authenticating with its resume token, which keeps working after the code changed.
        let (room, auth_key) = match (&self.rejoin_room, self.resume_token) {
            (Some(room), Some(token)) => (room.clone(), token),
            _ => {
                let keys = self.code.keys();
                (keys.room_id, keys.auth_key)
            }
        };
        signalling.send(ClientMessage::TurnCredentials);

        let mut ice_servers = self.config.peer.ice_servers.clone();
        let mut connection: Option<String> = None;
        let mut peer: Option<Peer> = None;
        let (peer_tx, mut peer_rx) = mpsc::unbounded_channel();
        let mut joined_sent = false;
        let mut connected = false;
        // Joining has one deadline across attempts; each reconnection attempt has its own.
        let deadline = if self.admitted_once {
            Instant::now() + self.config.connect_timeout
        } else {
            self.joined_at + self.config.connect_timeout
        };

        loop {
            tokio::select! {
                command = control.recv() => match command {
                    Some(Control::Interrupt(outage)) => return Outcome::Interrupted(outage),
                    Some(Control::Leave) | None => {
                        if let Some(peer) = &mut peer {
                            let _ = peer.send(Channel::Control, &control::encode(&ViewerMessage::Leave));
                        }
                        return Outcome::Left;
                    }
                },
                _ = tokio::time::sleep_until(deadline), if !connected => {
                    return if self.admitted_once { Outcome::Retry } else { Outcome::Final(ViewerEvent::CannotConnect) };
                }
                message = server.recv() => match message {
                    None if !connected => return Outcome::Retry,
                    // Once connected, the signalling service is no longer needed.
                    None => {}
                    Some(ServerMessage::TurnCredentials { urls, username, credential, .. }) => {
                        ice_servers.extend(urls.iter().filter_map(|url| turn_url_with_credentials(url, &username, &credential)));
                        if !joined_sent {
                            joined_sent = true;
                            signalling.send(ClientMessage::Join { room: room.clone() });
                        }
                    }
                    Some(ServerMessage::Error { code }) => match code {
                        ErrorCode::TurnUnavailable if !joined_sent => {
                            joined_sent = true;
                            signalling.send(ClientMessage::Join { room: room.clone() });
                        }
                        // After the host closed the session its room is gone.
                        ErrorCode::RoomNotFound if self.admitted_once => return Outcome::Final(ViewerEvent::Ended),
                        ErrorCode::RoomNotFound => return Outcome::Final(ViewerEvent::InvalidCode),
                        ErrorCode::RateLimited => return Outcome::Final(ViewerEvent::RateLimited),
                        _ => {}
                    },
                    Some(ServerMessage::Joined { connection: id }) => {
                        let config = PeerConfig { ice_servers: ice_servers.clone(), ..self.config.peer.clone() };
                        match Peer::offer(&config, peer_tx.clone()) {
                            Ok(new_peer) => peer = Some(new_peer),
                            Err(_) => return Outcome::Retry,
                        }
                        connection = Some(id);
                    }
                    Some(ServerMessage::Signal { payload, .. }) => {
                        let (Some(peer), Some(id)) = (&mut peer, &connection) else { continue };
                        match payload {
                            SignalPayload::Description(description) => {
                                // The answer must be made with this session's code.
                                let Ok(sdp) = description.verify(&auth_key, id, DescriptionKind::Answer) else {
                                    return Outcome::Retry;
                                };
                                if peer.set_remote_description(DescriptionKind::Answer, sdp).is_err() {
                                    return Outcome::Retry;
                                }
                            }
                            SignalPayload::Candidate { candidate, mid } => {
                                let _ = peer.add_remote_candidate(candidate, mid);
                            }
                        }
                    }
                    Some(ServerMessage::PeerLeft { .. }) if !connected => return Outcome::Retry,
                    Some(_) => {}
                },
                Some(event) = peer_rx.recv() => {
                    let (Some(current), Some(id)) = (&mut peer, &connection) else { continue };
                    match event {
                        PeerEvent::Description { kind, sdp } => {
                            let signed = SignedDescription::sign(&auth_key, id, kind, sdp);
                            signalling.send(ClientMessage::Signal { connection: id.clone(), payload: SignalPayload::Description(signed) });
                        }
                        PeerEvent::Candidate { candidate, mid } => {
                            signalling.send(ClientMessage::Signal { connection: id.clone(), payload: SignalPayload::Candidate { candidate, mid } });
                        }
                        // coturn can refuse an allocation (437) after a quick restart; a new peer
                        // connection gets a new socket.
                        PeerEvent::NoRelay => return Outcome::Retry,
                        PeerEvent::State(ConnectionState::Connecting) => current.flush_remote_candidates(),
                        PeerEvent::State(ConnectionState::Failed | ConnectionState::Disconnected | ConnectionState::Closed) => {
                            return Outcome::Retry;
                        }
                        PeerEvent::Open(Channel::Control) => {
                            connected = true;
                            let hello = ViewerMessage::Hello {
                                protocol_version: PROTOCOL_VERSION,
                                username: self.config.username.clone(),
                                last_output_seq: self.next_record.checked_sub(1),
                                last_event_seq: self.next_event.checked_sub(1),
                                resume_token: self.resume_token,
                            };
                            if current.send(Channel::Control, &control::encode(&hello)).is_err() {
                                return Outcome::Retry;
                            }
                        }
                        PeerEvent::Message(Channel::Control, bytes) => {
                            if let Some(outcome) = self.handle_control(&bytes) {
                                return outcome;
                            }
                        }
                        PeerEvent::Message(Channel::Stream, bytes) => {
                            if let Some(request) = self.handle_frame(&bytes) {
                                let _ = current.send(Channel::Control, &control::encode(&request));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_control(&mut self, bytes: &[u8]) -> Option<Outcome> {
        let Ok(message) = control::decode_host(bytes) else { return Some(Outcome::Retry) };
        match message {
            HostMessage::Welcome {
                viewer_id,
                session_public_key,
                rejoin_room,
                cols,
                rows,
                resume_token,
                agent,
                timeline_supported,
                ..
            } => {
                let Ok(verifier) = FrameVerifier::new(&session_public_key) else {
                    return Some(Outcome::Retry);
                };
                self.verifier = Some(verifier);
                self.resume_token = Some(resume_token);
                self.rejoin_room = Some(rejoin_room);
                let reconnected = self.admitted_once;
                self.admitted_once = true;
                let _ = self.events.send(ViewerEvent::Welcome {
                    viewer_id,
                    agent,
                    timeline_supported,
                    cols,
                    rows,
                    reconnected,
                });
            }
            HostMessage::Refused { reason } => {
                return Some(Outcome::Final(ViewerEvent::Refused(reason)));
            }
            HostMessage::Participants { participants } => {
                let _ = self.events.send(ViewerEvent::Participants(participants));
            }
            HostMessage::Status { status } => {
                let _ = self.events.send(ViewerEvent::Status(status));
            }
            HostMessage::SessionEnded => return Some(Outcome::Final(ViewerEvent::Ended)),
        }
        None
    }

    /// Applies a stream frame; returns a `Resend` request when records are missing or a frame
    /// failed verification.
    fn handle_frame(&mut self, bytes: &[u8]) -> Option<ViewerMessage> {
        let verifier = self.verifier.as_ref()?;
        let records = match verifier.decode(bytes) {
            Ok(Frame::Record(record)) => vec![record],
            Ok(Frame::History(records)) => records,
            Ok(Frame::Event { seq, event }) => {
                // Events arrive in order on one connection; after a reconnect the host continues
                // from the last one this viewer had.
                if seq >= self.next_event {
                    self.next_event = seq + 1;
                    let _ = self.events.send(ViewerEvent::Timeline { seq, event });
                }
                return None;
            }
            Err(_) => return self.request_resend(),
        };
        let fresh: Vec<Record> =
            records.into_iter().filter(|r| r.seq() >= self.next_record).collect();
        match fresh.first().map(Record::seq) {
            None => None,
            Some(first) if first == self.next_record => {
                self.next_record = fresh.last().map_or(first, Record::seq) + 1;
                self.resend_requested = None;
                let _ = self.events.send(ViewerEvent::Records(fresh));
                None
            }
            // A gap: drop this and ask for everything from the missing record on.
            Some(_) => self.request_resend(),
        }
    }

    /// Asks for records from the next expected one, once until they arrive.
    fn request_resend(&mut self) -> Option<ViewerMessage> {
        if self.resend_requested == Some(self.next_record) {
            return None;
        }
        self.resend_requested = Some(self.next_record);
        Some(ViewerMessage::Resend { from_output_seq: self.next_record })
    }
}

#[cfg(test)]
mod tests {
    use specdeck_protocol::frames::SessionKey;

    use super::*;

    fn viewer(key: &SessionKey) -> (Viewer, UnboundedReceiver<ViewerEvent>) {
        let (events, rx) = mpsc::unbounded_channel();
        let viewer = Viewer {
            config: ViewerConfig {
                signalling_url: String::new(),
                username: "v".into(),
                peer: PeerConfig::default(),
                connect_timeout: CONNECT_TIMEOUT,
            },
            code: InviteCode::generate(),
            events,
            resume_token: None,
            rejoin_room: None,
            verifier: Some(FrameVerifier::new(&key.public_key()).unwrap()),
            next_record: 0,
            next_event: 0,
            resend_requested: None,
            admitted_once: true,
            joined_at: Instant::now(),
        };
        (viewer, rx)
    }

    fn output(seq: u64) -> Record {
        Record::Output { seq, bytes: vec![b'a' + seq as u8] }
    }

    #[test]
    fn applies_records_in_order_and_asks_once_for_a_gap() {
        let key = SessionKey::generate();
        let (mut viewer, mut events) = viewer(&key);
        assert_eq!(viewer.handle_frame(&key.encode(&Frame::Record(output(0)))), None);
        // Record 1 got lost: 2 is dropped and 1 asked for, once.
        assert_eq!(
            viewer.handle_frame(&key.encode(&Frame::Record(output(2)))),
            Some(ViewerMessage::Resend { from_output_seq: 1 })
        );
        assert_eq!(viewer.handle_frame(&key.encode(&Frame::Record(output(3)))), None);
        // The resend arrives as a batch, including a duplicate of 0.
        let batch = Frame::History(vec![output(0), output(1), output(2), output(3)]);
        assert_eq!(viewer.handle_frame(&key.encode(&batch)), None);

        let mut applied = Vec::new();
        while let Ok(ViewerEvent::Records(records)) = events.try_recv() {
            applied.extend(records.iter().map(Record::seq));
        }
        assert_eq!(applied, [0, 1, 2, 3]);
        assert_eq!(viewer.next_record, 4);
    }

    #[test]
    fn tampered_frame_is_dropped_and_asked_for_again() {
        let key = SessionKey::generate();
        let (mut viewer, mut events) = viewer(&key);
        let mut bytes = key.encode(&Frame::Record(output(0)));
        bytes[10] ^= 1;
        assert_eq!(viewer.handle_frame(&bytes), Some(ViewerMessage::Resend { from_output_seq: 0 }));
        assert!(events.try_recv().is_err());
    }

    #[test]
    fn frames_from_another_key_are_not_applied() {
        let key = SessionKey::generate();
        let (mut viewer, mut events) = viewer(&key);
        let other = SessionKey::generate();
        viewer.handle_frame(&other.encode(&Frame::Record(output(0))));
        assert!(events.try_recv().is_err());
    }

    #[test]
    fn final_events() {
        assert!(ViewerEvent::Ended.is_final());
        assert!(!ViewerEvent::Reconnecting.is_final());
    }
}

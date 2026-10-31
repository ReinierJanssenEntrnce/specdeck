//! One WebRTC connection between the host and a viewer, with the two data channels of the
//! protocol (`control` and `stream`), reported as async events (design decision 6).
//!
//! The viewer offers and opens both channels; the host answers. Remote ICE candidates wait until
//! local gathering is complete: checks started before the local relay address exists are sent
//! without it and can leave a relayed connection stuck.

use std::sync::{Arc, Mutex};

use datachannel::{
    ConnectionState, DataChannelHandler, DataChannelInfo, GatheringState, IceCandidate,
    PeerConnectionHandler, RtcConfig, RtcDataChannel, RtcPeerConnection, SdpType,
    SessionDescription, TransportPolicy,
};
use specdeck_protocol::signal_auth::DescriptionKind;
use tokio::sync::mpsc::UnboundedSender;

/// Queued bytes per channel below which `BufferedLow` is reported.
pub const BUFFERED_LOW: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Channel {
    Control,
    Stream,
}

impl Channel {
    fn label(self) -> &'static str {
        match self {
            Channel::Control => "control",
            Channel::Stream => "stream",
        }
    }

    fn from_label(label: &str) -> Option<Self> {
        match label {
            "control" => Some(Channel::Control),
            "stream" => Some(Channel::Stream),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum PeerEvent {
    /// A local description to send to the other side, as SDP text.
    Description {
        kind: DescriptionKind,
        sdp: String,
    },
    /// A local ICE candidate to send to the other side.
    Candidate {
        candidate: String,
        mid: String,
    },
    /// The connection got its relay address, if TURN is configured.
    RelayGathered,
    /// Candidate gathering finished without a relay address although TURN was configured.
    NoRelay,
    Open(Channel),
    Message(Channel, Vec<u8>),
    /// The channel's queue drained below [`BUFFERED_LOW`].
    BufferedLow(Channel),
    State(ConnectionState),
}

#[derive(Clone, Debug, Default)]
pub struct PeerConfig {
    /// STUN and TURN URLs, TURN with credentials (see [`crate::ice`]).
    pub ice_servers: Vec<String>,
    /// Use only relayed candidates (tests, and networks where direct paths must not be tried).
    pub relay_only: bool,
    /// Bind to this local address only (tests use 127.0.0.1).
    pub bind_address: Option<String>,
}

impl PeerConfig {
    fn rtc(&self) -> RtcConfig {
        let mut config = RtcConfig::new(&self.ice_servers);
        if self.relay_only {
            config = config.ice_transport_policy(TransportPolicy::Relay);
        }
        if let Some(address) = &self.bind_address {
            config = config.bind_address(address);
        }
        config
    }

    fn uses_turn(&self) -> bool {
        self.ice_servers.iter().any(|url| url.starts_with("turn:") || url.starts_with("turns:"))
    }
}

type Channels = Arc<Mutex<[Option<Box<RtcDataChannel<ChannelHandler>>>; 2]>>;

pub struct Peer {
    pc: Box<RtcPeerConnection<Handler>>,
    channels: Channels,
    gathered: Arc<Mutex<Gathering>>,
}

#[derive(Default)]
struct Gathering {
    complete: bool,
    pending: Vec<IceCandidate>,
}

impl Peer {
    /// The viewer's side: creates both channels, which makes the connection produce its offer.
    pub fn offer(
        config: &PeerConfig,
        events: UnboundedSender<PeerEvent>,
    ) -> datachannel::Result<Self> {
        let mut peer = Self::new(config, events.clone())?;
        for channel in [Channel::Control, Channel::Stream] {
            let handler = ChannelHandler { channel, events: events.clone() };
            let mut dc = peer.pc.create_data_channel(channel.label(), handler)?;
            dc.set_buffered_amount_low_threshold(BUFFERED_LOW)?;
            peer.channels.lock().unwrap()[channel as usize] = Some(dc);
        }
        Ok(peer)
    }

    /// The host's side: waits for the viewer's offer and channels.
    pub fn answer(
        config: &PeerConfig,
        events: UnboundedSender<PeerEvent>,
    ) -> datachannel::Result<Self> {
        Self::new(config, events)
    }

    fn new(config: &PeerConfig, events: UnboundedSender<PeerEvent>) -> datachannel::Result<Self> {
        let channels: Channels = Arc::default();
        let gathered: Arc<Mutex<Gathering>> = Arc::default();
        let handler = Handler {
            events,
            channels: channels.clone(),
            gathered: gathered.clone(),
            expects_relay: config.uses_turn(),
            saw_relay: false,
        };
        let pc = RtcPeerConnection::new(&config.rtc(), handler)?;
        Ok(Self { pc, channels, gathered })
    }

    /// Applies the other side's offer or answer.
    pub fn set_remote_description(
        &mut self,
        kind: DescriptionKind,
        sdp: &str,
    ) -> Result<(), String> {
        let sdp = datachannel::sdp::parse_sdp(sdp, false).map_err(|e| e.to_string())?;
        let sdp_type = match kind {
            DescriptionKind::Offer => SdpType::Offer,
            DescriptionKind::Answer => SdpType::Answer,
        };
        self.pc
            .set_remote_description(&SessionDescription { sdp, sdp_type })
            .map_err(|e| e.to_string())
    }

    /// Adds the other side's candidate, or keeps it until local gathering is complete.
    pub fn add_remote_candidate(&mut self, candidate: String, mid: String) -> Result<(), String> {
        let candidate = IceCandidate { candidate, mid };
        {
            let mut gathered = self.gathered.lock().unwrap();
            if !gathered.complete {
                gathered.pending.push(candidate);
                return Ok(());
            }
        }
        self.pc.add_remote_candidate(&candidate).map_err(|e| e.to_string())
    }

    /// Call when [`PeerEvent::RelayGathered`] or the gathering-complete moment has passed: adds
    /// the candidates that were kept back. Safe to call more than once.
    pub fn flush_remote_candidates(&mut self) {
        let pending = {
            let mut gathered = self.gathered.lock().unwrap();
            if !gathered.complete {
                return;
            }
            std::mem::take(&mut gathered.pending)
        };
        for candidate in pending {
            let _ = self.pc.add_remote_candidate(&candidate);
        }
    }

    pub fn send(&mut self, channel: Channel, bytes: &[u8]) -> Result<(), String> {
        let mut channels = self.channels.lock().unwrap();
        let dc = channels[channel as usize].as_mut().ok_or("channel not open")?;
        dc.send(bytes).map_err(|e| e.to_string())
    }

    /// Bytes queued on the channel and not yet sent; `usize::MAX` if it is not open.
    pub fn buffered_amount(&self, channel: Channel) -> usize {
        let channels = self.channels.lock().unwrap();
        channels[channel as usize].as_ref().map_or(usize::MAX, |dc| dc.buffered_amount())
    }

    pub fn selected_pair(&self) -> Option<(String, String)> {
        self.pc.selected_candidate_pair().map(|pair| (pair.local, pair.remote))
    }
}

struct Handler {
    events: UnboundedSender<PeerEvent>,
    channels: Channels,
    gathered: Arc<Mutex<Gathering>>,
    expects_relay: bool,
    saw_relay: bool,
}

impl PeerConnectionHandler for Handler {
    type DCH = ChannelHandler;

    fn data_channel_handler(&mut self, info: DataChannelInfo) -> ChannelHandler {
        // Unknown labels get the stream channel's handler; their messages are ignored below.
        let channel = Channel::from_label(&info.label).unwrap_or(Channel::Stream);
        ChannelHandler { channel, events: self.events.clone() }
    }

    fn on_description(&mut self, description: SessionDescription) {
        let kind = match description.sdp_type {
            SdpType::Offer => DescriptionKind::Offer,
            _ => DescriptionKind::Answer,
        };
        let _ = self.events.send(PeerEvent::Description { kind, sdp: description.sdp.to_string() });
    }

    fn on_candidate(&mut self, candidate: IceCandidate) {
        if candidate.candidate.contains(" typ relay") && !self.saw_relay {
            self.saw_relay = true;
            let _ = self.events.send(PeerEvent::RelayGathered);
        }
        let _ = self
            .events
            .send(PeerEvent::Candidate { candidate: candidate.candidate, mid: candidate.mid });
    }

    fn on_gathering_state_change(&mut self, state: GatheringState) {
        if state != GatheringState::Complete {
            return;
        }
        self.gathered.lock().unwrap().complete = true;
        if self.expects_relay && !self.saw_relay {
            let _ = self.events.send(PeerEvent::NoRelay);
        }
        // Wake the owner so it flushes the remote candidates it kept back.
        let _ = self.events.send(PeerEvent::State(ConnectionState::Connecting));
    }

    fn on_connection_state_change(&mut self, state: ConnectionState) {
        let _ = self.events.send(PeerEvent::State(state));
    }

    fn on_data_channel(&mut self, mut channel: Box<RtcDataChannel<ChannelHandler>>) {
        let Some(kind) = Channel::from_label(&channel.label()) else { return };
        let _ = channel.set_buffered_amount_low_threshold(BUFFERED_LOW);
        self.channels.lock().unwrap()[kind as usize] = Some(channel);
        // An incoming channel is already open.
        let _ = self.events.send(PeerEvent::Open(kind));
    }
}

pub struct ChannelHandler {
    channel: Channel,
    events: UnboundedSender<PeerEvent>,
}

impl DataChannelHandler for ChannelHandler {
    fn on_open(&mut self) {
        let _ = self.events.send(PeerEvent::Open(self.channel));
    }

    fn on_message(&mut self, msg: &[u8]) {
        let _ = self.events.send(PeerEvent::Message(self.channel, msg.to_vec()));
    }

    fn on_buffered_amount_low(&mut self) {
        let _ = self.events.send(PeerEvent::BufferedLow(self.channel));
    }

    fn on_closed(&mut self) {
        let _ = self.events.send(PeerEvent::State(ConnectionState::Closed));
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

    use super::*;

    fn loopback() -> PeerConfig {
        PeerConfig { bind_address: Some("127.0.0.1".into()), ..PeerConfig::default() }
    }

    /// Delivers one side's events to the other until both channels are open on both sides.
    async fn connect(
        viewer: &mut Peer,
        viewer_events: &mut UnboundedReceiver<PeerEvent>,
        host: &mut Peer,
        host_events: &mut UnboundedReceiver<PeerEvent>,
    ) {
        let mut open = 0;
        while open < 4 {
            tokio::select! {
                Some(event) = viewer_events.recv() => match event {
                    PeerEvent::Description { kind, sdp } => host.set_remote_description(kind, &sdp).unwrap(),
                    PeerEvent::Candidate { candidate, mid } => host.add_remote_candidate(candidate, mid).unwrap(),
                    PeerEvent::Open(_) => open += 1,
                    PeerEvent::State(_) => viewer.flush_remote_candidates(),
                    _ => {}
                },
                Some(event) = host_events.recv() => match event {
                    PeerEvent::Description { kind, sdp } => viewer.set_remote_description(kind, &sdp).unwrap(),
                    PeerEvent::Candidate { candidate, mid } => viewer.add_remote_candidate(candidate, mid).unwrap(),
                    PeerEvent::Open(_) => open += 1,
                    PeerEvent::State(_) => host.flush_remote_candidates(),
                    _ => {}
                },
            }
        }
    }

    #[tokio::test]
    async fn both_channels_carry_messages_both_ways() {
        let (viewer_tx, mut viewer_events) = unbounded_channel();
        let (host_tx, mut host_events) = unbounded_channel();
        let mut viewer = Peer::offer(&loopback(), viewer_tx).unwrap();
        let mut host = Peer::answer(&loopback(), host_tx).unwrap();
        tokio::time::timeout(
            Duration::from_secs(10),
            connect(&mut viewer, &mut viewer_events, &mut host, &mut host_events),
        )
        .await
        .expect("channels did not open");

        viewer.send(Channel::Control, b"hello").unwrap();
        host.send(Channel::Stream, b"frame").unwrap();
        let mut got = Vec::new();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        while got.len() < 2 {
            let event = tokio::select! {
                Some(event) = host_events.recv() => event,
                Some(event) = viewer_events.recv() => event,
                _ = tokio::time::sleep_until(deadline) => panic!("messages did not arrive: {got:?}"),
            };
            if let PeerEvent::Message(channel, bytes) = event {
                got.push((channel, bytes));
            }
        }
        assert!(got.contains(&(Channel::Control, b"hello".to_vec())));
        assert!(got.contains(&(Channel::Stream, b"frame".to_vec())));
        assert!(host.buffered_amount(Channel::Stream) < BUFFERED_LOW);
    }
}

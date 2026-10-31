//! Checks that the vendored libdatachannel builds and two local peers can exchange a message.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use datachannel::{
    DataChannelHandler, DataChannelInfo, IceCandidate, PeerConnectionHandler, RtcConfig,
    RtcDataChannel, RtcPeerConnection, SessionDescription,
};

// The OpenSSL that libdatachannel vendors uses these Windows libraries, but its build does not
// tell the linker; without them a program that links little else fails with unresolved symbols.
#[cfg(windows)]
#[link(name = "advapi32")]
#[link(name = "bcrypt")]
#[link(name = "crypt32")]
#[link(name = "user32")]
unsafe extern "C" {}

enum Signal {
    Description(Box<SessionDescription>),
    Candidate(IceCandidate),
}

/// Forwards received messages and the open event to the test.
#[derive(Clone)]
struct Channel {
    opened: Sender<()>,
    received: Sender<Vec<u8>>,
}

impl DataChannelHandler for Channel {
    fn on_open(&mut self) {
        self.opened.send(()).ok();
    }

    fn on_message(&mut self, msg: &[u8]) {
        self.received.send(msg.to_vec()).ok();
    }
}

/// Sends its signalling to the other peer and answers every incoming channel with "pong".
struct Peer {
    signal: Sender<Signal>,
    channel: Channel,
    incoming: Option<Box<RtcDataChannel<Channel>>>,
}

impl PeerConnectionHandler for Peer {
    type DCH = Channel;

    fn data_channel_handler(&mut self, _info: DataChannelInfo) -> Channel {
        self.channel.clone()
    }

    fn on_description(&mut self, description: SessionDescription) {
        self.signal.send(Signal::Description(Box::new(description))).ok();
    }

    fn on_candidate(&mut self, candidate: IceCandidate) {
        self.signal.send(Signal::Candidate(candidate)).ok();
    }

    fn on_data_channel(&mut self, mut channel: Box<RtcDataChannel<Channel>>) {
        channel.send(b"pong").ok();
        self.incoming = Some(channel);
    }
}

fn channel() -> (Channel, Receiver<()>, Receiver<Vec<u8>>) {
    let (opened, on_open) = mpsc::channel();
    let (received, on_message) = mpsc::channel();
    (Channel { opened, received }, on_open, on_message)
}

/// Applies the other peer's signalling until the sender side closes.
fn relay(
    mut peer: Box<RtcPeerConnection<Peer>>,
    signals: Receiver<Signal>,
) -> Box<RtcPeerConnection<Peer>> {
    while let Ok(signal) = signals.recv_timeout(Duration::from_secs(2)) {
        match signal {
            Signal::Description(description) => peer.set_remote_description(&description).unwrap(),
            Signal::Candidate(candidate) => peer.add_remote_candidate(&candidate).unwrap(),
        }
    }
    peer
}

#[test]
fn two_local_peers_exchange_a_message() {
    // Loopback only: the test must not depend on the machine's network interfaces.
    let config = RtcConfig::new::<&str>(&[]).bind_address(&"127.0.0.1");
    let (to_answerer, answerer_signals) = mpsc::channel();
    let (to_offerer, offerer_signals) = mpsc::channel();

    let (offerer_channel, offerer_open, offerer_messages) = channel();
    let (answerer_channel, _answerer_open, answerer_messages) = channel();

    let mut offerer = RtcPeerConnection::new(
        &config,
        Peer { signal: to_answerer, channel: offerer_channel.clone(), incoming: None },
    )
    .unwrap();
    let answerer = RtcPeerConnection::new(
        &config,
        Peer { signal: to_offerer, channel: answerer_channel, incoming: None },
    )
    .unwrap();

    let answerer = thread::spawn(move || relay(answerer, answerer_signals));
    let mut ping = offerer.create_data_channel("ping", offerer_channel).unwrap();
    let offerer = thread::spawn(move || relay(offerer, offerer_signals));

    offerer_open.recv_timeout(Duration::from_secs(10)).expect("channel did not open");
    ping.send(b"ping").unwrap();

    let at_answerer = answerer_messages.recv_timeout(Duration::from_secs(10)).unwrap();
    let at_offerer = offerer_messages.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(at_answerer, b"ping");
    assert_eq!(at_offerer, b"pong");

    drop(ping);
    drop(offerer.join().unwrap());
    drop(answerer.join().unwrap());
}

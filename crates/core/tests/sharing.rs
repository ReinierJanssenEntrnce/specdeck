//! A hosted session shared with viewers through an in-process signalling service, over loopback
//! WebRTC. The agent is a fake terminal-only agent.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::time::Duration;

use specdeck_core::environment::Environment;
use specdeck_core::peer::PeerConfig;
use specdeck_core::session::{HostConfig, HostSession};
use specdeck_core::sharing::{Sharing, SharingConfig, SharingEvent};
use specdeck_core::viewer::{CONNECT_TIMEOUT, ViewerConfig, ViewerEvent, ViewerSession};
use specdeck_protocol::AgentId;
use specdeck_protocol::control::{Participant, RefusalReason};
use specdeck_protocol::frames::Record;
use specdeck_protocol::invite::InviteCode;
use tokio::net::TcpListener;
use tokio::sync::mpsc::UnboundedReceiver;

const WAIT: Duration = Duration::from_secs(15);

fn loopback() -> PeerConfig {
    PeerConfig { bind_address: Some("127.0.0.1".into()), ..PeerConfig::default() }
}

async fn signalling() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/signal", listener.local_addr().unwrap());
    tokio::spawn(specdeck_signalling::serve(
        listener,
        Arc::new(specdeck_signalling::Service::default()),
    ));
    url
}

struct Hosted {
    session: HostSession,
    sharing: Sharing,
    events: UnboundedReceiver<SharingEvent>,
    _dirs: (tempfile::TempDir, tempfile::TempDir),
}

/// Hosts a session running a fake agent that echoes each line it reads, and writes 3 MB of
/// output for the line "flood".
async fn host(url: &str, username: &str) -> Hosted {
    host_with(url, username, ECHO_AGENT).await
}

const ECHO_AGENT: &str = r#"#!/bin/sh
echo ready
while read line; do
  if [ "$line" = flood ]; then head -c 3000000 /dev/zero | tr '\0' x; echo; echo flood-done
  else echo "got:$line"; fi
done
"#;

async fn host_with(url: &str, username: &str, script: &str) -> Hosted {
    let bin = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    let agent = bin.path().join("opencode");
    std::fs::write(&agent, script).unwrap();
    std::fs::set_permissions(&agent, std::fs::Permissions::from_mode(0o755)).unwrap();
    let env: Environment =
        [("PATH", format!("{}:/usr/bin:/bin", bin.path().display()))].into_iter().collect();
    let config = HostConfig {
        agent: AgentId::OpenCode,
        folder: bin.path().to_owned(),
        cols: 80,
        rows: 24,
        data_dir: data.path().to_owned(),
    };
    let (session, _session_events) = HostSession::start(config, &env).await.unwrap();
    let (sharing, events) = Sharing::start(
        session.view(),
        SharingConfig {
            signalling_url: url.into(),
            username: username.into(),
            peer: loopback(),
            viewer_limit: 20,
        },
    );
    Hosted { session, sharing, events, _dirs: (bin, data) }
}

async fn ready(events: &mut UnboundedReceiver<SharingEvent>) -> InviteCode {
    loop {
        match tokio::time::timeout(WAIT, events.recv()).await.expect("no code").unwrap() {
            SharingEvent::Ready { code } => return code,
            _ => continue,
        }
    }
}

fn join(
    url: &str,
    code: &InviteCode,
    username: &str,
) -> (ViewerSession, UnboundedReceiver<ViewerEvent>) {
    ViewerSession::join(
        code.clone(),
        ViewerConfig {
            signalling_url: url.into(),
            username: username.into(),
            peer: loopback(),
            connect_timeout: CONNECT_TIMEOUT,
        },
    )
}

/// What a viewer has seen so far.
#[derive(Default)]
struct Seen {
    text: String,
    records: Vec<u64>,
    participants: Vec<Participant>,
    welcome: Option<u32>,
}

/// Collects viewer events until `done` holds, or panics after the timeout.
async fn until(
    events: &mut UnboundedReceiver<ViewerEvent>,
    seen: &mut Seen,
    done: impl Fn(&Seen, &ViewerEvent) -> bool,
) -> ViewerEvent {
    tokio::time::timeout(WAIT, async {
        loop {
            let event = events.recv().await.expect("viewer stopped");
            match &event {
                ViewerEvent::Records(records) => {
                    for record in records {
                        seen.records.push(record.seq());
                        if let Record::Output { bytes, .. } = record {
                            seen.text.push_str(&String::from_utf8_lossy(bytes));
                        }
                    }
                }
                ViewerEvent::Participants(participants) => seen.participants = participants.clone(),
                ViewerEvent::Welcome { viewer_id, .. } => seen.welcome = Some(*viewer_id),
                _ => {}
            }
            if done(seen, &event) {
                return event;
            }
        }
    })
    .await
    .unwrap_or_else(|_| panic!("timed out; seen text {:?}", seen.text))
}

#[tokio::test(flavor = "multi_thread")]
async fn viewer_joins_late_and_follows_live_output() {
    let url = signalling().await;
    let mut hosted = host(&url, "nick").await;
    let code = ready(&mut hosted.events).await;
    hosted.session.input(b"before\r").unwrap();

    let (_viewer, mut events) = join(&url, &code, "anna");
    let mut seen = Seen::default();
    // History from the start of the session, before the viewer joined.
    until(&mut events, &mut seen, |seen, _| seen.text.contains("got:before")).await;
    assert!(seen.text.contains("ready"));
    // Live output after joining.
    hosted.session.input(b"after\r").unwrap();
    until(&mut events, &mut seen, |seen, _| seen.text.contains("got:after")).await;
    // Every record exactly once, in order.
    assert!(seen.records.iter().enumerate().all(|(i, seq)| *seq == i as u64), "{:?}", seen.records);

    let names: Vec<_> = seen.participants.iter().map(|p| p.label.as_str()).collect();
    assert_eq!(names, ["nick", "anna"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn duplicate_usernames_can_be_told_apart() {
    let url = signalling().await;
    let mut hosted = host(&url, "nick").await;
    let code = ready(&mut hosted.events).await;
    let (_a, mut a) = join(&url, &code, "anna");
    until(&mut a, &mut Seen::default(), |seen, _| seen.welcome.is_some()).await;
    let (_b, mut b) = join(&url, &code, "anna");
    let mut seen = Seen::default();
    until(&mut b, &mut seen, |seen, _| seen.participants.len() == 3).await;
    let labels: Vec<_> = seen.participants.iter().map(|p| p.label.as_str()).collect();
    assert_eq!(labels, ["nick", "anna", "anna (2)"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn wrong_code_is_refused() {
    let url = signalling().await;
    let mut hosted = host(&url, "nick").await;
    ready(&mut hosted.events).await;
    let (_viewer, mut events) = join(&url, &InviteCode::generate(), "anna");
    let event = until(&mut events, &mut Seen::default(), |_, e| e.is_final()).await;
    assert_eq!(event, ViewerEvent::InvalidCode);
}

#[tokio::test(flavor = "multi_thread")]
async fn new_code_keeps_viewers_and_retires_the_old_code() {
    let url = signalling().await;
    let mut hosted = host(&url, "nick").await;
    let old = ready(&mut hosted.events).await;
    let (_viewer, mut events) = join(&url, &old, "anna");
    let mut seen = Seen::default();
    until(&mut events, &mut seen, |seen, _| seen.welcome.is_some()).await;

    hosted.sharing.new_code();
    let new = ready(&mut hosted.events).await;
    assert_ne!(new, old);

    let (_late, mut late) = join(&url, &old, "late");
    assert_eq!(
        until(&mut late, &mut Seen::default(), |_, e| e.is_final()).await,
        ViewerEvent::InvalidCode
    );

    // The viewer who joined with the old code still receives the session.
    hosted.session.input(b"still here\r").unwrap();
    until(&mut events, &mut seen, |seen, _| seen.text.contains("got:still here")).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn removed_viewer_is_disconnected_and_cannot_rejoin_with_the_code() {
    let url = signalling().await;
    let mut hosted = host(&url, "nick").await;
    let code = ready(&mut hosted.events).await;
    let (_viewer, mut events) = join(&url, &code, "anna");
    let mut seen = Seen::default();
    until(&mut events, &mut seen, |seen, _| seen.welcome.is_some()).await;

    hosted.sharing.remove(seen.welcome.unwrap());
    let event = until(&mut events, &mut seen, |_, e| e.is_final()).await;
    assert_eq!(event, ViewerEvent::Refused(RefusalReason::Removed));
    let new = ready(&mut hosted.events).await;
    assert_ne!(new, code);

    let (_again, mut again) = join(&url, &code, "anna");
    assert_eq!(
        until(&mut again, &mut Seen::default(), |_, e| e.is_final()).await,
        ViewerEvent::InvalidCode
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn viewer_limit_refuses_new_viewers_but_keeps_joined_ones() {
    let url = signalling().await;
    let mut hosted = host(&url, "nick").await;
    let code = ready(&mut hosted.events).await;
    let (_a, mut a) = join(&url, &code, "anna");
    let mut seen_a = Seen::default();
    until(&mut a, &mut seen_a, |seen, _| seen.welcome.is_some()).await;

    hosted.sharing.set_viewer_limit(1);
    let (_b, mut b) = join(&url, &code, "bob");
    let event = until(&mut b, &mut Seen::default(), |_, e| e.is_final()).await;
    assert_eq!(event, ViewerEvent::Refused(RefusalReason::SessionFull));

    hosted.session.input(b"for anna\r").unwrap();
    until(&mut a, &mut seen_a, |seen, _| seen.text.contains("got:for anna")).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn sharing_becomes_available_when_the_service_comes_up() {
    // A port with nothing listening yet.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let url = format!("ws://{address}/signal");

    let mut hosted = host(&url, "nick").await;
    let first = tokio::time::timeout(WAIT, hosted.events.recv()).await.unwrap().unwrap();
    assert_eq!(first, SharingEvent::Unavailable);
    // The session runs locally meanwhile.
    hosted.session.input(b"local\r").unwrap();

    let (_viewer, mut events) = join(&url, &InviteCode::generate(), "anna");
    let event = until(&mut events, &mut Seen::default(), |_, e| e.is_final()).await;
    assert_eq!(event, ViewerEvent::ServiceUnavailable);

    let listener = TcpListener::bind(address).await.unwrap();
    tokio::spawn(specdeck_signalling::serve(
        listener,
        Arc::new(specdeck_signalling::Service::default()),
    ));
    let code = ready(&mut hosted.events).await;

    let (_viewer, mut events) = join(&url, &code, "anna");
    let mut seen = Seen::default();
    until(&mut events, &mut seen, |seen, _| seen.text.contains("got:local")).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn viewer_reconnects_after_an_outage_without_losing_output() {
    let url = signalling().await;
    let mut hosted = host(&url, "nick").await;
    let code = ready(&mut hosted.events).await;
    let (viewer, mut events) = join(&url, &code, "anna");
    let mut seen = Seen::default();
    until(&mut events, &mut seen, |seen, _| seen.text.contains("ready")).await;

    viewer.interrupt(Duration::from_secs(3));
    until(&mut events, &mut seen, |_, e| *e == ViewerEvent::Reconnecting).await;
    hosted.session.input(b"during the outage\r").unwrap();

    let event =
        until(&mut events, &mut seen, |_, e| matches!(e, ViewerEvent::Welcome { .. })).await;
    assert!(matches!(event, ViewerEvent::Welcome { reconnected: true, .. }));
    until(&mut events, &mut seen, |seen, _| seen.text.contains("got:during the outage")).await;
    // Nothing missing and nothing twice.
    assert!(seen.records.iter().enumerate().all(|(i, seq)| *seq == i as u64), "{:?}", seen.records);
    // Still one viewer, under the same name.
    let labels: Vec<_> = seen.participants.iter().map(|p| p.label.as_str()).collect();
    assert_eq!(labels, ["nick", "anna"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn viewer_returns_after_a_new_code_was_made_during_its_outage() {
    let url = signalling().await;
    let mut hosted = host(&url, "nick").await;
    let code = ready(&mut hosted.events).await;
    let (viewer, mut events) = join(&url, &code, "anna");
    let mut seen = Seen::default();
    until(&mut events, &mut seen, |seen, _| seen.text.contains("ready")).await;

    viewer.interrupt(Duration::from_secs(2));
    until(&mut events, &mut seen, |_, e| *e == ViewerEvent::Reconnecting).await;
    hosted.sharing.new_code();
    assert_ne!(ready(&mut hosted.events).await, code);
    hosted.session.input(b"new code\r").unwrap();

    until(&mut events, &mut seen, |seen, _| seen.text.contains("got:new code")).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn session_closed_during_an_outage_ends_for_the_viewer() {
    let url = signalling().await;
    let mut hosted = host(&url, "nick").await;
    let code = ready(&mut hosted.events).await;
    let (viewer, mut events) = join(&url, &code, "anna");
    let mut seen = Seen::default();
    until(&mut events, &mut seen, |seen, _| seen.welcome.is_some()).await;

    viewer.interrupt(Duration::from_secs(2));
    until(&mut events, &mut seen, |_, e| *e == ViewerEvent::Reconnecting).await;
    hosted.sharing.close();

    let event = until(&mut events, &mut seen, |_, e| e.is_final()).await;
    assert_eq!(event, ViewerEvent::Ended);
}

#[tokio::test(flavor = "multi_thread")]
async fn host_closing_the_session_ends_it_for_connected_viewers() {
    let url = signalling().await;
    let mut hosted = host(&url, "nick").await;
    let code = ready(&mut hosted.events).await;
    let (_viewer, mut events) = join(&url, &code, "anna");
    let mut seen = Seen::default();
    until(&mut events, &mut seen, |seen, _| seen.welcome.is_some()).await;
    hosted.sharing.close();
    assert_eq!(until(&mut events, &mut seen, |_, e| e.is_final()).await, ViewerEvent::Ended);
}

#[tokio::test(flavor = "multi_thread")]
async fn viewer_is_told_when_no_connection_can_be_made() {
    let url = signalling().await;
    let mut hosted = host(&url, "nick").await;
    let code = ready(&mut hosted.events).await;
    // Relayed connections only, through a relay that does not exist: as on a network that
    // blocks everything but the signalling service.
    let peer = PeerConfig {
        ice_servers: vec!["turn:nobody:secret@127.0.0.1:9?transport=udp".into()],
        relay_only: true,
        bind_address: Some("127.0.0.1".into()),
    };
    let config = ViewerConfig {
        signalling_url: url.clone(),
        username: "anna".into(),
        peer,
        connect_timeout: Duration::from_secs(3),
    };
    let started = std::time::Instant::now();
    let (_viewer, mut events) = ViewerSession::join(code, config);
    let event = until(&mut events, &mut Seen::default(), |_, e| e.is_final()).await;
    assert_eq!(event, ViewerEvent::CannotConnect);
    assert!(started.elapsed() < Duration::from_secs(8), "took {:?}", started.elapsed());
}

#[tokio::test(flavor = "multi_thread")]
async fn two_viewers_receive_a_burst_of_output_completely() {
    let url = signalling().await;
    let mut hosted = host(&url, "nick").await;
    let code = ready(&mut hosted.events).await;
    let (_a, mut a) = join(&url, &code, "anna");
    let (_b, mut b) = join(&url, &code, "bob");
    let (mut seen_a, mut seen_b) = (Seen::default(), Seen::default());
    until(&mut a, &mut seen_a, |seen, _| seen.text.contains("ready")).await;
    until(&mut b, &mut seen_b, |seen, _| seen.text.contains("ready")).await;

    let started = std::time::Instant::now();
    hosted.session.input(b"flood\r").unwrap();
    for (events, seen) in [(&mut a, &mut seen_a), (&mut b, &mut seen_b)] {
        until(events, seen, |seen, _| seen.text.contains("flood-done")).await;
        assert_eq!(seen.text.matches('x').count(), 3_000_000);
        assert!(seen.records.iter().enumerate().all(|(i, seq)| *seq == i as u64));
    }
    eprintln!("both viewers had the 3 MB burst after {:?}", started.elapsed());
}

/// Twenty viewers while the agent writes continuously: the host's terminal stays responsive and
/// every viewer gets the output within a second (session-streaming: Session capacity).
#[tokio::test(flavor = "multi_thread")]
async fn twenty_viewers_under_continuous_output() {
    const BUSY_AGENT: &str = r#"#!/bin/sh
( i=0; while true; do i=$((i+1)); echo "tick $i: the agent is writing continuously, about a hundred lines per second"; sleep 0.01; done ) &
while read line; do echo "got:$line"; done
"#;
    let url = signalling().await;
    let mut hosted = host_with(&url, "nick", BUSY_AGENT).await;
    let code = ready(&mut hosted.events).await;

    let mut viewers = Vec::new();
    for i in 0..20 {
        let (session, events) = join(&url, &code, &format!("viewer {i}"));
        viewers.push((session, events, Seen::default()));
    }
    for (_, events, seen) in &mut viewers {
        until(events, seen, |seen, _| seen.text.contains("tick")).await;
    }

    let log = hosted.session.log().clone();
    let log_has = |marker: &str| {
        log.read(0, usize::MAX).unwrap().iter().rev().take(200).any(|r| match r {
            Record::Output { bytes, .. } => String::from_utf8_lossy(bytes).contains(marker),
            Record::Resize { .. } => false,
        })
    };
    let (mut echo_worst, mut viewer_worst) = (Duration::ZERO, Duration::ZERO);
    for round in 0..5 {
        let marker = format!("got:ping-{round}");
        let typed = std::time::Instant::now();
        hosted.session.input(format!("ping-{round}\r").as_bytes()).unwrap();
        while !log_has(&marker) {
            assert!(typed.elapsed() < Duration::from_secs(2), "no echo for {marker}");
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        let in_log = std::time::Instant::now();
        echo_worst = echo_worst.max(in_log - typed);
        for (_, events, seen) in &mut viewers {
            until(events, seen, |seen, _| seen.text.contains(&marker)).await;
            viewer_worst = viewer_worst.max(in_log.elapsed());
        }
    }
    eprintln!("20 viewers: worst host echo {echo_worst:?}, worst viewer delay {viewer_worst:?}");
    assert!(echo_worst < Duration::from_millis(50), "host echo took {echo_worst:?}");
    assert!(viewer_worst < Duration::from_secs(1), "a viewer took {viewer_worst:?}");
}

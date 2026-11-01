//! WebSocket connection to the signalling service (design decision 9).

use futures_util::{SinkExt, StreamExt};
use specdeck_protocol::signalling::{ClientMessage, ServerMessage};
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;

/// How often a quiet connection is pinged. A host's connection carries nothing for most of a
/// session, and proxies in front of the service close quiet connections (Envoy after 5 minutes,
/// Cloudflare after 100 seconds).
const KEEPALIVE: Duration = Duration::from_secs(30);

/// Sends messages to the signalling service. Dropping it closes the connection.
#[derive(Clone)]
pub struct SignallingSender(mpsc::UnboundedSender<ClientMessage>);

impl SignallingSender {
    /// Returns false if the connection is gone.
    pub fn send(&self, message: ClientMessage) -> bool {
        self.0.send(message).is_ok()
    }
}

/// Connects to `url` (`ws://` or `wss://…/signal`). The receiver ends when the connection closes.
pub async fn connect(
    url: &str,
) -> Result<
    (SignallingSender, mpsc::UnboundedReceiver<ServerMessage>),
    tokio_tungstenite::tungstenite::Error,
> {
    let (socket, _) = tokio_tungstenite::connect_async(url).await?;
    let (mut sink, mut stream) = socket.split();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<ClientMessage>();
    let (in_tx, in_rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let mut keepalive = tokio::time::interval_at(Instant::now() + KEEPALIVE, KEEPALIVE);
        loop {
            let message = tokio::select! {
                message = out_rx.recv() => match message {
                    Some(message) => Message::Text(
                        serde_json::to_string(&message).expect("client messages always encode").into(),
                    ),
                    None => break,
                },
                _ = keepalive.tick() => Message::Ping(Default::default()),
            };
            if sink.send(message).await.is_err() {
                break;
            }
        }
        let _ = sink.close().await;
    });
    tokio::spawn(async move {
        while let Some(Ok(message)) = stream.next().await {
            let Message::Text(text) = message else { continue };
            // A message this version does not understand is skipped, not fatal.
            if let Ok(message) = serde_json::from_str(&text)
                && in_tx.send(message).is_err()
            {
                break;
            }
        }
    });
    Ok((SignallingSender(out_tx), in_rx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test(start_paused = true)]
    async fn pings_a_quiet_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/signal", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
            socket.next().await.unwrap().unwrap()
        });
        let (_sender, _receiver) = connect(&url).await.unwrap();
        let started = Instant::now();
        let message = server.await.unwrap();
        assert!(matches!(message, Message::Ping(_)), "got {message:?}");
        assert!(started.elapsed() >= KEEPALIVE - Duration::from_secs(1));
    }
}

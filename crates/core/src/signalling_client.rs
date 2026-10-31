//! WebSocket connection to the signalling service (design decision 9).

use futures_util::{SinkExt, StreamExt};
use specdeck_protocol::signalling::{ClientMessage, ServerMessage};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

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
        while let Some(message) = out_rx.recv().await {
            let text = serde_json::to_string(&message).expect("client messages always encode");
            if sink.send(Message::Text(text.into())).await.is_err() {
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

//! Which sharing service a session uses, and whether one can be reached
//! (`configurable-sharing-service` design decisions 1 and 3).

use std::time::Duration;

use serde::Serialize;

/// Signalling service used when `SPECDECK_SIGNALLING_URL` is not set at build or run time: the
/// local stack from `just local-up`, reached through `just local-dev`'s port-forward.
const LOCAL_SIGNALLING_URL: &str = "ws://127.0.0.1:18081/signal";

/// How long the check in Settings waits for the service.
const CHECK_TIMEOUT: Duration = Duration::from_secs(5);

/// Where the service comes from, apart from the user's setting.
#[derive(Clone, Debug)]
pub struct Services {
    /// Baked in at build time.
    pub built_in: String,
    /// `SPECDECK_SIGNALLING_URL` when the app started; wins over the setting.
    pub overridden: Option<String>,
}

impl Services {
    pub fn from_environment() -> Self {
        Self {
            built_in: option_env!("SPECDECK_SIGNALLING_URL")
                .unwrap_or(LOCAL_SIGNALLING_URL)
                .to_owned(),
            overridden: std::env::var("SPECDECK_SIGNALLING_URL")
                .ok()
                .filter(|url| !url.trim().is_empty()),
        }
    }

    /// The service a session started now uses: the environment, the setting, then the built-in one.
    pub fn effective(&self, setting: Option<&str>) -> String {
        self.overridden.as_deref().or(setting).unwrap_or(&self.built_in).to_owned()
    }
}

/// The result of the check in Settings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceCheck {
    pub reachable: bool,
    /// Why it could not be reached, in plain words.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
}

/// Opens a connection to the service the way sessions do, and closes it again.
pub async fn check(url: &str) -> ServiceCheck {
    let problem =
        match tokio::time::timeout(CHECK_TIMEOUT, specdeck_core::signalling_client::connect(url))
            .await
        {
            Ok(Ok(_connection)) => None,
            Ok(Err(error)) => Some(describe(&error)),
            Err(_) => Some("It did not answer within 5 seconds.".to_owned()),
        };
    ServiceCheck { reachable: problem.is_none(), problem }
}

fn describe(error: &tokio_tungstenite::tungstenite::Error) -> String {
    use tokio_tungstenite::tungstenite::Error;
    match error {
        Error::Http(response) => format!(
            "It answered, but not as a SpecDeck sharing service (HTTP {}).",
            response.status().as_u16()
        ),
        Error::Tls(_) => "Its certificate is not valid.".to_owned(),
        Error::Io(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {
            "Nothing answers at that address.".to_owned()
        }
        Error::Io(_) => "It cannot be reached from this network.".to_owned(),
        _ => "It cannot be reached.".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn services(overridden: Option<&str>) -> Services {
        Services {
            built_in: "wss://built-in/signal".into(),
            overridden: overridden.map(Into::into),
        }
    }

    #[test]
    fn environment_then_setting_then_built_in() {
        assert_eq!(services(None).effective(None), "wss://built-in/signal");
        assert_eq!(services(None).effective(Some("wss://own/signal")), "wss://own/signal");
        assert_eq!(
            services(Some("ws://env/signal")).effective(Some("wss://own/signal")),
            "ws://env/signal"
        );
        assert_eq!(services(Some("ws://env/signal")).effective(None), "ws://env/signal");
    }

    #[tokio::test]
    async fn a_running_service_is_reachable() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/signal", listener.local_addr().unwrap());
        tokio::spawn(specdeck_signalling::serve(
            listener,
            Arc::new(specdeck_signalling::Service::default()),
        ));
        assert_eq!(check(&url).await, ServiceCheck { reachable: true, problem: None });
    }

    #[tokio::test]
    async fn a_closed_port_is_not() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("ws://{}/signal", listener.local_addr().unwrap());
        drop(listener);
        let result = check(&url).await;
        assert!(!result.reachable);
        assert_eq!(result.problem.as_deref(), Some("Nothing answers at that address."));
    }

    #[tokio::test]
    async fn a_web_server_that_is_not_the_service_is_not() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/signal", listener.local_addr().unwrap());
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            let (mut socket, _) = listener.accept().await.unwrap();
            let _ = socket.write_all(b"HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\n\r\n").await;
        });
        let result = check(&url).await;
        assert!(!result.reachable);
        assert!(result.problem.unwrap().contains("HTTP 404"));
    }
}

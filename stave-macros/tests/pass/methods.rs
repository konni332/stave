#![allow(dead_code)]

use std::time::Duration;

use stave_macros::{builder, methods};

#[builder]
struct Server {
    #[stave(required)]
    host: String,
    #[stave(required)]
    port: u16,
    timeout: Duration,
    note: String,
}

#[derive(Debug, PartialEq)]
struct Config {
    host: String,
    port: u16,
    timeout: Option<Duration>,
    note: Option<String>,
}

#[methods]
impl Server {
    #[sets(host)]
    fn sets_host(self, value: impl Into<String>) -> String {
        value.into()
    }

    #[sets(note)]
    #[requires(host)]
    fn sets_note_with_host(mut self, extra: &str) -> String {
        format!("{}-{extra}", self.__stave_host.0)
    }

    #[requires(host, port)]
    fn finish(self) -> Config {
        Config {
            host: self.__stave_host.0,
            port: self.__stave_port.0,
            timeout: self.timeout,
            note: self.note,
        }
    }
}

fn main() {
    let server1 = Server::new().sets_port(8080).sets_host("localhost");

    let server2 = Server::new()
        .sets_host("localhost")
        .sets_port(8080)
        .sets_timeout(Duration::from_secs(5))
        .sets_note_with_host("extra");

    let host = server1.host();
    let port = server2.port();
    let timeout = server1.timeout();
    let note = server1.note();

    assert_eq!(host, "localhost");
    assert_eq!(port, &8080);
    assert_eq!(timeout, &None);
    assert_eq!(note, &None);

    let config1 = server1.finish();
    let config2 = server2.finish();
    assert_eq!(
        config1,
        Config {
            host: "localhost".to_string(),
            port: 8080,
            timeout: None,
            note: None
        }
    );

    assert_eq!(
        config2,
        Config {
            host: "localhost".to_string(),
            port: 8080,
            timeout: Some(Duration::from_secs(5)),
            note: Some("localhost-extra".to_string()),
        }
    )
}

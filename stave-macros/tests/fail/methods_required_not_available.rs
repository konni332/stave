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
    fn set_host(self, value: impl Into<String>) -> String {
        value.into()
    }

    #[sets(note)]
    #[requires(host)]
    fn set_note_with_host(mut self, extra: &str) -> String {
        format!("{}-{extra}", self.__stave_host.0)
    }

    #[requires(host, port)]
    fn finish(self) -> Config {
        Config {
            host: self.host().clone(),
            port: self.port().clone(),
            timeout: self.timeout,
            note: self.note,
        }
    }
}

fn main() {
    let _ = Server::new().set_timeout(Duration::from_secs(5)).finish();

    let _ = Server::new().set_host("localhost").finish();

    let _ = Server::new().set_port(8080).set_note_with_host();
}

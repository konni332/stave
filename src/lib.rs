pub use stave_macros::{builder, methods};

#[cfg(test)]
mod tests {
    use super::*;

    #[builder]
    pub struct LoginBuilder {
        #[stave(required)]
        credentials: String,
        #[stave(required)]
        token_kind: String,
        #[stave(optional)]
        ip_address: String,
        #[stave(optional)]
        user_agent: String,
    }

    #[methods]
    #[optional_fields(ip_address, user_agent)]
    impl LoginBuilder {
        #[sets(credentials)]
        pub fn with_password(self, username: String, password: String) -> String {
            format!("password:{}:{}", username, password)
        }

        #[sets(credentials)]
        pub fn with_api_key(self, key: String) -> String {
            format!("apikey:{}", key)
        }

        #[sets(token_kind)]
        pub fn opaque_token(self, expires_at: String) -> String {
            format!("opaque:{}", expires_at)
        }

        #[requires(credentials)]
        pub fn ip_address(mut self, ip: String) -> Self {
            self.ip_address = Some(ip);
            self
        }

        #[requires(credentials)]
        pub fn user_agent(mut self, ua: String) -> Self {
            self.user_agent = Some(ua);
            self
        }

        #[requires(credentials, token_kind)]
        pub fn send(self) -> String {
            format!(
                "sending: creds={} token={} ip={:?} ua={:?}",
                self.__stave_credentials.0,
                self.__stave_token_kind.0,
                self.ip_address,
                self.user_agent,
            )
        }
    }

    #[test]
    fn full_chain_password() {
        let result = LoginBuilder::new()
            .with_password("alice".into(), "secret".into())
            .opaque_token("2099-01-01".into())
            .ip_address("127.0.0.1".into())
            .send();
        assert_eq!(
            result,
            r#"sending: creds=password:alice:secret token=opaque:2099-01-01 ip=Some("127.0.0.1") ua=None"#
        );
    }

    #[test]
    fn full_chain_api_key() {
        let result = LoginBuilder::new()
            .with_api_key("my-key".into())
            .opaque_token("2099-01-01".into())
            .send();
        assert_eq!(
            result,
            r#"sending: creds=apikey:my-key token=opaque:2099-01-01 ip=None ua=None"#
        );
    }
}

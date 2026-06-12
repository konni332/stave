use stave::{builder, methods};

#[derive(Debug, PartialEq)]
pub enum Credentials {
    Password { username: String, password: String },
    ApiKey { full_key: String },
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Opaque { expires_at: String },
    Jwt { claims: String },
}

#[builder]
pub struct LoginBuilder {
    #[stave(required)]
    credentials: Credentials,
    #[stave(required)]
    token_kind: TokenType,
    #[stave(optional)]
    ip_address: String,
    #[stave(optional)]
    user_agent: String,
    #[stave(optional)]
    refresh_expiry: String,
}

#[methods]
#[optional_fields(ip_address, user_agent, refresh_expiry)]
impl LoginBuilder {
    #[sets(credentials)]
    pub fn with_password(self, username: String, password: String) -> Credentials {
        Credentials::Password { username, password }
    }

    #[sets(credentials)]
    pub fn with_api_key(self, full_key: String) -> Credentials {
        Credentials::ApiKey { full_key }
    }

    #[sets(token_kind)]
    pub fn opaque_token(self, expires_at: String) -> TokenType {
        TokenType::Opaque { expires_at }
    }

    #[sets(token_kind)]
    pub fn jwt_token(self, claims: String) -> TokenType {
        TokenType::Jwt { claims }
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

    #[requires(credentials)]
    pub fn with_refresh(mut self, expires_at: String) -> Self {
        self.refresh_expiry = Some(expires_at);
        self
    }

    #[requires(credentials, token_kind)]
    pub fn send(
        self,
    ) -> (
        Credentials,
        TokenType,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        (
            self.__stave_credentials.0,
            self.__stave_token_kind.0,
            self.ip_address,
            self.user_agent,
            self.refresh_expiry,
        )
    }

    // Plain method -- no #[sets]/#[requires], available on any state
    pub fn describe(&self) -> &'static str {
        "a LoginBuilder"
    }
}

#[test]
fn password_with_jwt_and_all_optionals() {
    let (creds, token, ip, ua, refresh) = LoginBuilder::new()
        .with_password("alice".into(), "secret".into())
        .ip_address("10.0.0.1".into())
        .user_agent("test-agent".into())
        .with_refresh("2099-12-31".into())
        .jwt_token("claim-data".into())
        .send();

    assert_eq!(
        creds,
        Credentials::Password {
            username: "alice".into(),
            password: "secret".into()
        }
    );
    assert_eq!(
        token,
        TokenType::Jwt {
            claims: "claim-data".into()
        }
    );
    assert_eq!(ip, Some("10.0.0.1".to_string()));
    assert_eq!(ua, Some("test-agent".to_string()));
    assert_eq!(refresh, Some("2099-12-31".to_string()));
}

#[test]
fn api_key_with_opaque_no_optionals() {
    let (creds, token, ip, ua, refresh) = LoginBuilder::new()
        .with_api_key("key-123".into())
        .opaque_token("2099-01-01".into())
        .send();

    assert_eq!(
        creds,
        Credentials::ApiKey {
            full_key: "key-123".into()
        }
    );
    assert_eq!(
        token,
        TokenType::Opaque {
            expires_at: "2099-01-01".into()
        }
    );
    assert_eq!(ip, None);
    assert_eq!(ua, None);
    assert_eq!(refresh, None);
}

#[test]
fn order_independence_token_first_then_credentials() {
    // Setting token_kind before credentials should work identically
    let (creds, token, ..) = LoginBuilder::new()
        .jwt_token("claims".into())
        .with_password("bob".into(), "pw".into())
        .send();

    assert_eq!(
        creds,
        Credentials::Password {
            username: "bob".into(),
            password: "pw".into()
        }
    );
    assert_eq!(
        token,
        TokenType::Jwt {
            claims: "claims".into()
        }
    );
}

#[test]
fn plain_method_available_in_any_state() {
    // describe() should be callable even on the fully-unset state
    assert_eq!(LoginBuilder::new().describe(), "a LoginBuilder");

    // ...and on a partially-set state
    assert_eq!(
        LoginBuilder::new()
            .with_password("a".into(), "b".into())
            .describe(),
        "a LoginBuilder"
    );
}

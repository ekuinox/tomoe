use anyhow::{bail, ensure, Result};
pub use oauth2::Scope;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::http_client,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, StandardTokenResponse,
    TokenUrl,
};
use reqwest::Url;
use std::{collections::HashMap, ops::Deref};

// twitter oauth2 authorize endpoint
const TWITTER_AUTHORIZE_URL: &str = "https://twitter.com/i/oauth2/authorize";

// twitter oauth2 token endpoint
const TWITTER_TOKEN_URL: &str = "https://api.twitter.com/2/oauth2/token";

/// Twitter OAuth2 Client
pub struct TwitterOAuth2Client {
    inner: BasicClient,
}

pub struct TwitterOAuth2Authorizer {
    authorize_url: Url,
    csrf_state: CsrfToken,
    pkce_verifier: PkceCodeVerifier,
}

impl TwitterOAuth2Authorizer {
    /// create authorizer
    fn new(client: &TwitterOAuth2Client, scopes: Vec<Scope>) -> TwitterOAuth2Authorizer {
        let (code_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorize_url, csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes)
            .set_pkce_challenge(code_challenge)
            .url();
        TwitterOAuth2Authorizer {
            authorize_url,
            csrf_state,
            pkce_verifier,
        }
    }

    /// get token by redirect url and consume self
    fn try_into_token_with_redirect_url(
        self,
        client: &TwitterOAuth2Client,
        redirect_url: &str,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        let redirect_url = Url::parse(redirect_url)?;
        let params = redirect_url.query_pairs().collect::<HashMap<_, _>>();
        let code = match params.get("code") {
            Some(code) => AuthorizationCode::new(code.to_string()),
            None => bail!("couldn't find pair which key is 'code'"),
        };
        let state = match params.get("state") {
            Some(state) => CsrfToken::new(state.to_string()),
            None => bail!("couldn't find pair which key is 'state'"),
        };
        ensure!(state.secret() == self.csrf_state.secret());
        let token = client
            .exchange_code(code)
            .set_pkce_verifier(self.pkce_verifier)
            .request(http_client)?;
        Ok(token)
    }

    pub fn authorize_url(&self) -> &str {
        self.authorize_url.as_str()
    }
}

impl Deref for TwitterOAuth2Client {
    type Target = BasicClient;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TwitterOAuth2Client {
    /// create client with callback url
    pub fn new_with_callback_url(
        client_id: String,
        client_secret: String,
        callback_url: String,
    ) -> Result<TwitterOAuth2Client> {
        let TwitterOAuth2Client { inner: client } = Self::new(client_id, client_secret)?;
        let redirect_url = RedirectUrl::new(callback_url)?;
        let client = client.set_redirect_uri(redirect_url);
        Ok(TwitterOAuth2Client { inner: client })
    }

    /// create client with callback url
    pub fn new(client_id: String, client_secret: String) -> Result<TwitterOAuth2Client> {
        let client_id = ClientId::new(client_id);
        let client_secret = ClientSecret::new(client_secret);
        let auth_url = AuthUrl::new(TWITTER_AUTHORIZE_URL.to_owned())?;
        let token_url = TokenUrl::new(TWITTER_TOKEN_URL.to_owned())?;
        let client = BasicClient::new(client_id, client_secret.into(), auth_url, token_url.into());
        Ok(TwitterOAuth2Client { inner: client })
    }

    /// create authorizer instance
    pub fn authorizer(&self, scopes: Vec<Scope>) -> TwitterOAuth2Authorizer {
        TwitterOAuth2Authorizer::new(self, scopes)
    }

    /// request token by redirect url
    pub fn get_token(
        &self,
        authorizer: TwitterOAuth2Authorizer,
        redirect_url: &str,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        authorizer.try_into_token_with_redirect_url(self, redirect_url)
    }

    /// request new token
    pub fn refresh_token(
        &self,
        refresh_token: String,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        let refresh_token = RefreshToken::new(refresh_token);
        let token = self
            .exchange_refresh_token(&refresh_token)
            .request(http_client)?;
        Ok(token)
    }
}

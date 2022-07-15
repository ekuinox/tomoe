use super::TwitterCredentials;
use crate::twitter::{Scope, TwitterOAuth2Client};
use anyhow::Result;
use clap::Parser;
use oauth2::TokenResponse;
use std::{
    fmt::Display,
    fs::OpenOptions,
    io::{self, Write},
    ops::Deref,
    path::PathBuf,
    str::FromStr,
};

/// init authorization get refresh token
#[derive(Parser, Debug)]
pub struct InitSubcommand {
    /// twitter oauth2 client id
    #[clap(short = 'i', long)]
    client_id: String,
    /// twitter oauth2 client secret
    #[clap(short = 's', long)]
    client_secret: String,
    /// redirect url after signin
    #[clap(short = 'u', long)]
    callback_url: String,
    /// twitter oauth2 scopes e.g. `tweet.read`
    #[clap(long, default_value_t)]
    scopes: Scopes,
    /// verbosity. e.g give `-vv` to display self
    #[clap(short, parse(from_occurrences))]
    verbose: usize,
    /// env key to export access_token
    #[clap(short = 'e', long)]
    export_to: Option<PathBuf>,
}

impl InitSubcommand {
    pub fn run(self) -> Result<()> {
        if self.verbose >= 2 {
            dbg!(&self);
        }
        let client =
            TwitterOAuth2Client::new(self.client_id, self.client_secret, self.callback_url)?;
        let authorizer = client.authorizer(self.scopes.into());
        println!("authorize_url: {}", authorizer.authorize_url());
        println!("enter redirected_url: ");
        let redirect_url = {
            let mut buffer = String::new();
            let _ = io::stdin().read_line(&mut buffer)?;
            buffer
        };
        let token = client.get_token(authorizer, &redirect_url)?;
        let credentials = TwitterCredentials::new(
            token.access_token().secret().to_string(),
            token.refresh_token().map(|t| t.secret().to_string()),
        );
        let exports = serde_json::to_string(&credentials)?;
        if self.verbose >= 1 {
            println!("{exports}");
        }
        if let Some(export_to) = &self.export_to {
            let mut export_to = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(export_to)?;
            let _ = export_to.write_all(exports.as_bytes())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Scopes(Vec<Scope>);

impl FromStr for Scopes {
    type Err = std::io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let scopes = s
            .split(',')
            .into_iter()
            .map(ToString::to_string)
            .map(Scope::new)
            .collect::<Vec<_>>();
        Ok(Self(scopes))
    }
}

impl Display for Scopes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = f.write_str(
            &self
                .iter()
                .map(|s| s.as_ref())
                .collect::<Vec<_>>()
                .join(","),
        )?;
        Ok(())
    }
}

impl Deref for Scopes {
    type Target = Vec<Scope>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Scopes {
    fn default() -> Self {
        const DEFAULT_SCOPES: &[&str; 10] = &[
            "offline.access", // to refresh token
            "tweet.read",
            "users.read",
            "bookmark.read",
            "follows.read",
            "block.read",
            "like.read",
            "mute.read",
            "follows.read",
            "follows.read",
        ];
        let scopes = DEFAULT_SCOPES
            .into_iter()
            .map(ToString::to_string)
            .map(Scope::new)
            .collect();
        Self(scopes)
    }
}

impl From<Scopes> for Vec<Scope> {
    fn from(scopes: Scopes) -> Self {
        scopes.0
    }
}

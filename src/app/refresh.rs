use super::TwitterCredentials;
use crate::twitter::TwitterOAuth2Client;
use anyhow::{bail, Result};
use clap::Parser;
use oauth2::TokenResponse;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Write},
    path::PathBuf,
};

/// refresh twitter access token by refresh token
#[derive(Parser, Debug)]
pub struct RefreshSubcommand {
    /// path to credentials
    path: PathBuf,

    /// twitter oauth2 client id
    #[clap(short = 'i', long)]
    client_id: String,

    /// twitter oauth2 client secret
    #[clap(short = 's', long)]
    client_secret: String,

    /// verbosity
    #[clap(short, parse(from_occurrences))]
    verbose: usize,
}

impl RefreshSubcommand {
    pub fn run(self) -> Result<()> {
        let credentials: TwitterCredentials = {
            let file = File::open(&self.path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader)?
        };
        let refresh_token = match credentials.refresh_token.clone() {
            Some(t) => t,
            None => bail!("refresh token is not found"),
        };
        if self.verbose >= 1 {
            dbg!(&refresh_token);
        }

        let client = TwitterOAuth2Client::new(self.client_id, self.client_secret)?;
        let mut f = OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(&self.path)?;
        let credentials = match client.refresh_token(refresh_token) {
            Ok(token) => TwitterCredentials::new(
                token.access_token().secret().to_owned(),
                token.refresh_token().map(|t| t.secret().to_owned()),
            ),
            Err(e) => {
                eprintln!("{e}");
                credentials
            }
        };
        let json = serde_json::to_string(&credentials)?;
        f.write_all(json.as_bytes())?;

        Ok(())
    }
}

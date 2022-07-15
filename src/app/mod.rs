mod init;

use anyhow::Result;
use clap::Parser;
use init::InitSubcommand;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
pub enum AppSubcommand {
    Init(InitSubcommand),
}

#[derive(Parser, Debug)]
pub struct App {
    #[clap(subcommand)]
    subcommand: AppSubcommand,
}

impl App {
    pub fn run(self) -> Result<()> {
        match self.subcommand {
            AppSubcommand::Init(init) => init.run(),
        }
    }
    pub fn parse_and_run() -> Result<()> {
        let app = App::try_parse()?;
        app.run()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TwitterCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

impl TwitterCredentials {
    pub fn new(access_token: String, refresh_token: Option<String>) -> TwitterCredentials {
        TwitterCredentials { access_token, refresh_token }
    }
}

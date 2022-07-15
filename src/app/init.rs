use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct InitSubcommand {
    client_id: String,
    client_secret: String,
}

impl InitSubcommand {
    pub fn run(&self) -> Result<()> {
        dbg!(&self);
        Ok(())
    }
}

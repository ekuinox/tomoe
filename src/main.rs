mod app;
mod twitter;

use anyhow::Result;
use app::App;

fn main() -> Result<()> {
    App::parse_and_run()
}

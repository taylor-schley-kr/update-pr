mod app;
mod cli;
mod progress;
mod utils;

use std::process::ExitCode;

use anyhow::Result;
use app::App;
use clap::Parser;
use cli::Cli;

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();
    let app = App::new(cli)?;

    std::process::exit(app.run()?)
}

mod app;
mod cli;
mod progress;
mod utils;

use std::process::ExitCode;

use app::App;
use clap::Parser;
use cli::Cli;
use color_eyre::eyre::Result;

fn main() -> Result<ExitCode> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let app = App::new(cli)?;

    std::process::exit(app.run()?)
}

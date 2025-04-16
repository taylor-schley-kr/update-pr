use clap::Parser;
use humantime::Duration;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Optional working directory
    pub working_dir: Option<String>,

    /// Delay between attempts.
    /// If this is not set, it will try only once.
    /// Examples: -d 10s -d 3m
    #[arg(short = 'd')]
    pub delay: Option<Duration>,
}

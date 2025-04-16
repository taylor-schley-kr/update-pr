mod fetch;
mod merge;
mod push;
mod repo;

use anyhow::Result;
use fetch::Fetch;
use git2::{Oid, Repository};
use humantime::Duration;
use indicatif::ProgressBar;
use merge::Merge;
use push::Push;

use crate::cli::Cli;

static REMOTE_NAME: &str = "origin";
static REMOTE_BRANCH: &str = "main";

pub struct App {
    repo: Repository,
    delay: Option<Duration>,
}

impl App {
    pub fn new(cli: Cli) -> Result<Self> {
        let working_dir = cli.working_dir.as_deref().unwrap_or(".");

        Ok(Self {
            repo: Repository::open(working_dir)?,
            delay: cli.delay,
        })
    }

    pub fn run(&self) -> Result<i32> {
        let mut exit_code = 0;

        loop {
            if let Err(e) = self.try_git_ops() {
                eprintln!("Error: {}", e);
                exit_code = e.raw_code();

                break;
            }

            if let Some(delay) = self.delay {
                wait(delay.as_secs());
            } else {
                break;
            }
        }

        Ok(exit_code)
    }

    pub fn try_git_ops(&self) -> Result<(), git2::Error> {
        self.pull_current_branch()?;
        if self.merge_main()? {
            self.push_current_branch()?;
        }

        Ok(())
    }

    fn merge_main(&self) -> Result<bool, git2::Error> {
        let mut remote = self.repo.find_remote(REMOTE_NAME)?;
        let fetch_commit = self.fetch(REMOTE_BRANCH, &mut remote)?;

        let head = self.repo.head()?;
        let current_ref = head.name().unwrap();

        let current_oid = self.get_current_oid()?;

        self.merge(current_ref, fetch_commit)?;

        Ok(self.get_current_oid()? != current_oid)
    }

    fn pull_current_branch(&self) -> Result<(), git2::Error> {
        let mut remote = self.repo.find_remote(REMOTE_NAME)?;

        let head = self.repo.head()?;
        let current_ref = head.name().unwrap();
        let fetch_commit = self.fetch(current_ref, &mut remote)?;

        self.merge(current_ref, fetch_commit)?;

        Ok(())
    }

    fn push_current_branch(&self) -> Result<(), git2::Error> {
        let mut remote = self.repo.find_remote(REMOTE_NAME)?;

        let head = self.repo.head()?;
        let current_ref = head.name().unwrap();

        self.push(&[current_ref], &mut remote)?;

        Ok(())
    }

    fn get_current_oid(&self) -> Result<Oid, git2::Error> {
        self.repo.refname_to_id("HEAD")
    }
}

fn wait(delay: u64) {
    let pb = ProgressBar::new(delay).with_message("Waiting...");

    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner} {elapsed} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let stop_at = std::time::Instant::now() + std::time::Duration::from_secs(delay);

    while std::time::Instant::now() < stop_at {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    pb.finish();
}

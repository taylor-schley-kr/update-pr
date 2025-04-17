use std::{ffi::OsStr, fmt::Display, path::Path, process::Command};

use color_eyre::eyre::Result;
use console::{style, Style};
use tempfile::TempDir;
use update_pr::{app::App, cli::Cli};

lazy_static::lazy_static! {
    /// Style for the console output.
    static ref STYLE: Style = Style::new().cyan().italic();
}

fn main() -> Result<()> {
    color_eyre::install()?;

    Runner::new()?.run()
}

struct Runner {
    temp_dir: TempDir,
    remote_dir: TempDir,
}

impl Runner {
    pub fn new() -> Result<Self> {
        let temp_dir = tempfile::Builder::new()
            .prefix("update-pr-simple_")
            .tempdir()?;

        let remote_dir = tempfile::Builder::new()
            .prefix("remote_")
            .tempdir_in(&temp_dir)?;

        Ok(Self {
            temp_dir,
            remote_dir,
        })
    }

    pub fn run(&self) -> Result<()> {
        self.setup_remote()?;

        let local_dir = self.clone_remote("local")?;

        self.create_commit_on_remote()?;

        self.setup_local(&local_dir)?;

        println!(
            "{}",
            STYLE.apply_to(format!(
                "Running update-pr in {}",
                style(local_dir.path().display()).bold()
            ))
        );

        App::new(Cli {
            working_dir: Some(local_dir.path().to_str().unwrap().to_string()),
            delay: None,
        })?
        .run()?;

        println!(
            "{}",
            STYLE.apply_to(format!(
                "Git log for {}",
                style(local_dir.path().display()).bold()
            ))
        );

        Command::new("git")
            .arg("log")
            .arg("--oneline")
            .current_dir(local_dir.path())
            .status()?;

        Ok(())
    }

    fn setup_remote(&self) -> Result<()> {
        println!("{}", STYLE.apply_to("Setting up remote..."));

        Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(self.remote_dir.path())
            .status()?;

        // Add an initial commit to the remote repo
        {
            let init_dir = self.clone_remote("init")?;

            create_commit(init_dir.path())?;

            Command::new("git")
                .arg("push")
                .current_dir(init_dir.path())
                .status()?;
        }

        Ok(())
    }

    fn clone_remote<S: Display>(&self, prefix: S) -> Result<TempDir> {
        clone_remote(prefix, self.remote_dir.path(), self.temp_dir.path())
    }

    fn create_commit_on_remote(&self) -> Result<()> {
        println!("{}", STYLE.apply_to("Creating commit on remote..."));

        let other_dir = self.clone_remote("other")?;

        create_commit(other_dir.path())?;

        Command::new("git")
            .arg("push")
            .current_dir(other_dir.path())
            .status()?;

        Ok(())
    }

    fn setup_local(&self, local_dir: &TempDir) -> Result<()> {
        println!(
            "{}",
            STYLE.apply_to(format!(
                "Creating branch {} in {}",
                style("pr").bold(),
                style(local_dir.path().display()).bold()
            ))
        );

        Command::new("git")
            .arg("checkout")
            .arg("-b")
            .arg("pr")
            .current_dir(local_dir.path())
            .status()?;

        create_commit(local_dir.path())?;

        Command::new("git")
            .arg("push")
            .arg("--set-upstream")
            .arg("origin")
            .arg("pr")
            .current_dir(local_dir.path())
            .status()?;

        Ok(())
    }
}

fn clone_remote<S, R, P>(prefix: S, remote_dir: R, temp_dir: P) -> Result<TempDir>
where
    S: Display,
    R: AsRef<OsStr>,
    P: AsRef<Path>,
{
    let dir = tempfile::Builder::new()
        .prefix(format!("{prefix}_").as_str())
        .tempdir_in(temp_dir)?;

    println!(
        "{}",
        STYLE.apply_to(format!(
            "Cloning remote to {}",
            style(dir.path().display()).bold()
        ))
    );

    Command::new("git")
        .arg("clone")
        .arg(remote_dir)
        .arg(dir.path())
        .status()?;

    Ok(dir)
}

fn create_commit<S: AsRef<Path>>(dir: S) -> Result<()> {
    println!(
        "{}",
        STYLE.apply_to(format!(
            "Creating commit in {}",
            style(dir.as_ref().display()).bold()
        ))
    );

    let file = tempfile::Builder::new()
        .prefix("file_")
        .rand_bytes(10)
        .keep(true)
        .tempfile_in(&dir)?;

    Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(&dir)
        .status()?;

    Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(format!(
            "add file {}",
            file.path().file_name().unwrap().to_str().unwrap()
        ))
        .current_dir(&dir)
        .status()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner() -> Result<()> {
        let runner = Runner::new()?;
        runner.run()
    }
}

use console::{style, Emoji};
use git2::{AnnotatedCommit, AutotagOption, FetchOptions, Remote, RemoteCallbacks};

use crate::{progress::fetch::FetchProgress, utils::git::ssh_creds};

use super::{repo::Repo, App};

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");

pub trait Fetch: Repo {
    fn fetch(&self, refs: &[&str], remote: &mut Remote) -> Result<AnnotatedCommit, git2::Error> {
        let pb = FetchProgress::new();

        let mut cb = RemoteCallbacks::new();
        cb.credentials(ssh_creds).transfer_progress(|stats| {
            pb.process(stats);

            true
        });

        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);
        // Always fetch all tags.
        // Perform a download and also update tips
        fo.download_tags(AutotagOption::All);

        pb.println(format!(
            "{}Fetching {} for repo from {}...",
            LOOKING_GLASS,
            style(refs.join(",")).italic().dim().blue(),
            style(remote.name().unwrap()).italic().bold().green()
        ))
        .unwrap();

        if let Err(e) = remote.fetch(refs, Some(&mut fo), None) {
            match e.class() {
                git2::ErrorClass::Net => println!("Error: {}", e),
                _ => return Err(e),
            }
        }

        let fetch_head = self.repo().find_reference("FETCH_HEAD")?;
        self.repo().reference_to_annotated_commit(&fetch_head)
    }
}

impl Fetch for App {}

#[cfg(test)]
mod tests {
    use std::{
        fs::{create_dir, File},
        io::Write,
        process::Command,
    };

    use anyhow::Result;
    use git2::Repository;
    use testdir::testdir;

    use super::*;

    pub struct TestApp {
        pub repo: Repository,
    }

    impl Repo for TestApp {
        fn repo(&self) -> &Repository {
            &self.repo
        }
    }

    impl Fetch for TestApp {}

    // Using git commands instead of libigit2 to create a test repo
    // This way we know we are creating similiar behaviors
    #[test]
    fn test_fetch() -> Result<()> {
        let base_dir = testdir!();

        //
        // setup remote
        //
        let remote_dir = base_dir.join("remote");
        create_dir(&remote_dir)?;

        Command::new("git")
            .current_dir(&remote_dir)
            .arg("init")
            .arg("--bare")
            .output()?;

        //
        // create a commit
        //
        {
            let commit_dir = base_dir.join("base-commit");
            create_dir(&commit_dir)?;

            Command::new("git")
                .current_dir(&commit_dir)
                .arg("init")
                .output()
                .unwrap();
            Command::new("git")
                .current_dir(&commit_dir)
                .arg("remote")
                .arg("add")
                .arg("origin")
                .arg(remote_dir.to_owned().into_os_string().to_str().unwrap())
                .output()?;

            let file_path = commit_dir.join("README.md");
            let mut file = File::create(&file_path)?;
            file.write_all(b"# Test\n")?;

            Command::new("git")
                .current_dir(&commit_dir)
                .arg("add")
                .arg("README.md")
                .output()?;
            Command::new("git")
                .current_dir(&commit_dir)
                .arg("commit")
                .arg("-m")
                .arg("Initial commit")
                .output()?;

            assert!(file_path.exists());

            Command::new("git")
                .current_dir(&commit_dir)
                .arg("push")
                .arg("origin")
                .arg("HEAD:main")
                .output()?;
        }

        //
        // setup local repo
        //
        let usage_dir = base_dir.join("usage");
        create_dir(&usage_dir)?;

        Command::new("git")
            .current_dir(&usage_dir)
            .arg("init")
            .output()?;

        assert!(usage_dir.join(".git").exists());

        let app = TestApp {
            repo: Repository::open(&usage_dir)?,
        };
        let mut remote = app
            .repo()
            .remote(
                "origin",
                remote_dir.to_owned().into_os_string().to_str().unwrap(),
            )
            .unwrap();

        assert_eq!(
            app.fetch(&["main"], &mut remote).unwrap().id(),
            Repository::open(&remote_dir)?
                .find_reference("refs/heads/main")?
                .peel_to_commit()?
                .id()
        );

        Ok(())
    }
}

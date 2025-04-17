use console::{style, Emoji};
use git2::{IntoCString, PushOptions, Remote, RemoteCallbacks};

use crate::{progress::push::PushProgress, utils::git::ssh_creds};

use super::App;

static TRUCK: Emoji<'_, '_> = Emoji("🚚  ", "");

pub trait Push {
    fn push<Str: AsRef<str> + IntoCString + Clone>(
        &self,
        refspecs: &[Str],
        remote: &mut Remote,
    ) -> Result<(), git2::Error> {
        let pb = PushProgress::new();

        let mut cb = RemoteCallbacks::new();
        cb.credentials(ssh_creds)
            .push_transfer_progress(|current, total, _bytes| {
                let pb = pb.pb();
                pb.set_length(total as u64);
                pb.set_position(current as u64);
            });

        let mut po = PushOptions::new();
        po.remote_callbacks(cb);

        pb.pb().println(format!(
            "{}Pushing {} for repo...",
            TRUCK,
            style(remote.name().unwrap()).italic().bold()
        ));

        remote.push(refspecs, Some(&mut po))?;

        Ok(())
    }
}

impl Push for App {}

#[cfg(test)]
mod tests {
    use std::{
        fs::{create_dir, File},
        io::Write,
        path::{Path, PathBuf},
        process::Command,
    };

    use color_eyre::eyre::Result;
    use git2::{AnnotatedCommit, Repository};
    use testdir::testdir;

    use super::*;

    pub struct TestApp {
        pub repo: Repository,
    }

    impl TestApp {
        pub fn get_fetch_commit(&self) -> Result<AnnotatedCommit<'_>> {
            let reference = self.repo.find_reference("refs/remotes/origin/main")?;

            let ret = self.repo.reference_to_annotated_commit(&reference)?;

            Ok(ret)
        }
    }

    impl Push for TestApp {}

    fn git_cmd(dir: &Path) -> Command {
        let mut cmd = Command::new("git");

        cmd.current_dir(dir);

        cmd
    }

    pub struct TestFixture {
        base_dir: PathBuf,
        remote_dir: PathBuf,
    }

    impl TestFixture {
        pub fn new() -> Result<Self> {
            let base_dir = testdir!();

            //
            // setup remote
            //
            let remote_dir = base_dir.join("remote");
            create_dir(&remote_dir)?;

            git_cmd(&remote_dir)
                .arg("init")
                .arg("--bare")
                .arg("--initial-branch=main")
                .status()?;

            Ok(Self {
                base_dir,
                remote_dir,
            })
        }

        pub fn setup(self) -> Result<Self> {
            let setup_dir = self.get_repo_dir("setup")?;

            let file_path = setup_dir.join("README.md");
            let mut file = File::create(&file_path)?;
            file.write_all(b"# Test\n")?;

            git_cmd(&setup_dir).arg("add").arg(&file_path).status()?;
            git_cmd(&setup_dir)
                .arg("commit")
                .arg("-m")
                .arg("Initial commit")
                .status()?;
            git_cmd(&setup_dir)
                .arg("push")
                .arg("origin")
                .arg("HEAD:main")
                .status()?;

            Ok(self)
        }

        pub fn get_repo_dir(&self, name: &str) -> Result<PathBuf> {
            //
            // setup repo
            //
            let repo_dir = self.base_dir.join(name);
            create_dir(&repo_dir)?;

            git_cmd(&repo_dir)
                .arg("clone")
                .arg(&self.remote_dir)
                .arg(".")
                .status()?;

            Ok(repo_dir)
        }
    }

    #[test]
    fn test_push() -> Result<()> {
        let fixture = TestFixture::new()?.setup()?;

        let usage_dir = fixture.get_repo_dir("usage")?;

        // Create a commit
        {
            let file_path = usage_dir.join("file.txt");
            let mut file = File::create(&file_path)?;
            file.write_all(b"Hello World")?;

            git_cmd(&usage_dir).arg("add").arg(&file_path).status()?;
            git_cmd(&usage_dir)
                .arg("commit")
                .arg("-m")
                .arg("add file.txt")
                .status()?;
        }

        let app = TestApp {
            repo: Repository::open(&usage_dir)?,
        };

        let local_commit_id = app.repo.head()?.peel_to_commit()?.id();

        assert_ne!(app.get_fetch_commit()?.id(), local_commit_id);

        let head = app.repo.head()?;
        let current_ref = head.name().unwrap();

        app.push(&[current_ref], &mut app.repo.find_remote("origin")?)?;

        assert_eq!(app.get_fetch_commit()?.id(), local_commit_id);

        Ok(())
    }
}

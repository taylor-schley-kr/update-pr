use super::{repo::Repo, App};

pub trait Merge: Repo {
    /// Merge the current branch with the remote branch
    ///
    /// * `remote_branch` - The name of the remote branch to merge from
    /// * `fetch_commit` - The commit to merge
    fn merge(
        &self,
        remote_branch: &str,
        fetch_commit: git2::AnnotatedCommit<'_>,
    ) -> Result<(), git2::Error> {
        let repo = self.repo();

        // 1. do a merge analysis
        let analysis = repo.merge_analysis(&[&fetch_commit])?;

        // 2. Do the appropriate merge
        if analysis.0.is_fast_forward() {
            println!("Doing a fast forward");
            // do a fast forward
            let refname = format!("refs/heads/{}", remote_branch);
            match repo.find_reference(&refname) {
                Ok(mut r) => {
                    self.fast_forward(&mut r, &fetch_commit)?;
                }
                Err(_) => {
                    // The branch doesn't exist so just set the reference to the
                    // commit directly. Usually this is because you are pulling
                    // into an empty repository.
                    repo.reference(
                        &refname,
                        fetch_commit.id(),
                        true,
                        &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
                    )?;
                    repo.set_head(&refname)?;
                    repo.checkout_head(Some(
                        git2::build::CheckoutBuilder::default()
                            .allow_conflicts(true)
                            .conflict_style_merge(true)
                            .force(),
                    ))?;
                }
            };
        } else if analysis.0.is_normal() {
            // do a normal merge
            let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
            self.normal_merge(&head_commit, &fetch_commit)?;
        } else {
            println!("Nothing to do...");
        }
        Ok(())
    }

    fn normal_merge(
        &self,
        local: &git2::AnnotatedCommit,
        remote: &git2::AnnotatedCommit,
    ) -> Result<(), git2::Error> {
        let repo = self.repo();

        let local_tree = repo.find_commit(local.id())?.tree()?;
        let remote_tree = repo.find_commit(remote.id())?.tree()?;
        let ancestor = repo
            .find_commit(repo.merge_base(local.id(), remote.id())?)?
            .tree()?;
        let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

        if idx.has_conflicts() {
            println!("Merge conflicts detected...");
            repo.checkout_index(Some(&mut idx), None)?;
            return Ok(());
        }
        let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
        // now create the merge commit
        let msg = format!("Merge '{}' into {}", get_name(remote), get_name(local));
        let sig = repo.signature()?;
        let local_commit = repo.find_commit(local.id())?;
        let remote_commit = repo.find_commit(remote.id())?;
        // Do our merge commit and set current branch head to that commit.
        let _merge_commit = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &msg,
            &result_tree,
            &[&local_commit, &remote_commit],
        )?;
        // Set working tree to match head.
        repo.checkout_head(None)?;
        Ok(())
    }

    fn fast_forward(
        &self,
        lb: &mut git2::Reference,
        rc: &git2::AnnotatedCommit,
    ) -> Result<(), git2::Error> {
        let repo = self.repo();

        let name = match lb.name() {
            Some(s) => s.to_string(),
            None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
        };
        let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
        println!("{}", msg);
        lb.set_target(rc.id(), &msg)?;
        repo.set_head(&name)?;
        repo.checkout_head(Some(
            git2::build::CheckoutBuilder::default()
                // For some reason the force is required to make the working directory actually get updated
                // I suspect we should be adding some logic to handle dirty working directory states
                // but this is just an example so maybe not.
                .force(),
        ))?;
        Ok(())
    }
}

impl Merge for App {}

fn get_name(commit: &git2::AnnotatedCommit) -> String {
    commit
        .refname()
        .map_or_else(|| commit.id().to_string(), |refname| refname.to_string())
}

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

    impl Repo for TestApp {
        fn repo(&self) -> &Repository {
            &self.repo
        }
    }

    impl Merge for TestApp {}

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
    fn test_normal_merge() -> Result<()> {
        let fixture = TestFixture::new()?.setup()?;

        let usage_dir = fixture.get_repo_dir("usage")?;

        // Create a commit in the repo we want to merge to
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

        // Create and push a commit in the repo we want to merge from
        {
            let alt_dir = fixture.get_repo_dir("alt")?;

            let file_path = alt_dir.join("file2.txt");
            let mut file = File::create(&file_path)?;
            file.write_all(b"Hello World")?;

            git_cmd(&alt_dir).arg("add").arg(&file_path).status()?;
            git_cmd(&alt_dir)
                .arg("commit")
                .arg("-m")
                .arg("add file2.txt")
                .status()?;
            git_cmd(&alt_dir)
                .arg("push")
                .arg("origin")
                .arg("HEAD:main")
                .status()?;
        }

        // Fetch remote changes
        {
            git_cmd(&usage_dir)
                .arg("fetch")
                .arg("origin")
                .arg("main")
                .status()?;
        }

        let app = TestApp {
            repo: Repository::open(&usage_dir)?,
        };
        let fetch_commit = app.get_fetch_commit()?;

        let analysis = app.repo().merge_analysis(&[&fetch_commit])?;

        assert!(analysis.0.is_normal());

        app.merge("main", fetch_commit)?;

        let fetch_commit = app.get_fetch_commit()?;

        let analysis = app.repo().merge_analysis(&[&fetch_commit])?;

        assert!(analysis.0.is_up_to_date());

        Ok(())
    }

    #[test]
    fn test_fast_forward() -> Result<()> {
        let fixture = TestFixture::new()?.setup()?;

        let usage_dir = fixture.get_repo_dir("usage")?;

        // Create and push a commit in the repo we want to merge from
        {
            let alt_dir = fixture.get_repo_dir("alt")?;

            let file_path = alt_dir.join("file2.txt");
            let mut file = File::create(&file_path)?;
            file.write_all(b"Hello World")?;

            git_cmd(&alt_dir).arg("add").arg(&file_path).status()?;
            git_cmd(&alt_dir)
                .arg("commit")
                .arg("-m")
                .arg("add file2.txt")
                .status()?;
            git_cmd(&alt_dir)
                .arg("push")
                .arg("origin")
                .arg("HEAD:main")
                .status()?;
        }

        // Fetch remote changes
        {
            git_cmd(&usage_dir)
                .arg("fetch")
                .arg("origin")
                .arg("main")
                .status()?;
        }

        let app = TestApp {
            repo: Repository::open(&usage_dir)?,
        };

        let fetch_commit = app.get_fetch_commit()?;

        let analysis = app.repo().merge_analysis(&[&fetch_commit])?;

        assert!(analysis.0.is_fast_forward());

        app.merge("main", app.get_fetch_commit()?)?;

        let fetch_commit = app.get_fetch_commit()?;

        let analysis = app.repo().merge_analysis(&[&fetch_commit])?;

        assert!(analysis.0.is_up_to_date());

        Ok(())
    }
}

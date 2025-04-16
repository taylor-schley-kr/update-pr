use git2::Repository;

use super::App;

pub trait Repo {
    fn repo(&self) -> &Repository;
}

impl Repo for App {
    fn repo(&self) -> &Repository {
        &self.repo
    }
}

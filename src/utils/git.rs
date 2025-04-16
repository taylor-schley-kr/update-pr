use git2::{Cred, CredentialType};

pub fn ssh_creds(
    _url: &str,
    username_from_url: Option<&str>,
    _cred_type: CredentialType,
) -> Result<Cred, git2::Error> {
    Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
}


use crate::error::*;

use std::env;

/// Auth token for accessing github resources.
#[derive(Clone, RedactedDebug)]
pub struct GithubAuth {
    #[redacted]
    pub token: String,
}

impl From<String> for GithubAuth {
    fn from(token: String) -> Self {
        GithubAuth { token }
    }
}

impl GithubAuth {
    /// Retrieve the auth token from an environment variable.
    pub fn from_env(var: &str) -> Result<Self> {
        let token = env::var(var)?;
        Ok(GithubAuth { token })
    }
}
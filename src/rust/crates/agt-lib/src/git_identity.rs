//! Read git user identity from local git config.

use std::process::Command;

pub struct GitIdentity {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Read git config user.name and user.email. Returns None for each if not configured.
pub fn get_git_identity() -> GitIdentity {
    let name = Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        });

    let email = Command::new("git")
        .args(["config", "user.email"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        });

    GitIdentity { name, email }
}

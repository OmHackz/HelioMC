//! Resolve the Helio repository URL at build time.
//!
//! Prefers `git remote get-url origin`; falls back to the `repository`
//! field of `Cargo.toml`. The value is exposed to the crate as the
//! `HELIO_REPO_URL` environment variable (see `brand::REPO_URL`).

use std::path::PathBuf;

fn main() {
    let url = git_remote_url()
        .or_else(cargo_repository)
        .unwrap_or_else(|| panic!("could not determine repository URL"));
    println!("cargo:rustc-env=HELIO_REPO_URL={url}");
}

fn git_remote_url() -> Option<String> {
    let mut cmd = std::process::Command::new("git");
    cmd.args(["remote", "get-url", "origin"]);
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    let s = s.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn cargo_repository() -> Option<String> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("repository") {
            let rest = rest.trim_start();
            let rest = rest.strip_prefix('=')?.trim();
            let rest = rest.trim_matches('"');
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

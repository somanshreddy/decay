//! Auto-update. Every run asks the GitHub repo whether a newer tag exists and,
//! if so, kicks off `cargo install --git … --tag …` in the background — the
//! next invocation picks up the new binary.

use anyhow::{Context, Result, anyhow};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

/// How long we'll wait on the background check before giving up on it.
const CHECK_TIMEOUT: Duration = Duration::from_secs(5);
/// Don't retry a failing install for the same version more often than this.
const RETRY_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

/// A release newer than the one running.
pub struct Available {
    latest: Version,
    /// The tag exactly as the repo spells it (`v0.2.0`, `0.2.0`, …).
    tag: String,
}

fn state_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("decay")
}

fn state_path() -> PathBuf {
    state_dir().join("update-state.json")
}

fn log_path() -> PathBuf {
    state_dir().join("update.log")
}

#[derive(Default, Serialize, Deserialize)]
struct State {
    /// Version we last tried to install.
    last_attempt_version: Option<String>,
    /// Unix seconds of that attempt.
    last_attempt_at: Option<u64>,
}

fn load_state() -> State {
    std::fs::read_to_string(state_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_state(state: &State) {
    if std::fs::create_dir_all(state_dir()).is_err() {
        return;
    }
    if let Ok(json) = serde_json::to_string(state) {
        let _ = std::fs::write(state_path(), json);
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Auto-update is off when explicitly disabled, or when the running binary
/// isn't the cargo-installed one (don't clobber a dev build with a release).
fn auto_update_enabled() -> bool {
    let opted_out =
        std::env::var("DECAY_NO_UPDATE").is_ok_and(|val| !val.is_empty() && val != "0");
    !opted_out && is_cargo_install()
}

fn is_cargo_install() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let Some(bin_dir) = exe.parent() else {
        return false;
    };
    let cargo_home = std::env::var("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".cargo")
        });
    bin_dir == cargo_home.join("bin")
}

/// Highest semver tag on the remote, or `None` if it has no release tags yet.
///
/// `git ls-remote` rather than the GitHub API: no rate limit, no pagination,
/// and git is already required to install from the repo.
fn latest_tag() -> Result<Option<(Version, String)>> {
    let output = Command::new("git")
        .args(["ls-remote", "--tags", "--refs", REPO_URL])
        // Never block on a credential or host-key prompt, and bail out of a
        // stalled transfer instead of hanging the CLI.
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_HTTP_LOW_SPEED_LIMIT", "1000")
        .env("GIT_HTTP_LOW_SPEED_TIME", "5")
        .stdin(Stdio::null())
        .output()
        .context("failed to run git ls-remote (is git installed?)")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git ls-remote failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(pick_latest(stdout.lines().filter_map(parse_tag_ref)))
}

/// `<sha>\trefs/tags/v0.2.0` -> `v0.2.0`.
fn parse_tag_ref(line: &str) -> Option<String> {
    line.split_whitespace()
        .nth(1)?
        .strip_prefix("refs/tags/")
        .map(String::from)
}

/// Highest semver among tag names, ignoring anything that isn't `X.Y.Z` /
/// `vX.Y.Z`. Returns the version plus the tag spelling to pass to cargo.
fn pick_latest(names: impl Iterator<Item = String>) -> Option<(Version, String)> {
    names
        .filter_map(|name| {
            let version = Version::parse(name.trim_start_matches('v')).ok()?;
            Some((version, name))
        })
        .max_by(|a, b| a.0.cmp(&b.0))
}

fn current_version() -> Version {
    Version::parse(VERSION).expect("crate version is valid semver")
}

/// Start the version check on a background thread so it never delays the
/// command the user actually ran. Returns `None` when updates are disabled.
pub fn spawn_check() -> Option<Receiver<Available>> {
    if !auto_update_enabled() {
        return None;
    }
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        if let Ok(Some((latest, tag))) = latest_tag()
            && latest > current_version()
        {
            let _ = tx.send(Available { latest, tag });
        }
    });
    Some(rx)
}

/// Collect the background check and, if a newer release exists, launch the
/// install detached. Never fails the command that was actually run, and never
/// waits longer than [`CHECK_TIMEOUT`].
pub fn finish(check: Option<Receiver<Available>>) {
    let Some(rx) = check else { return };
    let Ok(Available { latest, tag }) = rx.recv_timeout(CHECK_TIMEOUT) else {
        // Timed out, or the check found nothing / failed. Either way: quiet.
        return;
    };

    let mut state = load_state();
    if attempted_recently(&state, &latest) {
        return;
    }

    match spawn_install(&tag) {
        Ok(()) => {
            record_attempt(&mut state, &latest);
            println!(
                "  ⬆️  Updating decay {VERSION} → {latest} in the background. \
                 The next run uses the new version."
            );
        }
        Err(e) => {
            println!("  ⚠️  decay {latest} is available but the update could not start: {e}");
            println!("     Run: {}", install_hint(&tag));
        }
    }
}

fn attempted_recently(state: &State, latest: &Version) -> bool {
    state.last_attempt_version.as_deref() == Some(&latest.to_string())
        && state
            .last_attempt_at
            .is_some_and(|at| now_secs().saturating_sub(at) < RETRY_INTERVAL.as_secs())
}

fn record_attempt(state: &mut State, latest: &Version) {
    state.last_attempt_version = Some(latest.to_string());
    state.last_attempt_at = Some(now_secs());
    save_state(state);
}

fn install_args(tag: &str) -> [String; 7] {
    [
        "install".into(),
        "--git".into(),
        REPO_URL.into(),
        "--tag".into(),
        tag.into(),
        "--force".into(),
        "--quiet".into(),
    ]
}

fn install_hint(tag: &str) -> String {
    format!("cargo install --git {REPO_URL} --tag {tag} --force")
}

/// Fire off `cargo install` and return immediately; the child outlives us.
fn spawn_install(tag: &str) -> Result<()> {
    std::fs::create_dir_all(state_dir()).context("failed to create data directory")?;
    let log = std::fs::File::create(log_path()).context("failed to open update log")?;
    let log_err = log.try_clone().context("failed to open update log")?;

    Command::new("cargo")
        .args(install_args(tag))
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err))
        .spawn()
        .context("failed to start cargo install")?;

    Ok(())
}

/// `decay update` — check now and, unless `check_only`, install in the
/// foreground so the user sees what happened.
pub fn run_command(check_only: bool) -> Result<()> {
    let current = current_version();
    println!();
    println!("  Installed: decay {current}");

    let Some((latest, tag)) = latest_tag()? else {
        println!("  Latest:    no releases tagged on {REPO_URL} yet.");
        println!();
        return Ok(());
    };

    if latest <= current {
        println!("  Latest:    decay {latest} — you're up to date.");
        println!();
        return Ok(());
    }

    println!("  Latest:    decay {latest}");
    println!();

    if check_only {
        println!("  Run `decay update` to install it.");
        println!();
        return Ok(());
    }

    println!("  Installing decay {latest}…");
    let status = Command::new("cargo")
        .args(install_args(&tag))
        .status()
        .context("failed to run cargo install")?;

    if status.success() {
        record_attempt(&mut load_state(), &latest);
        println!("  ✅ Updated to decay {latest}.");
    } else {
        println!("  ⚠️  cargo install failed (exit {status}).");
        println!("     Try: {}", install_hint(&tag));
    }
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pick(names: &[&str]) -> Option<(Version, String)> {
        pick_latest(names.iter().map(|s| s.to_string()))
    }

    #[test]
    fn picks_highest_semver_not_lexicographic() {
        let (version, tag) = pick(&["v0.1.0", "v0.10.0", "v0.9.0"]).unwrap();
        assert_eq!(version, Version::parse("0.10.0").unwrap());
        assert_eq!(tag, "v0.10.0");
    }

    #[test]
    fn keeps_the_tag_spelling_cargo_needs() {
        assert_eq!(pick(&["0.2.0"]).unwrap().1, "0.2.0");
        assert_eq!(pick(&["v0.2.0"]).unwrap().1, "v0.2.0");
    }

    #[test]
    fn ignores_non_semver_tags() {
        assert_eq!(pick(&["nightly", "release", "v1"]), None);
        assert_eq!(pick(&["latest", "v1.2.3"]).unwrap().1, "v1.2.3");
    }

    #[test]
    fn prereleases_rank_below_their_release() {
        let (version, _) = pick(&["v1.0.0-rc.1", "v1.0.0"]).unwrap();
        assert_eq!(version, Version::parse("1.0.0").unwrap());
    }

    #[test]
    fn no_tags_means_no_update() {
        assert_eq!(pick(&[]), None);
    }

    #[test]
    fn parses_ls_remote_output() {
        let out = "9f3a1b2\trefs/tags/v0.1.0\n1c4d5e6\trefs/tags/v0.2.0\n";
        let tags: Vec<String> = out.lines().filter_map(parse_tag_ref).collect();
        assert_eq!(tags, vec!["v0.1.0", "v0.2.0"]);
        assert_eq!(pick_latest(tags.into_iter()).unwrap().1, "v0.2.0");
    }

    #[test]
    fn ignores_malformed_ls_remote_lines() {
        assert_eq!(parse_tag_ref(""), None);
        assert_eq!(parse_tag_ref("9f3a1b2"), None);
        assert_eq!(parse_tag_ref("9f3a1b2\trefs/heads/main"), None);
    }

    #[test]
    fn crate_version_is_valid_semver() {
        current_version();
    }
}

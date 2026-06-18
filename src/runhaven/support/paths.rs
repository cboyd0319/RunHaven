use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::validators::validate_run_id;

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

#[cfg(test)]
pub(crate) static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn runhaven_cache_root() -> PathBuf {
    std::env::var_os("RUNHAVEN_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join("Library").join("Caches").join("runhaven"))
}

pub fn runs_log_path() -> PathBuf {
    runhaven_cache_root().join("runs.jsonl")
}

pub fn egress_policy_log_path() -> PathBuf {
    runhaven_cache_root().join("egress-policy.jsonl")
}

pub fn auth_broker_log_path() -> PathBuf {
    runhaven_cache_root().join("auth-broker.jsonl")
}

pub fn active_runs_dir() -> PathBuf {
    runhaven_cache_root().join("active-runs")
}

pub fn worktrees_dir() -> PathBuf {
    runhaven_cache_root().join("worktrees")
}

pub fn active_run_path(run_id: &str) -> Result<PathBuf> {
    validate_run_id(run_id)?;
    Ok(active_runs_dir().join(format!("{run_id}.json")))
}

pub fn state_lock_path(state_volume: &str) -> PathBuf {
    runhaven_cache_root()
        .join("locks")
        .join(format!("{state_volume}.lock"))
}

pub fn ensure_private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    restrict_dir_permissions(path)?;
    Ok(())
}

pub fn ensure_private_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        let root = runhaven_cache_root();
        if parent.starts_with(&root) {
            ensure_private_dir(&root)?;
        }
        ensure_private_dir(parent)?;
    }
    Ok(())
}

pub fn create_private_file(path: &Path) -> Result<File> {
    ensure_private_parent(path)?;
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    set_private_file_mode(&mut options);
    let file = options.open(path)?;
    restrict_file_permissions(&file)?;
    Ok(file)
}

pub fn open_private_append(path: &Path) -> Result<File> {
    ensure_private_parent(path)?;
    let mut options = OpenOptions::new();
    options.append(true).create(true);
    set_private_file_mode(&mut options);
    let file = options.open(path)?;
    restrict_file_permissions(&file)?;
    Ok(file)
}

pub fn open_private_read_write(path: &Path) -> Result<File> {
    ensure_private_parent(path)?;
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    set_private_file_mode(&mut options);
    let file = options.open(path)?;
    restrict_file_permissions(&file)?;
    Ok(file)
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(unix)]
fn set_private_file_mode(options: &mut OpenOptions) {
    options.mode(0o600);
}

#[cfg(not(unix))]
fn set_private_file_mode(_options: &mut OpenOptions) {}

#[cfg(unix)]
fn restrict_dir_permissions(path: &Path) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_dir_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn restrict_file_permissions(file: &File) -> Result<()> {
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_file_permissions(_file: &File) -> Result<()> {
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn mode(path: &Path) -> u32 {
        fs::metadata(path).expect("metadata").permissions().mode() & 0o777
    }

    #[test]
    fn private_cache_helpers_use_owner_only_permissions() {
        let cache = tempfile::tempdir().expect("cache");
        let dir = cache.path().join("logs");
        let append = dir.join("runs.jsonl");
        let lock = cache.path().join("locks").join("state.lock");

        ensure_private_dir(&dir).expect("private dir");
        drop(open_private_append(&append).expect("append file"));
        drop(open_private_read_write(&lock).expect("lock file"));

        assert_eq!(mode(&dir), 0o700);
        assert_eq!(mode(&append), 0o600);
        assert_eq!(mode(lock.parent().expect("lock parent")), 0o700);
        assert_eq!(mode(&lock), 0o600);
    }
}

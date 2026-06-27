use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::support::validators::validate_run_id;

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

#[cfg(any(test, feature = "test-support"))]
static TEST_CACHE_ROOT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(any(test, feature = "test-support"))]
static TEST_CACHE_ROOT: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
pub struct CacheRootOverride {
    previous: Option<PathBuf>,
    _guard: std::sync::MutexGuard<'static, ()>,
}

#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
impl Drop for CacheRootOverride {
    fn drop(&mut self) {
        *TEST_CACHE_ROOT.lock().expect("cache root override lock") = self.previous.clone();
    }
}

#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
pub fn override_cache_root_for_tests(path: &Path) -> CacheRootOverride {
    let guard = TEST_CACHE_ROOT_LOCK
        .lock()
        .expect("cache root override lock");
    let mut root = TEST_CACHE_ROOT.lock().expect("cache root override lock");
    let previous = root.clone();
    *root = Some(path.to_path_buf());
    drop(root);
    CacheRootOverride {
        previous,
        _guard: guard,
    }
}

pub fn runhaven_cache_root() -> PathBuf {
    #[cfg(any(test, feature = "test-support"))]
    if let Some(path) = TEST_CACHE_ROOT
        .lock()
        .expect("cache root override lock")
        .clone()
    {
        return path;
    }

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

pub fn oauth_token_path(agent: &str) -> PathBuf {
    runhaven_cache_root()
        .join("auth")
        .join(format!("{agent}-oauth-token"))
}

pub fn zork_save_path() -> PathBuf {
    runhaven_cache_root().join("tui").join("zork1-save.ifzs")
}

/// A stable empty workspace for `runhaven login`. The in-sandbox login does not
/// operate on a project, but a run plan requires a workspace, so it is mounted
/// read-only from here instead of exposing the user's current directory.
pub fn login_workspace_dir() -> Result<PathBuf> {
    let dir = runhaven_cache_root().join("login-workspace");
    ensure_private_dir(&dir)?;
    Ok(dir)
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

pub fn write_private_atomic(path: &Path, contents: &[u8]) -> Result<()> {
    ensure_private_parent(path)?;
    let parent = path
        .parent()
        .with_context(|| format!("path has no parent: {}", path.display()))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("runhaven-file");
    let mut last_error = None;

    for attempt in 0..16 {
        let temp_path = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), attempt));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        set_private_file_mode(&mut options);
        match options.open(&temp_path) {
            Ok(mut file) => {
                restrict_file_permissions(&file)?;
                if let Err(error) = file.write_all(contents).and_then(|_| file.sync_all()) {
                    let _ = fs::remove_file(&temp_path);
                    return Err(error).with_context(|| {
                        format!("could not write private file: {}", temp_path.display())
                    });
                }
                fs::rename(&temp_path, path).with_context(|| {
                    format!(
                        "could not replace private file {} with {}",
                        path.display(),
                        temp_path.display()
                    )
                })?;
                return Ok(());
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                last_error = Some(error);
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "could not create private temp file: {}",
                        temp_path.display()
                    )
                });
            }
        }
    }

    let error = last_error.unwrap_or_else(|| std::io::Error::other("could not create temp file"));
    Err(error).with_context(|| format!("could not create temp file for {}", path.display()))
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
        write_private_atomic(&cache.path().join("tui").join("zork1-save.ifzs"), b"IFZS")
            .expect("atomic file");

        assert_eq!(mode(&dir), 0o700);
        assert_eq!(mode(&append), 0o600);
        assert_eq!(mode(lock.parent().expect("lock parent")), 0o700);
        assert_eq!(mode(&lock), 0o600);
        assert_eq!(mode(&cache.path().join("tui")), 0o700);
        assert_eq!(
            mode(&cache.path().join("tui").join("zork1-save.ifzs")),
            0o600
        );
    }
}

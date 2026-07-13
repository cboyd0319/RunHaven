//! Ownership and keyed admission for desktop-only background processes.
//!
//! Image rebuilds and in-app agent sign-ins are long-running host children.
//! This registry gives each operation a stable key, keeps exactly one active
//! job per key, and owns every spawned child until it exits. Shutdown first
//! closes admission, then kills and reaps children without holding the registry
//! lock across process I/O.

use std::collections::HashMap;
use std::process::{Child, Command, ExitStatus};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const CHILD_POLL_INTERVAL: Duration = Duration::from_millis(50);
const SHUTDOWN_WAIT: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub(crate) struct BackgroundJobRegistry {
    inner: Arc<RegistryInner>,
}

struct RegistryInner {
    state: Mutex<RegistryState>,
    changed: Condvar,
}

#[derive(Default)]
struct RegistryState {
    shutting_down: bool,
    jobs: HashMap<String, JobRecord>,
}

struct JobRecord {
    id: String,
    child: Option<OwnedChild>,
}

pub(crate) enum JobClaim {
    Started(JobLease),
    Existing(String),
}

pub(crate) fn image_build_key(tag: &str) -> String {
    format!("image:{tag}")
}

pub(crate) struct JobLease {
    registry: BackgroundJobRegistry,
    key: String,
    id: String,
}

#[derive(Clone)]
pub(crate) struct OwnedChild {
    inner: Arc<Mutex<OwnedChildState>>,
}

struct OwnedChildState {
    child: Child,
    exit: Option<ExitStatus>,
}

impl Default for BackgroundJobRegistry {
    fn default() -> Self {
        Self {
            inner: Arc::new(RegistryInner {
                state: Mutex::new(RegistryState::default()),
                changed: Condvar::new(),
            }),
        }
    }
}

impl BackgroundJobRegistry {
    pub(crate) fn claim(&self, key: &str, id: &str) -> Result<JobClaim, String> {
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| "Background job tracking is unavailable.".to_string())?;
        if state.shutting_down {
            return Err("RunHaven is closing and cannot start another background job.".to_string());
        }
        if let Some(job) = state.jobs.get(key) {
            return Ok(JobClaim::Existing(job.id.clone()));
        }
        state.jobs.insert(
            key.to_string(),
            JobRecord {
                id: id.to_string(),
                child: None,
            },
        );
        Ok(JobClaim::Started(JobLease {
            registry: self.clone(),
            key: key.to_string(),
            id: id.to_string(),
        }))
    }

    pub(crate) fn active_id(&self, key: &str) -> Result<Option<String>, String> {
        let state = self
            .inner
            .state
            .lock()
            .map_err(|_| "Background job tracking is unavailable.".to_string())?;
        Ok(state.jobs.get(key).map(|job| job.id.clone()))
    }

    /// Close admission, terminate children, then give their owning workers time
    /// to unwind provider/network cleanup. No registry lock is held while a
    /// child is killed or reaped.
    pub(crate) fn shutdown(&self) {
        let children = {
            let mut state = self
                .inner
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.shutting_down = true;
            state
                .jobs
                .values()
                .filter_map(|job| job.child.clone())
                .collect::<Vec<_>>()
        };
        for child in children {
            if let Err(error) = child.terminate_and_wait() {
                log::warn!("background child cleanup failed: {error}");
            }
        }
        let deadline = Instant::now() + SHUTDOWN_WAIT;
        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !state.jobs.is_empty() {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                log::warn!("background job cleanup did not finish before shutdown timeout");
                break;
            }
            let (next, timeout) = self
                .inner
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next;
            if timeout.timed_out() && !state.jobs.is_empty() {
                log::warn!("background job cleanup did not finish before shutdown timeout");
                break;
            }
        }
    }

    #[cfg(test)]
    fn is_shutting_down(&self) -> bool {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .shutting_down
    }
}

impl JobLease {
    /// Spawn and register a child while admission is locked. This narrow lock
    /// window closes the race where app shutdown could begin after a worker
    /// checked the flag but before it called `Command::spawn`.
    pub(crate) fn spawn_child(&self, command: &mut Command) -> Result<OwnedChild, String> {
        let mut state = self
            .registry
            .inner
            .state
            .lock()
            .map_err(|_| "Background job tracking is unavailable.".to_string())?;
        if state.shutting_down {
            return Err("RunHaven is closing and did not start the background job.".to_string());
        }
        let job = state
            .jobs
            .get_mut(&self.key)
            .filter(|job| job.id == self.id)
            .ok_or_else(|| "Background job ownership is no longer active.".to_string())?;
        let child = command
            .spawn()
            .map_err(|error| format!("could not start background process: {error}"))?;
        let owned = OwnedChild {
            inner: Arc::new(Mutex::new(OwnedChildState { child, exit: None })),
        };
        job.child = Some(owned.clone());
        Ok(owned)
    }
}

impl Drop for JobLease {
    fn drop(&mut self) {
        let mut state = self
            .registry
            .inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state
            .jobs
            .get(&self.key)
            .is_some_and(|job| job.id == self.id)
        {
            state.jobs.remove(&self.key);
            self.registry.inner.changed.notify_all();
        }
    }
}

impl OwnedChild {
    pub(crate) fn take_stdout(&self) -> Option<std::process::ChildStdout> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .child
            .stdout
            .take()
    }

    pub(crate) fn take_stderr(&self) -> Option<std::process::ChildStderr> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .child
            .stderr
            .take()
    }

    pub(crate) fn wait(&self) -> Result<ExitStatus, String> {
        loop {
            {
                let mut state = self
                    .inner
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if let Some(status) = state.exit {
                    return Ok(status);
                }
                if let Some(status) = state
                    .child
                    .try_wait()
                    .map_err(|error| format!("could not wait for background process: {error}"))?
                {
                    state.exit = Some(status);
                    return Ok(status);
                }
            }
            thread::sleep(CHILD_POLL_INTERVAL);
        }
    }

    fn terminate_and_wait(&self) -> Result<(), String> {
        let mut state = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.exit.is_some() {
            return Ok(());
        }
        if let Some(status) = state
            .child
            .try_wait()
            .map_err(|error| format!("could not inspect background process: {error}"))?
        {
            state.exit = Some(status);
            return Ok(());
        }
        state
            .child
            .kill()
            .map_err(|error| format!("could not stop background process: {error}"))?;
        let status = state
            .child
            .wait()
            .map_err(|error| format!("could not reap background process: {error}"))?;
        state.exit = Some(status);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    #[test]
    fn duplicate_key_returns_the_existing_job_id() {
        let registry = BackgroundJobRegistry::default();
        let first = registry.claim("image:runhaven/codex:0.7.0", "build-one");
        let _lease = match first.expect("first claim") {
            JobClaim::Started(lease) => lease,
            JobClaim::Existing(_) => panic!("first claim must start"),
        };

        assert!(matches!(
            registry
                .claim("image:runhaven/codex:0.7.0", "build-two")
                .expect("duplicate claim"),
            JobClaim::Existing(ref id) if id == "build-one"
        ));
    }

    #[cfg(unix)]
    #[test]
    fn shutdown_kills_and_reaps_owned_children() {
        let registry = BackgroundJobRegistry::default();
        let lease = match registry
            .claim("sign-in:codex", "sign-in-one")
            .expect("claim")
        {
            JobClaim::Started(lease) => lease,
            JobClaim::Existing(_) => panic!("first claim must start"),
        };
        let mut command = Command::new("sh");
        command
            .args(["-c", "sleep 60"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let owned = lease.spawn_child(&mut command).expect("owned child");
        let worker = thread::spawn(move || {
            let result = owned.wait();
            drop(lease);
            result
        });

        registry.shutdown();

        assert!(
            worker.join().expect("worker").is_ok(),
            "shutdown must leave a reaped child"
        );
        assert!(registry.is_shutting_down());
    }

    #[cfg(unix)]
    #[test]
    fn shutdown_after_claim_prevents_a_late_child_spawn_and_releases_the_key() {
        let temp = tempfile::tempdir().expect("temp dir");
        let marker = temp.path().join("late-child-ran");
        let registry = BackgroundJobRegistry::default();
        let lease = match registry
            .claim("sign-in:codex", "sign-in-one")
            .expect("claim")
        {
            JobClaim::Started(lease) => lease,
            JobClaim::Existing(_) => panic!("first claim must start"),
        };
        let shutdown_registry = registry.clone();
        let shutdown = thread::spawn(move || shutdown_registry.shutdown());
        let deadline = Instant::now() + Duration::from_secs(2);
        while !registry.is_shutting_down() {
            assert!(Instant::now() < deadline, "shutdown did not start");
            thread::yield_now();
        }
        let mut command = Command::new("touch");
        command.arg(&marker);

        let error = match lease.spawn_child(&mut command) {
            Ok(_) => panic!("shutdown must prevent a late child"),
            Err(error) => error,
        };
        drop(lease);

        assert!(error.contains("closing"));
        assert!(!marker.exists(), "late child must never run");
        assert_eq!(
            registry.active_id("sign-in:codex").expect("active id"),
            None
        );
        shutdown.join().expect("shutdown");
    }
}

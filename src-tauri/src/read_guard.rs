//! Local-DoS throttle for read-only commands that shell out to Apple `container`.
//!
//! The WebView can invoke read commands concurrently on the Tauri thread pool,
//! so without a backend throttle a compromised WebView could spam
//! `get_managed_sessions`, `get_network_prune_preview`, or `get_image_status`
//! to fork many concurrent `container` subprocesses. That is self-inflicted
//! resource use, not a trust-boundary break, so the fix stays proportionate:
//! each command family gets a [`ReadGuard`] combining single-flight with a
//! short TTL cache. A caller either returns a fresh cached result, runs the
//! probe itself while concurrent callers wait, or retries after a failed probe
//! (errors are never cached).
//!
//! Single-flight mechanism: the guard's mutex is held across the probe. At
//! this scale (three read families, each probe one short `container`
//! invocation) that is the simplest correct coalescing: followers block on the
//! lock and then read the fresh entry the leader stored. A hung probe stalls
//! only its own family's read command; a panicking probe poisons the lock and
//! every later call fails closed, matching the registries in
//! `registry_support`. Mutating commands never go through a guard and are
//! never throttled or cached.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use runhaven_core::ui_contracts::ManagedSessionListData;

use crate::contracts::{ImageStatusResponse, NetworkPrunePreview};
use crate::registry_support::{BoundedMap, lock_or};

/// Fresh-enough window for cached read results: below the UI's 4-second poll,
/// so a poll never serves state older than 2 seconds. After a mutating command
/// (stop, kill, repair, cleanup, launch) the next poll may still serve a
/// cached result up to 2 seconds old, which is at worst one extra poll cycle
/// before the UI sees the new state.
const READ_PROBE_TTL: Duration = Duration::from_secs(2);

/// Cap on cached results per guard. Sessions and the network preview each use
/// one fixed key; image status keys by agent name. Only successful probes are
/// stored, but the map stays bounded regardless of caller-supplied keys.
const MAX_CACHED_RESULTS: usize = 16;

const CACHE_UNAVAILABLE: &str = "read cache is unavailable";

struct CachedResult<T> {
    value: T,
    stored_at: Instant,
}

/// Single-flight, TTL-cached wrapper around one read command family's probe.
pub(crate) struct ReadGuard<T> {
    ttl: Duration,
    state: Mutex<BoundedMap<CachedResult<T>>>,
}

impl<T: Clone> ReadGuard<T> {
    fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            state: Mutex::new(BoundedMap::default()),
        }
    }

    /// Return the cached result for `key` when it is fresher than the TTL,
    /// otherwise run `probe` while holding the guard's lock so concurrent
    /// callers coalesce into one subprocess-spawning probe. A failed probe
    /// stores nothing and releases the lock, so the next caller retries
    /// instead of seeing a cached error.
    pub(crate) fn run(
        &self,
        key: &str,
        probe: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        let mut state = lock_or(self.state.lock(), CACHE_UNAVAILABLE)?;
        if let Some(cached) = state.get(key)
            && cached.stored_at.elapsed() < self.ttl
        {
            return Ok(cached.value.clone());
        }
        let value = probe()?;
        state.insert_tracked(
            key,
            CachedResult {
                value: value.clone(),
                stored_at: Instant::now(),
            },
        );
        state.prune(MAX_CACHED_RESULTS);
        Ok(value)
    }
}

/// One guard per throttled read command family, registered as Tauri managed
/// state. Separate guards so a slow probe in one family cannot stall reads in
/// another.
pub(crate) struct ReadGuards {
    pub(crate) managed_sessions: ReadGuard<ManagedSessionListData>,
    pub(crate) network_prune_preview: ReadGuard<NetworkPrunePreview>,
    pub(crate) image_status: ReadGuard<ImageStatusResponse>,
}

impl Default for ReadGuards {
    fn default() -> Self {
        Self {
            managed_sessions: ReadGuard::new(READ_PROBE_TTL),
            network_prune_preview: ReadGuard::new(READ_PROBE_TTL),
            image_status: ReadGuard::new(READ_PROBE_TTL),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    use super::{MAX_CACHED_RESULTS, ReadGuard};

    #[test]
    fn fresh_cached_result_skips_the_probe() {
        let guard = ReadGuard::new(Duration::from_secs(60));
        let probes = AtomicUsize::new(0);
        let probe = || {
            probes.fetch_add(1, Ordering::SeqCst);
            Ok(7_u32)
        };
        assert_eq!(guard.run("key", probe), Ok(7));
        assert_eq!(guard.run("key", probe), Ok(7));
        assert_eq!(probes.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn expired_entry_probes_again() {
        let guard = ReadGuard::new(Duration::from_millis(25));
        let probes = AtomicUsize::new(0);
        let probe = || {
            probes.fetch_add(1, Ordering::SeqCst);
            Ok(7_u32)
        };
        assert_eq!(guard.run("key", probe), Ok(7));
        thread::sleep(Duration::from_millis(50));
        assert_eq!(guard.run("key", probe), Ok(7));
        assert_eq!(probes.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn distinct_keys_probe_and_cache_separately() {
        let guard = ReadGuard::new(Duration::from_secs(60));
        let probes = AtomicUsize::new(0);
        let probe = || {
            probes.fetch_add(1, Ordering::SeqCst);
            Ok(7_u32)
        };
        assert_eq!(guard.run("codex", probe), Ok(7));
        assert_eq!(guard.run("claude", probe), Ok(7));
        assert_eq!(guard.run("codex", probe), Ok(7));
        assert_eq!(probes.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn concurrent_callers_coalesce_to_one_probe() {
        let guard = Arc::new(ReadGuard::new(Duration::from_secs(60)));
        let probes = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(8));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let guard = Arc::clone(&guard);
                let probes = Arc::clone(&probes);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    guard.run("key", || {
                        probes.fetch_add(1, Ordering::SeqCst);
                        thread::sleep(Duration::from_millis(50));
                        Ok(42_u32)
                    })
                })
            })
            .collect();
        for handle in handles {
            assert_eq!(handle.join().expect("caller thread"), Ok(42));
        }
        assert_eq!(probes.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn failed_probe_is_not_cached_and_the_next_caller_retries() {
        let guard = ReadGuard::new(Duration::from_secs(60));
        let probes = AtomicUsize::new(0);
        let failed = guard.run("key", || -> Result<u32, String> {
            probes.fetch_add(1, Ordering::SeqCst);
            Err("probe failed".to_string())
        });
        assert_eq!(failed, Err("probe failed".to_string()));
        let retried = guard.run("key", || {
            probes.fetch_add(1, Ordering::SeqCst);
            Ok(9_u32)
        });
        assert_eq!(retried, Ok(9));
        assert_eq!(probes.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn cache_stays_bounded_and_evicts_the_oldest_key() {
        let guard = ReadGuard::new(Duration::from_secs(60));
        let probes = AtomicUsize::new(0);
        let probe = || {
            probes.fetch_add(1, Ordering::SeqCst);
            Ok(7_u32)
        };
        for index in 0..=MAX_CACHED_RESULTS {
            assert_eq!(guard.run(&format!("key-{index}"), probe), Ok(7));
        }
        // key-0 was evicted by the bound, so probing it again re-runs the probe.
        assert_eq!(guard.run("key-0", probe), Ok(7));
        assert_eq!(probes.load(Ordering::SeqCst), MAX_CACHED_RESULTS + 2);
    }
}

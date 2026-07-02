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
//! Single-flight mechanism: each cache key gets its own slot (a per-key
//! mutex around the cached entry) behind a short-held outer lock. A caller
//! takes the outer lock only to get-or-insert its key's slot, releases it,
//! then holds the slot lock across probe-plus-cache. Followers for the same
//! key block on the slot and read the fresh entry the leader stored, while
//! callers of different keys probe independently. A hung probe stalls only
//! its own key; a panicking probe poisons that key's slot and later calls
//! for that key fail closed, matching the registries in `registry_support`.
//! Mutating commands never go through a guard and are never throttled or
//! cached.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use runhaven_core::ui_contracts::ManagedSessionListData;

use crate::contracts::{ImageStatusResponse, ManagedSessionStatusResponse, NetworkPrunePreview};
use crate::registry_support::{BoundedMap, lock_or};

/// Fresh-enough window for cached read results: below the UI's 4-second poll,
/// so a poll never serves state older than 2 seconds. After a mutating command
/// (stop, kill, repair, cleanup, launch) the next poll may still serve a
/// cached result up to 2 seconds old, which is at worst one extra poll cycle
/// before the UI sees the new state.
const READ_PROBE_TTL: Duration = Duration::from_secs(2);

/// Cap on cached slots per guard. Sessions and the network preview each use
/// one fixed key; image status keys by agent name and per-session status keys
/// by session id. The slot map stays bounded regardless of caller-supplied
/// keys; evicting a slot while its probe is still in flight only costs one
/// extra probe for that key, never unbounded growth.
const MAX_CACHED_RESULTS: usize = 16;

const CACHE_UNAVAILABLE: &str = "read cache is unavailable";

struct CachedResult<T> {
    value: T,
    stored_at: Instant,
}

/// One key's cache entry plus the per-key lock held across its probe. `None`
/// means no successful probe has been cached yet (errors are never cached).
type CacheSlot<T> = Arc<Mutex<Option<CachedResult<T>>>>;

/// Per-key single-flight, TTL-cached wrapper around one read command
/// family's probe.
pub(crate) struct ReadGuard<T> {
    ttl: Duration,
    slots: Mutex<BoundedMap<CacheSlot<T>>>,
}

impl<T: Clone> ReadGuard<T> {
    fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            slots: Mutex::new(BoundedMap::default()),
        }
    }

    /// Return the cached result for `key` when it is fresher than the TTL,
    /// otherwise run `probe` while holding `key`'s slot lock so concurrent
    /// callers of the same key coalesce into one subprocess-spawning probe.
    /// Callers of different keys hold different slot locks and proceed
    /// independently. A failed probe stores nothing and releases the slot,
    /// so the next caller retries instead of seeing a cached error.
    pub(crate) fn run(
        &self,
        key: &str,
        probe: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        // Outer lock: get-or-insert the slot, prune, release. Never held
        // across a probe, so one key's hung probe cannot block another key.
        let slot = {
            let mut slots = lock_or(self.slots.lock(), CACHE_UNAVAILABLE)?;
            let slot = Arc::clone(slots.entry_tracked(key));
            slots.prune(MAX_CACHED_RESULTS);
            slot
        };
        let mut entry = lock_or(slot.lock(), CACHE_UNAVAILABLE)?;
        if let Some(cached) = entry.as_ref()
            && cached.stored_at.elapsed() < self.ttl
        {
            return Ok(cached.value.clone());
        }
        let value = probe()?;
        *entry = Some(CachedResult {
            value: value.clone(),
            stored_at: Instant::now(),
        });
        Ok(value)
    }
}

/// One guard per throttled read command family, registered as Tauri managed
/// state. Separate guards keep each family's cached type distinct; within a
/// guard, locking is per key, so a slow probe stalls only same-key callers.
pub(crate) struct ReadGuards {
    pub(crate) managed_sessions: ReadGuard<ManagedSessionListData>,
    /// Keyed by session id. Added with the per-session monitor windows: the
    /// main window plus up to six monitors can poll the same session's
    /// status, and each probe is a `container` subprocess, so same-session
    /// polls must coalesce.
    pub(crate) managed_session_status: ReadGuard<ManagedSessionStatusResponse>,
    pub(crate) network_prune_preview: ReadGuard<NetworkPrunePreview>,
    pub(crate) image_status: ReadGuard<ImageStatusResponse>,
}

impl Default for ReadGuards {
    fn default() -> Self {
        Self {
            managed_sessions: ReadGuard::new(READ_PROBE_TTL),
            managed_session_status: ReadGuard::new(READ_PROBE_TTL),
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

    use super::{CACHE_UNAVAILABLE, MAX_CACHED_RESULTS, ReadGuard};

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
    fn concurrent_callers_of_distinct_keys_probe_in_parallel() {
        // Both probes rendezvous on a barrier from inside the probe closure.
        // Under a single guard-wide lock held across probes, the second probe
        // could not start until the first finished, so this wait would
        // deadlock; per-key slots let both probes run at the same time.
        let guard = Arc::new(ReadGuard::new(Duration::from_secs(60)));
        let rendezvous = Arc::new(Barrier::new(2));
        let handles: Vec<_> = ["left", "right"]
            .into_iter()
            .map(|key| {
                let guard = Arc::clone(&guard);
                let rendezvous = Arc::clone(&rendezvous);
                thread::spawn(move || {
                    guard.run(key, || {
                        rendezvous.wait();
                        Ok(11_u32)
                    })
                })
            })
            .collect();
        for handle in handles {
            assert_eq!(handle.join().expect("caller thread"), Ok(11));
        }
    }

    #[test]
    fn panicked_probe_fails_closed_for_its_key_but_not_for_others() {
        let guard = Arc::new(ReadGuard::new(Duration::from_secs(60)));
        let poisoner = {
            let guard = Arc::clone(&guard);
            thread::spawn(move || {
                let _ = guard.run("poisoned", || -> Result<u32, String> {
                    panic!("probe died")
                });
            })
        };
        assert!(poisoner.join().is_err(), "poisoning thread must panic");
        // The poisoned key's slot fails closed on every later call.
        assert_eq!(
            guard.run("poisoned", || Ok(1_u32)),
            Err(CACHE_UNAVAILABLE.to_string())
        );
        // Other keys have their own slots and keep working.
        assert_eq!(guard.run("healthy", || Ok(2_u32)), Ok(2));
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

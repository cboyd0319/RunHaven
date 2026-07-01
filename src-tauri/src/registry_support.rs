//! Shared mechanics for the desktop app's bounded, in-memory status registries.
//!
//! Four registries (launch status, session output, image build, agent sign-in)
//! all keep a fixed-size map of per-run entries, prune the oldest entries when
//! the map grows past a cap, validate run-scoped ids, scrub control characters,
//! and fail closed when their lock is poisoned. Those mechanics live here so the
//! security-relevant bounding and fail-closed behavior has one implementation;
//! each registry keeps its own domain types, states, labels, and user-facing
//! messages local and explicit.

use std::collections::{HashMap, VecDeque};
use std::io::Read;
use std::sync::{LockResult, MutexGuard};

use runhaven_core::support::validators::validate_run_id;

/// Cap on the bytes buffered for a single output line before it is flushed to the
/// sink. Matches the fixed 8 KB read window `launch_runner` uses so a captured
/// stream with no newline cannot grow an in-memory line without bound before the
/// sink redacts and truncates it.
const MAX_CAPTURE_LINE_BYTES: usize = 8192;

/// Read `reader` in fixed 8 KB chunks and hand each complete line to `sink`. A
/// run of bytes with no newline is flushed once it reaches
/// [`MAX_CAPTURE_LINE_BYTES`], so a hostile or noisy stream cannot accumulate an
/// unbounded single line. Used by the background runners that stream image-build
/// and sign-in output into the sanitized tail registries.
pub(crate) fn capture_lines<R: Read>(
    mut reader: R,
    mut sink: impl FnMut(&str) -> Result<(), String>,
) -> Result<(), String> {
    let mut chunk = [0_u8; MAX_CAPTURE_LINE_BYTES];
    let mut pending: Vec<u8> = Vec::new();
    loop {
        let read = reader
            .read(&mut chunk)
            .map_err(|error| format!("could not read captured output: {error}"))?;
        if read == 0 {
            if !pending.is_empty() {
                sink(&String::from_utf8_lossy(&pending))?;
            }
            return Ok(());
        }
        for &byte in &chunk[..read] {
            if byte == b'\n' {
                sink(&String::from_utf8_lossy(&pending))?;
                pending.clear();
            } else {
                pending.push(byte);
                if pending.len() >= MAX_CAPTURE_LINE_BYTES {
                    sink(&String::from_utf8_lossy(&pending))?;
                    pending.clear();
                }
            }
        }
    }
}

/// Drop every control character (including terminal escapes) from `text`.
pub(crate) fn strip_control_chars(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_control()).collect()
}

/// Truncate `text` to at most `max_len` bytes without splitting a UTF-8 char.
pub(crate) fn truncate_on_char_boundary(text: &mut String, max_len: usize) {
    if text.len() <= max_len {
        return;
    }
    let mut end = max_len;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    text.truncate(end);
}

/// Validate a run-scoped registry id (run id, build id, or sign-in id) with the
/// shared core validator, mapping its error to the string the commands surface.
pub(crate) fn validate_registry_id(id: &str) -> Result<(), String> {
    validate_run_id(id).map_err(|error| error.to_string())
}

/// Lock a registry mutex, returning the caller's fixed fail-closed message when
/// the lock is poisoned so a panicked writer can never expose partial state.
pub(crate) fn lock_or<'a, T>(
    lock: LockResult<MutexGuard<'a, T>>,
    message: &'static str,
) -> Result<MutexGuard<'a, T>, String> {
    lock.map_err(|_| message.to_string())
}

/// A bounded, insertion-ordered map. Keys keep their first-insertion position in
/// `order` so `prune` can drop the oldest entries once the map exceeds a cap.
pub(crate) struct BoundedMap<V> {
    map: HashMap<String, V>,
    order: VecDeque<String>,
}

impl<V> Default for BoundedMap<V> {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }
}

impl<V> BoundedMap<V> {
    /// Insert `value` at `key`, recording insertion order the first time a key is
    /// seen so `prune` drops the oldest entries first.
    pub(crate) fn insert_tracked(&mut self, key: &str, value: V) {
        if !self.map.contains_key(key) {
            self.order.push_back(key.to_string());
        }
        self.map.insert(key.to_string(), value);
    }

    pub(crate) fn get(&self, key: &str) -> Option<&V> {
        self.map.get(key)
    }

    pub(crate) fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        self.map.get_mut(key)
    }

    /// Drop the oldest-inserted entries until at most `max` remain.
    pub(crate) fn prune(&mut self, max: usize) {
        while self.map.len() > max {
            let Some(key) = self.order.pop_front() else {
                return;
            };
            self.map.remove(&key);
        }
    }
}

impl<V: Default> BoundedMap<V> {
    /// Get the entry for `key`, inserting a default value (and tracking insertion
    /// order) when the key is absent.
    pub(crate) fn entry_tracked(&mut self, key: &str) -> &mut V {
        if !self.map.contains_key(key) {
            self.order.push_back(key.to_string());
        }
        self.map.entry(key.to_string()).or_default()
    }
}

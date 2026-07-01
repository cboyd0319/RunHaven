use std::sync::{Arc, Mutex};

use runhaven_core::runtime::active::{
    MAX_LOG_SNAPSHOT_BYTES, log_snapshot_payload_from_stdout, validate_log_snapshot_lines,
};

use crate::commands::log_snapshot::log_snapshot_response_from_payload;
use crate::contracts::LogSnapshotResponse;
use crate::registry_support::{BoundedMap, lock_or, validate_registry_id};

const REGISTRY_UNAVAILABLE: &str = "session output registry is unavailable";

#[derive(Clone, Default)]
pub(crate) struct SessionOutputRegistry {
    state: Arc<Mutex<BoundedMap<OutputBuffer>>>,
}

#[derive(Clone, Default)]
struct OutputBuffer {
    bytes: Vec<u8>,
    truncated: bool,
}

const MAX_SESSION_OUTPUT_BUFFERS: usize = 20;

impl SessionOutputRegistry {
    /// Pre-create the run's output buffer so a snapshot before any output returns
    /// an empty (not missing) buffer, and prune the oldest buffers. Output is
    /// retained after the run exits so late snapshots still work; there is no
    /// finish/unregister step.
    pub(crate) fn register(&self, run_id: &str) -> Result<(), String> {
        validate_registry_id(run_id)?;
        let mut state = lock_or(self.state.lock(), REGISTRY_UNAVAILABLE)?;
        state.entry_tracked(run_id);
        state.prune(MAX_SESSION_OUTPUT_BUFFERS);
        Ok(())
    }

    pub(crate) fn append(&self, run_id: &str, bytes: &[u8]) -> Result<(), String> {
        validate_registry_id(run_id)?;
        if bytes.is_empty() {
            return Ok(());
        }
        let mut state = lock_or(self.state.lock(), REGISTRY_UNAVAILABLE)?;
        let buffer = state.entry_tracked(run_id);
        buffer.bytes.extend_from_slice(bytes);
        if buffer.bytes.len() > MAX_LOG_SNAPSHOT_BYTES {
            let excess = buffer.bytes.len() - MAX_LOG_SNAPSHOT_BYTES;
            buffer.bytes.drain(0..excess);
            buffer.truncated = true;
        }
        state.prune(MAX_SESSION_OUTPUT_BUFFERS);
        Ok(())
    }

    pub(crate) fn snapshot(
        &self,
        run_id: &str,
        requested_lines: u32,
    ) -> Result<Option<LogSnapshotResponse>, String> {
        validate_registry_id(run_id)?;
        validate_log_snapshot_lines(requested_lines).map_err(|error| error.to_string())?;
        let buffer = {
            let state = lock_or(self.state.lock(), REGISTRY_UNAVAILABLE)?;
            state.get(run_id).cloned()
        };
        let Some(buffer) = buffer else {
            return Ok(None);
        };

        let (tail, line_truncated) =
            tail_lines(&String::from_utf8_lossy(&buffer.bytes), requested_lines);
        let payload = log_snapshot_payload_from_stdout(
            run_id,
            requested_lines,
            tail.as_bytes(),
            MAX_LOG_SNAPSHOT_BYTES,
        )
        .map_err(|error| error.to_string())?;
        let mut response = log_snapshot_response_from_payload(payload)?;
        if buffer.truncated || line_truncated {
            response.truncated = true;
        }
        Ok(Some(response))
    }
}

fn tail_lines(text: &str, requested_lines: u32) -> (String, bool) {
    let limit = requested_lines as usize;
    let lines = text.lines().collect::<Vec<_>>();
    if lines.len() <= limit {
        return (text.to_string(), false);
    }
    (lines[lines.len() - limit..].join("\n"), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_output_registry_returns_bounded_tail() {
        let registry = SessionOutputRegistry::default();
        registry.register("output-registry-test").expect("register");

        registry
            .append("output-registry-test", b"one\ntwo\nthree\n")
            .expect("append");
        let snapshot = registry
            .snapshot("output-registry-test", 2)
            .expect("snapshot")
            .expect("registered output");

        assert_eq!(snapshot.run_id, "output-registry-test");
        assert_eq!(snapshot.text, "two\nthree");
        assert_eq!(snapshot.returned_lines, 2);
        assert!(snapshot.truncated);
    }

    #[test]
    fn session_output_registry_retains_output_after_child_exit() {
        let registry = SessionOutputRegistry::default();
        {
            registry.register("output-registry-drop").expect("register");
            registry
                .append("output-registry-drop", b"hello\n")
                .expect("append");
            assert!(
                registry
                    .snapshot("output-registry-drop", 20)
                    .expect("snapshot")
                    .is_some()
            );
        }

        assert!(
            registry
                .snapshot("output-registry-drop", 20)
                .expect("snapshot")
                .expect("retained output")
                .text
                .contains("hello")
        );
    }

    #[test]
    fn session_output_registry_prunes_old_buffers() {
        let registry = SessionOutputRegistry::default();
        for idx in 0..=MAX_SESSION_OUTPUT_BUFFERS {
            let run_id = format!("output-prune-{idx}");
            registry.register(&run_id).expect("register");
            registry.append(&run_id, run_id.as_bytes()).expect("append");
        }

        assert!(
            registry
                .snapshot("output-prune-0", 20)
                .expect("snapshot")
                .is_none()
        );
        assert!(
            registry
                .snapshot(&format!("output-prune-{MAX_SESSION_OUTPUT_BUFFERS}"), 20)
                .expect("snapshot")
                .expect("newest retained")
                .text
                .contains(&format!("output-prune-{MAX_SESSION_OUTPUT_BUFFERS}"))
        );
    }
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use runhaven_core::runtime::active::{
    MAX_LOG_SNAPSHOT_BYTES, log_snapshot_payload_from_stdout, validate_log_snapshot_lines,
};
use runhaven_core::support::validators::validate_run_id;

use crate::commands::log_snapshot::log_snapshot_response_from_payload;
use crate::contracts::LogSnapshotResponse;

#[derive(Clone, Default)]
pub(crate) struct SessionOutputRegistry {
    buffers: Arc<Mutex<HashMap<String, OutputBuffer>>>,
}

#[derive(Clone, Default)]
struct OutputBuffer {
    bytes: Vec<u8>,
    truncated: bool,
}

impl SessionOutputRegistry {
    pub(crate) fn register(&self, run_id: &str) -> Result<OutputRegistration<'_>, String> {
        validate_run_id(run_id).map_err(|error| error.to_string())?;
        let mut buffers = self
            .buffers
            .lock()
            .map_err(|_| "session output registry is unavailable".to_string())?;
        buffers.entry(run_id.to_string()).or_default();
        Ok(OutputRegistration {
            registry: self,
            run_id: run_id.to_string(),
        })
    }

    pub(crate) fn append(&self, run_id: &str, bytes: &[u8]) -> Result<(), String> {
        validate_run_id(run_id).map_err(|error| error.to_string())?;
        if bytes.is_empty() {
            return Ok(());
        }
        let mut buffers = self
            .buffers
            .lock()
            .map_err(|_| "session output registry is unavailable".to_string())?;
        let buffer = buffers.entry(run_id.to_string()).or_default();
        buffer.bytes.extend_from_slice(bytes);
        if buffer.bytes.len() > MAX_LOG_SNAPSHOT_BYTES {
            let excess = buffer.bytes.len() - MAX_LOG_SNAPSHOT_BYTES;
            buffer.bytes.drain(0..excess);
            buffer.truncated = true;
        }
        Ok(())
    }

    pub(crate) fn snapshot(
        &self,
        run_id: &str,
        requested_lines: u32,
    ) -> Result<Option<LogSnapshotResponse>, String> {
        validate_run_id(run_id).map_err(|error| error.to_string())?;
        validate_log_snapshot_lines(requested_lines).map_err(|error| error.to_string())?;
        let buffer = {
            let buffers = self
                .buffers
                .lock()
                .map_err(|_| "session output registry is unavailable".to_string())?;
            buffers.get(run_id).cloned()
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

    fn unregister(&self, run_id: &str) -> Result<(), String> {
        validate_run_id(run_id).map_err(|error| error.to_string())?;
        let mut buffers = self
            .buffers
            .lock()
            .map_err(|_| "session output registry is unavailable".to_string())?;
        buffers.remove(run_id);
        Ok(())
    }
}

pub(crate) struct OutputRegistration<'a> {
    registry: &'a SessionOutputRegistry,
    run_id: String,
}

impl Drop for OutputRegistration<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.registry.unregister(&self.run_id) {
            eprintln!(
                "runhaven session output unregister {} failed: {error}",
                self.run_id
            );
        }
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
        let _registration = registry.register("output-registry-test").expect("register");

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
    fn session_output_registry_unregisters_output() {
        let registry = SessionOutputRegistry::default();
        {
            let _registration = registry.register("output-registry-drop").expect("register");
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
                .is_none()
        );
    }
}

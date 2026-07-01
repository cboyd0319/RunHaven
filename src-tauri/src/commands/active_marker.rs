use std::io::ErrorKind;

use runhaven_core::support::paths::active_run_path;

/// Shared active-run marker probe for the run-lifecycle commands.
///
/// Returns `Ok(true)` when an active-run marker still exists for `run_id`,
/// `Ok(false)` when it is gone, and `Err(())` when the path cannot be resolved
/// or stat'd. Callers map the outcomes to their own fail-closed messages so no
/// raw host path is ever surfaced to the WebView.
pub(crate) fn active_run_marker_present(run_id: &str) -> Result<bool, ()> {
    let path = active_run_path(run_id).map_err(|_| ())?;
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(_) => Err(()),
    }
}

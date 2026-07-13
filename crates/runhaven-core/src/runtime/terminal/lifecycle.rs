use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

/// One-shot child exit latch shared by every reap path.
pub(super) struct ExitSignal {
    code: Mutex<Option<i32>>,
    ready: Condvar,
}

impl ExitSignal {
    pub(super) fn new() -> Self {
        Self {
            code: Mutex::new(None),
            ready: Condvar::new(),
        }
    }

    pub(super) fn signal(&self, code: i32) {
        let mut guard = self.code.lock().unwrap_or_else(PoisonError::into_inner);
        if guard.is_none() {
            *guard = Some(code);
            self.ready.notify_all();
        }
    }

    pub(super) fn wait(&self) -> i32 {
        let mut guard = self.code.lock().unwrap_or_else(PoisonError::into_inner);
        while guard.is_none() {
            guard = self
                .ready
                .wait(guard)
                .unwrap_or_else(PoisonError::into_inner);
        }
        guard.expect("exit code recorded before wake")
    }
}

/// Outcome latch for provider/container cleanup owned by the detached launch
/// thread. EOF publication and app teardown wait for it only after reaping the
/// child.
pub(super) struct CleanupSignal {
    outcome: Mutex<Option<bool>>,
    ready: Condvar,
}

impl CleanupSignal {
    pub(super) fn new() -> Self {
        Self {
            outcome: Mutex::new(None),
            ready: Condvar::new(),
        }
    }

    fn complete(&self, succeeded: bool) {
        *self.outcome.lock().unwrap_or_else(PoisonError::into_inner) = Some(succeeded);
        self.ready.notify_all();
    }

    pub(super) fn wait(&self) -> bool {
        let mut guard = self.outcome.lock().unwrap_or_else(PoisonError::into_inner);
        while guard.is_none() {
            guard = self
                .ready
                .wait(guard)
                .unwrap_or_else(PoisonError::into_inner);
        }
        guard.expect("cleanup outcome recorded before wake")
    }

    pub(super) fn wait_timeout(&self, timeout: Duration) -> Option<bool> {
        let guard = self.outcome.lock().unwrap_or_else(PoisonError::into_inner);
        if guard.is_some() {
            return *guard;
        }
        let (guard, _) = self
            .ready
            .wait_timeout_while(guard, timeout, |outcome| outcome.is_none())
            .unwrap_or_else(PoisonError::into_inner);
        *guard
    }
}

pub(super) struct CleanupGuard {
    signal: Arc<CleanupSignal>,
    succeeded: bool,
}

impl CleanupGuard {
    pub(super) fn new(signal: Arc<CleanupSignal>) -> Self {
        Self {
            signal,
            succeeded: false,
        }
    }

    pub(super) fn finish(mut self, succeeded: bool) {
        self.succeeded = succeeded;
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        self.signal.complete(self.succeeded);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_guard_reports_success_failure_and_timeout() {
        let signal = Arc::new(CleanupSignal::new());
        let guard = CleanupGuard::new(signal.clone());
        assert_eq!(signal.wait_timeout(Duration::from_millis(1)), None);
        guard.finish(true);
        assert!(signal.wait());
        assert_eq!(signal.wait_timeout(Duration::from_millis(1)), Some(true));

        let signal = Arc::new(CleanupSignal::new());
        let guard = CleanupGuard::new(signal.clone());
        guard.finish(false);
        assert!(!signal.wait());
        assert_eq!(signal.wait_timeout(Duration::from_millis(1)), Some(false));
    }
}

use std::fs::{File, TryLockError};
use std::io::Write;

use anyhow::{Context, Result, bail};

use crate::support::paths::{open_private_read_write, state_lock_path};

pub struct StateLock {
    file: File,
}

impl Drop for StateLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

pub fn acquire_state_lock(state_volume: &str) -> Result<StateLock> {
    let path = state_lock_path(state_volume);
    let mut file = open_private_read_write(&path)
        .with_context(|| format!("could not open state lock: {}", path.display()))?;
    match file.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => bail!(
            "agent state for this workspace is already in use. Wait for the other run to finish, or use a different workspace/profile."
        ),
        Err(TryLockError::Error(error)) => {
            return Err(error).with_context(|| format!("could not lock {}", path.display()));
        }
    }
    file.set_len(0)?;
    writeln!(file, "{}", std::process::id())?;
    file.flush()?;
    Ok(StateLock { file })
}

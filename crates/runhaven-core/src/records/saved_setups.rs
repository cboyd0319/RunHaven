//! Private saved assistant/project setup persistence.

use std::fs::{self, File};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::runtime::profiles::get_profile;
use crate::support::paths::{open_private_read_write, runhaven_cache_root, write_private_atomic};
use crate::support::validators::validate_run_id;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SavedSetupRecord {
    pub id: String,
    pub assistant: String,
    pub project_path: PathBuf,
    pub pinned: bool,
}

struct SavedSetupsLock(File);

impl Drop for SavedSetupsLock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

pub fn saved_setups_path() -> PathBuf {
    runhaven_cache_root().join("saved-setups.json")
}

pub fn save_saved_setup(assistant: &str, project_path: &Path) -> Result<SavedSetupRecord> {
    get_profile(assistant)?;
    let project_path = fs::canonicalize(project_path)?;
    if !project_path.is_dir() {
        bail!("saved setup project is not a directory");
    }

    let _lock = acquire_saved_setups_lock()?;
    let mut records = read_saved_setups()?;
    if let Some(record) = records
        .iter()
        .find(|record| record.assistant == assistant && record.project_path == project_path)
    {
        return Ok(record.clone());
    }

    let record = SavedSetupRecord {
        id: new_saved_setup_id(&records),
        assistant: assistant.to_string(),
        project_path,
        pinned: false,
    };
    records.push(record.clone());
    write_saved_setups(&records)?;
    Ok(record)
}

pub fn read_saved_setups() -> Result<Vec<SavedSetupRecord>> {
    let bytes = match fs::read(saved_setups_path()) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let records: Vec<SavedSetupRecord> = serde_json::from_slice(&bytes)?;
    for record in &records {
        validate_record(record)?;
    }
    Ok(records)
}

pub fn set_saved_setup_pinned(id: &str, pinned: bool) -> Result<()> {
    validate_run_id(id)?;
    let _lock = acquire_saved_setups_lock()?;
    let mut records = read_saved_setups()?;
    let Some(record) = records.iter_mut().find(|record| record.id == id) else {
        bail!("saved setup not found");
    };
    record.pinned = pinned;
    write_saved_setups(&records)
}

fn write_saved_setups(records: &[SavedSetupRecord]) -> Result<()> {
    let bytes = serde_json::to_vec(records)?;
    write_private_atomic(&saved_setups_path(), &bytes)
}

fn acquire_saved_setups_lock() -> Result<SavedSetupsLock> {
    let path = saved_setups_path().with_extension("lock");
    let file = open_private_read_write(&path)
        .with_context(|| format!("could not open saved setup lock: {}", path.display()))?;
    file.lock()
        .with_context(|| format!("could not lock saved setups: {}", path.display()))?;
    Ok(SavedSetupsLock(file))
}

fn new_saved_setup_id(records: &[SavedSetupRecord]) -> String {
    loop {
        let id = format!("setup-{}", uuid::Uuid::new_v4().simple());
        if records.iter().all(|record| record.id != id) {
            return id;
        }
    }
}

fn validate_record(record: &SavedSetupRecord) -> Result<()> {
    validate_run_id(&record.id)?;
    get_profile(&record.assistant)?;
    if !record.project_path.is_absolute() {
        bail!("saved setup project path is invalid");
    }
    Ok(())
}

#[cfg(any(test, feature = "test-support"))]
pub fn saved_setups_path_for_test() -> PathBuf {
    saved_setups_path()
}

#[cfg(any(test, feature = "test-support"))]
pub fn save_saved_setup_for_test(assistant: &str, project_path: &Path) -> Result<SavedSetupRecord> {
    save_saved_setup(assistant, project_path)
}

#[cfg(any(test, feature = "test-support"))]
pub fn read_saved_setups_for_test() -> Result<Vec<SavedSetupRecord>> {
    read_saved_setups()
}

#[cfg(any(test, feature = "test-support"))]
pub fn set_saved_setup_pinned_for_test(id: &str, pinned: bool) -> Result<()> {
    set_saved_setup_pinned(id, pinned)
}

#[cfg(test)]
mod tests;

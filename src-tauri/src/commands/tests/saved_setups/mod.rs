use std::fs;
use std::path::PathBuf;

use crate::commands::saved_setups::{
    list_saved_setups_for_test, pin_saved_setup_for_test, save_saved_setup_for_test,
    select_saved_setup_for_test, set_saved_setup_pinned_for_test,
};

use super::*;

mod availability;
mod pinning;
mod selection;
mod summaries;

fn project_fixture(root: &Path, name: &str) -> PathBuf {
    let project = root.join(name);
    fs::create_dir_all(&project).expect("project directory");
    project
}

fn summary_id(summary: &serde_json::Value) -> &str {
    summary
        .get("id")
        .and_then(serde_json::Value::as_str)
        .expect("saved setup summary id")
}

use std::fs;
use std::path::PathBuf;

use crate::support::paths::{CacheRootOverride, override_cache_root_for_tests};

use super::*;

mod identity;
mod persistence;

fn isolated_cache() -> (tempfile::TempDir, CacheRootOverride) {
    let cache = tempfile::tempdir().expect("cache");
    let cache_home = override_cache_root_for_tests(cache.path());
    (cache, cache_home)
}

fn project_fixture(root: &std::path::Path, name: &str) -> PathBuf {
    let project = root.join(name);
    fs::create_dir_all(&project).expect("project directory");
    project
}

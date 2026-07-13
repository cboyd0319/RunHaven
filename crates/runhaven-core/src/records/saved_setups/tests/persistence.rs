use std::fs;
use std::sync::{Arc, Barrier};

use super::*;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
#[cfg(unix)]
fn saved_setup_record_is_owner_only() {
    let (cache, _cache_home) = isolated_cache();
    let project = project_fixture(cache.path(), "owner-only-project");

    save_saved_setup_for_test("codex", &project).expect("save setup");

    let record_path = saved_setups_path_for_test();
    let record_mode = fs::metadata(&record_path)
        .expect("saved setup record exists")
        .permissions()
        .mode()
        & 0o777;
    let parent_mode = fs::metadata(record_path.parent().expect("record parent"))
        .expect("saved setup parent exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(record_mode, 0o600);
    assert_eq!(parent_mode, 0o700);
}

#[test]
fn saved_setup_record_uses_the_private_atomic_writer() {
    let source = include_str!("../../saved_setups.rs");

    assert!(
        source.contains("write_private_atomic("),
        "saved setup persistence must replace one private record atomically"
    );
    for forbidden in ["fs::write(", "File::create(", "OpenOptions::new("] {
        assert!(
            !source.contains(forbidden),
            "saved setup persistence bypassed the private atomic writer with {forbidden}"
        );
    }
}

#[test]
fn concurrent_saved_setup_writes_do_not_lose_records() {
    let (cache, _cache_home) = isolated_cache();
    let projects = (0..8)
        .map(|index| project_fixture(cache.path(), &format!("project-{index}")))
        .collect::<Vec<_>>();
    let barrier = Arc::new(Barrier::new(projects.len()));
    let writers = projects
        .into_iter()
        .map(|project| {
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                save_saved_setup_for_test("codex", &project).expect("save setup")
            })
        })
        .collect::<Vec<_>>();

    for writer in writers {
        writer.join().expect("saved setup writer");
    }

    assert_eq!(
        read_saved_setups_for_test()
            .expect("read saved setups")
            .len(),
        8,
    );
}

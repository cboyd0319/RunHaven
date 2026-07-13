use super::*;

pub(super) fn internal_test_plan(workspace: &Path) -> AgentRunPlan {
    let mut plan = test_plan(workspace);
    plan.network_mode = NetworkMode::Internal;
    plan.network_name = Some("runhaven-test-internal".to_string());
    plan.preflight = vec![internal_network_create("runhaven-test-internal")];
    plan
}

fn internal_network_create(name: &str) -> Vec<String> {
    ["container", "network", "create", "--internal", name]
        .into_iter()
        .map(str::to_string)
        .collect()
}

#[test]
fn standard_internal_launch_cleans_its_per_run_network_after_runner_failure() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let plan = internal_test_plan(workspace.path());
    let mut runner = FailingRunner {
        expected_run_id: "runner-seam".to_string(),
        saw_active_marker: false,
    };
    let cleanup_calls = std::cell::Cell::new(0);

    let error = run_standard_launch_with_ops(
        &plan,
        &mut runner,
        |_| Ok(()),
        |cleaned_plan| {
            cleanup_calls.set(cleanup_calls.get() + 1);
            assert_eq!(cleaned_plan.network_name, plan.network_name);
            Ok(json!({
                "provider_network": "deleted",
                "provider_network_name": "runhaven-test-internal",
            }))
        },
    )
    .expect_err("runner failure");

    assert!(error.to_string().contains("backend runner failed"));
    assert_eq!(cleanup_calls.get(), 1);
    let record = crate::records::find_run_record("runner-seam").expect("run record");
    assert_eq!(record["cleanup"]["provider_network"], "deleted");
}

#[test]
fn standard_internal_launch_cleans_its_per_run_network_after_success() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let plan = internal_test_plan(workspace.path());
    let mut runner = RecordingRunner {
        expected_run_id: "runner-seam".to_string(),
        command: None,
        saw_active_marker: false,
        active_marker_auth_scope: None,
    };
    let cleanup_calls = std::cell::Cell::new(0);

    let code = run_standard_launch_with_ops(
        &plan,
        &mut runner,
        |_| Ok(()),
        |_| {
            cleanup_calls.set(cleanup_calls.get() + 1);
            Ok(json!({"provider_network": "deleted"}))
        },
    )
    .expect("standard launch");

    assert_eq!(code, 0);
    assert_eq!(cleanup_calls.get(), 1);
    let record = crate::records::find_run_record("runner-seam").expect("run record");
    assert_eq!(record["cleanup"]["provider_network"], "deleted");
}

#[test]
fn standard_preflight_failure_before_run_network_does_not_clean_or_launch() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = internal_test_plan(workspace.path());
    plan.preflight.insert(0, vec!["setup".to_string()]);
    let mut runner = CountingRunner::default();
    let cleanup_calls = std::cell::Cell::new(0);

    let error = run_standard_launch_with_ops(
        &plan,
        &mut runner,
        |_| Err(anyhow::anyhow!("preflight failed")),
        |_| {
            cleanup_calls.set(cleanup_calls.get() + 1);
            Ok(json!({"provider_network": "deleted"}))
        },
    )
    .expect_err("preflight failure");

    assert!(error.to_string().contains("preflight failed"));
    assert_eq!(cleanup_calls.get(), 0);
    assert_eq!(runner.calls, 0);
}

#[test]
fn standard_preflight_failure_after_run_network_cleans_without_masking_error() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = internal_test_plan(workspace.path());
    plan.preflight.push(vec!["later-setup".to_string()]);
    let mut runner = CountingRunner::default();
    let preflight_calls = std::cell::Cell::new(0);
    let cleanup_calls = std::cell::Cell::new(0);

    let error = run_standard_launch_with_ops(
        &plan,
        &mut runner,
        |_| {
            preflight_calls.set(preflight_calls.get() + 1);
            if preflight_calls.get() == 1 {
                Ok(())
            } else {
                Err(anyhow::anyhow!("later preflight failed"))
            }
        },
        |_| {
            cleanup_calls.set(cleanup_calls.get() + 1);
            Err(anyhow::anyhow!("cleanup transport failed"))
        },
    )
    .expect_err("later preflight failure");

    assert!(error.to_string().contains("later preflight failed"));
    assert_eq!(cleanup_calls.get(), 1);
    assert_eq!(runner.calls, 0);
}

#[test]
fn standard_internal_cleanup_error_is_recorded_without_failing_a_good_run() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let plan = internal_test_plan(workspace.path());
    let mut runner = RecordingRunner {
        expected_run_id: "runner-seam".to_string(),
        command: None,
        saw_active_marker: false,
        active_marker_auth_scope: None,
    };

    let code = run_standard_launch_with_ops(
        &plan,
        &mut runner,
        |_| Ok(()),
        |_| Err(anyhow::anyhow!("cleanup transport failed")),
    )
    .expect("agent result wins");

    assert_eq!(code, 0);
    let record = crate::records::find_run_record("runner-seam").expect("run record");
    assert_eq!(record["cleanup"]["provider_network"], "cleanup-error");
}

#[test]
fn standard_internet_launch_cleans_its_container_without_a_network() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let plan = test_plan(workspace.path());
    let mut runner = RecordingRunner {
        expected_run_id: "runner-seam".to_string(),
        command: None,
        saw_active_marker: false,
        active_marker_auth_scope: None,
    };

    let cleanup_calls = std::cell::Cell::new(0);
    let code = run_standard_launch_with_ops(
        &plan,
        &mut runner,
        |_| Ok(()),
        |cleaned_plan| {
            cleanup_calls.set(cleanup_calls.get() + 1);
            assert!(cleaned_plan.network_name.is_none());
            Ok(json!({
                "provider_network": "not-created",
                "run_container": "removed",
            }))
        },
    )
    .expect("internet launch");

    assert_eq!(code, 0);
    assert_eq!(cleanup_calls.get(), 1);
    let record = crate::records::find_run_record("runner-seam").expect("run record");
    assert_eq!(record["cleanup"]["provider_network"], "not-created");
    assert_eq!(record["cleanup"]["run_container"], "removed");
}

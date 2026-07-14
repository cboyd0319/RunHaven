use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::support_report::{
    OPEN_PROGRAM, fixed_report_for_test, perform_support_report_for_test,
};
use runhaven_core::support::report::SUPPORT_ISSUE_URL;

#[test]
fn confirmation_explains_the_privacy_and_submission_boundary() {
    let (title, button, message) =
        crate::commands::support_report::confirmation::confirmation_copy_for_test();

    assert_eq!(title, "Create a support report?");
    assert_eq!(button, "Create report");
    assert!(message.contains("Nothing is uploaded or submitted automatically"));
    assert!(message.contains("public GitHub support form"));
    assert!(message.contains("GitHub uploads it before you submit"));
    assert!(message.contains("raw logs, terminal output, prompts"));
    assert!(message.contains("Review the file before you attach it"));
}

#[test]
fn cancel_reads_nothing_writes_nothing_and_opens_nothing() {
    let mut created = false;
    let mut opened = false;
    let result = perform_support_report_for_test(
        "0.7.0",
        Path::new("/must/not/be/read"),
        Path::new("/must/not/be/read"),
        || Ok(false),
        |_, _| {
            created = true;
            anyhow::bail!("must not create")
        },
        |_, _| {
            opened = true;
            Ok(true)
        },
    )
    .expect("quiet cancellation");

    assert_eq!(result, None);
    assert!(!created);
    assert!(!opened);
}

#[test]
fn confirmed_report_uses_only_fixed_exact_process_arguments() {
    let root = tempfile::tempdir().expect("root");
    let report = root
        .path()
        .join("Support Reports")
        .join("RunHaven report 'one'.txt");
    fs::create_dir_all(report.parent().expect("parent")).expect("report dir");
    fs::write(&report, "safe report").expect("report");
    let mut commands: Vec<(PathBuf, Vec<OsString>)> = Vec::new();

    let response = perform_support_report_for_test(
        "0.7.0",
        report.parent().expect("parent"),
        root.path(),
        || Ok(true),
        |request, output_dir| {
            assert_eq!(request.app_version, "0.7.0");
            assert_eq!(output_dir, report.parent().expect("parent"));
            assert_eq!(
                request.app_log.expect("app log").path,
                root.path().join("runhaven.log")
            );
            Ok(fixed_report_for_test(report.clone()))
        },
        |program, args| {
            commands.push((program.to_path_buf(), args.to_vec()));
            Ok(true)
        },
    )
    .expect("support action")
    .expect("confirmed response");

    assert_eq!(response.file_name, "RunHaven report 'one'.txt");
    assert!(response.issue_opened);
    assert!(response.finder_revealed);
    assert_eq!(
        commands,
        vec![
            (
                PathBuf::from(OPEN_PROGRAM),
                vec![OsString::from("-u"), OsString::from(SUPPORT_ISSUE_URL)],
            ),
            (
                PathBuf::from(OPEN_PROGRAM),
                vec![OsString::from("-R"), report.into_os_string()],
            ),
        ]
    );
}

#[test]
fn report_creation_failure_opens_neither_finder_nor_github() {
    let mut opened = 0;
    let error = perform_support_report_for_test(
        "0.7.0",
        Path::new("/private/output"),
        Path::new("/private/log"),
        || Ok(true),
        |_, _| anyhow::bail!("poisoned raw filesystem error {PRIVATE_SENTINEL}"),
        |_, _| {
            opened += 1;
            Ok(true)
        },
    )
    .expect_err("creation must fail");

    assert_eq!(opened, 0);
    assert_eq!(
        error,
        "RunHaven could not create the support report. No apps were opened."
    );
    assert!(!error.contains(PRIVATE_SENTINEL));
}

#[test]
fn report_outside_the_verified_directory_opens_nothing() {
    let output = tempfile::tempdir().expect("output");
    let outside = tempfile::tempdir().expect("outside");
    let report = outside.path().join("RunHaven-support.txt");
    fs::write(&report, "safe").expect("report");
    let mut opened = 0;

    let error = perform_support_report_for_test(
        "0.7.0",
        output.path(),
        output.path(),
        || Ok(true),
        |_, _| Ok(fixed_report_for_test(report.clone())),
        |_, _| {
            opened += 1;
            Ok(true)
        },
    )
    .expect_err("outside report must fail");

    assert_eq!(opened, 0);
    assert_eq!(
        error,
        "RunHaven could not verify the support report. No apps were opened."
    );
}

#[test]
fn independent_open_failures_preserve_the_report_and_return_fixed_flags() {
    let root = tempfile::tempdir().expect("root");
    let report = root.path().join("RunHaven-support.txt");
    fs::write(&report, "safe").expect("report");
    let mut call = 0;
    let response = perform_support_report_for_test(
        "0.7.0",
        root.path(),
        root.path(),
        || Ok(true),
        |_, _| Ok(fixed_report_for_test(report.clone())),
        |_, _| {
            call += 1;
            if call == 1 {
                Err(std::io::Error::other("browser failed with private path"))
            } else {
                Ok(true)
            }
        },
    )
    .expect("partial success")
    .expect("response");

    assert_eq!(call, 2);
    assert!(!response.issue_opened);
    assert!(response.finder_revealed);
    assert!(report.exists());
    let serialized = serde_json::to_string(&response).expect("response JSON");
    assert!(!serialized.contains(root.path().to_string_lossy().as_ref()));
    assert!(!serialized.contains("safe report"));
}

const PRIVATE_SENTINEL: &str = "PRIVATE-SENTINEL";

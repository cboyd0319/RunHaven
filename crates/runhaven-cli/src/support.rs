use anyhow::Result;
use runhaven_core::support::report::{
    SUPPORT_ISSUE_URL, SupportReportRequest, create_support_report, default_support_reports_dir,
};
use time::OffsetDateTime;

pub(crate) fn support() -> Result<i32> {
    let output_dir = default_support_reports_dir()?;
    let report = create_support_report(
        SupportReportRequest {
            app_version: env!("CARGO_PKG_VERSION"),
            created_at: OffsetDateTime::now_utc(),
            app_log: None,
        },
        &output_dir,
    )?;
    println!("Support report created: {}", report.path.display());
    println!("Nothing was uploaded or submitted.");
    println!("Review the report, then attach it to this GitHub support form:");
    println!("{SUPPORT_ISSUE_URL}");
    Ok(0)
}

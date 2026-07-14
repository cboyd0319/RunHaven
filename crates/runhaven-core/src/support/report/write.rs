use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result, bail};
use time::OffsetDateTime;

use super::{CreatedSupportReport, MAX_REPORT_BYTES};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

pub(super) fn write_report(
    output_dir: &Path,
    app_version: &str,
    created_at: OffsetDateTime,
    report: &str,
) -> Result<CreatedSupportReport> {
    if report.len() > MAX_REPORT_BYTES {
        bail!("support report exceeded its size limit")
    }
    ensure_private_report_dir(output_dir)?;
    let version = file_component(app_version.split_whitespace().next().unwrap_or("unknown"));
    let timestamp = format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        created_at.year(),
        u8::from(created_at.month()),
        created_at.day(),
        created_at.hour(),
        created_at.minute(),
        created_at.second(),
    );
    let base = format!("RunHaven-{version}-support-{timestamp}");
    create_unique_report(output_dir, &base, report)
}

pub(super) fn prepare_default_support_reports_dir(data_dir: &Path) -> Result<std::path::PathBuf> {
    fs::create_dir_all(data_dir).context("could not create Application Support directory")?;
    let app_dir = data_dir.join("RunHaven");
    ensure_private_directory(&app_dir, "support data directory")?;
    Ok(app_dir.join("Support Reports"))
}

fn ensure_private_report_dir(path: &Path) -> Result<()> {
    ensure_private_directory(path, "support report directory")
}

fn ensure_private_directory(path: &Path, label: &str) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            bail!("{label} is not a private directory")
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(path).with_context(|| format!("could not create {label}"))?;
            let metadata = fs::symlink_metadata(path)?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                bail!("{label} is not a private directory")
            }
        }
        Err(error) => return Err(error).with_context(|| format!("could not inspect {label}")),
    }
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn create_unique_report(
    output_dir: &Path,
    base: &str,
    report: &str,
) -> Result<CreatedSupportReport> {
    for suffix in 1..=1000usize {
        let file_name = if suffix == 1 {
            format!("{base}.txt")
        } else {
            format!("{base}-{suffix}.txt")
        };
        let path = output_dir.join(&file_name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        match options.open(&path) {
            Ok(mut file) => {
                let write_result = (|| -> std::io::Result<()> {
                    #[cfg(unix)]
                    file.set_permissions(fs::Permissions::from_mode(0o600))?;
                    file.write_all(report.as_bytes())?;
                    file.sync_all()
                })();
                if let Err(error) = write_result {
                    let _ = fs::remove_file(&path);
                    return Err(error).context("could not write support report");
                }
                if let Err(error) = verify_report_file(&path) {
                    let _ = fs::remove_file(&path);
                    return Err(error);
                }
                return Ok(CreatedSupportReport { path, file_name });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error).context("could not create support report"),
        }
    }
    bail!("too many support reports have the same timestamp")
}

fn verify_report_file(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("support report is not a regular file")
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o600 {
        bail!("support report permissions are not private")
    }
    Ok(())
}

fn file_component(value: &str) -> String {
    let component = value
        .chars()
        .take(32)
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
        .collect::<String>();
    if component.is_empty() {
        "unknown".to_string()
    } else {
        component
    }
}

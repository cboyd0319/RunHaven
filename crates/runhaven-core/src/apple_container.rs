//! Resolution and process construction for Apple's `container` CLI.
//!
//! Finder-launched macOS apps do not reliably inherit `/usr/local/bin` in
//! `PATH`, but Apple's signed installer puts `container` there. Prefer that
//! reviewed location on every call. Do not fall back to `PATH`: that would let
//! an untrusted binary bypass the ownership and permission check. Plans keep
//! the stable literal `container`; only the process runner substitutes the
//! resolved executable.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Result, bail};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

pub const INSTALLED_PROGRAM: &str = "/usr/local/bin/container";
pub const INSTALLER_URL: &str = "https://github.com/apple/container/releases/download/1.1.0/container-1.1.0-installer-signed.pkg";

/// Resolve the CLI without caching, so installing it while RunHaven is open is
/// visible to the next setup refresh.
pub fn resolve_program() -> Option<PathBuf> {
    let installed = Path::new(INSTALLED_PROGRAM);
    select_program(installed, trusted_installed_program(installed))
}

pub fn command() -> Result<Command> {
    Ok(Command::new(program()?))
}

pub fn program() -> Result<PathBuf> {
    resolve_program().ok_or_else(|| {
        anyhow::anyhow!(
            "Apple container is missing or unsafe at {INSTALLED_PROGRAM}; reinstall Apple's signed package"
        )
    })
}

/// Preserve plan and display contracts while resolving the Apple CLI at the
/// last possible moment before process creation.
pub fn command_for_program(program: &str) -> Result<Command> {
    Ok(Command::new(program_for_program(program)?))
}

pub fn program_for_program(requested: &str) -> Result<PathBuf> {
    program_for_program_with_resolution(requested, resolve_program())
}

fn program_for_program_with_resolution(
    requested: &str,
    resolved_container: Option<PathBuf>,
) -> Result<PathBuf> {
    if requested == "container" || requested == INSTALLED_PROGRAM {
        let Some(program) = resolved_container else {
            bail!(
                "Apple container is missing or unsafe at {INSTALLED_PROGRAM}; reinstall Apple's signed package"
            );
        };
        return Ok(program);
    }
    Ok(PathBuf::from(requested))
}

fn select_program(installed: &Path, installed_is_trusted: bool) -> Option<PathBuf> {
    installed_is_trusted.then(|| installed.to_path_buf())
}

fn trusted_installed_program(path: &Path) -> bool {
    let Ok(metadata) = path.symlink_metadata() else {
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        let mode = metadata.permissions().mode();
        metadata.uid() == 0
            && mode & 0o111 != 0
            && mode & 0o022 == 0
            && path
                .parent()
                .and_then(|bin| bin.parent().map(|local| [bin, local]))
                .is_some_and(|parents| parents.into_iter().all(trusted_install_directory))
    }
    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(unix)]
fn trusted_install_directory(path: &Path) -> bool {
    let Ok(metadata) = path.symlink_metadata() else {
        return false;
    };
    let mode = metadata.permissions().mode();
    metadata.is_dir()
        && !metadata.file_type().is_symlink()
        && metadata.uid() == 0
        && mode & 0o022 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_installer_location_wins_when_finder_path_omits_it() {
        assert_eq!(
            select_program(Path::new(INSTALLED_PROGRAM), true),
            Some(PathBuf::from(INSTALLED_PROGRAM))
        );
    }

    #[test]
    fn untrusted_official_location_cannot_be_rediscovered_through_path() {
        assert_eq!(select_program(Path::new(INSTALLED_PROGRAM), false), None);
    }

    #[test]
    fn missing_or_untrusted_container_fails_closed_before_process_creation() {
        for requested in ["container", INSTALLED_PROGRAM] {
            let error = program_for_program_with_resolution(requested, None)
                .expect_err("an unresolved container executable must fail closed");

            assert!(error.to_string().contains("missing or unsafe"));
        }
    }

    #[test]
    fn trusted_container_resolution_is_the_only_container_program_returned() {
        let trusted = PathBuf::from(INSTALLED_PROGRAM);

        assert_eq!(
            program_for_program_with_resolution("container", Some(trusted.clone()))
                .expect("trusted container"),
            trusted
        );
    }

    #[test]
    fn non_container_programs_are_not_rewritten() {
        assert_eq!(
            command_for_program("/usr/bin/git")
                .expect("non-container command")
                .get_program(),
            std::ffi::OsStr::new("/usr/bin/git")
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_at_installer_location_is_not_trusted() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().expect("tempdir");
        let link = temp.path().join("container");
        symlink("/bin/sh", &link).expect("symlink");

        assert!(!trusted_installed_program(&link));
    }
}

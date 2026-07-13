use std::thread::sleep;
use std::time::Duration;

use anyhow::{Result, bail};

use crate::provider::agent_sign_in_record::{AGENT_SIGN_IN_SCOPE, record_agent_sign_out};
use crate::runtime::lock::acquire_state_lock;
use crate::runtime::profiles::agent_profiles;
use crate::runtime::session_state::{
    shared_state_volume_name, validate_session_name, volume_matches_session,
};

pub fn state_list(session: Option<&str>) -> Result<i32> {
    let volumes = list_state_volumes(session)?;
    if volumes.is_empty() {
        if let Some(session) = session {
            println!("No RunHaven state volumes found for session {session}.");
        } else {
            println!("No RunHaven state volumes found.");
        }
        return Ok(0);
    }
    for volume in volumes {
        println!("{volume}");
    }
    Ok(0)
}

pub fn state_prune(confirm: bool, session: Option<&str>) -> Result<i32> {
    let volumes = list_state_volumes(session)?;
    if volumes.is_empty() {
        if let Some(session) = session {
            println!("No RunHaven state volumes found for session {session}.");
        } else {
            println!("No RunHaven state volumes found.");
        }
        return Ok(0);
    }
    if !confirm {
        for volume in volumes {
            println!("{volume}");
        }
        println!("Rerun with --yes to delete these volumes.");
        return Ok(2);
    }
    prune_listed_volumes(volumes)
}

fn prune_listed_volumes(volumes: Vec<String>) -> Result<i32> {
    prune_listed_volumes_with(volumes, delete_volume, |agent| {
        record_agent_sign_out(agent, AGENT_SIGN_IN_SCOPE)
    })
}

fn prune_listed_volumes_with(
    volumes: Vec<String>,
    mut delete: impl FnMut(&str) -> Result<VolumeDeletion>,
    mut sign_out: impl FnMut(&str) -> Result<()>,
) -> Result<i32> {
    for volume in volumes {
        // A live run/login owns this same per-volume lock. Hold it through both
        // deletion and the matching sign-out marker so prune cannot erase state
        // while the agent is writing credentials or publish a premature sign-out.
        let _state_lock = acquire_state_lock(&volume)?;
        match delete(&volume)? {
            VolumeDeletion::Deleted | VolumeDeletion::Missing => {
                if let Some(agent) = shared_login_agent(&volume) {
                    sign_out(agent)?;
                }
            }
            VolumeDeletion::Failed => return Ok(1),
        }
    }
    Ok(0)
}

fn shared_login_agent(volume: &str) -> Option<&'static str> {
    agent_profiles()
        .into_iter()
        .find(|profile| shared_state_volume_name(profile.name) == volume)
        .map(|profile| profile.name)
}

/// Result of attempting to delete a RunHaven volume.
pub enum VolumeDeletion {
    /// The volume existed and was deleted.
    Deleted,
    /// The volume did not exist, so there was nothing to delete.
    Missing,
    /// The volume exists but could not be deleted (its error was printed).
    Failed,
}

/// Delete a RunHaven volume. A volume that does not exist is reported `Missing`
/// (resetting or pruning state that was never created is not an error). An
/// existing volume can stay briefly held after its container is stopped or
/// killed (Apple container does not auto-remove the container), so deletion
/// retries across a short window before reporting `Failed`.
pub fn delete_volume(volume: &str) -> Result<VolumeDeletion> {
    if !volume_exists(volume)? {
        return Ok(VolumeDeletion::Missing);
    }
    const ATTEMPTS: u32 = 6;
    const DELAY: Duration = Duration::from_secs(1);
    let mut last_stderr = String::new();
    for attempt in 0..ATTEMPTS {
        let output = crate::apple_container::command()?
            .args(["volume", "delete", volume])
            .output()?;
        if output.status.success() {
            return Ok(VolumeDeletion::Deleted);
        }
        last_stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        // It may have been released and reaped between the check and now.
        if !volume_exists(volume)? {
            return Ok(VolumeDeletion::Missing);
        }
        if attempt + 1 < ATTEMPTS {
            sleep(DELAY);
        }
    }
    if !last_stderr.is_empty() {
        eprintln!("{last_stderr}");
    }
    Ok(VolumeDeletion::Failed)
}

/// Delete one state volume while holding the same ownership lock used by live
/// runs and sign-in. This is the mutation entry point for targeted reset.
pub fn delete_volume_with_state_lock(volume: &str) -> Result<VolumeDeletion> {
    delete_volume_with_state_lock_with(volume, delete_volume)
}

fn delete_volume_with_state_lock_with(
    volume: &str,
    delete: impl FnOnce(&str) -> Result<VolumeDeletion>,
) -> Result<VolumeDeletion> {
    let _state_lock = acquire_state_lock(volume)?;
    delete(volume)
}

fn volume_exists(volume: &str) -> Result<bool> {
    let output = crate::apple_container::command()?
        .args(["volume", "list", "--quiet"])
        .output()?;
    if !output.status.success() {
        // If the list fails, do not assume the volume is gone; let delete try.
        return Ok(true);
    }
    Ok(volume_in_list(
        &String::from_utf8_lossy(&output.stdout),
        volume,
    ))
}

fn volume_in_list(stdout: &str, volume: &str) -> bool {
    stdout.lines().any(|line| line.trim() == volume)
}

pub fn list_state_volumes(session: Option<&str>) -> Result<Vec<String>> {
    if let Some(session) = session {
        validate_session_name(session)?;
    }
    let output = crate::apple_container::command()?
        .args(["volume", "list", "--quiet"])
        .output()?;
    if !output.status.success() {
        bail!("container volume list failed: {}", output.status);
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| volume_matches_session(line, session).unwrap_or(false))
        .map(str::to_string)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_in_list_matches_whole_lines_only() {
        let stdout = "runhaven-shell-shared-home\nrunhaven-shell-abc-s-sess-home\n";
        assert!(volume_in_list(stdout, "runhaven-shell-shared-home"));
        assert!(volume_in_list(stdout, "runhaven-shell-abc-s-sess-home"));
        // A prefix or substring must not count as present.
        assert!(!volume_in_list(stdout, "runhaven-shell-abc"));
        assert!(!volume_in_list(stdout, "shared-home"));
        assert!(!volume_in_list("", "runhaven-shell-shared-home"));
    }

    #[test]
    fn shared_login_volume_detection_covers_only_product_agent_shared_state() {
        for agent in ["antigravity", "claude", "codex", "copilot"] {
            assert_eq!(
                shared_login_agent(&shared_state_volume_name(agent)),
                Some(agent),
            );
        }
        assert_eq!(shared_login_agent("runhaven-shell-shared-home"), None);
        assert_eq!(shared_login_agent("runhaven-codex-project-home"), None);
    }

    #[test]
    fn state_prune_refuses_delete_and_sign_out_while_the_volume_is_locked() {
        use std::cell::Cell;

        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = crate::support::paths::override_cache_root_for_tests(cache.path());
        let volume = shared_state_volume_name("codex");
        let _held = acquire_state_lock(&volume).expect("hold state lock");
        let deleted = Cell::new(false);
        let signed_out = Cell::new(false);

        let error = prune_listed_volumes_with(
            vec![volume],
            |_| {
                deleted.set(true);
                Ok(VolumeDeletion::Deleted)
            },
            |_| {
                signed_out.set(true);
                Ok(())
            },
        )
        .expect_err("a busy state volume must not be pruned");

        assert!(error.to_string().contains("already in use"));
        assert!(!deleted.get(), "delete must stay behind the state lock");
        assert!(
            !signed_out.get(),
            "sign-out must stay behind the state lock"
        );
    }

    #[test]
    fn state_prune_holds_the_lock_through_sign_out_then_releases_it() {
        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = crate::support::paths::override_cache_root_for_tests(cache.path());
        let volume = shared_state_volume_name("codex");
        let sign_out_volume = volume.clone();

        assert_eq!(
            prune_listed_volumes_with(
                vec![volume.clone()],
                |_| Ok(VolumeDeletion::Deleted),
                move |_| {
                    assert!(
                        acquire_state_lock(&sign_out_volume).is_err(),
                        "the sign-out marker must be written before releasing the volume lock"
                    );
                    Ok(())
                },
            )
            .expect("prune"),
            0
        );

        acquire_state_lock(&volume).expect("prune releases the lock when complete");
    }

    #[test]
    fn targeted_state_delete_never_starts_while_the_volume_is_locked() {
        use std::cell::Cell;

        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = crate::support::paths::override_cache_root_for_tests(cache.path());
        let volume = "runhaven-codex-reset-home";
        let _held = acquire_state_lock(volume).expect("hold state lock");
        let delete_started = Cell::new(false);

        let error = match delete_volume_with_state_lock_with(volume, |_| {
            delete_started.set(true);
            Ok(VolumeDeletion::Deleted)
        }) {
            Ok(_) => panic!("a busy state volume must not be reset"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("already in use"));
        assert!(!delete_started.get(), "delete must remain behind the lock");
    }
}

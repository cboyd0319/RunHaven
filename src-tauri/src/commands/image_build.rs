//! Mutating agent-image rebuild command.
//!
//! Rebuilding is recovery for a stale or missing agent image. It reuses the
//! tested core `ImageBuildPlan` (no docker/container build logic lives here),
//! fails closed on unconfirmed or invalid input, and requires a Rust-owned
//! native confirmation naming the agent and effect before any build runs. The
//! build itself runs on a background thread; the command returns a build id the
//! WebView polls through the read-only `get_image_build_status`.

use runhaven_core::doctor::require_supported_runtime;
use runhaven_core::image::build::{ImageBuildPlan, build_image_plan};
use runhaven_core::runtime::launch::new_run_id;
use runhaven_core::runtime::profiles::get_profile;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;

use super::dialog::confirm_native;
use super::validation::{MAX_AGENT_NAME_LEN, validate_text_len};
use crate::background_jobs::{BackgroundJobRegistry, JobClaim, image_build_key};
use crate::contracts::{ImageBuildRequest, ImageBuildStartResponse};
use crate::image_build_runner::spawn_image_build;
use crate::tail_status::ImageBuildStatusRegistry;

const REBUILD_CANCELED: &str = "Agent installation canceled before it started.";

#[tauri::command]
pub(crate) async fn rebuild_agent_image(
    app: AppHandle,
    request: ImageBuildRequest,
    image_build_status: State<'_, ImageBuildStatusRegistry>,
    background_jobs: State<'_, BackgroundJobRegistry>,
) -> Result<ImageBuildStartResponse, String> {
    let registry = image_build_status.inner().clone();
    let prepared = authorize_image_rebuild(&request)?;
    if let Some(response) = existing_image_rebuild(&prepared, &registry, &background_jobs)? {
        return Ok(response);
    }
    if !trusted_image_rebuild_confirmed(app, &prepared.agent, &prepared.plan.tag).await? {
        return Err(REBUILD_CANCELED.to_string());
    }
    start_image_rebuild(prepared, &registry, &background_jobs)
}

/// A validated, ownership-checked rebuild ready for the confirmation dialog.
/// Building this does not start any build, so a denial or failure after this
/// point leaves the image untouched.
struct PreparedRebuild {
    agent: String,
    plan: ImageBuildPlan,
}

/// Validate intent and input before anything is shown or built. Fails closed on
/// an unconfirmed request, an over-long or unknown agent, a missing template, or
/// an unavailable container CLI. The canonical `profile.name` is used everywhere
/// downstream so arbitrary WebView input is never reflected back.
fn authorize_image_rebuild(request: &ImageBuildRequest) -> Result<PreparedRebuild, String> {
    if !request.confirm {
        return Err("Confirm before RunHaven prepares this assistant.".to_string());
    }
    validate_text_len("agent", &request.agent, MAX_AGENT_NAME_LEN)?;
    let profile =
        get_profile(&request.agent).map_err(|_| "Choose a supported assistant.".to_string())?;
    // build_image_plan surfaces host paths and template names on error, so it is
    // collapsed to a fixed, path-free message.
    let plan = build_image_plan(&profile, None)
        .map_err(|_| "RunHaven can’t prepare this assistant automatically.".to_string())?;
    Ok(PreparedRebuild {
        agent: profile.name.to_string(),
        plan,
    })
}

fn start_image_rebuild(
    prepared: PreparedRebuild,
    registry: &ImageBuildStatusRegistry,
    jobs: &BackgroundJobRegistry,
) -> Result<ImageBuildStartResponse, String> {
    // The container CLI is the last gate before the mutation, re-checked after
    // confirmation so a denied or canceled rebuild never touches the builder.
    require_supported_runtime().map_err(|_| {
        "RunHaven setup is not ready for image rebuilds. Open System details and fix the listed issue."
            .to_string()
    })?;
    let build_id = new_run_id();
    let tag = prepared.plan.tag.clone();
    let lease = match jobs.claim(&image_build_key(&tag), &build_id)? {
        JobClaim::Started(lease) => lease,
        JobClaim::Existing(existing_id) => return image_build_response(registry, &existing_id),
    };
    registry.mark_building(&build_id, &prepared.agent, &tag)?;
    if let Err(error) = spawn_image_build(build_id.clone(), prepared.plan, registry.clone(), lease)
    {
        let _ = registry.mark_failed(
            &build_id,
            "RunHaven could not start the rebuild.".to_string(),
        );
        return Err(error);
    }
    Ok(ImageBuildStartResponse {
        build_id,
        agent: prepared.agent,
        tag,
        state: "building".to_string(),
        label: "Installing".to_string(),
        detail: "RunHaven started preparing this assistant.".to_string(),
    })
}

fn existing_image_rebuild(
    prepared: &PreparedRebuild,
    registry: &ImageBuildStatusRegistry,
    jobs: &BackgroundJobRegistry,
) -> Result<Option<ImageBuildStartResponse>, String> {
    jobs.active_id(&image_build_key(&prepared.plan.tag))?
        .map(|id| image_build_response(registry, &id))
        .transpose()
}

fn image_build_response(
    registry: &ImageBuildStatusRegistry,
    build_id: &str,
) -> Result<ImageBuildStartResponse, String> {
    let status = registry.get(build_id)?;
    Ok(ImageBuildStartResponse {
        build_id: status.build_id,
        agent: status.agent,
        tag: status.tag,
        state: status.state,
        label: status.label,
        detail: "RunHaven is already preparing this assistant. Showing its current progress."
            .to_string(),
    })
}

async fn trusted_image_rebuild_confirmed(
    app: AppHandle,
    agent: &str,
    _tag: &str,
) -> Result<bool, String> {
    let label = match agent {
        "claude" => "Claude",
        "codex" => "Codex",
        "antigravity" => "Antigravity",
        "copilot" => "Copilot",
        other => other,
    };
    confirm_native(
        app,
        &format!("Install {label}?"),
        "Install",
        MessageDialogKind::Warning,
        image_install_confirmation_message(label),
    )
    .await
}

fn image_install_confirmation_message(agent: &str) -> String {
    format!(
        "RunHaven will prepare a fresh {agent} environment. Existing sessions will keep running. New sessions will use the new copy."
    )
}

#[cfg(test)]
pub(crate) fn image_install_confirmation_message_for_test(agent: &str) -> String {
    image_install_confirmation_message(agent)
}

#[cfg(test)]
pub(crate) fn rebuild_agent_image_for_test(
    request: ImageBuildRequest,
    registry: &ImageBuildStatusRegistry,
    jobs: &BackgroundJobRegistry,
    approve: impl FnOnce(&str, &str) -> Result<bool, String>,
) -> Result<ImageBuildStartResponse, String> {
    let prepared = authorize_image_rebuild(&request)?;
    if let Some(response) = existing_image_rebuild(&prepared, registry, jobs)? {
        return Ok(response);
    }
    if !approve(&prepared.agent, &prepared.plan.tag)? {
        return Err(REBUILD_CANCELED.to_string());
    }
    start_image_rebuild(prepared, registry, jobs)
}

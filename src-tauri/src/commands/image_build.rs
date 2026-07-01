//! Mutating agent-image rebuild command.
//!
//! Rebuilding is recovery for a stale or missing agent image. It reuses the
//! tested core `ImageBuildPlan` (no docker/container build logic lives here),
//! fails closed on unconfirmed or invalid input, and requires a Rust-owned
//! native confirmation naming the exact agent image and tag before any build
//! runs. The build itself runs on a background thread; the command returns a
//! build id the WebView polls through the read-only `get_image_build_status`.

use runhaven_core::image::build::{ImageBuildPlan, build_image_plan};
use runhaven_core::runtime::launch::{new_run_id, require_container_cli};
use runhaven_core::runtime::profiles::get_profile;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;

use super::dialog::confirm_native;
use super::validation::{MAX_AGENT_NAME_LEN, validate_text_len};
use crate::contracts::{ImageBuildRequest, ImageBuildStartResponse};
use crate::image_build_runner::spawn_image_build;
use crate::tail_status::ImageBuildStatusRegistry;

const REBUILD_CANCELED: &str = "Image rebuild canceled before it started.";

#[tauri::command]
pub(crate) async fn rebuild_agent_image(
    app: AppHandle,
    request: ImageBuildRequest,
    image_build_status: State<'_, ImageBuildStatusRegistry>,
) -> Result<ImageBuildStartResponse, String> {
    let registry = image_build_status.inner().clone();
    let prepared = authorize_image_rebuild(&request)?;
    if !trusted_image_rebuild_confirmed(app, &prepared.agent, &prepared.plan.tag).await? {
        return Err(REBUILD_CANCELED.to_string());
    }
    start_image_rebuild(prepared, &registry)
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
        return Err("Confirm the rebuild before RunHaven rebuilds this agent image.".to_string());
    }
    validate_text_len("agent", &request.agent, MAX_AGENT_NAME_LEN)?;
    let profile =
        get_profile(&request.agent).map_err(|_| "Choose a known agent to rebuild.".to_string())?;
    // build_image_plan surfaces host paths and template names on error, so it is
    // collapsed to a fixed, path-free message.
    let plan = build_image_plan(&profile, None)
        .map_err(|_| "RunHaven has no bundled image template for this agent.".to_string())?;
    Ok(PreparedRebuild {
        agent: profile.name.to_string(),
        plan,
    })
}

fn start_image_rebuild(
    prepared: PreparedRebuild,
    registry: &ImageBuildStatusRegistry,
) -> Result<ImageBuildStartResponse, String> {
    // The container CLI is the last gate before the mutation, re-checked after
    // confirmation so a denied or canceled rebuild never touches the builder.
    require_container_cli().map_err(|_| {
        "The container tool is not ready. Finish RunHaven setup before rebuilding an image."
            .to_string()
    })?;
    let build_id = new_run_id();
    let tag = prepared.plan.tag.clone();
    registry.mark_building(&build_id, &prepared.agent, &tag)?;
    if let Err(error) = spawn_image_build(build_id.clone(), prepared.plan, registry.clone()) {
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
        label: "Building".to_string(),
        detail: "RunHaven started rebuilding this agent image.".to_string(),
    })
}

async fn trusted_image_rebuild_confirmed(
    app: AppHandle,
    agent: &str,
    tag: &str,
) -> Result<bool, String> {
    let message = format!(
        "Agent: {agent}\nImage tag: {tag}\n\nRunHaven will rebuild this agent image from its bundled template after you confirm. Existing containers keep running until you launch a new session."
    );
    confirm_native(
        app,
        "Confirm RunHaven image rebuild",
        "Rebuild",
        MessageDialogKind::Warning,
        message,
    )
    .await
}

#[cfg(test)]
pub(crate) fn rebuild_agent_image_for_test(
    request: ImageBuildRequest,
    registry: &ImageBuildStatusRegistry,
    approve: impl FnOnce(&str, &str) -> Result<bool, String>,
) -> Result<ImageBuildStartResponse, String> {
    let prepared = authorize_image_rebuild(&request)?;
    if !approve(&prepared.agent, &prepared.plan.tag)? {
        return Err(REBUILD_CANCELED.to_string());
    }
    start_image_rebuild(prepared, registry)
}

//! Read-only progress for a background agent-image rebuild. Returns only the
//! sanitized status the `ImageBuildStatusRegistry` records (state, label,
//! detail, and a bounded, secret-redacted output tail).

use tauri::State;

use crate::contracts::{ImageBuildStatusRequest, ImageBuildStatusResponse};
use crate::tail_status::ImageBuildStatusRegistry;

#[tauri::command]
pub(crate) fn get_image_build_status(
    request: ImageBuildStatusRequest,
    image_build_status: State<'_, ImageBuildStatusRegistry>,
) -> Result<ImageBuildStatusResponse, String> {
    image_build_status.get(&request.build_id)
}

#[cfg(test)]
pub(crate) fn get_image_build_status_for_test(
    request: ImageBuildStatusRequest,
    registry: &ImageBuildStatusRegistry,
) -> Result<ImageBuildStatusResponse, String> {
    registry.get(&request.build_id)
}

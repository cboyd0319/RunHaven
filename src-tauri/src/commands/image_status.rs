use runhaven_core::image::doctor::{
    BuilderStatusSummary, ImageStatusReport, ProfileImageStatus as CoreProfileImageStatus,
    collect_image_status,
};

use super::validation::{MAX_AGENT_NAME_LEN, validate_text_len};
use crate::contracts::{
    BuilderStatus, ImageStatusRequest, ImageStatusResponse, ProfileImageStatus,
};

#[tauri::command]
pub(crate) fn get_image_status(request: ImageStatusRequest) -> Result<ImageStatusResponse, String> {
    validate_text_len("agent", &request.agent, MAX_AGENT_NAME_LEN)?;
    collect_image_status(&request.agent)
        .map(image_status_response)
        .map_err(|error| error.to_string())
}

fn image_status_response(report: ImageStatusReport) -> ImageStatusResponse {
    ImageStatusResponse {
        agent: report.agent,
        image: profile_image_status(report.image),
        builder: builder_status(report.builder),
    }
}

fn profile_image_status(status: CoreProfileImageStatus) -> ProfileImageStatus {
    ProfileImageStatus {
        agent: status.agent,
        image: status.image,
        status: status.status,
        ready: status.ready,
        expected_source_digest: status.expected_source_digest,
        local_source_digest: status.local_source_digest,
        fix_command: status.fix_command,
    }
}

fn builder_status(status: BuilderStatusSummary) -> BuilderStatus {
    BuilderStatus {
        status: status.status,
        detail: status.detail,
        image: status.image,
        cpus: status.cpus,
        memory: status.memory,
        rosetta: status.rosetta,
        started_date: status.started_date,
        ipv4_address: status.ipv4_address,
        warning: status.warning,
    }
}

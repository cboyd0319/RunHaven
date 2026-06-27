use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::image::assets::{context_digest, materialize_image_context, source_root};
use crate::runtime::plans::validate_image_reference;
use crate::runtime::profiles::AgentProfile;
use crate::support::shell;

pub const RUNHAVEN_PROFILE_LABEL: &str = "org.runhaven.profile";
pub const RUNHAVEN_SOURCE_DIGEST_LABEL: &str = "org.runhaven.source-sha256";

#[derive(Clone, Debug)]
pub struct ImageBuildPlan {
    pub command: Vec<String>,
    pub context: PathBuf,
    pub containerfile: PathBuf,
    pub tag: String,
}

impl ImageBuildPlan {
    pub fn shell_command(&self) -> String {
        shell::join(&self.command)
    }
}

pub fn build_image_plan(profile: &AgentProfile, tag: Option<&str>) -> Result<ImageBuildPlan> {
    let context_name = profile.image_context.ok_or_else(|| {
        anyhow::anyhow!(
            "agent {:?} does not have a bundled image template",
            profile.name
        )
    })?;
    let context = materialize_image_context()?;
    let containerfile = context.join(context_name).join("Containerfile");
    if !containerfile.exists() {
        bail!("missing bundled Containerfile for agent {:?}", profile.name);
    }
    let image_tag = tag.unwrap_or(profile.image).to_string();
    validate_image_reference(&image_tag, "image tag")?;
    let source_digest = image_source_digest(profile)?;
    let command = vec![
        "container".to_string(),
        "build".to_string(),
        "-t".to_string(),
        image_tag.clone(),
        "--label".to_string(),
        format!("{RUNHAVEN_PROFILE_LABEL}={}", profile.name),
        "--label".to_string(),
        format!("{RUNHAVEN_SOURCE_DIGEST_LABEL}={source_digest}"),
        "-f".to_string(),
        containerfile.display().to_string(),
        context.display().to_string(),
    ];
    Ok(ImageBuildPlan {
        command,
        context,
        containerfile,
        tag: image_tag,
    })
}

pub fn image_source_digest(profile: &AgentProfile) -> Result<String> {
    let context = profile.image_context.ok_or_else(|| {
        anyhow::anyhow!(
            "agent {:?} does not have a bundled image template",
            profile.name
        )
    })?;
    Ok(context_digest(context))
}

pub fn image_context_root() -> Result<PathBuf> {
    Ok(source_root().unwrap_or(materialize_image_context()?))
}

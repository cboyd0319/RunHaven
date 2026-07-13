use anyhow::{Result, bail};
use serde::Serialize;

pub const SUPPORTED_NETWORK_MODES: &[&str] = &["internet", "internal", "provider"];
pub const SUPPORTED_WORKSPACE_SCOPES: &[&str] = &["current", "git-root"];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkMode {
    Internet,
    Internal,
    Provider,
}

impl NetworkMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Internet => "internet",
            Self::Internal => "internal",
            Self::Provider => "provider",
        }
    }

    /// The plain user-facing label for this network mode, for consent and
    /// summary surfaces (e.g. the native launch confirmation dialog) instead of
    /// the raw `as_str` code. This is the shared launch vocabulary: it MUST
    /// return the exact strings the WebView shows in `ui/src/app/labels.ts`
    /// (`networkLabels`), so the native dialog and the desktop UI never drift
    /// into two different words for the same value. Change both sides together.
    pub fn friendly_label(&self) -> &'static str {
        match self {
            Self::Internet => "Full internet",
            Self::Internal => "Local only",
            Self::Provider => "Assistant services only",
        }
    }
}

impl TryFrom<&str> for NetworkMode {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "internet" => Ok(Self::Internet),
            "internal" => Ok(Self::Internal),
            "provider" => Ok(Self::Provider),
            _ => bail!("invalid network mode: {value:?}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceScope {
    Current,
    GitRoot,
}

impl WorkspaceScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::GitRoot => "git-root",
        }
    }
}

impl TryFrom<&str> for WorkspaceScope {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "current" => Ok(Self::Current),
            "git-root" => Ok(Self::GitRoot),
            _ => bail!("invalid workspace scope: {value:?}"),
        }
    }
}

/// Where an agent's login and state volume live, so an OAuth login can be done
/// once and reused.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthScope {
    /// One login per agent, reused across every workspace (log in once).
    #[default]
    Agent,
    /// A login isolated to this workspace (the per-workspace default state).
    Project,
}

impl AuthScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Project => "project",
        }
    }

    /// The plain user-facing label for this sign-in scope, for consent and
    /// summary surfaces (e.g. the native launch confirmation dialog) instead of
    /// the raw `as_str` code. This is the shared launch vocabulary: it MUST
    /// return the exact strings the WebView shows in `ui/src/app/labels.ts`
    /// (`authScopeLabels`), so the native dialog and the desktop UI speak with
    /// one voice. Change both sides together.
    pub fn friendly_label(&self) -> &'static str {
        match self {
            Self::Agent => "Shared assistant sign-in",
            Self::Project => "This project only",
        }
    }
}

impl TryFrom<&str> for AuthScope {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "agent" => Ok(Self::Agent),
            "project" => Ok(Self::Project),
            _ => bail!("invalid auth scope: {value:?} (use agent or project)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pin the plain user-facing labels for every variant against the RUST side
    /// of the shared launch vocabulary. This alone cannot see the WebView copy;
    /// the cross-boundary check below reads `ui/src/app/labels.ts` so a reword
    /// on either side fails a test instead of silently splitting the native
    /// consent dialog from the in-app screen.
    #[test]
    fn friendly_label_matches_the_shared_webview_vocabulary() {
        assert_eq!(
            NetworkMode::Provider.friendly_label(),
            "Assistant services only"
        );
        assert_eq!(NetworkMode::Internal.friendly_label(), "Local only");
        assert_eq!(NetworkMode::Internet.friendly_label(), "Full internet");

        assert_eq!(
            AuthScope::Agent.friendly_label(),
            "Shared assistant sign-in"
        );
        assert_eq!(AuthScope::Project.friendly_label(), "This project only");

        // The friendly label is never the raw enum code: that is the whole point
        // (COPY-1), so a non-technical user never reads "provider" or "agent".
        assert_ne!(
            NetworkMode::Provider.friendly_label(),
            NetworkMode::Provider.as_str()
        );
        assert_ne!(AuthScope::Agent.friendly_label(), AuthScope::Agent.as_str());
    }

    /// The REAL Rust<->TS drift pin: read the WebView label source and require
    /// every `friendly_label` string to appear in it. Rewording a label in
    /// `ui/src/app/labels.ts` without updating `friendly_label` (or vice versa)
    /// fails here, so the native launch consent dialog and the in-app summary
    /// cannot drift into two different words for the same value.
    #[test]
    fn friendly_labels_exist_in_the_webview_labels_source() {
        let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let labels = loop {
            let candidate = dir.join("ui/src/app/labels.ts");
            if candidate.is_file() {
                break candidate;
            }
            assert!(
                dir.pop(),
                "ui/src/app/labels.ts not found above CARGO_MANIFEST_DIR"
            );
        };
        let source = std::fs::read_to_string(&labels).expect("read ui/src/app/labels.ts");
        for (mode, label) in [
            ("network", NetworkMode::Provider.friendly_label()),
            ("network", NetworkMode::Internal.friendly_label()),
            ("network", NetworkMode::Internet.friendly_label()),
            ("auth scope", AuthScope::Agent.friendly_label()),
            ("auth scope", AuthScope::Project.friendly_label()),
        ] {
            assert!(
                source.contains(label),
                "ui/src/app/labels.ts no longer contains the {mode} label {label:?}; \
                 the WebView and friendly_label must be reworded together"
            );
        }
    }
}

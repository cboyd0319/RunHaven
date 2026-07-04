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
            Self::Internet => "Internet",
            Self::Internal => "Local only",
            Self::Provider => "Provider only",
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
            Self::Agent => "Shared agent login",
            Self::Project => "Project session",
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

    /// Pin the plain user-facing labels for every variant. These strings are the
    /// shared launch vocabulary and must stay byte-for-byte identical to the
    /// WebView labels in `ui/src/app/labels.ts` (`networkLabels`,
    /// `authScopeLabels`); if the WebView copy changes, this test should fail so
    /// the two surfaces cannot drift silently.
    #[test]
    fn friendly_label_matches_the_shared_webview_vocabulary() {
        assert_eq!(NetworkMode::Provider.friendly_label(), "Provider only");
        assert_eq!(NetworkMode::Internal.friendly_label(), "Local only");
        assert_eq!(NetworkMode::Internet.friendly_label(), "Internet");

        assert_eq!(AuthScope::Agent.friendly_label(), "Shared agent login");
        assert_eq!(AuthScope::Project.friendly_label(), "Project session");

        // The friendly label is never the raw enum code: that is the whole point
        // (COPY-1), so a non-technical user never reads "provider" or "agent".
        assert_ne!(
            NetworkMode::Provider.friendly_label(),
            NetworkMode::Provider.as_str()
        );
        assert_ne!(AuthScope::Agent.friendly_label(), AuthScope::Agent.as_str());
    }
}

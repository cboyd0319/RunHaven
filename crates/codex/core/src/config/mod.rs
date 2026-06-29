pub mod edit;
mod network_proxy_spec;
mod resolved_permission_profile;

use std::path::Path;
use std::path::PathBuf;

pub use codex_config::ConfigLayerStack;
pub use codex_config::ConfigLoadOptions;
pub use codex_config::Constrained;
pub use codex_config::ConstraintError;
pub use codex_config::ConstraintResult;
pub use codex_config::LoaderOverrides;
pub use codex_config::ProfileV2Name;
pub use codex_config::config_toml::ConfigToml;
pub use codex_config::types::AuthKeyringBackendKind;
pub use network_proxy_spec::NetworkProxySpec;
pub use resolved_permission_profile::PermissionProfileSnapshot;

use codex_config::CloudConfigBundleLoader;
use codex_config::config_toml::ConfigToml as UpstreamConfigToml;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::built_in_model_providers;
use codex_protocol::config_types::AltScreenMode;
use codex_protocol::config_types::ServiceTier;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use codex_protocol::config_types::WindowsSandboxLevel;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Default)]
pub struct Permissions {
    pub network: Option<NetworkProxySpec>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub codex_home: PathBuf,
    pub sqlite_home: PathBuf,
    pub cwd: PathBuf,
    pub log_dir: PathBuf,
    pub model: String,
    pub model_provider_id: String,
    pub model_provider: ModelProviderInfo,
    pub preferred_auth_method: Option<codex_protocol::config_types::ForcedLoginMethod>,
    pub disable_response_storage: bool,
    pub show_raw_agent_reasoning: bool,
    pub show_raw_agent_reasoning_ui: bool,
    pub hide_agent_reasoning: bool,
    pub model_reasoning_summary: Option<codex_protocol::config_types::ReasoningSummary>,
    pub model_reasoning_effort: Option<codex_protocol::openai_models::ReasoningEffort>,
    pub model_verbosity: Option<codex_protocol::config_types::Verbosity>,
    pub model_supports_reasoning_summaries: bool,
    pub model_context_window: Option<u64>,
    pub model_max_output_tokens: Option<u64>,
    pub show_tooltips: bool,
    pub approval_policy: codex_protocol::protocol::AskForApproval,
    pub sandbox_policy: codex_protocol::protocol::SandboxPolicy,
    pub shell_environment_policy: ShellEnvironmentPolicy,
    pub sandbox_workspace_write: codex_config::types::SandboxWorkspaceWrite,
    pub permissions: Permissions,
    pub tui_theme: Option<String>,
    pub tui_keymap: Option<codex_config::types::TuiKeymap>,
    pub tui_pet: Option<String>,
    pub tui_pet_anchor: codex_config::types::TuiPetAnchor,
    pub tui_notifications: codex_config::types::TuiNotificationSettings,
    pub tui_status_line: Option<Vec<String>>,
    pub tui_status_line_use_colors: bool,
    pub tui_terminal_title: Option<Vec<String>>,
    pub terminal_visualization: bool,
    pub terminal_resize_reflow: TerminalResizeReflowConfig,
    pub disable_paste_burst: bool,
    pub alt_screen: AltScreenMode,
    pub tools_web_search_mode: codex_protocol::config_types::WebSearchMode,
    pub tools_web_search_max_per_turn: Option<u64>,
    pub service_tier: Option<ServiceTier>,
    pub windows_sandbox_level: Option<WindowsSandboxLevel>,
    pub windows_sandbox_private_desktop: bool,
    pub config_layer_stack: ConfigLayerStack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TerminalResizeReflowMaxRows {
    #[default]
    Auto,
    Disabled,
    Limit(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TerminalResizeReflowConfig {
    pub max_rows: TerminalResizeReflowMaxRows,
}

#[derive(Debug, Clone, Default)]
pub struct ConfigOverrides {
    pub cwd: Option<PathBuf>,
    pub model: Option<String>,
    pub model_provider_id: Option<String>,
    pub model_provider: Option<ModelProviderInfo>,
}

#[derive(Debug, Clone)]
pub struct ConfigTomlLoadResult {
    pub config_toml: UpstreamConfigToml,
    pub config_layer_stack: ConfigLayerStack,
}

#[derive(Debug, Clone, Default)]
pub struct ConfigBuilder {
    codex_home: Option<PathBuf>,
    cwd: Option<PathBuf>,
    config_layer_stack: Option<ConfigLayerStack>,
    overrides: ConfigOverrides,
}

impl ConfigBuilder {
    pub fn without_managed_config_for_tests() -> Self {
        Self::default()
    }

    pub fn codex_home(mut self, codex_home: PathBuf) -> Self {
        self.codex_home = Some(codex_home);
        self
    }

    pub fn cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = Some(cwd);
        self
    }

    pub fn config_layer_stack(mut self, stack: ConfigLayerStack) -> Self {
        self.config_layer_stack = Some(stack);
        self
    }

    pub fn harness_overrides(mut self, overrides: ConfigOverrides) -> Self {
        self.overrides = overrides;
        self
    }

    pub async fn build(self) -> anyhow::Result<Config> {
        let codex_home = self.codex_home.unwrap_or_else(|| PathBuf::from(".codex"));
        let config_toml = load_reduced_config_toml(&codex_home).await;
        let cwd = self
            .overrides
            .cwd
            .clone()
            .or(self.cwd)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let config_layer_stack = self.config_layer_stack.unwrap_or_default();
        let model = self.overrides.model.unwrap_or_else(|| "gpt-5".to_string());
        let model_providers = built_in_model_providers(None);
        let (model_provider_id, model_provider) = match (
            self.overrides.model_provider_id,
            self.overrides.model_provider,
        ) {
            (Some(model_provider_id), Some(model_provider)) => (model_provider_id, model_provider),
            (None, Some(model_provider)) => {
                let model_provider_id = model_providers
                    .iter()
                    .find_map(|(id, provider)| (provider == &model_provider).then(|| id.clone()))
                    .unwrap_or_else(|| "custom".to_string());
                (model_provider_id, model_provider)
            }
            (Some(model_provider_id), None) => {
                let model_provider =
                    model_providers
                        .get(&model_provider_id)
                        .cloned()
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "model_provider_id `{model_provider_id}` requires a matching model_provider override"
                            )
                        })?;
                (model_provider_id, model_provider)
            }
            (None, None) => ("openai".to_string(), model_providers["openai"].clone()),
        };
        let log_dir = codex_home.join("log");

        Ok(Config {
            sqlite_home: codex_home.clone(),
            codex_home,
            cwd,
            log_dir,
            model,
            model_provider_id,
            model_provider,
            preferred_auth_method: None,
            disable_response_storage: false,
            show_raw_agent_reasoning: false,
            show_raw_agent_reasoning_ui: false,
            hide_agent_reasoning: false,
            model_reasoning_summary: None,
            model_reasoning_effort: None,
            model_verbosity: None,
            model_supports_reasoning_summaries: true,
            model_context_window: None,
            model_max_output_tokens: None,
            show_tooltips: config_toml
                .tui
                .as_ref()
                .map(|tui| tui.show_tooltips)
                .unwrap_or(true),
            approval_policy: codex_protocol::protocol::AskForApproval::UnlessTrusted,
            sandbox_policy: codex_protocol::protocol::SandboxPolicy::new_read_only_policy(),
            shell_environment_policy: ShellEnvironmentPolicy::default(),
            sandbox_workspace_write: codex_config::types::SandboxWorkspaceWrite::default(),
            permissions: Permissions::default(),
            tui_theme: None,
            tui_keymap: None,
            tui_pet: Some("cubby".to_string()),
            tui_pet_anchor: codex_config::types::TuiPetAnchor::default(),
            tui_notifications: codex_config::types::TuiNotificationSettings::default(),
            tui_status_line: None,
            tui_status_line_use_colors: false,
            tui_terminal_title: None,
            terminal_visualization: true,
            terminal_resize_reflow: TerminalResizeReflowConfig::default(),
            disable_paste_burst: false,
            alt_screen: AltScreenMode::Never,
            tools_web_search_mode: codex_protocol::config_types::WebSearchMode::Disabled,
            tools_web_search_max_per_turn: None,
            service_tier: None,
            windows_sandbox_level: Some(WindowsSandboxLevel::Disabled),
            windows_sandbox_private_desktop: false,
            config_layer_stack,
        })
    }
}

async fn load_reduced_config_toml(codex_home: &Path) -> UpstreamConfigToml {
    let path = codex_home.join("config.toml");
    let Ok(text) = tokio::fs::read_to_string(path).await else {
        return UpstreamConfigToml::default();
    };
    toml::from_str(&text).unwrap_or_default()
}

pub async fn load_config_toml_with_layer_stack(
    codex_home: &Path,
    _cwd: &Path,
    _options: ConfigLoadOptions,
) -> anyhow::Result<ConfigTomlLoadResult> {
    Ok(ConfigTomlLoadResult {
        config_toml: load_reduced_config_toml(codex_home).await,
        config_layer_stack: ConfigLayerStack::default(),
    })
}

pub fn resolve_profile_v2_config_path(codex_home: &Path, profile: &ProfileV2Name) -> PathBuf {
    codex_home.join("profiles").join(format!("{profile}.toml"))
}

pub fn resolve_bootstrap_auth_keyring_backend_kind(
    load_result: &ConfigTomlLoadResult,
) -> std::io::Result<AuthKeyringBackendKind> {
    Ok(load_result
        .config_toml
        .cli_auth_credentials_store
        .map(|mode| match mode {
            codex_config::types::AuthCredentialsStoreMode::Keyring
            | codex_config::types::AuthCredentialsStoreMode::Auto => AuthKeyringBackendKind::Direct,
            codex_config::types::AuthCredentialsStoreMode::File
            | codex_config::types::AuthCredentialsStoreMode::Ephemeral => {
                AuthKeyringBackendKind::default()
            }
        })
        .unwrap_or_default())
}

pub fn resolve_bootstrap_auth_route_config(_load_result: &ConfigTomlLoadResult) -> Option<()> {
    None
}

pub fn resolve_oss_provider(_load_result: &ConfigTomlLoadResult) -> Option<String> {
    None
}

pub fn cloud_config_bundle_loader() -> CloudConfigBundleLoader {
    CloudConfigBundleLoader::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn default_config_builder_stays_read_only() {
        let config = ConfigBuilder::without_managed_config_for_tests()
            .build()
            .await
            .unwrap();

        assert_eq!(
            config.sandbox_policy,
            codex_protocol::protocol::SandboxPolicy::new_read_only_policy()
        );
        assert!(config.permissions.network.is_none());
    }

    #[tokio::test]
    async fn default_config_builder_reports_openai_provider_id() {
        let config = ConfigBuilder::without_managed_config_for_tests()
            .build()
            .await
            .unwrap();

        assert_eq!(config.model_provider_id, "openai");
    }

    #[tokio::test]
    async fn config_builder_derives_builtin_provider_id_from_override() {
        let config = ConfigBuilder::without_managed_config_for_tests()
            .harness_overrides(ConfigOverrides {
                model_provider: Some(built_in_model_providers(None)["amazon-bedrock"].clone()),
                ..ConfigOverrides::default()
            })
            .build()
            .await
            .unwrap();

        assert_eq!(config.model_provider_id, "amazon-bedrock");
    }

    #[tokio::test]
    async fn config_builder_resolves_builtin_provider_from_id_override() {
        let config = ConfigBuilder::without_managed_config_for_tests()
            .harness_overrides(ConfigOverrides {
                model_provider_id: Some("amazon-bedrock".to_string()),
                ..ConfigOverrides::default()
            })
            .build()
            .await
            .unwrap();

        assert_eq!(config.model_provider_id, "amazon-bedrock");
        assert_eq!(
            config.model_provider,
            built_in_model_providers(None)["amazon-bedrock"]
        );
    }

    #[tokio::test]
    async fn config_builder_preserves_custom_provider_id_override() {
        let config = ConfigBuilder::without_managed_config_for_tests()
            .harness_overrides(ConfigOverrides {
                model_provider_id: Some("local-provider".to_string()),
                model_provider: Some(ModelProviderInfo {
                    name: "Local Provider".to_string(),
                    ..ModelProviderInfo::default()
                }),
                ..ConfigOverrides::default()
            })
            .build()
            .await
            .unwrap();

        assert_eq!(config.model_provider_id, "local-provider");
        assert_eq!(config.model_provider.name, "Local Provider");
    }

    #[tokio::test]
    async fn config_builder_rejects_unknown_provider_id_without_provider() {
        let err = ConfigBuilder::without_managed_config_for_tests()
            .harness_overrides(ConfigOverrides {
                model_provider_id: Some("local-provider".to_string()),
                ..ConfigOverrides::default()
            })
            .build()
            .await
            .unwrap_err();

        assert!(
            err.to_string()
                .contains("requires a matching model_provider override")
        );
    }

    #[tokio::test]
    async fn reduced_config_builder_loads_tui_show_tooltips() {
        let dir = tempfile::tempdir().unwrap();
        tokio::fs::write(
            dir.path().join("config.toml"),
            "[tui]\nshow_tooltips = false\n",
        )
        .await
        .unwrap();

        let config = ConfigBuilder::without_managed_config_for_tests()
            .codex_home(dir.path().to_path_buf())
            .build()
            .await
            .unwrap();

        assert!(!config.show_tooltips);
    }

    #[tokio::test]
    async fn reduced_config_toml_loader_loads_tui_show_tooltips() {
        let dir = tempfile::tempdir().unwrap();
        tokio::fs::write(
            dir.path().join("config.toml"),
            "[tui]\nshow_tooltips = false\n",
        )
        .await
        .unwrap();

        let loaded =
            load_config_toml_with_layer_stack(dir.path(), dir.path(), ConfigLoadOptions::default())
                .await
                .unwrap();

        assert_eq!(
            loaded.config_toml.tui.as_ref().map(|tui| tui.show_tooltips),
            Some(false)
        );
    }
}

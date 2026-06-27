use std::env;

use anyhow::{Context, Result, bail};

mod resources;
mod types;
mod validation;

pub use resources::{
    bind_mount, home_setup_command, project_identifier, safe_resource_name,
    strip_remainder_separator, volume_mount,
};
pub use types::{
    AgentRunPlan, AuthScope, CONTAINER_PATH, DEFAULT_ENV_PASSTHROUGH, NetworkMode, RunOptions,
    SUPPORTED_NETWORK_MODES, SUPPORTED_WORKSPACE_SCOPES, VOLUME_PREP_IMAGE, VOLUME_PREP_NETWORK,
    WorkspaceScope, WorktreeRun,
};
pub use validation::{
    apply_workspace_scope, default_network_mode, network_egress_summary, normalize_provider_hosts,
    normalize_session, provider_hosts_for_options, security_notices, sensitive_workspace_paths,
    uses_root_identity, validate_env_name, validate_image_reference, validate_resource_options,
    validate_workspace,
};

use crate::runtime::session_state::{shared_state_volume_name, state_volume_name};
use crate::support::validators::validate_run_id;

const SSH_FORWARDING_DISABLED_MESSAGE: &str = "SSH forwarding is disabled: Apple container 1.0.0 exposes the forwarded socket to RunHaven's non-root agent user, but ssh-add -l returns permission denied. Do not mount raw SSH keys or run the agent as root; track docs/APPLE_CONTAINER_GAP_ANALYSIS.md.";

pub fn build_run_plan(options: RunOptions) -> Result<AgentRunPlan> {
    if let Some(run_id) = &options.run_id {
        validate_run_id(run_id)?;
    }
    let mut workspace = options.workspace.canonicalize().with_context(|| {
        format!(
            "could not resolve workspace path: {}",
            options.workspace.display()
        )
    })?;
    if !workspace.exists() {
        bail!("workspace does not exist: {}", workspace.display());
    }
    if !workspace.is_dir() {
        bail!("workspace is not a directory: {}", workspace.display());
    }
    let (scoped_workspace, workspace_scope_note) =
        apply_workspace_scope(&workspace, options.workspace_scope)?;
    workspace = scoped_workspace;
    validate_workspace(&workspace, options.allow_sensitive_workspace)?;

    for name in &options.env {
        validate_env_name(name)?;
    }
    if let Some(name) = &options.api_key_broker_env {
        validate_env_name(name)?;
        if options.profile.name != "codex" {
            bail!("Codex API key broker requires codex profile");
        }
        if options.network != NetworkMode::Provider {
            bail!("Codex API key broker requires --network provider");
        }
    }
    validate_resource_options(&options.cpus, &options.memory, &options.user)?;
    if uses_root_identity(&options.user) && !options.allow_root_user {
        bail!("root user or group requires --allow-root-user");
    }
    if options.ssh {
        bail!(SSH_FORWARDING_DISABLED_MESSAGE);
    }

    let provider_allowed_hosts = provider_hosts_for_options(&options)?;
    let session = normalize_session(options.session.as_deref())?;
    let project_id = project_identifier(&workspace);
    // Per-agent (shared) auth scope keeps one login per agent across every
    // workspace; per-project keeps the isolated per-workspace volume.
    let state_volume = match options.auth_scope {
        AuthScope::Agent => shared_state_volume_name(options.profile.name),
        AuthScope::Project => state_volume_name(
            options.profile.name,
            &project_id,
            options.session.as_deref(),
        )?,
    };
    let container_name = safe_resource_name(&format!(
        "runhaven-{}-{project_id}-run",
        options.profile.name
    ));
    let default_network_name = safe_resource_name(&format!("runhaven-{project_id}-internal"));
    let image = options
        .image
        .clone()
        .unwrap_or_else(|| options.profile.image.to_string());
    validate_image_reference(&image, "image")?;

    let mut command = vec![
        "container".to_string(),
        "run".to_string(),
        "--rm".to_string(),
        "--init".to_string(),
        "--name".to_string(),
        container_name.clone(),
        "--read-only".to_string(),
        "--tmpfs".to_string(),
        "/tmp".to_string(),
        "--cap-drop".to_string(),
        "ALL".to_string(),
        "--cpus".to_string(),
        options.cpus.clone(),
        "--memory".to_string(),
        options.memory.clone(),
        "--user".to_string(),
        options.user.clone(),
        "--workdir".to_string(),
        "/workspace".to_string(),
        "--mount".to_string(),
        bind_mount(&workspace, "/workspace", options.read_only_workspace),
        "--mount".to_string(),
        volume_mount(&state_volume, "/home/agent"),
        "--env".to_string(),
        "HOME=/home/agent".to_string(),
        "--env".to_string(),
        format!("PATH={CONTAINER_PATH}"),
    ];
    if options.interactive {
        command.push("--interactive".to_string());
    }
    if options.tty {
        command.push("--tty".to_string());
    }
    for name in DEFAULT_ENV_PASSTHROUGH {
        if env::var_os(name).is_some() {
            command.extend(["--env".to_string(), (*name).to_string()]);
        }
    }
    for (key, value) in options.profile.env() {
        command.extend(["--env".to_string(), format!("{key}={value}")]);
    }
    for name in &options.env {
        command.extend(["--env".to_string(), name.clone()]);
    }

    let mut preflight = Vec::new();
    if options.user == "agent" {
        preflight.push(vec![
            "container".to_string(),
            "network".to_string(),
            "create".to_string(),
            "--internal".to_string(),
            VOLUME_PREP_NETWORK.to_string(),
        ]);
        preflight.push(vec![
            "container".to_string(),
            "run".to_string(),
            "--rm".to_string(),
            "--init".to_string(),
            // Drop all capabilities, then re-add only the three the home-volume
            // ownership setup needs: CHOWN (chown), FOWNER (chmod a non-owned
            // path), DAC_OVERRIDE (mkdir into the 1000-owned home). The runtime
            // resolves drop-ALL then add, so this yields exactly those three.
            "--cap-drop".to_string(),
            "ALL".to_string(),
            "--cap-add".to_string(),
            "CHOWN".to_string(),
            "--cap-add".to_string(),
            "FOWNER".to_string(),
            "--cap-add".to_string(),
            "DAC_OVERRIDE".to_string(),
            "--read-only".to_string(),
            "--no-dns".to_string(),
            "--network".to_string(),
            VOLUME_PREP_NETWORK.to_string(),
            "--cpus".to_string(),
            "1".to_string(),
            "--memory".to_string(),
            "256m".to_string(),
            "--user".to_string(),
            "root".to_string(),
            "--entrypoint".to_string(),
            "/bin/sh".to_string(),
            "--mount".to_string(),
            volume_mount(&state_volume, "/home/agent"),
            VOLUME_PREP_IMAGE.to_string(),
            "-c".to_string(),
            home_setup_command(&options.profile),
        ]);
    }

    let network_name = match options.network {
        NetworkMode::Internet => None,
        NetworkMode::Internal => {
            preflight.push(vec![
                "container".to_string(),
                "network".to_string(),
                "create".to_string(),
                "--internal".to_string(),
                default_network_name.clone(),
            ]);
            command.extend(["--network".to_string(), default_network_name.clone()]);
            Some(default_network_name)
        }
        NetworkMode::Provider => {
            let provider_network = safe_resource_name(&format!(
                "runhaven-{}-{project_id}-provider",
                options.profile.name
            ));
            preflight.push(vec![
                "container".to_string(),
                "network".to_string(),
                "create".to_string(),
                "--internal".to_string(),
                provider_network.clone(),
            ]);
            command.extend(["--network".to_string(), provider_network.clone()]);
            Some(provider_network)
        }
    };

    let mut agent_command = strip_remainder_separator(&options.agent_args);
    if agent_command.is_empty() {
        agent_command = options
            .profile
            .command
            .iter()
            .map(|s| (*s).to_string())
            .collect();
    }
    if options.api_key_broker_env.is_some()
        && agent_command.first().map(String::as_str) != Some("codex")
    {
        bail!("Codex API key broker requires the agent command to start with codex");
    }
    command.push(image.clone());
    command.extend(agent_command);

    let security_notices = security_notices(&options);

    Ok(AgentRunPlan {
        command,
        preflight,
        workspace,
        state_volume,
        session,
        container_name,
        profile_name: options.profile.name.to_string(),
        workspace_scope: options.workspace_scope,
        workspace_scope_note,
        auth_scope: options.auth_scope,
        worktree: options.worktree,
        run_id: options.run_id,
        network_name,
        network_mode: options.network,
        egress_summary: network_egress_summary(
            options.network,
            &provider_allowed_hosts,
            options.api_key_broker_env.is_some(),
        ),
        image,
        provider_allowed_hosts,
        api_key_broker_env: options.api_key_broker_env,
        security_notices,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::profiles::get_profile;

    #[test]
    fn default_plan_uses_non_root_read_only_root_and_project_mount() {
        let workspace = tempfile::tempdir().expect("workspace");
        let plan = build_run_plan(RunOptions {
            profile: get_profile("shell").expect("profile"),
            workspace: workspace.path().to_path_buf(),
            agent_args: vec![
                "/bin/bash".to_string(),
                "-lc".to_string(),
                "pwd".to_string(),
            ],
            image: None,
            cpus: "4".to_string(),
            memory: "4g".to_string(),
            network: NetworkMode::Internet,
            workspace_scope: WorkspaceScope::Current,
            session: None,
            auth_scope: AuthScope::Project,
            read_only_workspace: false,
            ssh: false,
            env: Vec::new(),
            user: "agent".to_string(),
            interactive: false,
            tty: false,
            allow_sensitive_workspace: false,
            allow_root_user: false,
            provider_hosts: Vec::new(),
            api_key_broker_env: None,
            worktree: None,
            run_id: None,
        })
        .expect("plan");

        assert!(
            plan.command
                .windows(2)
                .any(|items| items == ["--user", "agent"])
        );
        assert!(plan.command.iter().any(|arg| arg == "--read-only"));
        assert!(
            plan.command
                .iter()
                .any(|arg| arg.starts_with("type=bind,source="))
        );
        assert_eq!(plan.network_mode, NetworkMode::Internet);
        assert!(plan.egress_summary.contains("unrestricted internet"));
        assert!(
            plan.security_notices
                .iter()
                .any(|notice| notice.contains("Unrestricted internet egress"))
        );
    }

    #[test]
    fn volume_prep_container_drops_caps_to_minimum() {
        let workspace = tempfile::tempdir().expect("workspace");
        let plan = build_run_plan(RunOptions {
            profile: get_profile("shell").expect("profile"),
            workspace: workspace.path().to_path_buf(),
            agent_args: vec![
                "/bin/bash".to_string(),
                "-lc".to_string(),
                "true".to_string(),
            ],
            image: None,
            cpus: "4".to_string(),
            memory: "4g".to_string(),
            network: NetworkMode::Internal,
            workspace_scope: WorkspaceScope::Current,
            session: None,
            auth_scope: AuthScope::Project,
            read_only_workspace: false,
            ssh: false,
            env: Vec::new(),
            user: "agent".to_string(),
            interactive: false,
            tty: false,
            allow_sensitive_workspace: false,
            allow_root_user: false,
            provider_hosts: Vec::new(),
            api_key_broker_env: None,
            worktree: None,
            run_id: None,
        })
        .expect("plan");

        // The short-lived root home-volume prep container must drop all caps and
        // re-add only the three the ownership setup needs.
        let prep = plan
            .preflight
            .iter()
            .find(|cmd| cmd.windows(2).any(|w| w == ["--user", "root"]))
            .expect("volume-prep container present");
        assert!(prep.windows(2).any(|w| w == ["--cap-drop", "ALL"]));
        for cap in ["CHOWN", "FOWNER", "DAC_OVERRIDE"] {
            assert!(
                prep.windows(2).any(|w| w == ["--cap-add", cap]),
                "volume-prep missing --cap-add {cap}"
            );
        }
        // The agent run keeps cap-drop ALL with no capability re-adds.
        assert!(plan.command.windows(2).any(|w| w == ["--cap-drop", "ALL"]));
        assert!(!plan.command.iter().any(|arg| arg == "--cap-add"));
    }

    #[test]
    fn auth_scope_agent_shares_one_volume_across_workspaces() {
        let volume_for = |scope: AuthScope| {
            let workspace = tempfile::tempdir().expect("workspace");
            build_run_plan(RunOptions {
                profile: get_profile("shell").expect("profile"),
                workspace: workspace.path().to_path_buf(),
                agent_args: vec![
                    "/bin/bash".to_string(),
                    "-lc".to_string(),
                    "true".to_string(),
                ],
                image: None,
                cpus: "4".to_string(),
                memory: "4g".to_string(),
                network: NetworkMode::Internal,
                workspace_scope: WorkspaceScope::Current,
                session: None,
                auth_scope: scope,
                read_only_workspace: false,
                ssh: false,
                env: Vec::new(),
                user: "agent".to_string(),
                interactive: false,
                tty: false,
                allow_sensitive_workspace: false,
                allow_root_user: false,
                provider_hosts: Vec::new(),
                api_key_broker_env: None,
                worktree: None,
                run_id: None,
            })
            .expect("plan")
            .state_volume
        };
        // Agent scope: one shared per-agent volume, independent of the workspace.
        assert_eq!(volume_for(AuthScope::Agent), "runhaven-shell-shared-home");
        // Project scope: a per-workspace volume, never the shared one.
        assert_ne!(volume_for(AuthScope::Project), "runhaven-shell-shared-home");
    }

    #[test]
    fn provider_plan_normalizes_extra_hosts_and_rejects_ip_literals() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut options = RunOptions {
            profile: get_profile("codex").expect("profile"),
            workspace: workspace.path().to_path_buf(),
            agent_args: Vec::new(),
            image: None,
            cpus: "4".to_string(),
            memory: "4g".to_string(),
            network: NetworkMode::Provider,
            workspace_scope: WorkspaceScope::Current,
            session: None,
            auth_scope: AuthScope::Project,
            read_only_workspace: false,
            ssh: false,
            env: Vec::new(),
            user: "agent".to_string(),
            interactive: false,
            tty: false,
            allow_sensitive_workspace: false,
            allow_root_user: false,
            provider_hosts: vec!["API.Example.COM.".to_string()],
            api_key_broker_env: None,
            worktree: None,
            run_id: None,
        };
        let plan = build_run_plan(options.clone()).expect("plan");
        assert!(
            plan.provider_allowed_hosts
                .contains(&"api.example.com".to_string())
        );
        assert!(
            plan.security_notices
                .iter()
                .any(|notice| notice.contains("widened"))
        );

        options.provider_hosts = vec!["127.0.0.1".to_string()];
        assert!(build_run_plan(options).is_err());
    }

    #[test]
    fn security_notices_track_lower_security_choices() {
        let workspace = tempfile::tempdir().expect("workspace");
        let base = || RunOptions {
            profile: get_profile("shell").expect("profile"),
            workspace: workspace.path().to_path_buf(),
            agent_args: vec![
                "/bin/bash".to_string(),
                "-lc".to_string(),
                "true".to_string(),
            ],
            image: None,
            cpus: "4".to_string(),
            memory: "4g".to_string(),
            network: NetworkMode::Internal,
            workspace_scope: WorkspaceScope::Current,
            session: None,
            auth_scope: AuthScope::Project,
            read_only_workspace: false,
            ssh: false,
            env: Vec::new(),
            user: "agent".to_string(),
            interactive: false,
            tty: false,
            allow_sensitive_workspace: false,
            allow_root_user: false,
            provider_hosts: Vec::new(),
            api_key_broker_env: None,
            worktree: None,
            run_id: None,
        };

        let secure = build_run_plan(base()).expect("secure plan");
        assert!(
            secure.security_notices.is_empty(),
            "secure defaults should emit no notices, got {:?}",
            secure.security_notices
        );

        let mut env_options = base();
        env_options.env = vec!["ANTHROPIC_API_KEY".to_string()];
        let env_plan = build_run_plan(env_options).expect("env plan");
        assert!(
            env_plan
                .security_notices
                .iter()
                .any(|notice| notice.contains("ANTHROPIC_API_KEY"))
        );

        let mut root_options = base();
        root_options.user = "root".to_string();
        root_options.allow_root_user = true;
        let root_plan = build_run_plan(root_options).expect("root plan");
        assert!(
            root_plan
                .security_notices
                .iter()
                .any(|notice| notice.contains("runs as root"))
        );

        let mut image_options = base();
        image_options.image = Some("example.com/custom:1.0".to_string());
        let image_plan = build_run_plan(image_options).expect("image plan");
        assert!(
            image_plan
                .security_notices
                .iter()
                .any(|notice| notice.contains("custom --image"))
        );
    }

    #[test]
    fn default_network_mode_is_provider_for_bundled_hosts_else_internet() {
        assert_eq!(
            default_network_mode(&get_profile("claude").expect("profile")),
            NetworkMode::Provider
        );
        assert_eq!(
            default_network_mode(&get_profile("codex").expect("profile")),
            NetworkMode::Provider
        );
        // antigravity now has bundled provider hosts (its login/model
        // googleapis.com set), so it defaults to provider like claude/codex.
        assert_eq!(
            default_network_mode(&get_profile("antigravity").expect("profile")),
            NetworkMode::Provider
        );
        // shell has no bundled hosts, so it is the "else internet" case.
        assert_eq!(
            default_network_mode(&get_profile("shell").expect("profile")),
            NetworkMode::Internet
        );
    }

    #[test]
    fn ssh_forwarding_fails_closed_until_non_root_runtime_is_verified() {
        let workspace = tempfile::tempdir().expect("workspace");
        let error = build_run_plan(RunOptions {
            profile: get_profile("shell").expect("profile"),
            workspace: workspace.path().to_path_buf(),
            agent_args: Vec::new(),
            image: None,
            cpus: "4".to_string(),
            memory: "4g".to_string(),
            network: NetworkMode::Internet,
            workspace_scope: WorkspaceScope::Current,
            session: None,
            auth_scope: AuthScope::Project,
            read_only_workspace: false,
            ssh: true,
            env: Vec::new(),
            user: "agent".to_string(),
            interactive: false,
            tty: false,
            allow_sensitive_workspace: false,
            allow_root_user: false,
            provider_hosts: Vec::new(),
            api_key_broker_env: None,
            worktree: None,
            run_id: None,
        })
        .expect_err("ssh forwarding should be disabled");
        let message = error.to_string();

        assert!(message.contains("SSH forwarding is disabled"));
        assert!(message.contains("Apple container 1.0.0"));
        assert!(message.contains("raw SSH keys"));
    }
}

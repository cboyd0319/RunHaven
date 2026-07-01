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
fn session_names_get_distinct_container_names() {
    let workspace = tempfile::tempdir().expect("workspace");
    let plan_for = |session: &str| {
        build_run_plan(RunOptions {
            profile: get_profile("shell").expect("profile"),
            workspace: workspace.path().to_path_buf(),
            agent_args: Vec::new(),
            image: None,
            cpus: "4".to_string(),
            memory: "4g".to_string(),
            network: NetworkMode::Internal,
            workspace_scope: WorkspaceScope::Current,
            session: Some(session.to_string()),
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
        .expect("plan")
    };

    let alpha = plan_for("alpha");
    let beta = plan_for("beta");

    assert_ne!(alpha.container_name, beta.container_name);
    assert!(alpha.container_name.starts_with("runhaven-shell-"));
    assert!(alpha.container_name.ends_with("-run"));
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
fn api_key_broker_accepts_brokered_profiles_only_on_provider_network() {
    let workspace = tempfile::tempdir().expect("workspace");
    let options_for = |agent: &str, network: NetworkMode, api_key_broker_env: &str| RunOptions {
        profile: get_profile(agent).expect("profile"),
        workspace: workspace.path().to_path_buf(),
        agent_args: Vec::new(),
        image: None,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        network,
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
        api_key_broker_env: Some(api_key_broker_env.to_string()),
        worktree: None,
        run_id: None,
    };

    for (agent, env_name) in [
        ("codex", "OPENAI_API_KEY"),
        ("claude", "ANTHROPIC_API_KEY"),
        ("gemini", "GEMINI_API_KEY"),
    ] {
        let plan = build_run_plan(options_for(agent, NetworkMode::Provider, env_name))
            .unwrap_or_else(|error| panic!("{agent} broker should plan: {error}"));
        assert_eq!(plan.api_key_broker_env.as_deref(), Some(env_name));
        assert!(plan.egress_summary.contains("API key broker enabled"));
    }

    let wrong_env_error = build_run_plan(options_for(
        "codex",
        NetworkMode::Provider,
        "AWS_SECRET_ACCESS_KEY",
    ))
    .expect_err("broker rejects unrelated host env");
    assert!(wrong_env_error.to_string().contains("OPENAI_API_KEY"));

    let shell_error = build_run_plan(options_for(
        "shell",
        NetworkMode::Provider,
        "OPENAI_API_KEY",
    ))
    .expect_err("shell has no broker");
    assert!(shell_error.to_string().contains("has no API-key broker"));

    let network_error = build_run_plan(options_for(
        "codex",
        NetworkMode::Internet,
        "OPENAI_API_KEY",
    ))
    .expect_err("broker requires provider network");
    assert!(network_error.to_string().contains("--network provider"));
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
fn ssh_forwarding_adds_the_agent_socket_flag_warns_and_mounts_no_keys() {
    let workspace = tempfile::tempdir().expect("workspace");
    let plan = build_run_plan(RunOptions {
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
    .expect("ssh forwarding is supported");

    // Apple `container --ssh` forwards the agent socket, verified no-secret on
    // macOS 26+/Apple container: a non-root guest connects to the world-readable
    // launchd agent socket (ssh-add -l returns the no-identities status).
    assert!(plan.command.contains(&"--ssh".to_string()));
    // Explicit lower-security choice must warn about the exposure.
    assert!(
        plan.security_notices
            .iter()
            .any(|notice| notice.contains("SSH agent forwarding")),
        "expected an SSH forwarding security notice"
    );
    // The agent socket is forwarded, never raw keys: no mount or arg references
    // an SSH key or an .ssh directory, and the guest still runs as non-root.
    assert!(
        !plan.command.iter().any(|arg| arg.contains(".ssh")
            || arg.contains("id_rsa")
            || arg.contains("id_ed25519"))
    );
    assert!(plan.command.contains(&"agent".to_string()));
    assert!(!plan.command.contains(&"root".to_string()));
}

use runhaven_core::runtime::plans::{AgentRunPlan, NetworkMode};

pub(super) fn print_run_plan(plan: &AgentRunPlan) {
    println!("Workspace: {}", plan.workspace.display());
    println!("Workspace scope: {}", plan.workspace_scope.as_str());
    if let Some(note) = &plan.workspace_scope_note {
        println!("Workspace note: {note}");
    }
    if let Some(worktree) = &plan.worktree {
        println!("Worktree: {}", worktree.worktree_root.display());
        println!("Worktree branch: {}", worktree.branch);
    }
    println!("Session: {}", plan.session);
    println!("State volume: {}", plan.state_volume);
    println!(
        "Network: {}",
        plan.network_name
            .as_deref()
            .unwrap_or("default internet network")
    );
    println!("Egress: {}", plan.egress_summary);
    if plan.network_mode == NetworkMode::Provider {
        println!("Provider hosts: {}", plan.provider_allowed_hosts.join(", "));
        println!("Provider proxy: RunHaven injects proxy environment variables at runtime.");
        println!(
            "Provider egress is limited to these hosts. For package installs or other hosts, add --provider-host HOST or use --network internet."
        );
    }
    if let Some(name) = plan.api_key_broker_env_name() {
        println!(
            "API key broker: enabled from host environment variable {name}; value is not printed or planned."
        );
    }
    if !plan.preflight.is_empty() {
        println!("Preflight:");
        for command in plan.shell_preflight() {
            println!("  {command}");
        }
    }
    println!("Run:");
    println!("  {}", plan.shell_command());
}

pub(super) fn eprint_security_notices(plan: &AgentRunPlan) {
    if plan.security_notices.is_empty() {
        return;
    }
    eprintln!("Security notices:");
    for notice in &plan.security_notices {
        eprintln!("  - {notice}");
    }
    eprintln!(
        "Choose RunHaven's secure defaults (non-root agent, internal or provider network, no extra env, hosts, or custom image) to clear these notices."
    );
}

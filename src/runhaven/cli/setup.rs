use crate::doctor::Check;
use crate::profiles::get_profile;

pub fn print_checks(checks: &[Check]) {
    for check in checks {
        let status = if check.ok { "ok" } else { "fail" };
        println!("{status:4} {}: {}", check.name, check.detail);
        if !check.ok && !check.remedy.is_empty() {
            println!("     fix: {}", check.remedy);
        }
    }
}

pub fn print_setup_guide(agent: &str, checks: &[Check]) -> anyhow::Result<i32> {
    let profile = get_profile(agent)?;
    let ready = checks.iter().all(|check| check.ok);
    println!("RunHaven setup\n");
    println!("1. Host prerequisites");
    print_checks(checks);
    println!();
    if !ready {
        println!("Next steps");
        for check in checks {
            if !check.ok && !check.remedy.is_empty() {
                println!("- {}: {}", check.name, check.remedy);
            }
        }
        println!("- After fixing the items above, run `runhaven setup` again.");
        return Ok(1);
    }
    println!("Selected agent: {agent} - {}", profile.description);
    println!("\n2. Build the agent image");
    println!("   runhaven image build {agent}");
    println!("\n3. Preview the container boundary");
    println!("   runhaven plan {agent}");
    println!("\n4. Run from your project directory");
    println!("   runhaven run {agent}");
    println!("\nSafety defaults");
    println!("- One selected project is mounted at /workspace.");
    println!("- No host home, raw SSH keys, or cloud credential folders are mounted by default.");
    print_setup_workspace_and_credentials();
    print_setup_network_choices(agent);
    Ok(0)
}

fn print_setup_workspace_and_credentials() {
    println!("\nWorkspace and credentials");
    println!(
        "- Run from the smallest project directory you want the agent to see; that directory is mounted at /workspace."
    );
    println!(
        "- Do not run from your home directory, a cloud sync root, or a credential folder unless you intentionally allow that broader scope."
    );
    println!(
        "- RunHaven does not mount raw SSH keys, browser profiles, cloud credential folders, or provider login caches by default."
    );
    println!("- Use `--ssh` for SSH agent forwarding instead of mounting key files.");
    println!("- Use `--env NAME` only for a reviewed variable that the agent really needs.");
    println!("- Use `runhaven plan` to confirm the mounted host path.");
}

fn print_setup_network_choices(agent: &str) {
    println!("\nNetwork choices");
    println!(
        "- Local-only: use `runhaven run {agent} --network internal` for tests, builds, and commands that do not need internet."
    );
    println!(
        "- Provider-only: use `runhaven run {agent} --network provider` to allow reviewed provider hosts through the proxy. Login, telemetry, package registries, or feature paths may need extra reviewed hosts."
    );
    println!(
        "- Package install: use default internet mode with `runhaven run {agent}` when package managers or dependency updates need broad registry and CDN access."
    );
    println!(
        "- Unrestricted internet: default `runhaven run {agent}` leaves egress unrestricted inside Apple `container` and your host network."
    );
    println!("- Use `runhaven plan` before changing network modes.");
}

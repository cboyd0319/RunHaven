use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use super::{NetworkMode, RunOptions, WorkspaceScope};
use crate::provider::egress::{is_ip_literal, normalize_host};
use crate::runtime::profiles::AgentProfile;
use crate::runtime::session_state::{SESSION_DEFAULT, validate_session_name};
use crate::support::git::git_repo_root;

pub fn apply_workspace_scope(
    workspace: &Path,
    scope: WorkspaceScope,
) -> Result<(PathBuf, Option<String>)> {
    let (repo_root, reason) = git_repo_root(workspace);
    if scope == WorkspaceScope::GitRoot {
        let Some(repo_root) = repo_root else {
            bail!("--workspace-scope git-root requires a git worktree: {reason}");
        };
        let root = PathBuf::from(repo_root).canonicalize()?;
        if root != workspace {
            return Ok((
                root.clone(),
                Some(format!(
                    "expanded from {} to git repository root {}",
                    workspace.display(),
                    root.display()
                )),
            ));
        }
        return Ok((
            root,
            Some("selected workspace is the git repository root".to_string()),
        ));
    }
    let Some(repo_root) = repo_root else {
        return Ok((workspace.to_path_buf(), None));
    };
    let root = PathBuf::from(repo_root).canonicalize()?;
    if root != workspace && workspace.starts_with(&root) {
        return Ok((
            workspace.to_path_buf(),
            Some(format!(
                "selected workspace is inside git repository root {}; RunHaven mounts only the selected directory. Use --workspace-scope git-root to mount the full repository.",
                root.display()
            )),
        ));
    }
    Ok((workspace.to_path_buf(), None))
}

pub fn provider_hosts_for_options(options: &RunOptions) -> Result<Vec<String>> {
    if options.network != NetworkMode::Provider {
        return Ok(Vec::new());
    }
    let hosts = options
        .profile
        .provider_hosts
        .iter()
        .map(|s| (*s).to_string())
        .chain(options.provider_hosts.iter().cloned())
        .collect::<Vec<_>>();
    let hosts = normalize_provider_hosts(&hosts)?;
    if hosts.is_empty() {
        bail!(
            "provider hosts are required for --network provider. Use a bundled provider profile or pass --provider-host HOST."
        );
    }
    Ok(hosts)
}

pub fn normalize_provider_hosts(hosts: &[String]) -> Result<Vec<String>> {
    let mut normalized_hosts = Vec::new();
    for host in hosts {
        let normalized = normalize_host(host)?;
        if is_ip_literal(&normalized) {
            bail!("provider hosts cannot be IP literals");
        }
        if !normalized.contains('.') {
            bail!("provider hosts must be fully qualified domain names");
        }
        if !normalized_hosts.contains(&normalized) {
            normalized_hosts.push(normalized);
        }
    }
    Ok(normalized_hosts)
}

pub fn validate_env_name(name: &str) -> Result<()> {
    if name.contains('=') {
        bail!("pass only environment variable names, not NAME=value pairs");
    }
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        bail!("invalid environment variable name: {name:?}");
    };
    if !(first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        bail!("invalid environment variable name: {name:?}");
    }
    Ok(())
}

pub fn validate_workspace(workspace: &Path, allow_sensitive: bool) -> Result<()> {
    let text = workspace.display().to_string();
    if text.contains(',') {
        bail!("workspace paths containing a comma cannot be mounted safely");
    }
    if allow_sensitive {
        return Ok(());
    }
    let (root_paths, secret_paths) = sensitive_workspace_paths();
    for path in root_paths {
        if workspace == path {
            bail!(
                "sensitive workspace requires --allow-sensitive-workspace: {}",
                workspace.display()
            );
        }
    }
    for path in secret_paths {
        if workspace == path || workspace.starts_with(&path) || path.starts_with(workspace) {
            bail!(
                "sensitive workspace requires --allow-sensitive-workspace: {}",
                workspace.display()
            );
        }
    }
    Ok(())
}

pub fn sensitive_workspace_paths() -> (Vec<PathBuf>, Vec<PathBuf>) {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let home = home.canonicalize().unwrap_or(home);
    let root_paths = [
        "/", "/Users", "/private", "/var", "/usr", "/bin", "/sbin", "/opt",
    ]
    .iter()
    .map(PathBuf::from)
    .chain([home.clone()])
    .filter_map(|path| path.canonicalize().ok())
    .collect::<Vec<_>>();
    let secret_paths = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/Library"),
        PathBuf::from("/System"),
        PathBuf::from("/etc"),
        PathBuf::from("/private/etc"),
        PathBuf::from("/private/var/audit"),
        PathBuf::from("/private/var/db"),
        PathBuf::from("/private/var/log"),
        PathBuf::from("/private/var/root"),
        PathBuf::from("/private/var/run"),
        home.join(".ssh"),
        home.join(".aws"),
        home.join(".azure"),
        home.join(".config").join("gcloud"),
        home.join(".docker"),
        home.join(".gnupg"),
        home.join(".kube"),
        home.join("Library")
            .join("Application Support")
            .join("Google")
            .join("Chrome"),
        home.join("Library")
            .join("Application Support")
            .join("Firefox"),
        home.join("Library").join("Keychains"),
    ]
    .into_iter()
    .map(|path| path.canonicalize().unwrap_or(path))
    .collect();
    (root_paths, secret_paths)
}

pub fn validate_resource_options(cpus: &str, memory: &str, user: &str) -> Result<()> {
    if !valid_cpus(cpus) {
        bail!("invalid cpus value: {cpus:?}");
    }
    if !valid_memory(memory) {
        bail!("invalid memory value: {memory:?}");
    }
    if !valid_user(user) {
        bail!("invalid user value: {user:?}");
    }
    Ok(())
}

pub fn normalize_session(session: Option<&str>) -> Result<String> {
    match session {
        Some(session) => validate_session_name(session),
        None => Ok(SESSION_DEFAULT.to_string()),
    }
}

pub fn network_egress_summary(
    network: NetworkMode,
    provider_allowed_hosts: &[String],
    api_key_broker: bool,
) -> String {
    match network {
        NetworkMode::Internet => {
            "unrestricted internet egress; domain allowlisting is not enforced".to_string()
        }
        NetworkMode::Internal => "host-only internal network; internet egress disabled".to_string(),
        NetworkMode::Provider => {
            let mut summary = format!(
                "provider allowlist egress through runtime proxy: {}",
                provider_allowed_hosts.join(", ")
            );
            if api_key_broker {
                summary.push_str("; API key broker enabled");
            }
            summary
        }
    }
}

/// Plain-language notices for every lower-security choice active in a run.
///
/// Secure defaults produce no notices; each supported but less-secure option
/// adds one line so the tradeoff is visible at plan and run time.
pub fn security_notices(options: &RunOptions) -> Vec<String> {
    let mut notices = Vec::new();
    if options.network == NetworkMode::Internet {
        notices.push(
            "Unrestricted internet egress is enabled; untrusted workspace code can reach any host. Use --network provider (the agent's own hosts) or --network internal to restrict egress."
                .to_string(),
        );
    }
    if !options.env.is_empty() {
        notices.push(format!(
            "Host environment variable(s) {} are exposed to the agent and can be read by workspace code; prefer --network provider or internal to limit exfiltration.",
            options.env.join(", ")
        ));
    }
    if uses_root_identity(&options.user) {
        notices.push(
            "The agent runs as root inside the container, which weakens the non-root isolation boundary."
                .to_string(),
        );
    } else if options.user != "agent" {
        notices.push(format!(
            "The agent runs as container user {:?} instead of the default non-root agent user.",
            options.user
        ));
    }
    if !options.provider_hosts.is_empty() {
        notices.push(format!(
            "The provider allowlist is widened with: {}; each added host increases what the agent can reach.",
            options.provider_hosts.join(", ")
        ));
    }
    if options.allow_sensitive_workspace {
        notices.push(
            "--allow-sensitive-workspace permits mounting broad or credential-bearing host paths at /workspace."
                .to_string(),
        );
    }
    if options.image.is_some() {
        notices.push(
            "A custom --image is used; it may not follow RunHaven's non-root, read-only-root hardening."
                .to_string(),
        );
    }
    notices
}

/// Default network mode when the user does not pass `--network`: provider where
/// the agent's own hosts are bundled (the secure path that still works out of
/// the box), internet where no hosts are bundled so provider would be an
/// empty-allowlist dead end.
pub fn default_network_mode(profile: &AgentProfile) -> NetworkMode {
    if profile.provider_hosts.is_empty() {
        NetworkMode::Internet
    } else {
        NetworkMode::Provider
    }
}

pub fn uses_root_identity(user: &str) -> bool {
    user.split(':')
        .any(|part| part == "root" || part.parse::<u32>().is_ok_and(|id| id == 0))
}

pub fn validate_image_reference(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.starts_with('-')
        || value.contains(',')
        || value.chars().any(char::is_whitespace)
        || value.contains("://")
        || !value.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '/' | '@' | '+' | '-')
        })
    {
        bail!("invalid {label}: {value:?}");
    }
    Ok(())
}

fn valid_cpus(value: &str) -> bool {
    if value.is_empty() || value.starts_with('0') {
        return false;
    }
    let mut dot = false;
    for c in value.chars() {
        if c == '.' && !dot {
            dot = true;
        } else if !c.is_ascii_digit() {
            return false;
        }
    }
    !value.ends_with('.')
}

fn valid_memory(value: &str) -> bool {
    if value.is_empty() || value.starts_with('0') {
        return false;
    }
    let suffix = value.chars().last().unwrap_or_default();
    let digits = if matches!(
        suffix,
        'K' | 'M' | 'G' | 'T' | 'P' | 'k' | 'm' | 'g' | 't' | 'p'
    ) {
        &value[..value.len() - suffix.len_utf8()]
    } else {
        value
    };
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

fn valid_user(value: &str) -> bool {
    let mut parts = value.split(':');
    let first = parts.next().unwrap_or_default();
    let second = parts.next();
    if parts.next().is_some() || !valid_user_part(first, true) {
        return false;
    }
    second.is_none_or(|part| part.chars().all(|c| c.is_ascii_digit()))
}

fn valid_user_part(value: &str, allow_name: bool) -> bool {
    if value.is_empty() {
        return false;
    }
    if value.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    allow_name
        && value
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_roots_are_sensitive_workspace_roots() {
        let (root_paths, _secret) = sensitive_workspace_paths();
        // Each system root that resolves on the host must be a blocked root.
        for root in ["/usr", "/bin", "/sbin"] {
            if let Ok(canon) = Path::new(root).canonicalize() {
                assert!(
                    root_paths.contains(&canon),
                    "{root} ({canon:?}) should be a sensitive workspace root"
                );
            }
        }
    }

    #[test]
    fn validate_workspace_blocks_system_root_without_override() {
        let usr = Path::new("/usr").canonicalize().expect("/usr resolves");
        assert!(
            validate_workspace(&usr, false).is_err(),
            "/usr must require --allow-sensitive-workspace"
        );
        assert!(
            validate_workspace(&usr, true).is_ok(),
            "/usr is allowed with the explicit override"
        );
    }
}

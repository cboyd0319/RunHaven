use anyhow::{Result, bail};

pub fn require_string<'a>(value: Option<&'a serde_json::Value>, message: &str) -> Result<&'a str> {
    match value
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty())
    {
        Some(value) => Ok(value),
        None => bail!("{message}"),
    }
}

pub fn validate_run_id(run_id: &str) -> Result<()> {
    if run_id.is_empty()
        || run_id.starts_with('-')
        || run_id
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || matches!(c, '/' | '\\'))
    {
        bail!("invalid run id: {run_id:?}");
    }
    Ok(())
}

pub fn validate_runhaven_container_name(container_name: &str) -> Result<()> {
    if !container_name.starts_with("runhaven-") {
        bail!("active run container {container_name:?} is not a RunHaven-owned container");
    }
    if container_name.starts_with('-')
        || container_name
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || matches!(c, '/' | '\\' | ','))
    {
        bail!("invalid active run container name: {container_name:?}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_id_rejects_terminal_control_bytes() {
        assert!(validate_run_id("run-123").is_ok());
        assert!(validate_run_id("run-\u{1b}]2;bad\u{7}").is_err());
    }

    #[test]
    fn container_name_rejects_terminal_control_bytes() {
        assert!(validate_runhaven_container_name("runhaven-codex-project-run").is_ok());
        assert!(validate_runhaven_container_name("runhaven-\u{1b}]2;bad\u{7}").is_err());
    }
}

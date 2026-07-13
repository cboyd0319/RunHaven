use crate::contracts::{PlanWarning, RunPlanRequest};

use super::defaulted;

pub(super) fn plan_warnings(request: &RunPlanRequest, active_run_count: usize) -> Vec<PlanWarning> {
    let mut warnings = Vec::new();
    if active_run_count > 0 {
        warnings.push(warning(
            "active-runs",
            &active_runs_warning_message(active_run_count),
        ));
    }
    if active_run_count > 0 && material_memory_request(&defaulted(&request.memory, "4g")) {
        warnings.push(warning(
            "resource-memory",
            "This session may use a large amount of memory while other sessions are running.",
        ));
    }
    if request.network_mode == "internet" {
        warnings.push(warning(
            "full-internet",
            "Full internet lets the assistant connect to any internet service.",
        ));
    }
    if request.allow_sensitive_workspace {
        warnings.push(warning(
            "sensitive-workspace",
            "The selected folder may contain private files. The assistant can read them.",
        ));
    }
    if request.allow_root_user || matches!(request.user.as_str(), "root" | "0") {
        warnings.push(warning(
            "root-user",
            "The assistant will have full control inside its protected environment. This removes a normal safety limit.",
        ));
    }
    if !request.env_names.is_empty() {
        warnings.push(warning(
            "environment",
            "RunHaven will share the selected Mac settings with this session. Their values stay hidden.",
        ));
    }
    if request
        .image
        .as_deref()
        .is_some_and(|image| !image.trim().is_empty())
    {
        warnings.push(warning(
            "custom-image",
            "You chose a custom assistant environment that RunHaven has not reviewed.",
        ));
    }
    if !request.provider_hosts.is_empty() {
        warnings.push(warning(
            "provider-host",
            "The assistant can connect to the additional services you entered and their subdomains.",
        ));
    }
    if !request.worktree && !request.read_only_workspace {
        warnings.push(warning(
            "direct-workspace",
            "The assistant can change files in your original project folder. The safer choice is a protected copy.",
        ));
    }
    warnings
}

fn active_runs_warning_message(active_run_count: usize) -> String {
    let noun = if active_run_count == 1 {
        "session"
    } else {
        "sessions"
    };
    let verb = if active_run_count == 1 {
        "exists"
    } else {
        "exist"
    };
    format!(
        "{active_run_count} RunHaven {noun} already {verb}. Starting another may use more memory and battery."
    )
}

fn material_memory_request(memory: &str) -> bool {
    const TWO_GIB: u64 = 2 * 1024 * 1024 * 1024;
    memory_bytes(memory).is_some_and(|bytes| bytes >= TWO_GIB)
}

fn memory_bytes(memory: &str) -> Option<u64> {
    let memory = memory.trim();
    if memory.is_empty() {
        return None;
    }
    let suffix = memory.chars().last()?;
    let (digits, multiplier) = match suffix {
        'K' | 'k' => (&memory[..memory.len() - suffix.len_utf8()], 1024),
        'M' | 'm' => (&memory[..memory.len() - suffix.len_utf8()], 1024_u64.pow(2)),
        'G' | 'g' => (&memory[..memory.len() - suffix.len_utf8()], 1024_u64.pow(3)),
        'T' | 't' => (&memory[..memory.len() - suffix.len_utf8()], 1024_u64.pow(4)),
        _ => (memory, 1),
    };
    digits.parse::<u64>().ok()?.checked_mul(multiplier)
}

fn warning(code: &str, message: &str) -> PlanWarning {
    PlanWarning {
        code: code.to_string(),
        message: message.to_string(),
    }
}

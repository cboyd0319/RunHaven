use runhaven_core::runtime::plans::{AgentRunPlan, AuthScope};

use crate::contracts::RunPlanAccess;

pub(super) fn plan_access(plan: &AgentRunPlan) -> RunPlanAccess {
    if let Some(name) = &plan.api_key_broker_env {
        let present = std::env::var_os(name).is_some_and(|value| !value.is_empty());
        if present {
            return RunPlanAccess {
                status: "ready".to_string(),
                label: "API key variable found".to_string(),
                detail: format!(
                    "{name} is set for this app. RunHaven keeps its value on this Mac and gives the session a placeholder."
                ),
                remedy: String::new(),
                blocking: false,
                api_key_broker_env: Some(name.clone()),
                api_key_broker_env_present: Some(true),
            };
        }
        return RunPlanAccess {
            status: "blocked".to_string(),
            label: "API key variable missing".to_string(),
            detail: format!("{name} is not set for this RunHaven app."),
            remedy: format!(
                "Set {name} before opening RunHaven, or remove the API key variable and sign in inside the session."
            ),
            blocking: true,
            api_key_broker_env: Some(name.clone()),
            api_key_broker_env_present: Some(false),
        };
    }

    if plan.profile_name == "shell" {
        return RunPlanAccess {
            status: "not-needed".to_string(),
            label: "No sign-in needed".to_string(),
            detail: "This shell profile does not need RunHaven-managed sign-in.".to_string(),
            remedy: String::new(),
            blocking: false,
            api_key_broker_env: None,
            api_key_broker_env_present: None,
        };
    }

    match plan.auth_scope {
        AuthScope::Agent => RunPlanAccess {
            status: "setup".to_string(),
            label: "Shared agent login".to_string(),
            detail: format!(
                "RunHaven will use the shared {} login state across projects on this Mac.",
                plan.profile_name
            ),
            remedy: format!(
                "If the agent asks for sign-in, finish it in the session or run `runhaven login {}`.",
                plan.profile_name
            ),
            blocking: false,
            api_key_broker_env: None,
            api_key_broker_env_present: None,
        },
        AuthScope::Project => RunPlanAccess {
            status: "setup".to_string(),
            label: "Project session login".to_string(),
            detail: "RunHaven will keep login state separate for this project session.".to_string(),
            remedy: "If the agent asks for sign-in, finish it inside this session.".to_string(),
            blocking: false,
            api_key_broker_env: None,
            api_key_broker_env_present: None,
        },
    }
}

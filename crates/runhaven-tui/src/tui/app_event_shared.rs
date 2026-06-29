//! Shared type bridge for activating the vendored Codex `app_event.rs`.
//!
//! These are narrow inert leaf types whose owning upstream modules are still
//! dormant because they carry broader app, chat, filesystem, or history
//! behavior. Remove this bridge as the real modules are promoted.

pub(crate) mod app {
    pub(crate) mod app_server_requests {
        use codex_app_server_protocol::RequestId as AppServerRequestId;

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub(crate) enum ResolvedAppServerRequest {
            ExecApproval {
                id: String,
            },
            FileChangeApproval {
                id: String,
            },
            PermissionsApproval {
                id: String,
            },
            UserInput {
                call_id: String,
            },
            McpElicitation {
                server_name: String,
                request_id: AppServerRequestId,
            },
        }
    }
}

pub(crate) mod app_server_session {
    use codex_app_server_protocol::Turn;

    #[derive(Debug)]
    pub(crate) struct ThreadSessionState;

    #[derive(Debug)]
    pub(crate) struct AppServerStartedThread {
        pub(crate) session: ThreadSessionState,
        pub(crate) turns: Vec<Turn>,
    }
}

pub(crate) mod chatwidget {
    use codex_protocol::user_input::TextElement;

    #[derive(Debug, Clone, PartialEq)]
    pub(crate) struct UserMessage {
        pub(crate) text: String,
        pub(crate) local_images: Vec<()>,
        pub(crate) remote_image_urls: Vec<String>,
        pub(crate) text_elements: Vec<TextElement>,
        pub(crate) mention_bindings: Vec<()>,
    }
}

pub(crate) mod goal_files {
    use codex_protocol::user_input::TextElement;

    #[derive(Clone, Debug, Default)]
    pub(crate) struct GoalDraft {
        pub(crate) objective: String,
        pub(crate) text_elements: Vec<TextElement>,
        pub(crate) pending_pastes: Vec<(String, String)>,
        pub(crate) local_images: Vec<()>,
        pub(crate) remote_image_urls: Vec<String>,
    }
}

pub(crate) mod hooks_rpc {
    use codex_app_server_protocol::HookMetadata;
    use codex_app_server_protocol::HookTrustStatus;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct HookTrustUpdate {
        pub(crate) key: String,
        pub(crate) current_hash: String,
    }

    pub(crate) fn hook_needs_review(hook: &HookMetadata) -> bool {
        matches!(
            hook.trust_status,
            HookTrustStatus::Untrusted | HookTrustStatus::Modified
        )
    }
}

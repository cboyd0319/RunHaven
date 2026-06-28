//! Shared type bridge for activating the vendored Codex `app_event.rs`.
//!
//! These are narrow inert leaf types whose owning upstream modules are still
//! dormant because they carry broader app, chat, filesystem, or history
//! behavior. Remove this bridge as the real modules are promoted.

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

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub(crate) struct StatusLineGitSummary;
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

pub(crate) mod history_cell {
    pub(crate) trait HistoryCell: std::fmt::Debug + Send + Sync {}
}

pub(crate) mod hooks_rpc {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct HookTrustUpdate;
}

pub(crate) mod workspace_messages {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct WorkspaceHeadlineFetchResult;
}

pub(crate) mod session_log {
    use crate::app_command::AppCommand;
    use crate::app_event::AppEvent;

    pub(crate) fn log_inbound_app_event(_event: &AppEvent) {}

    pub(crate) fn log_outbound_op(_op: &AppCommand) {}

    pub(crate) fn log_session_end() {}
}

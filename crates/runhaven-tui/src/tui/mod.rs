use anyhow::Result;

#[allow(dead_code)]
pub(crate) mod app_command;
#[allow(dead_code)]
pub(crate) mod app_event;
#[allow(dead_code)]
mod app_event_shared;
#[allow(dead_code)]
pub(crate) mod app_server_approval_conversions;
mod app_shell;
mod runhaven;

#[allow(dead_code)]
pub(crate) mod color;
#[allow(dead_code)]
pub(crate) mod custom_terminal;

pub(crate) use app_event_shared::app;
pub(crate) use app_event_shared::app_server_session;
pub(crate) use app_event_shared::chatwidget;
pub(crate) use app_event_shared::goal_files;
pub(crate) use app_event_shared::hooks_rpc;

#[allow(dead_code)]
pub(crate) mod app_event_sender;

#[allow(dead_code, unused_imports)]
pub(crate) mod bottom_pane;
#[allow(dead_code)]
pub(crate) mod branch_summary;

#[allow(dead_code)]
pub(crate) mod clipboard_paste;

#[allow(dead_code)]
pub(crate) mod approval_events;
#[allow(dead_code)]
pub(crate) mod diff_model;
#[allow(dead_code)]
pub(crate) mod diff_render;
#[allow(dead_code, unused_imports)]
pub(crate) mod exec_cell;
#[allow(dead_code)]
pub(crate) mod exec_command;

#[allow(dead_code, unused_imports)]
pub(crate) mod key_hint;
#[allow(dead_code)]
pub(crate) mod keymap;
#[allow(dead_code)]
pub(crate) mod line_truncation;
#[allow(dead_code)]
pub(crate) mod live_wrap;
#[allow(dead_code)]
pub(crate) mod markdown;
#[allow(dead_code)]
pub(crate) mod markdown_render;
#[allow(dead_code)]
pub(crate) mod markdown_stream;
#[allow(dead_code)]
pub(crate) mod markdown_text_merge;
#[allow(dead_code)]
pub(crate) mod mention_codec;
#[allow(dead_code)]
pub(crate) mod motion;
#[allow(dead_code)]
pub(crate) mod notifications;
#[allow(dead_code)]
pub(crate) mod onboarding {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    pub(crate) fn mark_underlined_hyperlink(buf: &mut Buffer, area: Rect, url: &str) {
        crate::terminal_hyperlinks::mark_underlined_hyperlink(buf, area, url);
    }
}
#[allow(dead_code, unused_imports)]
pub(crate) mod pets;
#[allow(dead_code)]
pub(crate) mod render;
#[allow(dead_code)]
pub(crate) mod session_log;
#[allow(dead_code)]
pub(crate) mod session_state;
#[allow(dead_code)]
pub(crate) mod shimmer;
#[allow(dead_code)]
pub(crate) mod skills_helpers;
#[allow(dead_code)]
pub(crate) mod slash_command;
#[allow(dead_code)]
pub(crate) mod status_indicator_widget;
#[allow(dead_code)]
pub(crate) mod style;
#[allow(dead_code)]
pub(crate) mod table_detect;
#[allow(dead_code)]
pub(crate) mod terminal_hyperlinks;
#[allow(dead_code)]
pub(crate) mod terminal_palette;
#[allow(dead_code)]
pub(crate) mod terminal_probe;
#[allow(dead_code)]
pub(crate) mod terminal_title;
#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod test_backend;
#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod test_support;
#[allow(dead_code)]
pub(crate) mod text_formatting;
#[allow(dead_code)]
pub(crate) mod token_usage;
#[allow(dead_code)]
pub(crate) mod tooltips;
#[allow(dead_code)]
pub(crate) mod ui_consts;
#[allow(dead_code)]
pub(crate) mod update_action;
#[allow(dead_code)]
pub(crate) mod version;
#[allow(dead_code)]
pub(crate) mod width;
#[allow(dead_code)]
pub(crate) mod workspace_command;
#[allow(dead_code)]
pub(crate) mod workspace_messages;
#[allow(dead_code)]
pub(crate) mod wrapping;

#[allow(dead_code, unused_imports)]
pub(crate) mod history_cell;
#[allow(dead_code)]
pub(crate) mod insert_history;

#[allow(dead_code)]
#[path = "tui.rs"]
pub(crate) mod codex_runtime;

pub use codex_runtime::FrameRequester;

pub fn run() -> Result<i32> {
    if let Some(exit_code) = runhaven::terminal_handoff::run_smoke_from_env()? {
        return Ok(exit_code);
    }

    app_shell::run()
}

#[cfg(test)]
mod drift_tests;

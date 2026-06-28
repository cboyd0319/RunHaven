extern crate self as codex_terminal_detection;

mod tui;

#[cfg(all(test, feature = "codex-vendored-tests"))]
pub(crate) use tui::app_event;
pub(crate) use tui::app_event_sender;
#[cfg(all(test, feature = "codex-vendored-tests"))]
pub(crate) use tui::bottom_pane;
pub(crate) use tui::clipboard_paste;
pub(crate) use tui::custom_terminal;
pub(crate) use tui::insert_history;
pub(crate) use tui::key_hint;
pub(crate) use tui::keymap;
pub(crate) use tui::line_truncation;
pub(crate) use tui::notifications;
pub(crate) use tui::pets;
pub(crate) use tui::render;
pub(crate) use tui::status;
pub(crate) use tui::style;
pub use tui::terminal_detection::Multiplexer;
pub use tui::terminal_detection::TerminalInfo;
pub use tui::terminal_detection::TerminalName;
pub use tui::terminal_detection::terminal_info;
pub(crate) use tui::terminal_hyperlinks;
pub(crate) use tui::terminal_palette;
pub(crate) use tui::terminal_probe;
#[cfg(test)]
pub(crate) use tui::test_backend;
pub(crate) use tui::ui_consts;
pub(crate) use tui::wrapping;

pub use tui::run;

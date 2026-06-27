mod tui;

#[cfg(all(test, feature = "codex-vendored-tests"))]
pub(crate) use tui::app_event;
pub(crate) use tui::app_event_sender;
#[cfg(all(test, feature = "codex-vendored-tests"))]
pub(crate) use tui::bottom_pane;
pub(crate) use tui::clipboard_paste;
pub(crate) use tui::key_hint;
pub(crate) use tui::keymap;
pub(crate) use tui::line_truncation;
pub(crate) use tui::render;
pub(crate) use tui::style;
pub(crate) use tui::wrapping;

pub use tui::run;

use anyhow::Result;

mod app_shell;
mod runhaven;

#[allow(dead_code)]
pub(crate) mod color;

#[allow(dead_code)]
pub(crate) mod app_event {
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[allow(clippy::enum_variant_names)]
    pub(crate) enum AppEvent {
        OpenApprovalsPopup,
        PetPreviewRequested { pet_id: String },
        PetSelected { pet_id: String },
        PetDisabled,
    }
}

#[allow(dead_code)]
pub(crate) mod app_event_sender {
    use super::app_event::AppEvent;
    use tokio::sync::mpsc::UnboundedSender;

    #[derive(Clone, Debug, Default)]
    pub(crate) struct AppEventSender {
        app_event_tx: Option<UnboundedSender<AppEvent>>,
    }

    impl AppEventSender {
        pub(crate) fn new(app_event_tx: UnboundedSender<AppEvent>) -> Self {
            Self {
                app_event_tx: Some(app_event_tx),
            }
        }

        pub(crate) fn send(&self, event: AppEvent) {
            if let Some(app_event_tx) = &self.app_event_tx {
                let _ = app_event_tx.send(event);
            }
        }
    }
}

#[allow(dead_code, unused_imports)]
pub(crate) mod bottom_pane {
    use crossterm::event::KeyEvent;

    use super::render::renderable::Renderable;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum CancellationEvent {
        Handled,
        NotHandled,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum ViewCompletion {
        Accepted,
        Cancelled,
    }

    pub(crate) trait BottomPaneView: Renderable {
        fn handle_key_event(&mut self, _key_event: KeyEvent) {}

        fn is_complete(&self) -> bool {
            false
        }

        fn completion(&self) -> Option<ViewCompletion> {
            None
        }

        fn dismiss_after_child_accept(&self) -> bool {
            false
        }

        fn clear_dismiss_after_child_accept(&mut self) {}

        fn view_id(&self) -> Option<&'static str> {
            None
        }

        fn selected_index(&self) -> Option<usize> {
            None
        }

        fn active_tab_id(&self) -> Option<&str> {
            None
        }

        fn on_ctrl_c(&mut self) -> CancellationEvent {
            CancellationEvent::NotHandled
        }

        fn prefer_esc_to_handle_key_event(&self) -> bool {
            false
        }

        fn handle_paste(&mut self, _pasted: String) -> bool {
            false
        }
    }

    pub(crate) mod bottom_pane_view {
        pub(crate) use super::BottomPaneView;
        pub(crate) use super::ViewCompletion;
    }

    #[path = "list_selection_view.rs"]
    mod list_selection_view;
    #[path = "popup_consts.rs"]
    pub(crate) mod popup_consts;
    #[path = "scroll_state.rs"]
    mod scroll_state;
    #[path = "selection_popup_common.rs"]
    mod selection_popup_common;
    #[path = "selection_tabs.rs"]
    mod selection_tabs;

    pub(crate) use list_selection_view::ColumnWidthMode;
    pub(crate) use list_selection_view::ListSelectionView;
    pub(crate) use list_selection_view::OnSelectionChangedCallback;
    pub(crate) use list_selection_view::SelectionAction;
    pub(crate) use list_selection_view::SelectionItem;
    pub(crate) use list_selection_view::SelectionRowDisplay;
    pub(crate) use list_selection_view::SelectionViewParams;
    pub(crate) use list_selection_view::SideContentWidth;
}

#[allow(dead_code)]
pub(crate) mod clipboard_paste {
    pub(crate) fn normalize_pasted_search_query(pasted: &str) -> Option<String> {
        let normalized = pasted.split_whitespace().collect::<Vec<_>>().join(" ");
        (!normalized.is_empty()).then_some(normalized)
    }
}

#[allow(dead_code, unused_imports)]
pub(crate) mod key_hint;
#[allow(dead_code)]
pub(crate) mod keymap {
    use crossterm::event::KeyCode;

    use super::key_hint;
    use super::key_hint::KeyBinding;

    #[derive(Clone, Debug)]
    pub(crate) struct RuntimeKeymap {
        pub(crate) list: ListKeymap,
    }

    impl RuntimeKeymap {
        pub(crate) fn defaults() -> Self {
            Self {
                list: ListKeymap::defaults(),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub(crate) struct ListKeymap {
        pub(crate) move_up: Vec<KeyBinding>,
        pub(crate) move_down: Vec<KeyBinding>,
        pub(crate) move_left: Vec<KeyBinding>,
        pub(crate) move_right: Vec<KeyBinding>,
        pub(crate) page_up: Vec<KeyBinding>,
        pub(crate) page_down: Vec<KeyBinding>,
        pub(crate) jump_top: Vec<KeyBinding>,
        pub(crate) jump_bottom: Vec<KeyBinding>,
        pub(crate) accept: Vec<KeyBinding>,
        pub(crate) cancel: Vec<KeyBinding>,
    }

    impl ListKeymap {
        fn defaults() -> Self {
            Self {
                move_up: vec![
                    key_hint::plain(KeyCode::Up),
                    key_hint::ctrl(KeyCode::Char('p')),
                    key_hint::ctrl(KeyCode::Char('k')),
                    key_hint::plain(KeyCode::Char('k')),
                ],
                move_down: vec![
                    key_hint::plain(KeyCode::Down),
                    key_hint::ctrl(KeyCode::Char('n')),
                    key_hint::ctrl(KeyCode::Char('j')),
                    key_hint::plain(KeyCode::Char('j')),
                ],
                move_left: vec![
                    key_hint::plain(KeyCode::Left),
                    key_hint::ctrl(KeyCode::Char('h')),
                ],
                move_right: vec![
                    key_hint::plain(KeyCode::Right),
                    key_hint::ctrl(KeyCode::Char('l')),
                ],
                page_up: vec![
                    key_hint::plain(KeyCode::PageUp),
                    key_hint::ctrl(KeyCode::Char('b')),
                ],
                page_down: vec![
                    key_hint::plain(KeyCode::PageDown),
                    key_hint::ctrl(KeyCode::Char('f')),
                ],
                jump_top: vec![key_hint::plain(KeyCode::Home)],
                jump_bottom: vec![key_hint::plain(KeyCode::End)],
                accept: vec![key_hint::plain(KeyCode::Enter)],
                cancel: vec![key_hint::plain(KeyCode::Esc)],
            }
        }
    }

    pub(crate) fn primary_binding(bindings: &[KeyBinding]) -> Option<KeyBinding> {
        bindings.first().copied()
    }
}
#[allow(dead_code)]
pub(crate) mod line_truncation;
#[allow(dead_code)]
pub(crate) mod motion;
#[allow(dead_code, unused_imports)]
pub(crate) mod pets;
#[allow(dead_code)]
pub(crate) mod render {
    use ratatui::layout::Rect;

    #[path = "line_utils.rs"]
    pub(crate) mod line_utils;
    #[path = "renderable.rs"]
    pub(crate) mod renderable;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Insets {
        left: u16,
        top: u16,
        right: u16,
        bottom: u16,
    }

    impl Insets {
        pub fn tlbr(top: u16, left: u16, bottom: u16, right: u16) -> Self {
            Self {
                top,
                left,
                bottom,
                right,
            }
        }

        pub fn vh(v: u16, h: u16) -> Self {
            Self {
                top: v,
                left: h,
                bottom: v,
                right: h,
            }
        }
    }

    pub trait RectExt {
        fn inset(&self, insets: Insets) -> Rect;
    }

    impl RectExt for Rect {
        fn inset(&self, insets: Insets) -> Rect {
            let horizontal = insets.left.saturating_add(insets.right);
            let vertical = insets.top.saturating_add(insets.bottom);
            Rect {
                x: self.x.saturating_add(insets.left),
                y: self.y.saturating_add(insets.top),
                width: self.width.saturating_sub(horizontal),
                height: self.height.saturating_sub(vertical),
            }
        }
    }
}
#[allow(dead_code)]
pub(crate) mod shimmer;
#[allow(dead_code)]
pub(crate) mod style;
#[allow(dead_code)]
pub(crate) mod terminal_detection;
#[allow(dead_code)]
pub(crate) mod terminal_palette;
#[allow(dead_code)]
pub(crate) mod terminal_probe;
#[allow(dead_code)]
pub(crate) mod terminal_title;
#[allow(dead_code)]
pub(crate) mod text_formatting;
#[allow(dead_code)]
pub(crate) mod wrapping;

#[allow(dead_code)]
#[path = "tui/frame_rate_limiter.rs"]
mod frame_rate_limiter;
#[allow(dead_code)]
#[path = "tui/frame_requester.rs"]
mod frame_requester;

pub use frame_requester::FrameRequester;

pub fn run() -> Result<i32> {
    app_shell::run()
}

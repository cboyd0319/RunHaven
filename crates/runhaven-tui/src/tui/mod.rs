use anyhow::Result;

mod app_shell;
mod runhaven;

#[allow(dead_code)]
pub(crate) mod color;
#[allow(dead_code)]
pub(crate) mod custom_terminal;

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

    #[path = "footer.rs"]
    mod footer;
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
    #[path = "textarea.rs"]
    pub(crate) mod textarea;

    pub(crate) use footer::FooterKeyHints;
    pub(crate) use footer::FooterMode;
    pub(crate) use footer::FooterProps;
    pub(crate) use footer::footer_height;
    pub(crate) use footer::render_footer_from_props;
    pub(crate) use footer::render_footer_hint_items;
    pub(crate) use list_selection_view::ColumnWidthMode;
    pub(crate) use list_selection_view::ListSelectionView;
    pub(crate) use list_selection_view::OnSelectionChangedCallback;
    pub(crate) use list_selection_view::SelectionAction;
    pub(crate) use list_selection_view::SelectionItem;
    pub(crate) use list_selection_view::SelectionRowDisplay;
    pub(crate) use list_selection_view::SelectionViewParams;
    pub(crate) use list_selection_view::SideContentWidth;
    pub(crate) use selection_popup_common::menu_surface_inset;
    pub(crate) use selection_popup_common::render_menu_surface;
    pub(crate) use textarea::TextArea;
    pub(crate) use textarea::TextAreaState;
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
    use crossterm::event::KeyModifiers;

    use super::key_hint;
    use super::key_hint::KeyBinding;

    #[derive(Clone, Debug)]
    pub(crate) struct RuntimeKeymap {
        pub(crate) chat: ChatKeymap,
        pub(crate) composer: ComposerKeymap,
        pub(crate) editor: EditorKeymap,
        pub(crate) vim_normal: VimNormalKeymap,
        pub(crate) vim_operator: VimOperatorKeymap,
        pub(crate) vim_text_object: VimTextObjectKeymap,
        pub(crate) list: ListKeymap,
    }

    impl RuntimeKeymap {
        pub(crate) fn defaults() -> Self {
            Self {
                chat: ChatKeymap {
                    interrupt_turn: vec![key_hint::plain(KeyCode::Esc)],
                    decrease_reasoning_effort: vec![
                        key_hint::alt(KeyCode::Char(',')),
                        key_hint::shift(KeyCode::Down),
                    ],
                    increase_reasoning_effort: vec![
                        key_hint::alt(KeyCode::Char('.')),
                        key_hint::shift(KeyCode::Up),
                    ],
                    edit_queued_message: vec![
                        key_hint::alt(KeyCode::Up),
                        key_hint::shift(KeyCode::Left),
                    ],
                },
                composer: ComposerKeymap {
                    submit: vec![key_hint::plain(KeyCode::Enter)],
                    queue: vec![key_hint::plain(KeyCode::Tab)],
                    toggle_shortcuts: vec![
                        key_hint::plain(KeyCode::Char('?')),
                        key_hint::shift(KeyCode::Char('?')),
                    ],
                    history_search_previous: vec![key_hint::ctrl(KeyCode::Char('r'))],
                    history_search_next: vec![key_hint::ctrl(KeyCode::Char('s'))],
                },
                editor: EditorKeymap {
                    insert_newline: vec![
                        key_hint::ctrl(KeyCode::Char('j')),
                        key_hint::ctrl(KeyCode::Char('m')),
                        key_hint::plain(KeyCode::Enter),
                        key_hint::shift(KeyCode::Enter),
                        key_hint::alt(KeyCode::Enter),
                    ],
                    move_left: vec![
                        key_hint::plain(KeyCode::Left),
                        key_hint::ctrl(KeyCode::Char('b')),
                    ],
                    move_right: vec![
                        key_hint::plain(KeyCode::Right),
                        key_hint::ctrl(KeyCode::Char('f')),
                    ],
                    move_up: vec![
                        key_hint::plain(KeyCode::Up),
                        key_hint::ctrl(KeyCode::Char('p')),
                    ],
                    move_down: vec![
                        key_hint::plain(KeyCode::Down),
                        key_hint::ctrl(KeyCode::Char('n')),
                    ],
                    move_word_left: vec![
                        key_hint::alt(KeyCode::Char('b')),
                        KeyBinding::new(KeyCode::Left, KeyModifiers::ALT),
                        KeyBinding::new(KeyCode::Left, KeyModifiers::CONTROL),
                    ],
                    move_word_right: vec![
                        key_hint::alt(KeyCode::Char('f')),
                        KeyBinding::new(KeyCode::Right, KeyModifiers::ALT),
                        KeyBinding::new(KeyCode::Right, KeyModifiers::CONTROL),
                    ],
                    move_line_start: vec![
                        key_hint::plain(KeyCode::Home),
                        key_hint::ctrl(KeyCode::Char('a')),
                    ],
                    move_line_end: vec![
                        key_hint::plain(KeyCode::End),
                        key_hint::ctrl(KeyCode::Char('e')),
                    ],
                    delete_backward: vec![
                        key_hint::plain(KeyCode::Backspace),
                        key_hint::shift(KeyCode::Backspace),
                        key_hint::ctrl(KeyCode::Char('h')),
                    ],
                    delete_forward: vec![
                        key_hint::plain(KeyCode::Delete),
                        key_hint::shift(KeyCode::Delete),
                        key_hint::ctrl(KeyCode::Char('d')),
                    ],
                    delete_backward_word: vec![
                        key_hint::alt(KeyCode::Backspace),
                        key_hint::ctrl(KeyCode::Backspace),
                        KeyBinding::new(
                            KeyCode::Backspace,
                            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                        ),
                        key_hint::ctrl(KeyCode::Char('w')),
                        KeyBinding::new(
                            KeyCode::Char('h'),
                            KeyModifiers::CONTROL | KeyModifiers::ALT,
                        ),
                    ],
                    delete_forward_word: vec![
                        key_hint::alt(KeyCode::Delete),
                        key_hint::ctrl(KeyCode::Delete),
                        KeyBinding::new(
                            KeyCode::Delete,
                            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                        ),
                        key_hint::alt(KeyCode::Char('d')),
                    ],
                    kill_line_start: vec![key_hint::ctrl(KeyCode::Char('u'))],
                    kill_whole_line: Vec::new(),
                    kill_line_end: vec![key_hint::ctrl(KeyCode::Char('k'))],
                    yank: vec![key_hint::ctrl(KeyCode::Char('y'))],
                },
                vim_normal: VimNormalKeymap::defaults(),
                vim_operator: VimOperatorKeymap::defaults(),
                vim_text_object: VimTextObjectKeymap::defaults(),
                list: ListKeymap::defaults(),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub(crate) struct ChatKeymap {
        pub(crate) interrupt_turn: Vec<KeyBinding>,
        pub(crate) decrease_reasoning_effort: Vec<KeyBinding>,
        pub(crate) increase_reasoning_effort: Vec<KeyBinding>,
        pub(crate) edit_queued_message: Vec<KeyBinding>,
    }

    #[derive(Clone, Debug)]
    pub(crate) struct ComposerKeymap {
        pub(crate) submit: Vec<KeyBinding>,
        pub(crate) queue: Vec<KeyBinding>,
        pub(crate) toggle_shortcuts: Vec<KeyBinding>,
        pub(crate) history_search_previous: Vec<KeyBinding>,
        pub(crate) history_search_next: Vec<KeyBinding>,
    }

    #[derive(Clone, Debug)]
    pub(crate) struct EditorKeymap {
        pub(crate) insert_newline: Vec<KeyBinding>,
        pub(crate) move_left: Vec<KeyBinding>,
        pub(crate) move_right: Vec<KeyBinding>,
        pub(crate) move_up: Vec<KeyBinding>,
        pub(crate) move_down: Vec<KeyBinding>,
        pub(crate) move_word_left: Vec<KeyBinding>,
        pub(crate) move_word_right: Vec<KeyBinding>,
        pub(crate) move_line_start: Vec<KeyBinding>,
        pub(crate) move_line_end: Vec<KeyBinding>,
        pub(crate) delete_backward: Vec<KeyBinding>,
        pub(crate) delete_forward: Vec<KeyBinding>,
        pub(crate) delete_backward_word: Vec<KeyBinding>,
        pub(crate) delete_forward_word: Vec<KeyBinding>,
        pub(crate) kill_line_start: Vec<KeyBinding>,
        pub(crate) kill_whole_line: Vec<KeyBinding>,
        pub(crate) kill_line_end: Vec<KeyBinding>,
        pub(crate) yank: Vec<KeyBinding>,
    }

    #[derive(Clone, Debug)]
    pub(crate) struct VimNormalKeymap {
        pub(crate) enter_insert: Vec<KeyBinding>,
        pub(crate) append_after_cursor: Vec<KeyBinding>,
        pub(crate) append_line_end: Vec<KeyBinding>,
        pub(crate) insert_line_start: Vec<KeyBinding>,
        pub(crate) open_line_below: Vec<KeyBinding>,
        pub(crate) open_line_above: Vec<KeyBinding>,
        pub(crate) move_left: Vec<KeyBinding>,
        pub(crate) move_right: Vec<KeyBinding>,
        pub(crate) move_up: Vec<KeyBinding>,
        pub(crate) move_down: Vec<KeyBinding>,
        pub(crate) move_word_forward: Vec<KeyBinding>,
        pub(crate) move_word_backward: Vec<KeyBinding>,
        pub(crate) move_word_end: Vec<KeyBinding>,
        pub(crate) move_line_start: Vec<KeyBinding>,
        pub(crate) move_line_end: Vec<KeyBinding>,
        pub(crate) delete_char: Vec<KeyBinding>,
        pub(crate) substitute_char: Vec<KeyBinding>,
        pub(crate) delete_to_line_end: Vec<KeyBinding>,
        pub(crate) change_to_line_end: Vec<KeyBinding>,
        pub(crate) yank_line: Vec<KeyBinding>,
        pub(crate) paste_after: Vec<KeyBinding>,
        pub(crate) start_delete_operator: Vec<KeyBinding>,
        pub(crate) start_yank_operator: Vec<KeyBinding>,
        pub(crate) start_change_operator: Vec<KeyBinding>,
        pub(crate) cancel_operator: Vec<KeyBinding>,
    }

    impl VimNormalKeymap {
        fn defaults() -> Self {
            Self {
                enter_insert: vec![
                    key_hint::plain(KeyCode::Char('i')),
                    key_hint::plain(KeyCode::Insert),
                ],
                append_after_cursor: vec![key_hint::plain(KeyCode::Char('a'))],
                append_line_end: vec![
                    key_hint::shift(KeyCode::Char('a')),
                    key_hint::plain(KeyCode::Char('A')),
                ],
                insert_line_start: vec![
                    key_hint::shift(KeyCode::Char('i')),
                    key_hint::plain(KeyCode::Char('I')),
                ],
                open_line_below: vec![key_hint::plain(KeyCode::Char('o'))],
                open_line_above: vec![
                    key_hint::shift(KeyCode::Char('o')),
                    key_hint::plain(KeyCode::Char('O')),
                ],
                move_left: vec![
                    key_hint::plain(KeyCode::Char('h')),
                    key_hint::plain(KeyCode::Left),
                ],
                move_right: vec![
                    key_hint::plain(KeyCode::Char('l')),
                    key_hint::plain(KeyCode::Right),
                ],
                move_up: vec![
                    key_hint::plain(KeyCode::Char('k')),
                    key_hint::plain(KeyCode::Up),
                ],
                move_down: vec![
                    key_hint::plain(KeyCode::Char('j')),
                    key_hint::plain(KeyCode::Down),
                ],
                move_word_forward: vec![key_hint::plain(KeyCode::Char('w'))],
                move_word_backward: vec![key_hint::plain(KeyCode::Char('b'))],
                move_word_end: vec![key_hint::plain(KeyCode::Char('e'))],
                move_line_start: vec![key_hint::plain(KeyCode::Char('0'))],
                move_line_end: vec![
                    key_hint::plain(KeyCode::Char('$')),
                    key_hint::shift(KeyCode::Char('$')),
                ],
                delete_char: vec![key_hint::plain(KeyCode::Char('x'))],
                substitute_char: vec![key_hint::plain(KeyCode::Char('s'))],
                delete_to_line_end: vec![
                    key_hint::shift(KeyCode::Char('d')),
                    key_hint::plain(KeyCode::Char('D')),
                ],
                change_to_line_end: vec![
                    key_hint::shift(KeyCode::Char('c')),
                    key_hint::plain(KeyCode::Char('C')),
                ],
                yank_line: vec![
                    key_hint::shift(KeyCode::Char('y')),
                    key_hint::plain(KeyCode::Char('Y')),
                ],
                paste_after: vec![key_hint::plain(KeyCode::Char('p'))],
                start_delete_operator: vec![key_hint::plain(KeyCode::Char('d'))],
                start_yank_operator: vec![key_hint::plain(KeyCode::Char('y'))],
                start_change_operator: vec![key_hint::plain(KeyCode::Char('c'))],
                cancel_operator: vec![key_hint::plain(KeyCode::Esc)],
            }
        }
    }

    #[derive(Clone, Debug)]
    pub(crate) struct VimOperatorKeymap {
        pub(crate) delete_line: Vec<KeyBinding>,
        pub(crate) yank_line: Vec<KeyBinding>,
        pub(crate) motion_left: Vec<KeyBinding>,
        pub(crate) motion_right: Vec<KeyBinding>,
        pub(crate) motion_up: Vec<KeyBinding>,
        pub(crate) motion_down: Vec<KeyBinding>,
        pub(crate) motion_word_forward: Vec<KeyBinding>,
        pub(crate) motion_word_backward: Vec<KeyBinding>,
        pub(crate) motion_word_end: Vec<KeyBinding>,
        pub(crate) motion_line_start: Vec<KeyBinding>,
        pub(crate) motion_line_end: Vec<KeyBinding>,
        pub(crate) select_inner_text_object: Vec<KeyBinding>,
        pub(crate) select_around_text_object: Vec<KeyBinding>,
        pub(crate) cancel: Vec<KeyBinding>,
    }

    impl VimOperatorKeymap {
        fn defaults() -> Self {
            Self {
                delete_line: vec![key_hint::plain(KeyCode::Char('d'))],
                yank_line: vec![key_hint::plain(KeyCode::Char('y'))],
                motion_left: vec![key_hint::plain(KeyCode::Char('h'))],
                motion_right: vec![key_hint::plain(KeyCode::Char('l'))],
                motion_up: vec![key_hint::plain(KeyCode::Char('k'))],
                motion_down: vec![key_hint::plain(KeyCode::Char('j'))],
                motion_word_forward: vec![key_hint::plain(KeyCode::Char('w'))],
                motion_word_backward: vec![key_hint::plain(KeyCode::Char('b'))],
                motion_word_end: vec![key_hint::plain(KeyCode::Char('e'))],
                motion_line_start: vec![key_hint::plain(KeyCode::Char('0'))],
                motion_line_end: vec![
                    key_hint::plain(KeyCode::Char('$')),
                    key_hint::shift(KeyCode::Char('$')),
                ],
                select_inner_text_object: vec![key_hint::plain(KeyCode::Char('i'))],
                select_around_text_object: vec![key_hint::plain(KeyCode::Char('a'))],
                cancel: vec![key_hint::plain(KeyCode::Esc)],
            }
        }
    }

    #[derive(Clone, Debug)]
    pub(crate) struct VimTextObjectKeymap {
        pub(crate) word: Vec<KeyBinding>,
        pub(crate) big_word: Vec<KeyBinding>,
        pub(crate) parentheses: Vec<KeyBinding>,
        pub(crate) brackets: Vec<KeyBinding>,
        pub(crate) braces: Vec<KeyBinding>,
        pub(crate) double_quote: Vec<KeyBinding>,
        pub(crate) single_quote: Vec<KeyBinding>,
        pub(crate) backtick: Vec<KeyBinding>,
        pub(crate) cancel: Vec<KeyBinding>,
    }

    impl VimTextObjectKeymap {
        fn defaults() -> Self {
            Self {
                word: vec![key_hint::plain(KeyCode::Char('w'))],
                big_word: vec![
                    key_hint::shift(KeyCode::Char('w')),
                    key_hint::plain(KeyCode::Char('W')),
                ],
                parentheses: vec![
                    key_hint::plain(KeyCode::Char('(')),
                    key_hint::shift(KeyCode::Char('(')),
                    key_hint::plain(KeyCode::Char(')')),
                    key_hint::shift(KeyCode::Char(')')),
                    key_hint::plain(KeyCode::Char('b')),
                ],
                brackets: vec![
                    key_hint::plain(KeyCode::Char('[')),
                    key_hint::plain(KeyCode::Char(']')),
                ],
                braces: vec![
                    key_hint::plain(KeyCode::Char('{')),
                    key_hint::shift(KeyCode::Char('{')),
                    key_hint::plain(KeyCode::Char('}')),
                    key_hint::shift(KeyCode::Char('}')),
                    key_hint::shift(KeyCode::Char('b')),
                    key_hint::plain(KeyCode::Char('B')),
                ],
                double_quote: vec![
                    key_hint::plain(KeyCode::Char('"')),
                    key_hint::shift(KeyCode::Char('"')),
                ],
                single_quote: vec![key_hint::plain(KeyCode::Char('\''))],
                backtick: vec![key_hint::plain(KeyCode::Char('`'))],
                cancel: vec![key_hint::plain(KeyCode::Esc)],
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
#[allow(dead_code)]
pub(crate) mod notifications;
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
pub(crate) mod status {
    pub(crate) fn format_tokens_compact(value: i64) -> String {
        let value = value.max(0);
        if value == 0 {
            return "0".to_string();
        }
        if value < 1_000 {
            return value.to_string();
        }

        let value_f64 = value as f64;
        let (scaled, suffix) = if value >= 1_000_000_000_000 {
            (value_f64 / 1_000_000_000_000.0, "T")
        } else if value >= 1_000_000_000 {
            (value_f64 / 1_000_000_000.0, "B")
        } else if value >= 1_000_000 {
            (value_f64 / 1_000_000.0, "M")
        } else {
            (value_f64 / 1_000.0, "K")
        };

        let decimals = if scaled < 10.0 {
            2
        } else if scaled < 100.0 {
            1
        } else {
            0
        };

        let mut formatted = format!("{scaled:.decimals$}");
        if formatted.contains('.') {
            while formatted.ends_with('0') {
                formatted.pop();
            }
            if formatted.ends_with('.') {
                formatted.pop();
            }
        }

        format!("{formatted}{suffix}")
    }
}
#[allow(dead_code)]
pub(crate) mod terminal_detection;
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
#[allow(dead_code)]
pub(crate) mod text_formatting;
#[allow(dead_code)]
pub(crate) mod ui_consts;
#[allow(dead_code)]
pub(crate) mod wrapping;

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
mod drift_tests {
    fn inline_module_declarations(module_source: &str) -> Vec<String> {
        module_source
            .lines()
            .map(str::trim)
            .filter_map(|line| {
                ["pub(crate) mod ", "pub mod ", "mod "]
                    .iter()
                    .find_map(|prefix| {
                        line.strip_prefix(prefix)
                            .and_then(|rest| rest.strip_suffix(" {"))
                    })
            })
            .map(str::to_string)
            .collect()
    }

    fn module_declared(module_source: &str, module: &str) -> bool {
        let private_decl = format!("mod {module};");
        let crate_decl = format!("pub(crate) mod {module};");
        let public_decl = format!("pub mod {module};");
        module_source
            .lines()
            .map(str::trim)
            .any(|line| line == private_decl || line == crate_decl || line == public_decl)
    }

    fn assert_risky_markers_absent_when_active(
        module_source: &str,
        module: &str,
        source_path: &str,
        source: &str,
        markers: &[&str],
    ) {
        if !module_declared(module_source, module) {
            return;
        }

        for marker in markers {
            assert!(
                !source.contains(marker),
                "{module} is declared in tui/mod.rs, but {source_path} still contains risky upstream marker {marker:?}; remove or fail-close that behavior before activating the module"
            );
        }
    }

    #[test]
    fn staging_facade_inline_modules_do_not_grow() {
        let module_source = include_str!("mod.rs");
        let inline_modules = inline_module_declarations(module_source);

        assert_eq!(
            inline_modules,
            [
                "app_event",
                "app_event_sender",
                "bottom_pane",
                "bottom_pane_view",
                "clipboard_paste",
                "keymap",
                "render",
                "status",
                "drift_tests",
            ],
            "tui/mod.rs may shrink inline staging modules, but must not grow new stand-ins"
        );
    }

    #[test]
    fn protocol_crates_are_vendored_dependencies() {
        let module_source = include_str!("mod.rs");
        let manifest_source = include_str!("../../Cargo.toml");

        assert!(
            !module_declared(module_source, "codex_protocol"),
            "codex_protocol must be consumed from the vendored crate, not staged inside runhaven-tui"
        );
        assert!(
            manifest_source.contains("codex-protocol = { path = \"../codex/protocol\" }")
                && manifest_source.contains(
                    "codex-app-server-protocol = { path = \"../codex/app-server-protocol\" }"
                ),
            "runhaven-tui must depend on the real vendored Codex protocol crates"
        );
    }

    #[test]
    fn codex_self_aliases_do_not_grow() {
        let lib_source = include_str!("../lib.rs");
        let aliases = lib_source
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("extern crate self as codex_"))
            .collect::<Vec<_>>();

        assert_eq!(
            aliases,
            [
                "extern crate self as codex_config;",
                "extern crate self as codex_terminal_detection;",
            ],
            "do not add new codex_* self-aliases; vendor real Codex crates or shrink local shims"
        );
    }

    #[test]
    fn native_app_entrypoint_cannot_share_temporary_shell() {
        let module_source = include_str!("mod.rs");

        if module_declared(module_source, "app") {
            assert!(
                !module_source.contains("app_shell::run()"),
                "native app activation must move run() off the temporary app_shell entrypoint"
            );
        }
    }

    #[test]
    fn host_reaching_codex_surfaces_stay_dormant_until_sanitized() {
        let module_source = include_str!("mod.rs");

        assert_risky_markers_absent_when_active(
            module_source,
            "app",
            "app.rs",
            include_str!("app.rs"),
            &["std::env::vars().collect"],
        );
        assert_risky_markers_absent_when_active(
            module_source,
            "app_server_session",
            "app_server_session.rs",
            include_str!("app_server_session.rs"),
            &["mod fs;"],
        );
        assert_risky_markers_absent_when_active(
            module_source,
            "onboarding",
            "onboarding/auth.rs",
            include_str!("onboarding/auth.rs"),
            &["read_openai_api_key_from_env", "webbrowser::open"],
        );
        assert_risky_markers_absent_when_active(
            module_source,
            "local_chatgpt_auth",
            "local_chatgpt_auth.rs",
            include_str!("local_chatgpt_auth.rs"),
            &["OPENAI_API_KEY", "ChatGPT"],
        );
        assert_risky_markers_absent_when_active(
            module_source,
            "external_editor",
            "external_editor.rs",
            include_str!("external_editor.rs"),
            &["std::process::Command", "EDITOR"],
        );
        assert_risky_markers_absent_when_active(
            module_source,
            "clipboard_copy",
            "clipboard_copy.rs",
            include_str!("clipboard_copy.rs"),
            &["std::process::Command"],
        );
        assert_risky_markers_absent_when_active(
            module_source,
            "hooks_rpc",
            "hooks_rpc.rs",
            include_str!("hooks_rpc.rs"),
            &["hook", "Hook"],
        );
    }
}

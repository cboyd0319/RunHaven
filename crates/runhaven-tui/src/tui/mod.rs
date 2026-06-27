use anyhow::Result;

mod app_shell;

#[allow(dead_code)]
pub(crate) mod color;

#[allow(dead_code)]
pub(crate) mod app_event {
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[allow(clippy::enum_variant_names)]
    pub(crate) enum AppEvent {
        PetPreviewRequested { pet_id: String },
        PetSelected { pet_id: String },
        PetDisabled,
    }
}

#[allow(dead_code)]
pub(crate) mod app_event_sender {
    use super::app_event::AppEvent;

    #[derive(Clone, Debug, Default)]
    pub(crate) struct AppEventSender;

    impl AppEventSender {
        pub(crate) fn send(&self, _event: AppEvent) {}
    }
}

#[allow(dead_code)]
pub(crate) mod bottom_pane {
    use ratatui::text::Line;
    use ratatui::text::Span;

    use super::app_event_sender::AppEventSender;
    use super::key_hint::KeyBinding;
    use super::render::renderable::Renderable;

    pub(crate) mod popup_consts {
        use crossterm::event::KeyCode;
        use ratatui::text::Line;

        use crate::tui::key_hint;

        pub(crate) fn standard_popup_hint_line() -> Line<'static> {
            Line::from(vec![
                "Press ".into(),
                key_hint::plain(KeyCode::Enter).into(),
                " to confirm or ".into(),
                key_hint::plain(KeyCode::Esc).into(),
                " to go back".into(),
            ])
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub(crate) enum ColumnWidthMode {
        #[default]
        AutoVisible,
        AutoAllRows,
        Fixed,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub(crate) enum SelectionRowDisplay {
        #[default]
        Wrapped,
        SingleLine,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) enum SideContentWidth {
        Fixed(u16),
        Half,
    }

    impl Default for SideContentWidth {
        fn default() -> Self {
            Self::Fixed(0)
        }
    }

    pub(crate) type SelectionAction = Box<dyn Fn(&AppEventSender) + Send + Sync>;
    pub(crate) type SelectionToggleAction = dyn Fn(bool, &AppEventSender) + Send + Sync;
    pub(crate) type OnSelectionChangedCallback =
        Option<Box<dyn Fn(usize, &AppEventSender) + Send + Sync>>;
    pub(crate) type OnCancelCallback = Option<Box<dyn Fn(&AppEventSender) + Send + Sync>>;

    pub(crate) struct SelectionToggle {
        pub is_on: bool,
        pub action: Box<SelectionToggleAction>,
    }

    pub(crate) struct SelectionTab;

    #[derive(Default)]
    pub(crate) struct SelectionItem {
        pub name: String,
        pub name_prefix_spans: Vec<Span<'static>>,
        pub toggle: Option<SelectionToggle>,
        pub toggle_placeholder: Option<&'static str>,
        pub display_shortcut: Option<KeyBinding>,
        pub description: Option<String>,
        pub selected_description: Option<String>,
        pub is_current: bool,
        pub is_default: bool,
        pub is_disabled: bool,
        pub actions: Vec<SelectionAction>,
        pub dismiss_on_select: bool,
        pub dismiss_parent_on_child_accept: bool,
        pub search_value: Option<String>,
        pub disabled_reason: Option<String>,
    }

    pub(crate) struct SelectionViewParams {
        pub view_id: Option<&'static str>,
        pub title: Option<String>,
        pub subtitle: Option<String>,
        pub footer_note: Option<Line<'static>>,
        pub footer_hint: Option<Line<'static>>,
        pub tab_footer_hints: Vec<(String, Line<'static>)>,
        pub items: Vec<SelectionItem>,
        pub tabs: Vec<SelectionTab>,
        pub initial_tab_id: Option<String>,
        pub is_searchable: bool,
        pub search_placeholder: Option<String>,
        pub col_width_mode: ColumnWidthMode,
        pub row_display: SelectionRowDisplay,
        pub name_column_width: Option<usize>,
        pub header: Box<dyn Renderable>,
        pub initial_selected_idx: Option<usize>,
        pub side_content: Box<dyn Renderable>,
        pub side_content_width: SideContentWidth,
        pub side_content_min_width: u16,
        pub stacked_side_content: Option<Box<dyn Renderable>>,
        pub preserve_side_content_bg: bool,
        pub on_selection_changed: OnSelectionChangedCallback,
        pub allow_cancel: bool,
        pub on_cancel: OnCancelCallback,
    }

    impl Default for SelectionViewParams {
        fn default() -> Self {
            Self {
                view_id: None,
                title: None,
                subtitle: None,
                footer_note: None,
                footer_hint: None,
                tab_footer_hints: Vec::new(),
                items: Vec::new(),
                tabs: Vec::new(),
                initial_tab_id: None,
                is_searchable: false,
                search_placeholder: None,
                col_width_mode: ColumnWidthMode::AutoVisible,
                row_display: SelectionRowDisplay::Wrapped,
                name_column_width: None,
                header: Box::new(()),
                initial_selected_idx: None,
                side_content: Box::new(()),
                side_content_width: SideContentWidth::default(),
                side_content_min_width: 0,
                stacked_side_content: None,
                preserve_side_content_bg: false,
                on_selection_changed: None,
                allow_cancel: true,
                on_cancel: None,
            }
        }
    }
}

#[allow(dead_code, unused_imports)]
pub(crate) mod key_hint;
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

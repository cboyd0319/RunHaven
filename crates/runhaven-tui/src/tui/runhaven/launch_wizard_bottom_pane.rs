use std::sync::atomic::Ordering;

use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;

use crate::render::renderable::Renderable;
use crate::tui::bottom_pane::BottomPaneView;
use crate::tui::bottom_pane::CancellationEvent;
use crate::tui::bottom_pane::ViewCompletion;

use super::LAUNCH_WIZARD_VIEW_ID;
use super::LaunchWizardScreen;
use super::LaunchWizardView;
use super::rendering;

impl BottomPaneView for LaunchWizardView {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.completion.is_some() {
            return;
        }
        match self.screen {
            LaunchWizardScreen::ChooseWorkspace => self.handle_workspace_key(key_event),
            LaunchWizardScreen::ChooseAgent => self.handle_picker_key(key_event),
            LaunchWizardScreen::ReviewPlan => self.handle_review_key(key_event),
            LaunchWizardScreen::ConfirmLaunch => self.handle_confirm_key(key_event),
        }
    }

    fn is_complete(&self) -> bool {
        self.completion.is_some()
    }

    fn completion(&self) -> Option<ViewCompletion> {
        self.completion
    }

    fn selected_index(&self) -> Option<usize> {
        Some(self.selected_idx.load(Ordering::Relaxed))
    }

    fn view_id(&self) -> Option<&'static str> {
        Some(LAUNCH_WIZARD_VIEW_ID)
    }

    fn terminal_title(&self) -> Option<String> {
        Some(LaunchWizardView::terminal_title(self))
    }

    fn footer_status_line(&self) -> Option<Line<'static>> {
        Some(LaunchWizardView::footer_status_line(self))
    }

    fn accepts_text_input(&self) -> bool {
        self.confirm_accepts_text_input()
    }

    fn footer_help_items(&self) -> Option<Vec<(String, String)>> {
        let mut items = if self.confirm_accepts_text_input() {
            vec![
                ("esc".to_string(), "back".to_string()),
                ("enter".to_string(), "confirm".to_string()),
            ]
        } else if self.screen == LaunchWizardScreen::ChooseWorkspace {
            vec![
                ("up/down".to_string(), "choose".to_string()),
                ("enter".to_string(), "select".to_string()),
                ("q".to_string(), "quit".to_string()),
            ]
        } else if self.is_reviewing() || self.is_confirming() {
            vec![
                ("b".to_string(), "back".to_string()),
                ("esc".to_string(), "back".to_string()),
                ("enter".to_string(), "confirm".to_string()),
                ("q".to_string(), "quit".to_string()),
            ]
        } else {
            vec![
                ("up/down".to_string(), "choose".to_string()),
                ("enter".to_string(), "review".to_string()),
                ("q".to_string(), "quit".to_string()),
            ]
        };
        items.push(("?".to_string(), "hide help".to_string()));
        Some(items)
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.completion = Some(ViewCompletion::Cancelled);
        CancellationEvent::Handled
    }

    fn prefer_esc_to_handle_key_event(&self) -> bool {
        true
    }

    fn handle_paste(&mut self, pasted: String) -> bool {
        if !self.confirm_accepts_text_input() || pasted.is_empty() {
            return false;
        }
        LaunchWizardView::handle_paste(self, &pasted);
        true
    }
}

impl Renderable for LaunchWizardView {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        rendering::render(self, area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        rendering::desired_height(self, width)
    }

    fn cursor_pos(&self, area: Rect) -> Option<(u16, u16)> {
        self.confirm_cursor_position(area)
    }
}

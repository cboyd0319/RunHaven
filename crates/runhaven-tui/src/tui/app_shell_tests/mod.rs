use super::*;
use crate::tui::runhaven::service::confirm_required_preview_for_tests;
use crossterm::event::KeyModifiers;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use std::cell::Cell;
use std::rc::Rc;

mod launch;
mod navigation;
mod rendering;

fn confirm_required_shell_state() -> ShellState {
    ShellState::from_launch_wizard(LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![confirm_required_preview_for_tests()],
    ))
    .expect("state")
}

fn render_to_text(state: &mut ShellState, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test terminal");
    terminal.draw(|frame| render(frame, state)).expect("draw");
    terminal
        .backend()
        .buffer()
        .content()
        .chunks(width as usize)
        .map(|row| {
            row.iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    terminal
        .backend()
        .buffer()
        .content()
        .chunks(terminal.size().expect("terminal size").width as usize)
        .map(|row| {
            row.iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

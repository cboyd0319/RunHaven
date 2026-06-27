//! Terminal UI: the default interface when `runhaven` runs on a TTY with no
//! subcommand. It is a launcher and manager over the same profiles and planner
//! the CLI uses, never a replacement for the explicit CLI surface.
//!
//! This first slice is the scaffold: terminal setup via `ratatui::init` (which
//! also installs a panic hook that restores the terminal), a draw and key-event
//! loop, and a home screen listing the bundled agents. Later slices add the
//! agent picker, plan and egress review, the run dashboard, and brand graphics.

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListItem, Paragraph};
use ratatui::{DefaultTerminal, Frame};

use crate::profiles::profiles;

/// Launch the terminal UI. The terminal is restored on exit and on panic.
pub fn run() -> Result<i32> {
    let mut terminal = ratatui::init();
    let result = App::new().run(&mut terminal);
    ratatui::restore();
    result
}

struct App {
    agents: Vec<(&'static str, &'static str)>,
}

impl App {
    fn new() -> Self {
        let agents = profiles()
            .into_iter()
            .map(|profile| (profile.name, profile.description))
            .collect();
        Self { agents }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<i32> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
            {
                return Ok(0);
            }
        }
    }

    fn render(&self, frame: &mut Frame) {
        let [header, body, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        let title = Paragraph::new(Line::from("RunHaven".bold()))
            .block(Block::bordered().title(format!(" v{} ", env!("CARGO_PKG_VERSION"))))
            .centered();
        frame.render_widget(title, header);

        let items: Vec<ListItem> = self
            .agents
            .iter()
            .map(|(name, description)| ListItem::new(format!("{name:<12}  {description}")))
            .collect();
        let list = List::new(items).block(Block::bordered().title(" Agents "));
        frame.render_widget(list, body);

        let hint = Paragraph::new(Line::from("q quit".dim())).centered();
        frame.render_widget(hint, footer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn app_loads_all_agent_profiles() {
        let app = App::new();
        let names: Vec<&str> = app.agents.iter().map(|(name, _)| *name).collect();
        assert!(names.contains(&"claude"));
        assert!(names.contains(&"shell"));
        assert_eq!(app.agents.len(), 6);
    }

    #[test]
    fn home_screen_renders_without_panicking() {
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).expect("terminal");
        let app = App::new();
        terminal.draw(|frame| app.render(frame)).expect("draw");
    }
}

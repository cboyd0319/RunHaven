//! Terminal UI: the default interface when `runhaven` runs on a TTY with no
//! subcommand. It is a launcher and manager over the same profiles and planner
//! the CLI uses, never a replacement for the explicit CLI surface.
//!
//! Slices so far: the scaffold (terminal setup via `ratatui::init`, a draw and
//! key-event loop) and an agent picker (a navigable home list and a per-agent
//! detail screen). Later slices add workspace selection, plan and egress review,
//! the run dashboard, and brand graphics.

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};
use ratatui::{DefaultTerminal, Frame};

use super::app::{agent_broker, agent_sign_in};
use crate::plans::default_network_mode;
use crate::profiles::{AgentProfile, profiles};

mod mascot;

/// Launch the terminal UI. The terminal is restored on exit and on panic.
pub fn run() -> Result<i32> {
    let mut terminal = ratatui::init();
    let result = App::new().run(&mut terminal);
    ratatui::restore();
    result
}

#[derive(Clone, Copy)]
enum Screen {
    Home,
    Detail,
}

struct App {
    agents: Vec<AgentProfile>,
    list: ListState,
    screen: Screen,
}

impl App {
    fn new() -> Self {
        let agents = profiles();
        let mut list = ListState::default();
        if !agents.is_empty() {
            list.select(Some(0));
        }
        Self {
            agents,
            list,
            screen: Screen::Home,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<i32> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && let Some(code) = self.handle_key(key.code)
            {
                return Ok(code);
            }
        }
    }

    /// Handle a key press. Returns `Some(exit_code)` to quit, `None` to continue.
    fn handle_key(&mut self, code: KeyCode) -> Option<i32> {
        match self.screen {
            Screen::Home => match code {
                KeyCode::Char('q') | KeyCode::Esc => return Some(0),
                KeyCode::Down | KeyCode::Char('j') => self.select_next(),
                KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
                KeyCode::Enter | KeyCode::Char('l') => self.screen = Screen::Detail,
                _ => {}
            },
            Screen::Detail => match code {
                KeyCode::Char('q') => return Some(0),
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') => {
                    self.screen = Screen::Home;
                }
                _ => {}
            },
        }
        None
    }

    fn select_next(&mut self) {
        if self.agents.is_empty() {
            return;
        }
        let next = self.list.selected().unwrap_or(0) + 1;
        self.list.select(Some(next.min(self.agents.len() - 1)));
    }

    fn select_previous(&mut self) {
        let current = self.list.selected().unwrap_or(0);
        self.list.select(Some(current.saturating_sub(1)));
    }

    fn selected(&self) -> Option<&AgentProfile> {
        self.list.selected().and_then(|i| self.agents.get(i))
    }

    fn render(&mut self, frame: &mut Frame) {
        match self.screen {
            Screen::Home => self.render_home(frame),
            Screen::Detail => self.render_detail(frame),
        }
    }

    fn render_home(&mut self, frame: &mut Frame) {
        // Reserve rows for the agent list and footer, then show the largest
        // Cubby hero that still fits the banner.
        const RESERVED_ROWS: u16 = 11;
        let available = frame.area().height.saturating_sub(RESERVED_ROWS);
        let hero = mascot::hero_for_banner(available);

        let [banner, body, footer] = Layout::vertical([
            Constraint::Length(hero.cell_height()),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        render_banner(frame, banner, hero);

        let items: Vec<ListItem> = self
            .agents
            .iter()
            .map(|profile| ListItem::new(format!("{:<12}  {}", profile.name, profile.description)))
            .collect();
        let list = List::new(items)
            .block(Block::bordered().title(" Agents "))
            .highlight_symbol("> ")
            .highlight_style(Style::new().reversed());
        frame.render_stateful_widget(list, body, &mut self.list);

        let hint =
            Paragraph::new(Line::from("up/down move · enter select · q quit".dim())).centered();
        frame.render_widget(hint, footer);
    }

    fn render_detail(&self, frame: &mut Frame) {
        let [header, body, footer] = layout(frame);
        let Some(agent) = self.selected() else {
            return;
        };

        let title = Paragraph::new(Line::from(agent.name.bold()))
            .block(Block::bordered())
            .centered();
        frame.render_widget(title, header);

        let lines = vec![
            Line::from(agent.description),
            Line::from(""),
            Line::from(format!("image:           {}", agent.image)),
            Line::from(format!("sign-in:         {}", agent_sign_in(agent.name))),
            Line::from(format!(
                "default network: {}",
                default_network_mode(agent).as_str()
            )),
            Line::from(format!("api-key broker:  {}", agent_broker(agent.name))),
        ];
        let detail = Paragraph::new(Text::from(lines)).block(Block::bordered().title(" Agent "));
        frame.render_widget(detail, body);

        let hint = Paragraph::new(Line::from("esc back · q quit".dim())).centered();
        frame.render_widget(hint, footer);
    }
}

/// The home banner: Cubby on the left, brand and tagline on the right.
fn render_banner(frame: &mut Frame, area: Rect, hero: &mascot::HeroSprite) {
    let [mascot_area, brand_area] = Layout::horizontal([
        Constraint::Length(hero.cell_width() + 2),
        Constraint::Min(0),
    ])
    .areas(area);

    frame.render_widget(Paragraph::new(hero.lines()), mascot_area);

    // Vertically center the brand against the mascot.
    let brand = [
        Line::from("RunHaven".bold()),
        Line::from(format!("v{}", env!("CARGO_PKG_VERSION")).dim()),
        Line::from(""),
        Line::from("run agents in a safe haven".dim()),
    ];
    let pad = area.height.saturating_sub(brand.len() as u16) / 2;
    let mut lines = vec![Line::from(""); pad as usize];
    lines.extend(brand);
    frame.render_widget(Paragraph::new(lines), brand_area);
}

/// The shared three-row layout: a header, a flexible body, and a one-line hint.
fn layout(frame: &Frame) -> [Rect; 3] {
    Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn app_loads_all_agent_profiles() {
        let app = App::new();
        assert_eq!(app.agents.len(), 6);
        assert_eq!(app.list.selected(), Some(0));
    }

    #[test]
    fn home_banner_shows_mascot_and_brand() {
        let mut terminal = Terminal::new(TestBackend::new(60, 30)).unwrap();
        let mut app = App::new();
        terminal.draw(|f| app.render(f)).unwrap();
        let buf = terminal.backend().buffer();
        let mut text = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                text.push_str(buf[(x, y)].symbol());
            }
        }
        // The brand renders in the banner.
        assert!(text.contains("RunHaven"), "brand text missing");
        // Cubby drew a meaningful number of half-block pixels.
        let blocks = text.matches('\u{2580}').count() + text.matches('\u{2584}').count();
        assert!(blocks > 40, "expected mascot half-blocks, got {blocks}");
    }

    #[test]
    fn navigation_clamps_within_bounds() {
        let mut app = App::new();
        let last = app.agents.len() - 1;
        // Up at the top stays at 0.
        app.handle_key(KeyCode::Up);
        assert_eq!(app.list.selected(), Some(0));
        app.handle_key(KeyCode::Down);
        assert_eq!(app.list.selected(), Some(1));
        // Past the end clamps to the last row.
        for _ in 0..app.agents.len() + 3 {
            app.handle_key(KeyCode::Down);
        }
        assert_eq!(app.list.selected(), Some(last));
    }

    #[test]
    fn enter_opens_detail_and_esc_returns_home() {
        let mut app = App::new();
        assert!(matches!(app.screen, Screen::Home));
        app.handle_key(KeyCode::Enter);
        assert!(matches!(app.screen, Screen::Detail));
        app.handle_key(KeyCode::Esc);
        assert!(matches!(app.screen, Screen::Home));
    }

    #[test]
    fn q_quits_from_either_screen() {
        let mut app = App::new();
        assert_eq!(app.handle_key(KeyCode::Char('q')), Some(0));
        app.handle_key(KeyCode::Enter);
        assert_eq!(app.handle_key(KeyCode::Char('q')), Some(0));
    }

    #[test]
    fn both_screens_render_without_panicking() {
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).expect("terminal");
        let mut app = App::new();
        terminal.draw(|frame| app.render(frame)).expect("home");
        app.handle_key(KeyCode::Enter);
        terminal.draw(|frame| app.render(frame)).expect("detail");
    }
}

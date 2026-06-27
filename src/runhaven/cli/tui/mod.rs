//! Terminal UI: the default interface when `runhaven` runs on a TTY with no
//! subcommand. It is a launcher and manager over the same profiles and planner
//! the CLI uses, never a replacement for the explicit CLI surface.
//!
//! Slices so far: the scaffold, the agent picker, portable Cubby hero art, and
//! the Phase 0 foundation (theme/settings, poll-driven ticks, VT100 snapshots).
//! Later slices add workspace selection, plan and egress review, the run
//! dashboard, and high-fidelity brand graphics.

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::time::{Duration, Instant};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::app::{agent_broker, agent_sign_in};
use crate::plans::default_network_mode;
use crate::profiles::{AgentProfile, profiles};

mod codex;
mod color;
mod event_loop;
mod mascot;
#[cfg(test)]
mod snapshot;
#[cfg(test)]
mod test_backend;
mod theme;

use event_loop::{Tick, Ticker};
use theme::{Palette, TuiSettings};

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
    settings: TuiSettings,
    palette: Palette,
    screen: Screen,
    ticks: u64,
    last_tick_elapsed: Duration,
}

impl App {
    fn new() -> Self {
        Self::with_settings(TuiSettings::from_env())
    }

    fn with_settings(settings: TuiSettings) -> Self {
        let agents = profiles();
        let mut list = ListState::default();
        if !agents.is_empty() {
            list.select(Some(0));
        }
        let palette = Palette::for_settings(settings);
        Self {
            agents,
            list,
            settings,
            palette,
            screen: Screen::Home,
            ticks: 0,
            last_tick_elapsed: Duration::ZERO,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<i32> {
        let mut ticker = Ticker::new(Instant::now(), self.settings.tick_rate);
        loop {
            terminal.draw(|frame| self.render(frame))?;
            let now = Instant::now();
            if event::poll(ticker.timeout(now))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && let Some(code) = self.handle_key(key.code)
            {
                return Ok(code);
            }
            if let Some(tick) = ticker.tick(Instant::now()) {
                self.on_tick(tick);
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

    fn on_tick(&mut self, tick: Tick) {
        self.ticks = self.ticks.saturating_add(1);
        self.last_tick_elapsed = tick.elapsed;
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
        let hero = (!self.settings.line_mode).then(|| mascot::hero_for_banner(available));
        let banner_height = hero.map_or(4, mascot::HeroSprite::cell_height);

        let [banner, body, footer] = Layout::vertical([
            Constraint::Length(banner_height),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        if let Some(hero) = hero {
            render_banner(frame, banner, hero, self.settings, self.palette);
        } else {
            render_line_banner(frame, banner, self.palette);
        }

        let item_width =
            body.width
                .saturating_sub(if self.settings.line_mode { 2 } else { 4 }) as usize;
        let items: Vec<ListItem> = self
            .agents
            .iter()
            .map(|profile| agent_list_item(profile, item_width))
            .collect();
        let mut list = List::new(items)
            .highlight_symbol("> ")
            .highlight_style(self.palette.selected())
            .style(self.palette.text());
        if !self.settings.line_mode {
            list = list.block(
                Block::bordered()
                    .title(" Agents ")
                    .border_style(self.palette.border()),
            );
        }
        frame.render_stateful_widget(list, body, &mut self.list);

        let hint = Paragraph::new(Line::styled(
            "up/down move · enter select · q quit",
            self.palette.muted(),
        ))
        .centered();
        frame.render_widget(hint, footer);
    }

    fn render_detail(&self, frame: &mut Frame) {
        let [header, body, footer] = layout(frame);
        let Some(agent) = self.selected() else {
            return;
        };

        let mut title = Paragraph::new(Line::styled(agent.name, self.palette.accent())).centered();
        if !self.settings.line_mode {
            title = title.block(Block::bordered().border_style(self.palette.border()));
        }
        frame.render_widget(title, header);

        let lines = vec![
            Line::styled(agent.description, self.palette.text()),
            Line::from(""),
            Line::from(format!("image:           {}", agent.image)),
            Line::from(format!("sign-in:         {}", agent_sign_in(agent.name))),
            Line::from(format!(
                "default network: {}",
                default_network_mode(agent).as_str()
            )),
            Line::from(format!("api-key broker:  {}", agent_broker(agent.name))),
        ];
        let mut detail = Paragraph::new(Text::from(lines)).style(self.palette.text());
        if !self.settings.line_mode {
            detail = detail.block(
                Block::bordered()
                    .title(" Agent ")
                    .border_style(self.palette.border()),
            );
        }
        frame.render_widget(detail, body);

        let hint =
            Paragraph::new(Line::styled("esc back · q quit", self.palette.muted())).centered();
        frame.render_widget(hint, footer);
    }
}

/// The home banner: Cubby on the left, brand and tagline on the right.
fn render_banner(
    frame: &mut Frame,
    area: Rect,
    hero: &mascot::HeroSprite,
    settings: TuiSettings,
    palette: Palette,
) {
    let [mascot_area, brand_area] = Layout::horizontal([
        Constraint::Length(hero.cell_width() + 2),
        Constraint::Min(0),
    ])
    .areas(area);

    frame.render_widget(
        Paragraph::new(hero.lines_with_color(settings.color_enabled)),
        mascot_area,
    );

    // Vertically center the brand against the mascot.
    let brand = [
        Line::styled("RunHaven", palette.accent()),
        Line::styled(format!("v{}", env!("CARGO_PKG_VERSION")), palette.muted()),
        Line::from(""),
        Line::styled("run agents in a safe haven", palette.muted()),
    ];
    let pad = area.height.saturating_sub(brand.len() as u16) / 2;
    let mut lines = vec![Line::from(""); pad as usize];
    lines.extend(brand);
    frame.render_widget(Paragraph::new(lines), brand_area);
}

fn render_line_banner(frame: &mut Frame, area: Rect, palette: Palette) {
    let lines = vec![
        Line::styled("RunHaven", palette.accent()),
        Line::styled(format!("v{}", env!("CARGO_PKG_VERSION")), palette.muted()),
        Line::styled("run agents in a safe haven", palette.muted()),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

fn agent_list_item(profile: &AgentProfile, width: usize) -> ListItem<'static> {
    const NAME_WIDTH: usize = 12;
    const GAP: usize = 2;
    let prefix_width = NAME_WIDTH + GAP;
    let description_width = width.saturating_sub(prefix_width);
    let description = truncate_to_width(profile.description, description_width);
    ListItem::new(format!(
        "{:<name_width$}  {}",
        profile.name,
        description,
        name_width = NAME_WIDTH
    ))
}

fn truncate_to_width(text: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    if max_width == 0 {
        return String::new();
    }
    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let suffix = "...";
    let body_width = max_width - suffix.len();
    let mut out = String::new();
    let mut width = 0;
    for ch in text.chars() {
        let ch_width = ch.width().unwrap_or(0);
        if width + ch_width > body_width {
            break;
        }
        out.push(ch);
        width += ch_width;
    }
    out.push_str(suffix);
    out
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
    use ratatui::style::Color;

    fn test_app() -> App {
        App::with_settings(TuiSettings::default())
    }

    #[test]
    fn app_loads_all_agent_profiles() {
        let app = test_app();
        assert_eq!(app.agents.len(), 6);
        assert_eq!(app.list.selected(), Some(0));
    }

    #[test]
    fn home_banner_shows_mascot_and_brand() {
        let mut terminal = Terminal::new(TestBackend::new(60, 30)).unwrap();
        let mut app = test_app();
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
        let mut app = test_app();
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
        let mut app = test_app();
        assert!(matches!(app.screen, Screen::Home));
        app.handle_key(KeyCode::Enter);
        assert!(matches!(app.screen, Screen::Detail));
        app.handle_key(KeyCode::Esc);
        assert!(matches!(app.screen, Screen::Home));
    }

    #[test]
    fn q_quits_from_either_screen() {
        let mut app = test_app();
        assert_eq!(app.handle_key(KeyCode::Char('q')), Some(0));
        app.handle_key(KeyCode::Enter);
        assert_eq!(app.handle_key(KeyCode::Char('q')), Some(0));
    }

    #[test]
    fn both_screens_render_without_panicking() {
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).expect("terminal");
        let mut app = test_app();
        terminal.draw(|frame| app.render(frame)).expect("home");
        app.handle_key(KeyCode::Enter);
        terminal.draw(|frame| app.render(frame)).expect("detail");
    }

    #[test]
    fn no_color_rendering_leaves_color_cells_reset() {
        let settings = TuiSettings {
            color_enabled: false,
            ..TuiSettings::default()
        };
        let mut app = App::with_settings(settings);
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("terminal");

        terminal.draw(|frame| app.render(frame)).expect("home");
        let buf = terminal.backend().buffer();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let cell = &buf[(x, y)];
                assert_eq!(cell.fg, Color::Reset);
                assert_eq!(cell.bg, Color::Reset);
            }
        }
    }

    #[test]
    fn line_mode_uses_text_banner_without_mascot_blocks() {
        let settings = TuiSettings {
            line_mode: true,
            ..TuiSettings::default()
        };
        let mut app = App::with_settings(settings);
        let snapshot = snapshot::render_vt100(60, 20, |frame| app.render(frame)).unwrap();

        assert!(snapshot.contains("RunHaven"));
        assert!(!snapshot.contains('\u{2580}'));
        assert!(!snapshot.contains('\u{2584}'));
    }

    #[test]
    fn tick_updates_app_clock_state() {
        let mut app = test_app();
        app.on_tick(Tick {
            elapsed: Duration::from_millis(250),
        });

        assert_eq!(app.ticks, 1);
        assert_eq!(app.last_tick_elapsed, Duration::from_millis(250));
    }

    #[test]
    fn agent_list_items_truncate_with_ascii_affordance() {
        assert_eq!(truncate_to_width("abcdef", 6), "abcdef");
        assert_eq!(truncate_to_width("abcdef", 5), "ab...");
        assert_eq!(truncate_to_width("abcdef", 2), "..");
        assert_eq!(truncate_to_width("abcdef", 0), "");
    }

    #[test]
    fn home_snapshot_80x24() {
        let mut app = test_app();
        let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
        insta::assert_snapshot!("tui_home_80x24", snapshot);
    }

    #[test]
    fn home_snapshot_120x36() {
        let mut app = test_app();
        let snapshot = snapshot::render_vt100(120, 36, |frame| app.render(frame)).unwrap();
        insta::assert_snapshot!("tui_home_120x36", snapshot);
    }

    #[test]
    fn detail_snapshot_80x24() {
        let mut app = test_app();
        app.handle_key(KeyCode::Enter);
        let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
        insta::assert_snapshot!("tui_detail_80x24", snapshot);
    }
}

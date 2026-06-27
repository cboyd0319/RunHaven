//! Terminal UI: the default interface when `runhaven` runs on a TTY with no
//! subcommand. It is a launcher and manager over the same profiles and planner
//! the CLI uses, never a replacement for the explicit CLI surface.
//!
//! Slices so far: the scaffold, the agent picker, portable and high-resolution
//! Cubby branding, the Phase 0 foundation, and the Phase 1 pet/tooltips layer.
//! Later slices add the run dashboard and history/diagnostics surfaces.

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, List, ListState, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::app::{agent_broker, agent_sign_in};
use crate::launch::launch_run_plan;
use crate::plans::{AgentRunPlan, default_network_mode};
use crate::profiles::{AgentProfile, profiles};

mod codex;
mod color;
mod event_loop;
mod launcher;
mod mascot;
mod pet;
#[cfg(test)]
mod snapshot;
#[cfg(test)]
mod test_backend;
mod theme;
mod tooltips;
mod widgets;

use event_loop::{Tick, Ticker};
use theme::{MotionMode, Palette, TuiSettings};
use widgets::{
    agent_list_item, layout, plan_review_lines, push_wrapped_line, render_banner, render_footer,
    render_launcher_summary, render_line_banner, render_screen_body, render_screen_title,
    workspace_candidate_item,
};

/// Launch the terminal UI. The terminal is restored on exit and on panic.
pub fn run() -> Result<i32> {
    let mut terminal = ratatui::init();
    let action = App::new().run(&mut terminal);
    ratatui::restore();
    match action? {
        TuiAction::Exit(code) => Ok(code),
        TuiAction::Launch(plan) => launch_run_plan(&plan),
    }
}

#[derive(Clone, Copy)]
enum Screen {
    Home,
    Detail,
    Workspace,
    Plan,
    Confirm,
}

#[derive(Debug)]
enum TuiAction {
    Exit(i32),
    Launch(Box<AgentRunPlan>),
}

struct App {
    agents: Vec<AgentProfile>,
    list: ListState,
    launcher: launcher::LauncherState,
    settings: TuiSettings,
    palette: Palette,
    screen: Screen,
    ticks: u64,
    last_tick_elapsed: Duration,
    pet_animation_elapsed: Duration,
    pet: Option<pet::CubbyPet>,
    pet_image_protocol: Option<codex::image_protocol::ImageProtocol>,
    pending_pet_draw: Option<pet::PetImageDraw>,
    pet_image_state: pet::PetImageRenderState,
}

impl App {
    fn new() -> Self {
        let workspace = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::with_settings_and_workspace(TuiSettings::from_env(), workspace)
    }

    fn with_settings_and_workspace(settings: TuiSettings, workspace: PathBuf) -> Self {
        let agents = profiles();
        let mut list = ListState::default();
        if !agents.is_empty() {
            list.select(Some(0));
        }
        let palette = Palette::for_settings(settings);
        let pet = if settings.pet_enabled {
            pet::CubbyPet::load().ok()
        } else {
            None
        };
        let pet_image_protocol = pet::detect_image_protocol(settings);
        Self {
            agents,
            list,
            launcher: launcher::LauncherState::new(workspace),
            settings,
            palette,
            screen: Screen::Home,
            ticks: 0,
            last_tick_elapsed: Duration::ZERO,
            pet_animation_elapsed: Duration::ZERO,
            pet,
            pet_image_protocol,
            pending_pet_draw: None,
            pet_image_state: pet::PetImageRenderState::default(),
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<TuiAction> {
        let mut ticker = Ticker::new(Instant::now(), self.settings.tick_rate);
        loop {
            terminal.draw(|frame| self.render(frame))?;
            self.render_terminal_overlay();
            let now = Instant::now();
            if event::poll(ticker.timeout(now))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && let Some(code) = self.handle_key(key.code)
            {
                self.clear_terminal_overlay();
                return Ok(code);
            }
            if let Some(tick) = ticker.tick(Instant::now()) {
                self.on_tick(tick);
            }
        }
    }

    /// Handle a key press. Returns `Some(action)` to leave the TUI, `None` to continue.
    fn handle_key(&mut self, code: KeyCode) -> Option<TuiAction> {
        match self.screen {
            Screen::Home => match code {
                KeyCode::Char('q') | KeyCode::Esc => return Some(TuiAction::Exit(0)),
                KeyCode::Down | KeyCode::Char('j') => self.select_next(),
                KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
                KeyCode::Enter | KeyCode::Char('l') => self.screen = Screen::Detail,
                KeyCode::Char('r') => self.open_plan_review(),
                KeyCode::Char('w') => {
                    self.launcher.open_workspace_picker();
                    self.screen = Screen::Workspace;
                }
                KeyCode::Char('p') => self.toggle_pet(),
                _ => {}
            },
            Screen::Detail => match code {
                KeyCode::Char('q') => return Some(TuiAction::Exit(0)),
                KeyCode::Enter | KeyCode::Char('r') => self.open_plan_review(),
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') => {
                    self.screen = Screen::Home;
                }
                _ => {}
            },
            Screen::Workspace => match code {
                KeyCode::Char('q') => return Some(TuiAction::Exit(0)),
                KeyCode::Esc => self.screen = Screen::Home,
                KeyCode::Down | KeyCode::Char('j') => self.launcher.workspace_picker.select_next(),
                KeyCode::Up | KeyCode::Char('k') => {
                    self.launcher.workspace_picker.select_previous()
                }
                KeyCode::Backspace => self.launcher.workspace_picker.pop_query_char(),
                KeyCode::Enter => {
                    if let Err(error) = self.launcher.confirm_workspace_selection() {
                        self.launcher.plan_error = Some(error.to_string());
                    }
                    self.screen = Screen::Home;
                }
                KeyCode::Char(ch) => self.launcher.workspace_picker.push_query_char(ch),
                _ => {}
            },
            Screen::Plan => match code {
                KeyCode::Char('q') => return Some(TuiAction::Exit(0)),
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') => {
                    self.screen = Screen::Home
                }
                KeyCode::Char('w') => {
                    self.launcher.open_workspace_picker();
                    self.screen = Screen::Workspace;
                }
                KeyCode::Enter if self.launcher.review.is_some() => {
                    self.launcher.confirm_input.clear();
                    self.screen = Screen::Confirm;
                }
                _ => {}
            },
            Screen::Confirm => match code {
                KeyCode::Char('q') | KeyCode::Esc => self.screen = Screen::Plan,
                KeyCode::Backspace => {
                    self.launcher.confirm_input.pop();
                }
                KeyCode::Enter if self.launcher.confirm_ready() => {
                    let plan = self.launcher.launch_plan()?;
                    return Some(TuiAction::Launch(Box::new(plan)));
                }
                KeyCode::Char(ch) if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' => {
                    self.launcher.confirm_input.push(ch);
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

    fn open_plan_review(&mut self) {
        if let Some(agent) = self.selected().cloned() {
            self.launcher.build_review(&agent);
            self.screen = Screen::Plan;
        }
    }

    fn on_tick(&mut self, tick: Tick) {
        self.ticks = self.ticks.saturating_add(1);
        self.last_tick_elapsed = tick.elapsed;
        if self.settings.pet_enabled && matches!(self.screen, Screen::Home) {
            self.pet_animation_elapsed = self.pet_animation_elapsed.saturating_add(tick.elapsed);
        }
    }

    fn toggle_pet(&mut self) {
        self.settings.pet_enabled = !self.settings.pet_enabled;
        if self.settings.pet_enabled {
            self.pet_animation_elapsed = Duration::ZERO;
            if self.pet.is_none() {
                self.pet = pet::CubbyPet::load().ok();
            }
            self.pet_image_protocol = pet::detect_image_protocol(self.settings);
        } else {
            self.pending_pet_draw = None;
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        self.pending_pet_draw = None;
        match self.screen {
            Screen::Home => self.render_home(frame),
            Screen::Detail => self.render_detail(frame),
            Screen::Workspace => self.render_workspace(frame),
            Screen::Plan => self.render_plan(frame),
            Screen::Confirm => self.render_confirm(frame),
        }
    }

    fn render_home(&mut self, frame: &mut Frame) {
        // Reserve rows for the agent list and footer, then show the largest
        // Cubby hero that still fits the banner.
        const RESERVED_ROWS: u16 = 15;
        let available = frame.area().height.saturating_sub(RESERVED_ROWS);
        let brand_min_width = 22;
        let mascot_columns = frame.area().width.saturating_sub(brand_min_width);
        let pet_size = (self.settings.pet_enabled && !self.settings.line_mode)
            .then(|| {
                self.pet
                    .as_ref()
                    .and_then(|pet| pet.size_for_area(available, mascot_columns))
            })
            .flatten();
        let fallback_hero =
            (self.settings.pet_enabled && !self.settings.line_mode && pet_size.is_none())
                .then(|| mascot::hero_for_area(available, mascot_columns))
                .flatten();
        let banner_height = pet_size
            .map(|size| size.rows)
            .or_else(|| fallback_hero.map(mascot::HeroSprite::cell_height))
            .unwrap_or(4);

        let [banner, body, footer] = Layout::vertical([
            Constraint::Length(banner_height),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .areas(frame.area());

        if let Some(size) = pet_size {
            let animated = self.settings.motion_mode == MotionMode::Animated;
            let pet_lines = self.pet.as_mut().and_then(|pet| {
                pet.idle_lines(
                    size,
                    self.settings.color_enabled,
                    self.pet_animation_elapsed,
                    animated,
                )
                .ok()
            });
            if let Some(pet_lines) = pet_lines {
                let mascot_area =
                    render_banner(frame, banner, size.columns, pet_lines, self.palette);
                if let (Some(pet), Some(protocol)) = (self.pet.as_ref(), self.pet_image_protocol) {
                    self.pending_pet_draw = pet.draw_request(
                        mascot_area,
                        self.pet_animation_elapsed,
                        animated,
                        protocol,
                    );
                }
            } else if let Some(hero) = mascot::hero_for_area(available, mascot_columns) {
                render_banner(
                    frame,
                    banner,
                    hero.cell_width(),
                    hero.lines_with_color(self.settings.color_enabled),
                    self.palette,
                );
            } else {
                render_line_banner(frame, banner, self.palette);
            }
        } else if let Some(hero) = fallback_hero {
            render_banner(
                frame,
                banner,
                hero.cell_width(),
                hero.lines_with_color(self.settings.color_enabled),
                self.palette,
            );
        } else {
            render_line_banner(frame, banner, self.palette);
        }

        let workspace_rows = if self.settings.line_mode { 3 } else { 5 };
        let [workspace_area, list_area] =
            Layout::vertical([Constraint::Length(workspace_rows), Constraint::Min(0)]).areas(body);
        render_launcher_summary(
            frame,
            workspace_area,
            self.selected(),
            &self.launcher,
            self.settings,
            self.palette,
        );

        let item_width = list_area
            .width
            .saturating_sub(if self.settings.line_mode { 2 } else { 4 })
            as usize;
        let items: Vec<_> = self
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
        frame.render_stateful_widget(list, list_area, &mut self.list);

        render_footer(
            frame,
            footer,
            "w workspace · enter inspect · r review · p pet · q quit",
            tooltips::tip_for_tick(self.ticks),
            self.palette,
        );
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

        render_footer(
            frame,
            footer,
            "esc back · q quit",
            tooltips::tip_for_tick(self.ticks),
            self.palette,
        );
    }

    fn render_workspace(&self, frame: &mut Frame) {
        let [header, body, footer] = layout(frame);
        render_screen_title(
            frame,
            header,
            "Choose Workspace",
            self.settings,
            self.palette,
        );

        let [query_area, list_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(body);
        let query = if self.launcher.workspace_picker.query().is_empty() {
            "filter or type a path: ".to_string()
        } else {
            format!(
                "filter or type a path: {}",
                self.launcher.workspace_picker.query()
            )
        };
        let mut query_paragraph = Paragraph::new(Line::styled(query, self.palette.text()));
        if !self.settings.line_mode {
            query_paragraph = query_paragraph.block(
                Block::bordered()
                    .title(" Workspace ")
                    .border_style(self.palette.border()),
            );
        }
        frame.render_widget(query_paragraph, query_area);

        let items = self
            .launcher
            .workspace_picker
            .visible_candidates()
            .map(|(_, candidate)| {
                workspace_candidate_item(candidate, list_area.width as usize, self.palette)
            })
            .collect::<Vec<_>>();
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(
                self.launcher.workspace_picker.selected_visible_index(),
            ));
        }
        let mut list = List::new(items)
            .highlight_symbol("> ")
            .highlight_style(self.palette.selected())
            .style(self.palette.text());
        if !self.settings.line_mode {
            list = list.block(
                Block::bordered()
                    .title(" Matches ")
                    .border_style(self.palette.border()),
            );
        }
        frame.render_stateful_widget(list, list_area, &mut state);

        render_footer(
            frame,
            footer,
            "type filter/path · up/down move · enter choose · backspace edit · esc back",
            tooltips::tip_for_tick(self.ticks),
            self.palette,
        );
    }

    fn render_plan(&self, frame: &mut Frame) {
        let [header, body, footer] = layout(frame);
        render_screen_title(frame, header, "Review Plan", self.settings, self.palette);

        let lines = if let Some(review) = &self.launcher.review {
            plan_review_lines(review, body.width as usize, self.palette)
        } else {
            let message = self
                .launcher
                .plan_error
                .as_deref()
                .unwrap_or("No plan is available for the selected workspace and agent.");
            let mut lines = Vec::new();
            push_wrapped_line(
                &mut lines,
                format!("Plan error: {message}"),
                self.palette.text(),
                body.width as usize,
            );
            push_wrapped_line(
                &mut lines,
                "Choose a different workspace with w, or go back and pick another agent.",
                self.palette.muted(),
                body.width as usize,
            );
            lines
        };
        render_screen_body(
            frame,
            body,
            " Boundary ",
            lines,
            self.settings,
            self.palette,
        );

        let hints = if self.launcher.review.is_some() {
            "enter confirm · w workspace · esc back · q quit"
        } else {
            "w workspace · esc back · q quit"
        };
        render_footer(
            frame,
            footer,
            hints,
            tooltips::tip_for_tick(self.ticks),
            self.palette,
        );
    }

    fn render_confirm(&self, frame: &mut Frame) {
        let [header, body, footer] = layout(frame);
        render_screen_title(frame, header, "Confirm Launch", self.settings, self.palette);

        let mut lines = Vec::new();
        if let Some(review) = &self.launcher.review {
            push_wrapped_line(
                &mut lines,
                format!("Mount: {} -> /workspace", review.plan.workspace.display()),
                self.palette.text(),
                body.width as usize,
            );
            push_wrapped_line(
                &mut lines,
                format!("Network: {}", review.plan.network_mode.as_str()),
                self.palette.text(),
                body.width as usize,
            );
            push_wrapped_line(
                &mut lines,
                format!("Egress: {}", review.plan.egress_summary),
                self.palette.text(),
                body.width as usize,
            );
            lines.push(Line::from(""));
            push_wrapped_line(
                &mut lines,
                format!("CLI: {}", review.cli_command),
                self.palette.muted(),
                body.width as usize,
            );
            lines.push(Line::from(""));
            if review.requires_typed_confirm {
                push_wrapped_line(
                    &mut lines,
                    format!(
                        "This plan has security notices. Type {} to launch.",
                        launcher::CONFIRM_PHRASE
                    ),
                    self.palette.accent(),
                    body.width as usize,
                );
                push_wrapped_line(
                    &mut lines,
                    format!("confirm: {}", self.launcher.confirm_input),
                    self.palette.text(),
                    body.width as usize,
                );
            } else {
                push_wrapped_line(
                    &mut lines,
                    "Secure-default plan. Press enter to launch.",
                    self.palette.accent(),
                    body.width as usize,
                );
            }
        } else {
            push_wrapped_line(
                &mut lines,
                "No reviewed plan is available. Go back and review the plan first.",
                self.palette.text(),
                body.width as usize,
            );
        }
        render_screen_body(frame, body, " Launch ", lines, self.settings, self.palette);

        let hints = if self
            .launcher
            .review
            .as_ref()
            .is_some_and(|review| review.requires_typed_confirm)
        {
            "type run · enter launch · esc back"
        } else {
            "enter launch · esc back"
        };
        render_footer(
            frame,
            footer,
            hints,
            tooltips::tip_for_tick(self.ticks),
            self.palette,
        );
    }

    fn render_terminal_overlay(&mut self) {
        let mut stdout = std::io::stdout().lock();
        if pet::render_pet_image(
            &mut stdout,
            &mut self.pet_image_state,
            self.pending_pet_draw.clone(),
        )
        .is_err()
        {
            self.pet_image_protocol = None;
            let _ = pet::render_pet_image(&mut stdout, &mut self.pet_image_state, None);
        }
    }

    fn clear_terminal_overlay(&mut self) {
        let mut stdout = std::io::stdout().lock();
        let _ = pet::render_pet_image(&mut stdout, &mut self.pet_image_state, None);
    }
}

#[cfg(test)]
mod tests;

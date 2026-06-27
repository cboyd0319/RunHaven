//! Terminal UI: the default interface when `runhaven` runs on a TTY with no
//! subcommand. It is a launcher and manager over the same profiles and planner
//! the CLI uses, never a replacement for the explicit CLI surface.
//!
//! Slices so far: the scaffold, the agent picker, portable and high-resolution
//! Cubby branding, the Phase 0 foundation, and the Phase 1 pet/tooltips layer.
//! Later slices add the run dashboard and history/diagnostics surfaces.

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, List, ListState, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::runhaven::provider::auth_profiles::{agent_broker, agent_sign_in};
use crate::runhaven::runtime::launch::launch_run_plan;
use crate::runhaven::runtime::plans::{AgentRunPlan, default_network_mode};
use crate::runhaven::runtime::profiles::{AgentProfile, profiles};
use crate::runhaven::support::paths::runs_log_path;

mod codex;
mod color;
mod event_loop;
mod guide_views;
mod history;
mod history_views;
mod input;
mod launcher;
mod mascot;
mod pet;
mod run_views;
mod runs;
#[cfg(test)]
mod snapshot;
#[cfg(test)]
mod test_backend;
mod theme;
mod tooltips;
mod widgets;
mod zork;

use event_loop::{Tick, Ticker};
use theme::{MotionMode, Palette, TuiSettings};
use widgets::{
    LaunchStep, agent_list_item, launch_stepper_text, layout, plan_review_lines, push_wrapped_line,
    render_banner, render_footer, render_launch_stepper, render_launcher_summary,
    render_line_banner, render_screen_body, render_screen_title, workspace_candidate_item,
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
    Runs,
    Logs,
    Control,
    History,
    HistoryDetail,
    Diagnostics,
    Doctor,
    Guide,
    Zork,
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
    run_manager: runs::RunManagerState,
    history: history::HistoryState,
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
    zork: zork::ZorkState,
}

impl App {
    fn new() -> Self {
        let workspace = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut app = Self::with_settings_and_workspace(TuiSettings::from_env(), workspace);
        if Self::should_start_with_guide(&runs_log_path()) {
            app.screen = Screen::Guide;
        }
        app
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
            run_manager: runs::RunManagerState::default(),
            history: history::HistoryState::new(settings, pet_image_protocol),
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
            zork: zork::ZorkState::new(),
        }
    }

    fn should_start_with_guide(runs_log: &Path) -> bool {
        match fs::metadata(runs_log) {
            Ok(metadata) => metadata.len() == 0,
            Err(error) => error.kind() == std::io::ErrorKind::NotFound,
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

    fn on_tick(&mut self, tick: Tick) {
        self.ticks = self.ticks.saturating_add(1);
        self.last_tick_elapsed = tick.elapsed;
        if self.settings.pet_enabled && matches!(self.screen, Screen::Home) {
            self.pet_animation_elapsed = self.pet_animation_elapsed.saturating_add(tick.elapsed);
        }
        if matches!(self.screen, Screen::Runs | Screen::Logs) && self.ticks.is_multiple_of(8) {
            self.run_manager.refresh_dashboard();
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
            Screen::Runs => {
                run_views::render_runs(frame, &self.run_manager, self.settings, self.palette)
            }
            Screen::Logs => {
                run_views::render_logs(frame, &self.run_manager, self.settings, self.palette)
            }
            Screen::Control => {
                run_views::render_control(frame, &self.run_manager, self.settings, self.palette)
            }
            Screen::History => {
                history_views::render_history(frame, &self.history, self.settings, self.palette)
            }
            Screen::HistoryDetail => history_views::render_history_detail(
                frame,
                &self.history,
                self.settings,
                self.palette,
            ),
            Screen::Diagnostics => {
                history_views::render_diagnostics(frame, &self.history, self.settings, self.palette)
            }
            Screen::Doctor => history_views::render_doctor(
                frame,
                &self.history,
                self.settings,
                self.palette,
                self.ticks,
            ),
            Screen::Guide => guide_views::render_guide(frame, self.settings, self.palette),
            Screen::Zork => zork::render_zork(frame, &self.zork, self.settings, self.palette),
        }
    }

    fn render_home(&mut self, frame: &mut Frame) {
        // Reserve rows for the agent list and footer, then show the largest
        // Cubby hero that still fits the banner without dominating the screen.
        const RESERVED_ROWS: u16 = 15;
        let available = frame.area().height.saturating_sub(RESERVED_ROWS);
        let mascot_max_rows = available.saturating_add(1) / 2;
        let brand_min_width = 22;
        let mascot_columns = frame.area().width.saturating_sub(brand_min_width);
        let pet_size = (self.settings.pet_enabled && !self.settings.line_mode)
            .then(|| {
                self.pet
                    .as_ref()
                    .and_then(|pet| pet.size_for_area(mascot_max_rows, mascot_columns))
            })
            .flatten();
        let fallback_hero =
            (self.settings.pet_enabled && !self.settings.line_mode && pet_size.is_none())
                .then(|| mascot::hero_for_area(mascot_max_rows, mascot_columns))
                .flatten();
        let banner_context = self.home_banner_context();
        let banner_height = pet_size
            .map(|size| size.rows)
            .or_else(|| fallback_hero.map(mascot::HeroSprite::cell_height))
            .unwrap_or(5)
            .max(banner_context.len() as u16);

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
                let mascot_area = render_banner(
                    frame,
                    banner,
                    size.columns,
                    pet_lines,
                    &banner_context,
                    self.palette,
                );
                if let (Some(pet), Some(protocol)) = (self.pet.as_ref(), self.pet_image_protocol) {
                    self.pending_pet_draw = pet.draw_request(
                        mascot_area,
                        self.pet_animation_elapsed,
                        animated,
                        protocol,
                    );
                }
            } else if let Some(hero) = mascot::hero_for_area(mascot_max_rows, mascot_columns) {
                render_banner(
                    frame,
                    banner,
                    hero.cell_width(),
                    hero.lines_with_color(self.settings.color_enabled),
                    &banner_context,
                    self.palette,
                );
            } else {
                render_line_banner(frame, banner, &banner_context, self.palette);
            }
        } else if let Some(hero) = fallback_hero {
            render_banner(
                frame,
                banner,
                hero.cell_width(),
                hero.lines_with_color(self.settings.color_enabled),
                &banner_context,
                self.palette,
            );
        } else {
            render_line_banner(frame, banner, &banner_context, self.palette);
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
            "up/down select · enter inspect · r review plan · w workspace · ? guide · q quit",
            self.home_tip(),
            self.palette,
        );
    }

    fn home_tip(&self) -> &'static str {
        tooltips::tip_for_tick(self.ticks)
    }

    fn home_banner_context(&self) -> Vec<String> {
        let (agent, network) = self.selected().map_or_else(
            || ("none".to_string(), "-".to_string()),
            |agent| {
                (
                    agent.name.to_string(),
                    default_network_mode(agent).as_str().to_string(),
                )
            },
        );
        let next = if self.run_manager.runs.is_empty() {
            "next: r review plan  d dashboard  h history".to_string()
        } else {
            format!(
                "next: d dashboard ({} active)  h history",
                self.run_manager.runs.len()
            )
        };

        vec![
            format!("RunHaven v{}", env!("CARGO_PKG_VERSION")),
            format!("launch: {}", launch_stepper_text(LaunchStep::Agent)),
            format!("agent: {agent}  network: {network}"),
            format!("workspace: {}", self.launcher.workspace.display()),
            "boundary: /workspace only  no host home/creds".to_string(),
            next,
        ]
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
            "enter review · d dashboard · esc back · q quit",
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

        let [stepper_area, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(body);
        render_launch_stepper(frame, stepper_area, LaunchStep::Workspace, self.palette);
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

        let [stepper_area, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(body);
        render_launch_stepper(frame, stepper_area, LaunchStep::Review, self.palette);
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

        let [stepper_area, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(body);
        render_launch_stepper(frame, stepper_area, LaunchStep::Confirm, self.palette);
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

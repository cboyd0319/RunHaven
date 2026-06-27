//! Terminal UI: the default interface when `runhaven` runs on a TTY with no
//! subcommand. It is a launcher and manager over the same profiles and planner
//! the CLI uses, never a replacement for the explicit CLI surface.
//!
//! Slices so far: the scaffold, the agent picker, the source-first Codex TUI
//! foundation, RunHaven logo branding, the native Cubby pet, launcher flow, run
//! dashboard, history/diagnostics, and polish.

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
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

mod brand;
mod codex;
mod color;
mod event_loop;
mod guide_views;
mod history;
mod history_views;
mod input;
mod launcher;
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
    logo: Option<brand::BrandLogo>,
    pet: Option<pet::CubbyPet>,
    terminal_image_protocol: Option<codex::image_protocol::ImageProtocol>,
    pending_logo_draw: Option<codex::ambient::AmbientImageDraw>,
    pending_pet_draw: Option<codex::ambient::AmbientImageDraw>,
    logo_image_state: codex::ambient::AmbientImageRenderState,
    pet_image_state: codex::ambient::AmbientImageRenderState,
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
        let logo = if settings.line_mode {
            None
        } else {
            brand::BrandLogo::load().ok()
        };
        let pet = if settings.pet_enabled && !settings.line_mode {
            pet::CubbyPet::load().ok()
        } else {
            None
        };
        let terminal_image_protocol = pet::detect_image_protocol(settings);
        Self {
            agents,
            list,
            launcher: launcher::LauncherState::new(workspace),
            run_manager: runs::RunManagerState::default(),
            history: history::HistoryState::new(settings, terminal_image_protocol),
            settings,
            palette,
            screen: Screen::Home,
            ticks: 0,
            last_tick_elapsed: Duration::ZERO,
            pet_animation_elapsed: Duration::ZERO,
            logo,
            pet,
            terminal_image_protocol,
            pending_logo_draw: None,
            pending_pet_draw: None,
            logo_image_state: codex::ambient::AmbientImageRenderState::default(),
            pet_image_state: codex::ambient::AmbientImageRenderState::default(),
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
            self.terminal_image_protocol = pet::detect_image_protocol(self.settings);
        } else {
            self.pending_pet_draw = None;
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        self.pending_logo_draw = None;
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
        // RunHaven logo that still fits the banner without dominating the screen.
        const RESERVED_ROWS: u16 = 15;
        let available = frame.area().height.saturating_sub(RESERVED_ROWS);
        let logo_max_rows = available.saturating_add(1) / 2;
        let brand_min_width = 22;
        let logo_columns = frame.area().width.saturating_sub(brand_min_width);
        let logo_size = (!self.settings.line_mode)
            .then(|| {
                self.logo
                    .as_ref()
                    .and_then(|logo| logo.size_for_area(logo_max_rows, logo_columns))
            })
            .flatten();
        let banner_context = self.home_banner_context();
        let banner_height = logo_size
            .map(|size| size.rows)
            .unwrap_or(5)
            .max(banner_context.len() as u16);

        let [banner, body, footer] = Layout::vertical([
            Constraint::Length(banner_height),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .areas(frame.area());

        if let Some(size) = logo_size {
            let logo_lines = if self.terminal_image_protocol.is_some() {
                Some(blank_lines(size.rows))
            } else {
                self.logo
                    .as_mut()
                    .and_then(|logo| logo.lines(size, self.settings.color_enabled).ok())
            };
            if let Some(logo_lines) = logo_lines {
                let logo_area = render_banner(
                    frame,
                    banner,
                    size.columns,
                    logo_lines,
                    &banner_context,
                    self.palette,
                );
                if let (Some(logo), Some(protocol)) =
                    (self.logo.as_ref(), self.terminal_image_protocol)
                {
                    self.pending_logo_draw = logo.draw_request(logo_area, protocol);
                }
            } else {
                render_line_banner(frame, banner, &banner_context, self.palette);
            }
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
        self.render_ambient_pet(frame, list_area);

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

    fn render_ambient_pet(&mut self, frame: &mut Frame, list_area: Rect) {
        if !self.settings.pet_enabled || self.settings.line_mode {
            return;
        }
        let Some(pet) = self.pet.as_mut() else {
            return;
        };
        let size = pet.ambient_size();
        let inner = Rect {
            x: list_area.x.saturating_add(1),
            y: list_area.y.saturating_add(1),
            width: list_area.width.saturating_sub(2),
            height: list_area.height.saturating_sub(2),
        };
        let pet_area_y = inner
            .y
            .saturating_add(self.agents.len() as u16)
            .saturating_add(1);
        let pet_area = Rect {
            x: inner.x,
            y: pet_area_y,
            width: inner.width,
            height: inner
                .y
                .saturating_add(inner.height)
                .saturating_sub(pet_area_y),
        };
        let composer_bottom_y = pet_area.y.saturating_add(pet_area.height);
        let animated = self.settings.motion_mode == MotionMode::Animated;
        if self.terminal_image_protocol.is_none()
            && let Some(area) = pet.ambient_area(pet_area, composer_bottom_y)
            && let Ok(lines) = pet.ambient_lines(
                self.settings.color_enabled,
                self.pet_animation_elapsed,
                animated,
            )
        {
            debug_assert_eq!(area.width, size.columns);
            debug_assert_eq!(area.height, size.rows);
            frame.render_widget(Paragraph::new(lines), area);
        }
        if let Some(protocol) = self.terminal_image_protocol
            && let Some(request) = pet.ambient_draw_request(
                pet_area,
                composer_bottom_y,
                self.pet_animation_elapsed,
                animated,
                protocol,
            )
        {
            self.pending_pet_draw = Some(request);
        }
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
            "boundary: /workspace only  no home folder or credentials".to_string(),
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
                "network:         {}",
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
                format!("Workspace folder: {}", review.plan.workspace.display()),
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
                format!("Online access: {}", review.plan.egress_summary),
                self.palette.text(),
                body.width as usize,
            );
            lines.push(Line::from(""));
            push_wrapped_line(
                &mut lines,
                format!("Command: {}", review.cli_command),
                self.palette.muted(),
                body.width as usize,
            );
            lines.push(Line::from(""));
            if review.requires_typed_confirm {
                push_wrapped_line(
                    &mut lines,
                    format!(
                        "This plan has safety notes. Type {} to launch.",
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
                    "Safe plan. Press enter to launch.",
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
        if codex::ambient::render_ambient_image(
            &mut stdout,
            &mut self.logo_image_state,
            brand::LOGO_IMAGE_ID,
            self.pending_logo_draw.clone(),
        )
        .is_err()
        {
            self.terminal_image_protocol = None;
            let _ = codex::ambient::render_ambient_image(
                &mut stdout,
                &mut self.logo_image_state,
                brand::LOGO_IMAGE_ID,
                None,
            );
        }
        if codex::ambient::render_ambient_image(
            &mut stdout,
            &mut self.pet_image_state,
            pet::PET_IMAGE_ID,
            self.pending_pet_draw.clone(),
        )
        .is_err()
        {
            self.terminal_image_protocol = None;
            let _ = codex::ambient::render_ambient_image(
                &mut stdout,
                &mut self.pet_image_state,
                pet::PET_IMAGE_ID,
                None,
            );
        }
    }

    fn clear_terminal_overlay(&mut self) {
        let mut stdout = std::io::stdout().lock();
        let _ = codex::ambient::render_ambient_image(
            &mut stdout,
            &mut self.logo_image_state,
            brand::LOGO_IMAGE_ID,
            None,
        );
        let _ = codex::ambient::render_ambient_image(
            &mut stdout,
            &mut self.pet_image_state,
            pet::PET_IMAGE_ID,
            None,
        );
    }
}

fn blank_lines(rows: u16) -> Vec<Line<'static>> {
    (0..rows).map(|_| Line::from("")).collect()
}

#[cfg(test)]
mod tests;

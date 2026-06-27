use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;

use runhaven_core::provider::auth_profiles::agent_broker;
use runhaven_core::provider::auth_profiles::agent_sign_in;
use runhaven_core::runtime::plans::AuthScope;
use runhaven_core::runtime::plans::RunOptions;
use runhaven_core::runtime::plans::WorkspaceScope;
use runhaven_core::runtime::plans::build_run_plan;
use runhaven_core::runtime::plans::default_network_mode;
use runhaven_core::runtime::profiles::profiles;
use runhaven_core::ui_contracts::LaunchPlanData;
use tokio::runtime::Runtime;
use tokio::sync::broadcast;

const TICK_RATE: Duration = Duration::from_millis(250);
const IMAGE_SMOKE_TICK_RATE: Duration = Duration::from_millis(100);
const IMAGE_SMOKE_ENV: &str = "RUNHAVEN_TUI_IMAGE_SMOKE";
const IMAGE_SMOKE_PET_ENV: &str = "RUNHAVEN_TUI_IMAGE_SMOKE_PET";
const DEFAULT_IMAGE_SMOKE_PET: &str = "custom:cubby";

pub(crate) fn run() -> Result<i32> {
    let mut state = ShellState::for_current_dir()?;
    let mut terminal = ratatui::try_init()?;
    let _restore = TerminalRestoreGuard;
    run_loop(&mut terminal, &mut state)?;
    Ok(0)
}

struct TerminalRestoreGuard;

impl Drop for TerminalRestoreGuard {
    fn drop(&mut self) {
        let _ = ratatui::try_restore();
    }
}

fn run_loop(terminal: &mut DefaultTerminal, state: &mut ShellState) -> Result<()> {
    let mut redraw = true;
    loop {
        if state.drain_image_smoke_draws() {
            redraw = true;
        }

        if redraw {
            terminal.draw(|frame| render(frame, state))?;
            state.draw_image_smoke(terminal)?;
            redraw = false;
        }

        if !event::poll(state.tick_rate())? {
            if state.image_smoke_animates() {
                redraw = true;
            }
            continue;
        }

        match event::read()? {
            Event::Key(key) => match state.handle_key(key) {
                ShellAction::Continue => redraw = true,
                ShellAction::Quit => break,
            },
            Event::Resize(_, _) => redraw = true,
            _ => {}
        }
    }
    state.clear_image_smoke(terminal)?;
    Ok(())
}

struct ShellState {
    workspace: PathBuf,
    agents: Vec<AgentPreview>,
    selected: usize,
    image_smoke: ImageSmoke,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentPreview {
    name: String,
    description: String,
    sign_in: String,
    broker: String,
    plan: Result<LaunchPlanData, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellAction {
    Continue,
    Quit,
}

impl ShellState {
    fn for_current_dir() -> Result<Self> {
        Self::for_workspace(std::env::current_dir()?)
    }

    fn for_workspace(workspace: impl AsRef<Path>) -> Result<Self> {
        let workspace = workspace.as_ref().to_path_buf();
        let agents = profiles()
            .into_iter()
            .map(|profile| {
                let network = default_network_mode(&profile);
                let name = profile.name.to_string();
                let description = profile.description.to_string();
                let sign_in = agent_sign_in(profile.name).to_string();
                let broker = agent_broker(profile.name).to_string();
                let plan = build_run_plan(RunOptions {
                    profile,
                    workspace: workspace.clone(),
                    agent_args: Vec::new(),
                    image: None,
                    cpus: "4".to_string(),
                    memory: "4g".to_string(),
                    network,
                    workspace_scope: WorkspaceScope::Current,
                    session: None,
                    auth_scope: AuthScope::Agent,
                    read_only_workspace: false,
                    ssh: false,
                    env: Vec::new(),
                    user: "agent".to_string(),
                    interactive: true,
                    tty: true,
                    allow_sensitive_workspace: false,
                    allow_root_user: false,
                    provider_hosts: Vec::new(),
                    api_key_broker_env: None,
                    worktree: None,
                    run_id: None,
                })
                .map(|plan| LaunchPlanData::from(&plan))
                .map_err(|error| error.to_string());

                AgentPreview {
                    name,
                    description,
                    sign_in,
                    broker,
                    plan,
                }
            })
            .collect();

        Ok(Self {
            workspace,
            agents,
            selected: 0,
            image_smoke: ImageSmoke::from_env(),
        })
    }

    fn selected_agent(&self) -> Option<&AgentPreview> {
        self.agents.get(self.selected)
    }

    fn select_next(&mut self) {
        if self.agents.is_empty() {
            return;
        }
        self.selected = (self.selected + 1).min(self.agents.len() - 1);
    }

    fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn handle_key(&mut self, key: KeyEvent) -> ShellAction {
        if key.kind != KeyEventKind::Press {
            return ShellAction::Continue;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => ShellAction::Quit,
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                ShellAction::Continue
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
                ShellAction::Continue
            }
            KeyCode::Home => {
                self.selected = 0;
                ShellAction::Continue
            }
            KeyCode::End => {
                if !self.agents.is_empty() {
                    self.selected = self.agents.len() - 1;
                }
                ShellAction::Continue
            }
            _ => ShellAction::Continue,
        }
    }

    fn tick_rate(&self) -> Duration {
        if self.image_smoke_animates() {
            IMAGE_SMOKE_TICK_RATE
        } else {
            TICK_RATE
        }
    }

    fn image_smoke_animates(&self) -> bool {
        self.image_smoke.animates()
    }

    fn image_smoke_status(&self) -> Option<Line<'static>> {
        self.image_smoke.status_line()
    }

    fn prepare_image_smoke_draw(&mut self, area: ratatui::layout::Rect, composer_bottom_y: u16) {
        self.image_smoke.prepare_draw(area, composer_bottom_y);
    }

    fn draw_image_smoke(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.image_smoke.draw(terminal)
    }

    fn clear_image_smoke(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.image_smoke.clear(terminal)
    }

    fn drain_image_smoke_draws(&mut self) -> bool {
        self.image_smoke.drain_draws()
    }
}

enum ImageSmoke {
    Disabled,
    Ready(Box<ImageSmokeState>),
    Error(String),
}

impl ImageSmoke {
    fn from_env() -> Self {
        if !env_flag_enabled(std::env::var_os(IMAGE_SMOKE_ENV).as_deref()) {
            return Self::Disabled;
        }

        match ImageSmokeState::load() {
            Ok(state) => Self::Ready(Box::new(state)),
            Err(error) => Self::Error(format!("pet image smoke unavailable: {error}")),
        }
    }

    fn animates(&self) -> bool {
        matches!(self, Self::Ready(state) if state.image_enabled)
    }

    fn status_line(&self) -> Option<Line<'static>> {
        match self {
            Self::Disabled => None,
            Self::Ready(state) => Some(Line::from(state.status.clone())),
            Self::Error(message) => Some(Line::from(vec![Span::styled(
                message.clone(),
                Style::default().fg(Color::Yellow),
            )])),
        }
    }

    fn prepare_draw(&mut self, area: ratatui::layout::Rect, composer_bottom_y: u16) {
        if let Self::Ready(state) = self {
            state.prepare_draw(area, composer_bottom_y);
        }
    }

    fn draw(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        if let Self::Ready(state) = self {
            state.draw(terminal)?;
        }
        Ok(())
    }

    fn clear(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        if let Self::Ready(state) = self {
            state.clear(terminal)?;
        }
        Ok(())
    }

    fn drain_draws(&mut self) -> bool {
        match self {
            Self::Ready(state) => state.drain_draws(),
            Self::Disabled | Self::Error(_) => false,
        }
    }
}

struct ImageSmokeState {
    runtime: Runtime,
    draw_rx: broadcast::Receiver<()>,
    pet: crate::tui::pets::AmbientPet,
    render_state: crate::tui::pets::PetImageRenderState,
    pending_draw: Option<crate::tui::pets::AmbientPetDraw>,
    status: String,
    image_enabled: bool,
}

impl ImageSmokeState {
    fn load() -> Result<Self> {
        let codex_home = codex_home().context("CODEX_HOME or HOME is not available")?;
        let pet_id = std::env::var(IMAGE_SMOKE_PET_ENV)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_IMAGE_SMOKE_PET.to_string());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .context("start pet frame scheduler")?;
        let (draw_tx, draw_rx) = broadcast::channel(16);
        let frame_requester = {
            let _guard = runtime.enter();
            crate::tui::FrameRequester::new(draw_tx)
        };
        let mut pet =
            crate::tui::pets::AmbientPet::load(Some(&pet_id), &codex_home, frame_requester, true)
                .with_context(|| format!("load {pet_id} from {}", codex_home.display()))?;
        pet.set_notification(
            crate::tui::pets::PetNotificationKind::Review,
            Some("Image smoke".to_string()),
        );
        let image_enabled = pet.image_enabled();
        let status = if image_enabled {
            format!("pet image smoke: Codex native renderer using {pet_id}")
        } else {
            "pet image smoke: terminal image protocol unavailable".to_string()
        };

        Ok(Self {
            runtime,
            draw_rx,
            pet,
            render_state: crate::tui::pets::PetImageRenderState::default(),
            pending_draw: None,
            status,
            image_enabled,
        })
    }

    fn prepare_draw(&mut self, area: ratatui::layout::Rect, composer_bottom_y: u16) {
        self.pending_draw = self.pet.draw_request(area, composer_bottom_y);
        self.pet.schedule_next_frame();
    }

    fn draw(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        crate::tui::pets::render_ambient_pet_image(
            terminal.backend_mut(),
            &mut self.render_state,
            self.pending_draw.take(),
        )?;
        Ok(())
    }

    fn clear(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        crate::tui::pets::render_ambient_pet_image(
            terminal.backend_mut(),
            &mut self.render_state,
            None,
        )?;
        Ok(())
    }

    fn drain_draws(&mut self) -> bool {
        self.runtime.block_on(async {
            tokio::task::yield_now().await;
        });

        let mut requested = false;
        loop {
            match self.draw_rx.try_recv() {
                Ok(()) => requested = true,
                Err(broadcast::error::TryRecvError::Lagged(_)) => requested = true,
                Err(
                    broadcast::error::TryRecvError::Empty | broadcast::error::TryRecvError::Closed,
                ) => {
                    break;
                }
            }
        }
        requested
    }
}

fn env_flag_enabled(value: Option<&std::ffi::OsStr>) -> bool {
    let Some(value) = value.and_then(|value| value.to_str()) else {
        return false;
    };
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn codex_home() -> Option<PathBuf> {
    std::env::var_os("CODEX_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(|home| PathBuf::from(home).join(".codex"))
        })
}

fn render(frame: &mut Frame<'_>, state: &mut ShellState) {
    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(area);

    render_header(frame, vertical[0], state);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(34), Constraint::Min(30)])
        .split(vertical[1]);

    render_agents(frame, body[0], state);
    render_plan(frame, body[1], state);
    render_footer(frame, vertical[2]);
    state.prepare_image_smoke_draw(area, vertical[2].y);
}

fn render_header(frame: &mut Frame<'_>, area: ratatui::layout::Rect, state: &ShellState) {
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                "RunHaven",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" v{}", env!("CARGO_PKG_VERSION"))),
        ]),
        Line::from("launch: 1 agent > 2 workspace > 3 review > 4 run"),
        Line::from("This preview uses the same planner as the CLI."),
    ];
    if let Some(status) = state.image_smoke_status() {
        lines.push(status);
    }
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_agents(frame: &mut Frame<'_>, area: ratatui::layout::Rect, state: &ShellState) {
    let items = state
        .agents
        .iter()
        .map(|agent| {
            ListItem::new(vec![
                Line::from(vec![Span::styled(
                    agent.name.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Line::from(agent.description.clone()),
            ])
        })
        .collect::<Vec<_>>();
    let mut list_state = ListState::default();
    if !state.agents.is_empty() {
        list_state.select(Some(state.selected));
    }
    let list = List::new(items)
        .block(Block::default().title(" Agents ").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Gray)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_plan(frame: &mut Frame<'_>, area: ratatui::layout::Rect, state: &ShellState) {
    let mut lines = Vec::new();
    lines.push(label_value(
        "workspace",
        state.workspace.display().to_string(),
    ));

    match state.selected_agent() {
        Some(agent) => {
            lines.push(label_value("agent", agent.name.clone()));
            lines.push(label_value("sign in", agent.sign_in.clone()));
            lines.push(label_value("broker", agent.broker.clone()));
            lines.push(Line::from(""));

            match &agent.plan {
                Ok(plan) => append_plan_lines(&mut lines, plan),
                Err(message) => {
                    lines.push(Line::from(vec![Span::styled(
                        "Plan could not be built.",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    )]));
                    lines.push(Line::from(message.clone()));
                }
            }
        }
        None => lines.push(Line::from("No agents are configured.")),
    }

    let block = Block::default()
        .title(" Launch preview ")
        .borders(Borders::ALL);
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn append_plan_lines(lines: &mut Vec<Line<'static>>, plan: &LaunchPlanData) {
    lines.push(label_value("network", network_label(plan)));
    lines.push(label_value("state", plan.state_volume.clone()));
    lines.push(label_value("image", plan.image.clone()));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Not shared",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    for item in plan.boundary.not_shared.iter().take(5) {
        lines.push(Line::from(format!("- {item}")));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Shared with the agent",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(plan.boundary.mounted_workspace.clone()));
    lines.push(Line::from(plan.boundary.mounted_state_volume.clone()));

    if !plan.network.provider_allowed_hosts.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Provider hosts",
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        for host in plan.network.provider_allowed_hosts.iter().take(4) {
            lines.push(Line::from(format!("- {host}")));
        }
        if plan.network.provider_allowed_hosts.len() > 4 {
            lines.push(Line::from(format!(
                "- {} more",
                plan.network.provider_allowed_hosts.len() - 4
            )));
        }
    }

    if !plan.safety_notes.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Safety notes",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        for note in plan.safety_notes.iter().take(3) {
            lines.push(Line::from(format!("- {note}")));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Command",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(plan.command.clone()));
}

fn network_label(plan: &LaunchPlanData) -> String {
    match plan.network.mode.as_str() {
        "provider" => "provider allowlist".to_string(),
        "internal" => "local only".to_string(),
        "internet" => "internet".to_string(),
        _ => plan.network.summary.clone(),
    }
}

fn render_footer(frame: &mut Frame<'_>, area: ratatui::layout::Rect) {
    let footer = Paragraph::new(vec![
        Line::from("up/down select  |  enter review coming next  |  q quit"),
        Line::from("RunHaven will show the full review before launch."),
    ]);
    frame.render_widget(footer, area);
}

fn label_value(label: &'static str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label}: "),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(value),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn shell_state_builds_default_launch_previews() {
        let workspace = tempfile::tempdir().expect("workspace");
        let state = ShellState::for_workspace(workspace.path()).expect("state");

        assert!(!state.agents.is_empty());
        let antigravity = state.selected_agent().expect("selected agent");
        assert_eq!(antigravity.name, "antigravity");
        assert!(antigravity.plan.is_ok(), "{:?}", antigravity.plan);
    }

    #[test]
    fn shell_key_handling_moves_selection_and_quits() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut state = ShellState::for_workspace(workspace.path()).expect("state");

        assert_eq!(state.selected, 0);
        assert_eq!(
            state.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            ShellAction::Continue
        );
        assert_eq!(state.selected, 1);
        assert_eq!(
            state.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            ShellAction::Continue
        );
        assert_eq!(state.selected, 0);
        assert_eq!(
            state.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            ShellAction::Quit
        );
    }

    #[test]
    fn shell_render_shows_launch_contract_data() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut state = ShellState::for_workspace(workspace.path()).expect("state");
        let output = render_to_text(&mut state, 120, 48);

        assert!(output.contains("RunHaven"));
        assert!(output.contains("Launch preview"));
        assert!(output.contains("Not shared"));
        assert!(output.contains("host home folder"));
        assert!(output.contains("container run"));
    }

    #[test]
    fn image_smoke_flag_accepts_plain_true_values() {
        assert!(env_flag_enabled(Some(std::ffi::OsStr::new("1"))));
        assert!(env_flag_enabled(Some(std::ffi::OsStr::new("true"))));
        assert!(env_flag_enabled(Some(std::ffi::OsStr::new("YES"))));
        assert!(env_flag_enabled(Some(std::ffi::OsStr::new("on"))));
        assert!(!env_flag_enabled(Some(std::ffi::OsStr::new("0"))));
        assert!(!env_flag_enabled(Some(std::ffi::OsStr::new("false"))));
        assert!(!env_flag_enabled(None));
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
}

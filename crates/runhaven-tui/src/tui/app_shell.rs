use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

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

const TICK_RATE: Duration = Duration::from_millis(250);

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
        if redraw {
            terminal.draw(|frame| render(frame, state))?;
            redraw = false;
        }

        if !event::poll(TICK_RATE)? {
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
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShellState {
    workspace: PathBuf,
    agents: Vec<AgentPreview>,
    selected: usize,
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
}

fn render(frame: &mut Frame<'_>, state: &ShellState) {
    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(area);

    render_header(frame, vertical[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(34), Constraint::Min(30)])
        .split(vertical[1]);

    render_agents(frame, body[0], state);
    render_plan(frame, body[1], state);
    render_footer(frame, vertical[2]);
}

fn render_header(frame: &mut Frame<'_>, area: ratatui::layout::Rect) {
    let lines = vec![
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
        let state = ShellState::for_workspace(workspace.path()).expect("state");
        let output = render_to_text(&state, 120, 48);

        assert!(output.contains("RunHaven"));
        assert!(output.contains("Launch preview"));
        assert!(output.contains("Not shared"));
        assert!(output.contains("host home folder"));
        assert!(output.contains("container run"));
    }

    fn render_to_text(state: &ShellState, width: u16, height: u16) -> String {
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

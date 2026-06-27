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
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use super::runhaven::launch_wizard::AgentLaunchPreview;
use super::runhaven::launch_wizard::LaunchWizardView;
use crate::render::renderable::Renderable;
use runhaven_core::runtime::plans::AuthScope;
use runhaven_core::runtime::plans::RunOptions;
use runhaven_core::runtime::plans::WorkspaceScope;
use runhaven_core::runtime::plans::build_run_plan;
use runhaven_core::runtime::plans::default_network_mode;
use runhaven_core::runtime::profiles::profiles;
use runhaven_core::ui_contracts::AgentCatalogItemData;
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
    launch_wizard: LaunchWizardView,
    image_smoke: ImageSmoke,
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
        let image_smoke = ImageSmoke::from_env();
        let image_smoke_status = image_smoke.status_line();
        let previews = profiles()
            .into_iter()
            .map(|profile| {
                let network = default_network_mode(&profile);
                let agent = AgentCatalogItemData::from_profile(&profile);
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

                AgentLaunchPreview { agent, plan }
            })
            .collect();
        let launch_wizard = LaunchWizardView::new(workspace, previews, image_smoke_status);

        Ok(Self {
            launch_wizard,
            image_smoke,
        })
    }

    fn handle_key(&mut self, key: KeyEvent) -> ShellAction {
        if key.kind != KeyEventKind::Press {
            return ShellAction::Continue;
        }

        if matches!(key.code, KeyCode::Char('q')) {
            return ShellAction::Quit;
        }

        self.launch_wizard.handle_key(key);
        if self.launch_wizard.is_cancelled() {
            ShellAction::Quit
        } else {
            ShellAction::Continue
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
    state.launch_wizard.render(area, frame.buffer_mut());
    state.prepare_image_smoke_draw(area, area.y.saturating_add(area.height));
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

        assert!(state.launch_wizard.agent_count() > 0);
        assert_eq!(
            state.launch_wizard.selected_agent_name(),
            Some("antigravity")
        );
        assert!(state.launch_wizard.search_values_are_populated());
    }

    #[test]
    fn shell_key_handling_moves_selection_and_quits() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut state = ShellState::for_workspace(workspace.path()).expect("state");

        assert_eq!(state.launch_wizard.selected_index(), 0);
        assert_eq!(
            state.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            ShellAction::Continue
        );
        assert_eq!(state.launch_wizard.selected_index(), 1);
        assert_eq!(
            state.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            ShellAction::Continue
        );
        assert_eq!(state.launch_wizard.selected_index(), 0);
        assert_eq!(
            state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            ShellAction::Continue
        );
        assert_eq!(
            state.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            ShellAction::Quit
        );
    }

    #[test]
    fn shell_escape_cancels_source_picker() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut state = ShellState::for_workspace(workspace.path()).expect("state");

        assert_eq!(
            state.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            ShellAction::Quit
        );
    }

    #[test]
    fn shell_render_shows_launch_contract_data() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut state = ShellState::for_workspace(workspace.path()).expect("state");
        let output = render_to_text(&mut state, 120, 48);

        assert!(output.contains("RunHaven"));
        assert!(output.contains("Step 1/4: Choose agent"));
        assert!(output.contains("Plan Preview"));
        assert!(output.contains("Boundary"));
        assert!(output.contains("/workspace only"));
        assert!(output.contains("Host home"));
        assert!(output.contains("not mounted"));
        assert!(output.contains("Credentials"));
        assert!(output.contains("provider allowlist"));
        assert!(output.contains("Not shared"));
        assert!(output.contains("host home folder"));
        assert!(output.contains("Exact command before launch"));
        assert!(output.contains("container run"));
    }

    #[test]
    fn shell_render_keeps_boundary_visible_on_80x24() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut state = ShellState::for_workspace(workspace.path()).expect("state");
        let output = render_to_text(&mut state, 80, 24);

        assert!(output.contains("Boundary"));
        assert!(output.contains("/workspace only"));
        assert!(output.contains("Host home"));
        assert!(output.contains("Credentials"));
        assert!(output.contains("provider allowlist"));
    }

    #[test]
    fn shell_selection_updates_source_picker_preview_state() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut state = ShellState::for_workspace(workspace.path()).expect("state");

        assert_eq!(
            state.launch_wizard.selected_agent_name(),
            Some("antigravity")
        );
        assert_eq!(
            state.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            ShellAction::Continue
        );
        assert_eq!(state.launch_wizard.selected_agent_name(), Some("claude"));
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

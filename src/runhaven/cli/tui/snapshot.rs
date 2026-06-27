use anyhow::Result;
use ratatui::{Frame, Terminal};

use super::test_backend::Vt100Backend;

pub(crate) fn render_vt100(
    width: u16,
    height: u16,
    mut render: impl FnMut(&mut Frame<'_>),
) -> Result<String> {
    let mut terminal = Terminal::new(Vt100Backend::new(width, height))?;
    terminal.draw(|frame| render(frame))?;
    Ok(terminal.backend().to_string())
}

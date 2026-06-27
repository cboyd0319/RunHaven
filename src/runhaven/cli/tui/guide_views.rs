use ratatui::Frame;

use super::theme::{Palette, TuiSettings};
use super::widgets::{
    layout, push_wrapped_line, render_footer, render_screen_body, render_screen_title,
};

pub(super) fn render_guide(frame: &mut Frame, settings: TuiSettings, palette: Palette) {
    let [header, body, footer] = layout(frame);
    render_screen_title(frame, header, "RunHaven Guide", settings, palette);

    let mut lines = Vec::new();
    for line in [
        "Start with the smallest workspace the agent needs. RunHaven mounts that one directory at /workspace.",
        "Pick an agent, then review the plan before launch. The plan shows the workspace, state volume, network mode, egress posture, and equivalent CLI command.",
        "Secure-default plans launch with enter. Lower-security choices require typing run.",
        "Use d for the active-run dashboard, h for completed run history and diffs, and g for diagnostics. The CLI remains available for scripts, pipes, recovery, and worktree merge/discard.",
        "Accessibility controls: NO_COLOR disables color, RUNHAVEN_TUI_REDUCED_MOTION=1 keeps motion static, RUNHAVEN_TUI_LINE_MODE=1 uses a text-first layout, and RUNHAVEN_TUI_COLOR_MODE=light or dark selects the palette.",
    ] {
        push_wrapped_line(&mut lines, line, palette.text(), body.width as usize);
        lines.push(ratatui::text::Line::from(""));
    }

    render_screen_body(frame, body, " First Run ", lines, settings, palette);
    render_footer(
        frame,
        footer,
        "w workspace | r review | d dashboard | h history | g diagnostics | esc home",
        "Every TUI action maps back to a named CLI command.",
        palette,
    );
}

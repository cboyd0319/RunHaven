use ratatui::Frame;

use super::theme::{Palette, TuiSettings};
use super::widgets::{
    layout, push_wrapped_line, render_footer, render_screen_body, render_screen_title,
};

pub(super) fn render_guide(frame: &mut Frame, settings: TuiSettings, palette: Palette) {
    let [header, body, footer] = layout(frame);
    render_screen_title(frame, header, "RunHaven Guide", settings, palette);

    let mut lines = Vec::new();
    let guide_lines = [
        "Launch is a four-step wizard: agent, workspace, review boundary, confirm launch.",
        "Home shows Cubby plus the selected agent, workspace, network, boundary, and next safe action.",
        "Footers list only actions for the current screen. Use ? or F1 to return here.",
        "Secure-default plans launch with enter. Lower-security choices require typing run.",
        "Side flows: d dashboard/logs, h history/diffs, and g diagnostics/doctor.",
        "Display: p toggles Cubby. USAGE documents NO_COLOR, reduced motion, line mode, and light/dark palette environment controls.",
    ];
    for (index, line) in guide_lines.iter().enumerate() {
        push_wrapped_line(&mut lines, line, palette.text(), body.width as usize);
        if index + 1 < guide_lines.len() {
            lines.push(ratatui::text::Line::from(""));
        }
    }

    render_screen_body(frame, body, " Actions ", lines, settings, palette);
    render_footer(
        frame,
        footer,
        "w workspace · r review · d dash · h history · g diag · p pet · esc home",
        "Every TUI action maps back to a named CLI command.",
        palette,
    );
}

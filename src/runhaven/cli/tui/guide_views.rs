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
        "Launch has four steps: choose agent, choose workspace, review safety, confirm launch.",
        "Home shows the RunHaven logo, selected agent, workspace, network, boundary, and next safe action.",
        "Footers show only actions for the current screen. Use ? or F1 to return here.",
        "Safe plans start with enter. Riskier choices ask you to type run.",
        "Other screens: d dashboard/logs, h history/diffs, and g checks/doctor.",
        "Display: p shows or hides Cubby. USAGE explains color, motion, line mode, and light or dark mode.",
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
        "w workspace · r review · d dash · h history · g checks · p pet · esc home",
        "Every screen action has a matching CLI command.",
        palette,
    );
}

use ratatui::style::Color;
use ratatui::style::Style;

use crate::tui::color::blend;
use crate::tui::color::is_light;
use crate::tui::terminal_palette::StdoutColorLevel;
use crate::tui::terminal_palette::best_color;
use crate::tui::terminal_palette::default_bg;
use crate::tui::terminal_palette::default_fg;
use crate::tui::terminal_palette::rgb_color;
use crate::tui::terminal_palette::stdout_color_level;

const LIGHT_BG_ACCENT_RGB: (u8, u8, u8) = (0, 95, 135);
const LIGHT_BG_SAFE_RGB: (u8, u8, u8) = (0, 108, 60);
const LIGHT_BG_WARNING_RGB: (u8, u8, u8) = (142, 92, 0);
const LIGHT_BG_DANGER_RGB: (u8, u8, u8) = (170, 36, 42);
const LIGHT_BG_MUTED_RGB: (u8, u8, u8) = (87, 96, 106);
// Decorative table rules should remain visible without competing with cell content.
const TABLE_SEPARATOR_FG_ALPHA: f32 = 0.20;

pub fn user_message_style() -> Style {
    user_message_style_for(default_bg())
}

pub fn proposed_plan_style() -> Style {
    proposed_plan_style_for(default_bg())
}

/// Returns a low-contrast rule style for separators within markdown tables.
pub(crate) fn table_separator_style() -> Style {
    table_separator_style_for(default_fg(), default_bg(), stdout_color_level())
}

/// Returns the shared accent style for active or selected TUI controls.
pub(crate) fn accent_style() -> Style {
    accent_style_for(default_bg())
}

/// Safety-positive state such as blocked host home or provider allowlist.
pub(crate) fn safe_style() -> Style {
    safe_style_for(default_bg())
}

/// Lower-security but supported choices that need explicit review.
pub(crate) fn warning_style() -> Style {
    warning_style_for(default_bg())
}

/// Unsafe, invalid, or blocked state.
pub(crate) fn danger_style() -> Style {
    danger_style_for(default_bg())
}

/// Security boundary text that must remain readable.
pub(crate) fn boundary_style() -> Style {
    boundary_style_for(default_bg())
}

/// Secondary text that is readable enough for security-relevant labels.
pub(crate) fn muted_but_readable_style() -> Style {
    muted_but_readable_style_for(default_bg())
}

/// Active RunHaven row or summary text.
pub(crate) fn selected_row_style() -> Style {
    selected_row_style_for(default_bg())
}

/// Returns the style for a user-authored message using the provided terminal background.
pub fn user_message_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    match terminal_bg {
        Some(bg) => Style::default().bg(user_message_bg(bg)),
        None => Style::default(),
    }
}

pub fn proposed_plan_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    match terminal_bg {
        Some(bg) => Style::default().bg(proposed_plan_bg(bg)),
        None => Style::default(),
    }
}

/// Returns the shared accent style for the provided terminal background.
pub(crate) fn accent_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    if terminal_bg.is_some_and(is_light) {
        Style::default().fg(best_color(LIGHT_BG_ACCENT_RGB)).bold()
    } else {
        Style::default().fg(Color::Cyan).bold()
    }
}

fn safe_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    if terminal_bg.is_some_and(is_light) {
        Style::default().fg(best_color(LIGHT_BG_SAFE_RGB)).bold()
    } else {
        Style::default().fg(Color::Green).bold()
    }
}

fn warning_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    if terminal_bg.is_some_and(is_light) {
        Style::default().fg(best_color(LIGHT_BG_WARNING_RGB)).bold()
    } else {
        Style::default().fg(Color::Yellow).bold()
    }
}

fn danger_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    if terminal_bg.is_some_and(is_light) {
        Style::default().fg(best_color(LIGHT_BG_DANGER_RGB)).bold()
    } else {
        Style::default().fg(Color::Red).bold()
    }
}

fn boundary_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    accent_style_for(terminal_bg)
}

fn muted_but_readable_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    if terminal_bg.is_some_and(is_light) {
        Style::default().fg(best_color(LIGHT_BG_MUTED_RGB))
    } else {
        Style::default().fg(Color::Gray)
    }
}

fn selected_row_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    boundary_style_for(terminal_bg).bold()
}

fn table_separator_style_for(
    terminal_fg: Option<(u8, u8, u8)>,
    terminal_bg: Option<(u8, u8, u8)>,
    color_level: StdoutColorLevel,
) -> Style {
    let (Some(fg), Some(bg)) = (terminal_fg, terminal_bg) else {
        return Style::default().dim();
    };
    let separator_rgb = blend(fg, bg, TABLE_SEPARATOR_FG_ALPHA);
    match color_level {
        StdoutColorLevel::TrueColor => Style::default().fg(rgb_color(separator_rgb)),
        StdoutColorLevel::Ansi256 => Style::default().fg(best_color(separator_rgb)),
        StdoutColorLevel::Ansi16 | StdoutColorLevel::Unknown => Style::default().dim(),
    }
}

#[allow(clippy::disallowed_methods)]
pub fn user_message_bg(terminal_bg: (u8, u8, u8)) -> Color {
    let (top, alpha) = if is_light(terminal_bg) {
        ((0, 0, 0), 0.04)
    } else {
        ((255, 255, 255), 0.12)
    };
    best_color(blend(top, terminal_bg, alpha))
}

#[allow(clippy::disallowed_methods)]
pub fn proposed_plan_bg(terminal_bg: (u8, u8, u8)) -> Color {
    user_message_bg(terminal_bg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use ratatui::style::Modifier;

    #[test]
    fn accent_style_uses_darker_cyan_on_light_backgrounds() {
        let style = accent_style_for(Some((255, 255, 255)));

        assert_eq!(style.fg, Some(best_color(LIGHT_BG_ACCENT_RGB)));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn accent_style_uses_cyan_on_dark_or_unknown_backgrounds() {
        let expected = Style::default().fg(Color::Cyan).bold();

        assert_eq!(accent_style_for(Some((0, 0, 0))), expected);
        assert_eq!(accent_style_for(/*terminal_bg*/ None), expected);
    }

    #[test]
    fn semantic_styles_stay_readable_on_dark_backgrounds() {
        assert_eq!(safe_style_for(Some((0, 0, 0))).fg, Some(Color::Green));
        assert_eq!(warning_style_for(Some((0, 0, 0))).fg, Some(Color::Yellow));
        assert_eq!(danger_style_for(Some((0, 0, 0))).fg, Some(Color::Red));
        assert_eq!(
            muted_but_readable_style_for(Some((0, 0, 0))).fg,
            Some(Color::Gray)
        );
    }

    #[test]
    fn semantic_styles_avoid_dim_text_on_light_backgrounds() {
        let styles = [
            safe_style_for(Some((255, 255, 255))),
            warning_style_for(Some((255, 255, 255))),
            danger_style_for(Some((255, 255, 255))),
            boundary_style_for(Some((255, 255, 255))),
            selected_row_style_for(Some((255, 255, 255))),
        ];

        for style in styles {
            assert!(!style.add_modifier.contains(Modifier::DIM));
            assert!(style.add_modifier.contains(Modifier::BOLD));
        }
        assert!(
            !muted_but_readable_style_for(Some((255, 255, 255)))
                .add_modifier
                .contains(Modifier::DIM)
        );
    }

    #[test]
    fn table_separator_blends_toward_dark_background() {
        let style = table_separator_style_for(
            Some((255, 255, 255)),
            Some((0, 0, 0)),
            StdoutColorLevel::TrueColor,
        );

        assert_eq!(style.fg, Some(rgb_color((51, 51, 51))));
    }

    #[test]
    fn table_separator_blends_toward_light_background() {
        let style = table_separator_style_for(
            Some((0, 0, 0)),
            Some((255, 255, 255)),
            StdoutColorLevel::TrueColor,
        );

        assert_eq!(style.fg, Some(rgb_color((204, 204, 204))));
    }

    #[test]
    fn table_separator_dims_when_palette_aware_color_is_unavailable() {
        let expected = Style::default().dim();

        assert_eq!(
            table_separator_style_for(
                Some((255, 255, 255)),
                Some((0, 0, 0)),
                StdoutColorLevel::Ansi16,
            ),
            expected
        );
        assert_eq!(
            table_separator_style_for(
                /*terminal_fg*/ None,
                Some((0, 0, 0)),
                StdoutColorLevel::TrueColor,
            ),
            expected
        );
    }
}

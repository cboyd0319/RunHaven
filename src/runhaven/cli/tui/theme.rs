use std::ffi::OsString;
use std::time::Duration;

use ratatui::style::{Color, Modifier, Style};

use super::color;
use super::event_loop::DEFAULT_TICK_RATE;

const REDUCED_MOTION_ENV: &str = "RUNHAVEN_TUI_REDUCED_MOTION";
const LINE_MODE_ENV: &str = "RUNHAVEN_TUI_LINE_MODE";
const COLOR_MODE_ENV: &str = "RUNHAVEN_TUI_COLOR_MODE";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ColorMode {
    Dark,
    Light,
}

impl ColorMode {
    #[allow(dead_code)]
    pub(crate) fn from_background(rgb: (u8, u8, u8)) -> Self {
        if color::is_light(rgb) {
            Self::Light
        } else {
            Self::Dark
        }
    }

    fn from_env_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "dark" => Some(Self::Dark),
            "light" => Some(Self::Light),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MotionMode {
    Animated,
    Reduced,
}

impl MotionMode {
    pub(crate) fn from_animations_enabled(animations_enabled: bool) -> Self {
        if animations_enabled {
            Self::Animated
        } else {
            Self::Reduced
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TuiSettings {
    pub color_enabled: bool,
    pub color_mode: ColorMode,
    pub motion_mode: MotionMode,
    pub line_mode: bool,
    pub tick_rate: Duration,
}

impl Default for TuiSettings {
    fn default() -> Self {
        Self {
            color_enabled: true,
            color_mode: ColorMode::Dark,
            motion_mode: MotionMode::Animated,
            line_mode: false,
            tick_rate: DEFAULT_TICK_RATE,
        }
    }
}

impl TuiSettings {
    pub(crate) fn from_env() -> Self {
        Self::from_env_lookup(|name| std::env::var_os(name))
    }

    fn from_env_lookup(mut var: impl FnMut(&str) -> Option<OsString>) -> Self {
        let color_enabled = var("NO_COLOR").is_none();
        let color_mode = var(COLOR_MODE_ENV)
            .and_then(|value| value.into_string().ok())
            .and_then(|value| ColorMode::from_env_value(&value))
            .unwrap_or(ColorMode::Dark);
        let motion_mode = MotionMode::from_animations_enabled(!truthy(var(REDUCED_MOTION_ENV)));
        let line_mode = truthy(var(LINE_MODE_ENV));

        Self {
            color_enabled,
            color_mode,
            motion_mode,
            line_mode,
            tick_rate: DEFAULT_TICK_RATE,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Palette {
    color_enabled: bool,
    text: Color,
    muted: Color,
    accent: Color,
    border: Color,
    selected_fg: Color,
    selected_bg: Color,
}

impl Palette {
    pub(crate) fn for_settings(settings: TuiSettings) -> Self {
        let mut palette = match settings.color_mode {
            ColorMode::Dark => Self {
                color_enabled: settings.color_enabled,
                text: Color::Gray,
                muted: Color::DarkGray,
                accent: Color::LightCyan,
                border: Color::Indexed(67),
                selected_fg: Color::Black,
                selected_bg: Color::LightCyan,
            },
            ColorMode::Light => Self {
                color_enabled: settings.color_enabled,
                text: Color::Black,
                muted: Color::DarkGray,
                accent: Color::Blue,
                border: Color::Indexed(31),
                selected_fg: Color::White,
                selected_bg: Color::Blue,
            },
        };
        if !settings.color_enabled {
            palette = palette.without_color();
        }
        palette
    }

    pub(crate) fn text(self) -> Style {
        self.fg(self.text)
    }

    pub(crate) fn muted(self) -> Style {
        self.fg(self.muted).add_modifier(Modifier::DIM)
    }

    pub(crate) fn accent(self) -> Style {
        self.fg(self.accent).add_modifier(Modifier::BOLD)
    }

    pub(crate) fn border(self) -> Style {
        self.fg(self.border)
    }

    pub(crate) fn selected(self) -> Style {
        if self.color_enabled {
            Style::new()
                .fg(self.selected_fg)
                .bg(self.selected_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().add_modifier(Modifier::REVERSED)
        }
    }

    fn fg(self, color: Color) -> Style {
        if self.color_enabled {
            Style::new().fg(color)
        } else {
            Style::new()
        }
    }

    fn without_color(self) -> Self {
        Self {
            color_enabled: false,
            text: Color::Reset,
            muted: Color::Reset,
            accent: Color::Reset,
            border: Color::Reset,
            selected_fg: Color::Reset,
            selected_bg: Color::Reset,
        }
    }
}

fn truthy(value: Option<OsString>) -> bool {
    let Some(value) = value else {
        return false;
    };
    let Some(value) = value.to_str() else {
        return true;
    };
    !matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "0" | "false" | "no" | "off"
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ratatui::style::Modifier;

    use super::*;

    fn settings_from(entries: &[(&str, &str)]) -> TuiSettings {
        let vars = entries
            .iter()
            .map(|(key, value)| ((*key).to_string(), OsString::from(value)))
            .collect::<HashMap<_, _>>();
        TuiSettings::from_env_lookup(|name| vars.get(name).cloned())
    }

    #[test]
    fn settings_honor_no_color_and_accessibility_switches() {
        let settings = settings_from(&[
            ("NO_COLOR", "1"),
            (REDUCED_MOTION_ENV, "true"),
            (LINE_MODE_ENV, "yes"),
            (COLOR_MODE_ENV, "light"),
        ]);

        assert!(!settings.color_enabled);
        assert_eq!(settings.motion_mode, MotionMode::Reduced);
        assert!(settings.line_mode);
        assert_eq!(settings.color_mode, ColorMode::Light);
    }

    #[test]
    fn false_like_env_values_do_not_enable_switches() {
        let settings = settings_from(&[(REDUCED_MOTION_ENV, "0"), (LINE_MODE_ENV, "off")]);
        assert_eq!(settings.motion_mode, MotionMode::Animated);
        assert!(!settings.line_mode);
    }

    #[test]
    fn color_mode_can_be_derived_from_background_luminance() {
        assert_eq!(
            ColorMode::from_background((250, 250, 250)),
            ColorMode::Light
        );
        assert_eq!(ColorMode::from_background((8, 12, 20)), ColorMode::Dark);
    }

    #[test]
    fn no_color_palette_uses_attributes_without_color() {
        let palette = Palette::for_settings(settings_from(&[("NO_COLOR", "1")]));
        assert_eq!(palette.accent().fg, None);
        assert_eq!(palette.selected().fg, None);
        assert!(palette.selected().add_modifier.contains(Modifier::REVERSED));
    }
}

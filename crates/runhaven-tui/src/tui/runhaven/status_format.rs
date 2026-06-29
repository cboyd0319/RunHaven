use std::path::Path;

use unicode_width::UnicodeWidthStr;

pub(crate) fn format_tokens_compact(value: i64) -> String {
    let value = value.max(0);
    if value == 0 {
        return "0".to_string();
    }
    if value < 1_000 {
        return value.to_string();
    }

    let value_f64 = value as f64;
    let (scaled, suffix) = if value >= 1_000_000_000_000 {
        (value_f64 / 1_000_000_000_000.0, "T")
    } else if value >= 1_000_000_000 {
        (value_f64 / 1_000_000_000.0, "B")
    } else if value >= 1_000_000 {
        (value_f64 / 1_000_000.0, "M")
    } else {
        (value_f64 / 1_000.0, "K")
    };

    let decimals = if scaled < 10.0 {
        2
    } else if scaled < 100.0 {
        1
    } else {
        0
    };

    let mut formatted = format!("{scaled:.decimals$}");
    if formatted.contains('.') {
        while formatted.ends_with('0') {
            formatted.pop();
        }
        if formatted.ends_with('.') {
            formatted.pop();
        }
    }

    format!("{formatted}{suffix}")
}

pub(crate) fn format_directory_display(directory: &Path, max_width: Option<usize>) -> String {
    let formatted = if let Some(rel) = crate::exec_command::relativize_to_home(directory) {
        if rel.as_os_str().is_empty() {
            "~".to_string()
        } else {
            format!("~{}{}", std::path::MAIN_SEPARATOR, rel.display())
        }
    } else {
        directory.display().to_string()
    };

    if let Some(max_width) = max_width {
        if max_width == 0 {
            return String::new();
        }
        if UnicodeWidthStr::width(formatted.as_str()) > max_width {
            return crate::text_formatting::center_truncate_path(&formatted, max_width);
        }
    }

    formatted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_tokens_compact_matches_status_bridge_behavior() {
        assert_eq!(format_tokens_compact(-1), "0");
        assert_eq!(format_tokens_compact(999), "999");
        assert_eq!(format_tokens_compact(1_200), "1.2K");
        assert_eq!(format_tokens_compact(12_345_678), "12.3M");
    }

    #[test]
    fn format_directory_display_honors_zero_width() {
        assert_eq!(
            format_directory_display(Path::new("/tmp/runhaven"), Some(0)),
            ""
        );
    }
}

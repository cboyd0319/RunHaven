use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::sync::LazyLock;

use regex::Regex;

use super::{AppEvent, AppLogSource};

const MAX_LOG_TAIL_BYTES: u64 = 64 * 1024;
const MAX_LOG_TAIL_LINES: usize = 200;
const MAX_APP_EVENTS: usize = 30;

static ENVELOPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9]{4}-[0-9]{2}-[0-9]{2})\]\[([0-9]{2}:[0-9]{2}:[0-9]{2})\]\[([^\]]{1,240})\]\[(DEBUG|INFO|WARN|ERROR)\] (.*)$")
        .expect("valid app log envelope regex")
});
static RUN_SUMMARY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^run [0-9a-f]{32}: summary profile=(antigravity|claude|codex|copilot) status=([a-z-]{1,24}) return_code=(-?[0-9]{1,5}) duration=([0-9]{1,10})s .*$")
        .expect("valid run summary regex")
});
static RUN_LAUNCH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^run [0-9a-f]{32}: (launching container process|container process spawned and running after ([0-9]{1,10}) ms|container process exited code=(-?[0-9]{1,5}) after ([0-9]{1,12}) ms total)$")
        .expect("valid run launch regex")
});

pub(super) fn read_app_events(source: &AppLogSource<'_>) -> (Vec<AppEvent>, usize) {
    let Ok(text) = read_trusted_log_tail(source) else {
        return (Vec::new(), 0);
    };
    parse_app_log(&text)
}

fn read_trusted_log_tail(source: &AppLogSource<'_>) -> anyhow::Result<String> {
    let parent_metadata = fs::symlink_metadata(source.trusted_parent)?;
    if parent_metadata.file_type().is_symlink() || !parent_metadata.is_dir() {
        anyhow::bail!("untrusted app log directory")
    }
    if source.path.file_name().and_then(|name| name.to_str()) != Some("runhaven.log") {
        anyhow::bail!("unexpected app log file")
    }
    let metadata = fs::symlink_metadata(source.path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        anyhow::bail!("untrusted app log file")
    }
    let trusted_parent = fs::canonicalize(source.trusted_parent)?;
    let path_parent = source
        .path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("app log has no parent"))?;
    if fs::canonicalize(path_parent)? != trusted_parent {
        anyhow::bail!("app log escaped its trusted directory")
    }

    let mut file = fs::File::open(source.path)?;
    let length = file.metadata()?.len();
    let start = length.saturating_sub(MAX_LOG_TAIL_BYTES);
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::new();
    file.take(MAX_LOG_TAIL_BYTES).read_to_end(&mut bytes)?;
    if start > 0 {
        if let Some(newline) = bytes.iter().position(|byte| *byte == b'\n') {
            bytes.drain(..=newline);
        } else {
            bytes.clear();
        }
    }
    let text = String::from_utf8_lossy(&bytes);
    Ok(text
        .lines()
        .rev()
        .take(MAX_LOG_TAIL_LINES)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n"))
}

pub(super) fn parse_app_log(text: &str) -> (Vec<AppEvent>, usize) {
    let mut events = Vec::new();
    let mut omitted = 0usize;
    for line in text.lines() {
        match parse_line(line) {
            Some(event) => events.push(event),
            None => omitted += 1,
        }
    }
    if events.len() > MAX_APP_EVENTS {
        omitted += events.len() - MAX_APP_EVENTS;
        events.drain(..events.len() - MAX_APP_EVENTS);
    }
    (events, omitted)
}

fn parse_line(line: &str) -> Option<AppEvent> {
    let captures = ENVELOPE.captures(line)?;
    let timestamp = format!("{}T{}Z", &captures[1], &captures[2]);
    let target = &captures[3];
    if target == "webview" || target.starts_with("webview:") {
        return None;
    }
    let message = &captures[5];
    if message.starts_with("RunHaven desktop ") && message.contains(" started; log file:") {
        return Some(AppEvent {
            timestamp,
            component: "app".to_string(),
            action: "start".to_string(),
            outcome: "ok".to_string(),
            duration_ms: None,
            exit_code: None,
        });
    }
    if message == "main window destroyed; closing terminal sessions" {
        return Some(AppEvent {
            timestamp,
            component: "app".to_string(),
            action: "close".to_string(),
            outcome: "ok".to_string(),
            duration_ms: None,
            exit_code: None,
        });
    }
    if let Some(summary) = RUN_SUMMARY.captures(message) {
        return Some(AppEvent {
            timestamp,
            component: summary[1].to_string(),
            action: "session-finished".to_string(),
            outcome: allowed_status(&summary[2]),
            duration_ms: summary[4]
                .parse::<u64>()
                .ok()
                .and_then(|seconds| seconds.checked_mul(1000)),
            exit_code: summary[3].parse().ok(),
        });
    }
    if let Some(launch) = RUN_LAUNCH.captures(message) {
        let phrase = &launch[1];
        let (action, outcome, duration_ms, exit_code) = if phrase == "launching container process" {
            ("launch", "started", None, None)
        } else if phrase.starts_with("container process spawned") {
            (
                "launch",
                "running",
                launch.get(2).and_then(|m| m.as_str().parse().ok()),
                None,
            )
        } else {
            (
                "launch",
                "exited",
                launch.get(4).and_then(|m| m.as_str().parse().ok()),
                launch.get(3).and_then(|m| m.as_str().parse().ok()),
            )
        };
        return Some(AppEvent {
            timestamp,
            component: "runtime".to_string(),
            action: action.to_string(),
            outcome: outcome.to_string(),
            duration_ms,
            exit_code,
        });
    }
    None
}

fn allowed_status(value: &str) -> String {
    match value {
        "ended" | "failed" | "killed" | "running" | "stopped" | "succeeded" => value.to_string(),
        _ => "unknown".to_string(),
    }
}

use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn check_script_descriptions(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    for path in maintained_script_files(root) {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let Ok(text) = fs::read_to_string(&path) else {
            failures.push(format!("{relative}: unreadable maintained script"));
            continue;
        };
        if !has_top_description(&text, path.file_name().and_then(|name| name.to_str())) {
            failures.push(format!(
                "{relative}: add two top comment lines describing what this file is and what it does"
            ));
        }
    }
    failures
}

fn maintained_script_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let init = root.join("init.sh");
    if init.is_file() {
        files.push(init);
    }
    files.extend(files_in(root.join("scripts"), |path| {
        path.extension().and_then(|value| value.to_str()) == Some("sh")
    }));
    files.extend(files_in(root.join("images"), |path| {
        path.file_name().and_then(|value| value.to_str()) == Some("Containerfile")
            || path.extension().and_then(|value| value.to_str()) == Some("sh")
    }));
    files.sort();
    files.dedup();
    files
}

fn files_in(dir: PathBuf, keep: impl Fn(&Path) -> bool + Copy) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(files_in(path, keep));
        } else if path.is_file() && keep(&path) {
            files.push(path);
        }
    }
    files
}

fn has_top_description(text: &str, file_name: Option<&str>) -> bool {
    let mut lines = text.lines();
    if file_name != Some("Containerfile") && lines.next().is_none_or(|line| !line.starts_with("#!"))
    {
        return false;
    }
    lines
        .take(3)
        .filter(|line| is_description_line(line))
        .count()
        >= 2
}

fn is_description_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("# ") && trimmed.trim().len() > 10
}

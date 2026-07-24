fn inline_module_declarations(module_source: &str) -> Vec<String> {
    module_source
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            ["pub(crate) mod ", "pub mod ", "mod "]
                .iter()
                .find_map(|prefix| {
                    line.strip_prefix(prefix)
                        .and_then(|rest| rest.strip_suffix(" {"))
                })
        })
        .map(str::to_string)
        .collect()
}

fn top_level_inline_module_declarations(module_source: &str) -> Vec<String> {
    module_source
        .lines()
        .filter_map(|line| {
            ["pub(crate) mod ", "pub mod ", "mod "]
                .iter()
                .find_map(|prefix| {
                    line.strip_prefix(prefix)
                        .and_then(|rest| rest.strip_suffix(" {"))
                })
        })
        .map(str::to_string)
        .collect()
}

fn tui_source_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tui")
}

fn tui_rust_sources() -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![tui_source_dir()];
    while let Some(path) = pending.pop() {
        let metadata = std::fs::metadata(&path).expect("metadata should be readable");
        if metadata.is_dir() {
            for entry in std::fs::read_dir(&path).expect("directory should be readable") {
                pending.push(entry.expect("directory entry should be readable").path());
            }
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn relative_tui_source(path: &std::path::Path) -> std::path::PathBuf {
    path.strip_prefix(tui_source_dir())
        .expect("path should be under tui source")
        .to_path_buf()
}

fn is_guard_source(path: &std::path::Path) -> bool {
    matches!(path.to_str(), Some("mod.rs")) || path.starts_with("drift_tests")
}

fn module_declared(module_source: &str, module: &str) -> bool {
    let private_decl = format!("mod {module};");
    let crate_decl = format!("pub(crate) mod {module};");
    let public_decl = format!("pub mod {module};");
    module_source
        .lines()
        .map(str::trim)
        .any(|line| line == private_decl || line == crate_decl || line == public_decl)
}

fn source_file_declared_by_path(module_source: &str, source_path: &str) -> bool {
    let expected_path_attr = format!("#[path = \"{source_path}\"]");
    let mut pending_path_attr = false;
    let mut pending_block_comment = false;

    'next_line: for line in module_source.lines().map(str::trim) {
        if line == expected_path_attr {
            pending_path_attr = true;
            continue;
        }

        if pending_path_attr {
            let mut remaining = line;
            loop {
                let trimmed = remaining.trim_start();
                if trimmed.is_empty() || trimmed.starts_with("//") {
                    continue 'next_line;
                }

                if pending_block_comment {
                    if let Some(end) = trimmed.find("*/") {
                        pending_block_comment = false;
                        remaining = &trimmed[end + 2..];
                        continue;
                    }
                    continue 'next_line;
                }

                if let Some(after_start) = trimmed.strip_prefix("/*") {
                    if let Some(end) = after_start.find("*/") {
                        remaining = &after_start[end + 2..];
                        continue;
                    }
                    pending_block_comment = true;
                    continue 'next_line;
                }

                if trimmed.starts_with("#[") {
                    continue 'next_line;
                }

                remaining = trimmed;
                break;
            }

            let declaration = remaining.split("//").next().unwrap_or(remaining).trim();
            return ["mod ", "pub(crate) mod ", "pub mod "]
                .iter()
                .any(|prefix| declaration.starts_with(prefix) && declaration.ends_with(';'));
        }
    }

    false
}

#[test]
fn source_file_path_guard_skips_comments_before_module_item() {
    let module_source = r#"
#[path = "app.rs"]

// allowed between a path attribute and the item
/* also allowed */
pub(crate) mod native_app;
"#;
    let multiline_block = r#"
#[path = "chatwidget.rs"]
/*
also allowed
*/
mod native_chatwidget;
"#;
    let inline_block = r#"
#[path = "app.rs"]
/* allowed */ mod native_app;
"#;

    assert!(source_file_declared_by_path(module_source, "app.rs"));
    assert!(source_file_declared_by_path(
        multiline_block,
        "chatwidget.rs"
    ));
    assert!(source_file_declared_by_path(inline_block, "app.rs"));
}

fn inline_module_block<'a>(module_source: &'a str, module: &str) -> &'a str {
    let declaration = format!("pub(crate) mod {module} {{");
    let start = module_source
        .find(&declaration)
        .unwrap_or_else(|| panic!("inline module {module} should be declared"));
    let rest = &module_source[start..];
    let mut depth = 0usize;

    for (offset, character) in rest.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth = depth
                    .checked_sub(1)
                    .expect("inline module block should have balanced braces");
                if depth == 0 {
                    return &rest[..=offset];
                }
            }
            _ => {}
        }
    }

    panic!("inline module {module} should have a closing brace");
}

fn workspace_member_declared(root_manifest_source: &str, member: &str) -> bool {
    let quoted = format!("\"{member}\"");
    root_manifest_source
        .lines()
        .map(str::trim)
        .any(|line| line.trim_end_matches(',') == quoted)
}

fn assert_risky_markers_absent_when_active(
    module_source: &str,
    module: &str,
    source_path: &str,
    source: &str,
    markers: &[&str],
) {
    if !module_declared(module_source, module)
        && !source_file_declared_by_path(module_source, source_path)
    {
        return;
    }

    for marker in markers {
        assert!(
            !source.contains(marker),
            "{module} is declared in tui/mod.rs, but {source_path} still contains risky upstream marker {marker:?}; remove or fail-close that behavior before activating the module"
        );
    }
}

mod activation;
mod controls;
mod facade;
mod observability;

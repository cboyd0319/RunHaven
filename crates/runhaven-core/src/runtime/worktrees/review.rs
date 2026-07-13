use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File, Metadata};
use std::io::Read;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::WorktreeLifecycle;

mod git;

use git::{collect_git_stdout, stream_git_stdout_into};

const MAX_GIT_REVIEW_OUTPUT_BYTES: usize = 1024 * 1024;
const MAX_DISPLAY_PATH_BYTES: usize = 1024;
const MAX_DISPLAY_CHANGES: usize = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WorktreeChangeKind {
    Added,
    Changed,
    Removed,
}

impl WorktreeChangeKind {
    fn digest_tag(self) -> u8 {
        match self {
            Self::Added => b'A',
            Self::Changed => b'M',
            Self::Removed => b'D',
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeFileChange {
    pub path: String,
    pub kind: WorktreeChangeKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeChangeSummary {
    pub total_count: usize,
    pub file_changes: Vec<WorktreeFileChange>,
    pub file_changes_truncated: bool,
    pub review_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeFileSummary {
    pub total_count: usize,
    pub file_changes: Vec<WorktreeFileChange>,
    pub file_changes_truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RawChange {
    path: Vec<u8>,
    kind: WorktreeChangeKind,
}

pub fn worktree_change_summary(lifecycle: &WorktreeLifecycle) -> Result<WorktreeChangeSummary> {
    validate_base_head(&lifecycle.base_head)?;
    let (changes, untracked_output) =
        collect_changes(&lifecycle.worktree_root, &lifecycle.base_head)?;
    let review_id = review_digest(lifecycle, &changes, &untracked_output)?;
    let summary = summarize_files(&changes);
    Ok(WorktreeChangeSummary {
        total_count: summary.total_count,
        file_changes: summary.file_changes,
        file_changes_truncated: summary.file_changes_truncated,
        review_id,
    })
}

/// Check whether a protected copy has file changes without hashing file
/// contents for a review id. Session-list polling needs only this boolean; the
/// full review command computes the digest immediately before showing actions.
pub fn worktree_has_changes(lifecycle: &WorktreeLifecycle) -> Result<bool> {
    validate_base_head(&lifecycle.base_head)?;
    let (changes, _) = collect_changes(&lifecycle.worktree_root, &lifecycle.base_head)?;
    Ok(!changes.is_empty())
}

pub fn source_worktree_change_summary(
    lifecycle: &WorktreeLifecycle,
) -> Result<WorktreeFileSummary> {
    let head_output = collect_git_stdout(
        &lifecycle.source_repo_root,
        &["rev-parse", "--verify", "HEAD"],
        "resolve source HEAD",
    )?;
    let head = std::str::from_utf8(&head_output)
        .context("source HEAD is not UTF-8")?
        .trim();
    validate_base_head(head)?;
    let (changes, _) = collect_changes(&lifecycle.source_repo_root, head)?;
    Ok(summarize_files(&changes))
}

fn collect_changes(root: &Path, base_head: &str) -> Result<(Vec<RawChange>, Vec<u8>)> {
    let tracked_output = collect_git_stdout(
        root,
        &[
            "diff",
            "--name-status",
            "-z",
            "--no-ext-diff",
            "--no-renames",
            base_head,
            "--",
        ],
        "list tracked worktree changes",
    )?;
    let untracked_output = collect_git_stdout(
        root,
        &["ls-files", "--others", "--exclude-standard", "-z", "--"],
        "list untracked worktree files",
    )?;
    let mut changes = BTreeMap::new();
    for change in parse_name_status(&tracked_output)?
        .into_iter()
        .chain(parse_untracked_paths(&untracked_output)?)
    {
        if changes.insert(change.path, change.kind).is_some() {
            bail!("git returned the same worktree path more than once");
        }
    }
    let changes = changes
        .into_iter()
        .map(|(path, kind)| RawChange { path, kind })
        .collect::<Vec<_>>();
    Ok((changes, untracked_output))
}

fn summarize_files(changes: &[RawChange]) -> WorktreeFileSummary {
    let (file_changes, file_changes_truncated) = visible_changes(changes);
    WorktreeFileSummary {
        total_count: changes.len(),
        file_changes,
        file_changes_truncated,
    }
}

fn parse_name_status(output: &[u8]) -> Result<Vec<RawChange>> {
    validate_output_bound(output)?;
    let tokens = nul_tokens(output)?;
    if tokens.len() % 2 != 0 {
        bail!("git returned incomplete worktree status data");
    }
    tokens
        .chunks_exact(2)
        .map(|pair| {
            let kind = match pair[0] {
                b"A" => WorktreeChangeKind::Added,
                b"D" => WorktreeChangeKind::Removed,
                b"M" | b"T" | b"U" => WorktreeChangeKind::Changed,
                _ => bail!("git returned an unsupported worktree status"),
            };
            validate_relative_path(pair[1])?;
            Ok(RawChange {
                path: pair[1].to_vec(),
                kind,
            })
        })
        .collect()
}

fn parse_untracked_paths(output: &[u8]) -> Result<Vec<RawChange>> {
    validate_output_bound(output)?;
    nul_tokens(output)?
        .into_iter()
        .map(|path| {
            validate_relative_path(path)?;
            Ok(RawChange {
                path: path.to_vec(),
                kind: WorktreeChangeKind::Added,
            })
        })
        .collect()
}

fn nul_tokens(output: &[u8]) -> Result<Vec<&[u8]>> {
    if output.is_empty() {
        return Ok(Vec::new());
    }
    if output.last() != Some(&0) {
        bail!("git returned unterminated worktree path data");
    }
    Ok(output[..output.len() - 1]
        .split(|byte| *byte == 0)
        .collect())
}

fn validate_output_bound(output: &[u8]) -> Result<()> {
    if output.len() > MAX_GIT_REVIEW_OUTPUT_BYTES {
        bail!("git worktree review output exceeded the safe limit");
    }
    Ok(())
}

fn validate_relative_path(path: &[u8]) -> Result<()> {
    if path.is_empty() || path.starts_with(b"/") {
        bail!("git returned a non-relative worktree path");
    }
    if path
        .split(|byte| *byte == b'/')
        .any(|part| part.is_empty() || part == b"." || part == b"..")
    {
        bail!("git returned an unsafe worktree path");
    }
    Ok(())
}

fn visible_changes(changes: &[RawChange]) -> (Vec<WorktreeFileChange>, bool) {
    let mut visible = Vec::new();
    for change in changes {
        if visible.len() == MAX_DISPLAY_CHANGES {
            break;
        }
        let Some(path) = display_path(&change.path) else {
            continue;
        };
        visible.push(WorktreeFileChange {
            path,
            kind: change.kind,
        });
    }
    let truncated = visible.len() != changes.len();
    (visible, truncated)
}

fn display_path(path: &[u8]) -> Option<String> {
    if path.len() > MAX_DISPLAY_PATH_BYTES {
        return None;
    }
    let path = std::str::from_utf8(path).ok()?;
    if path.chars().any(is_unsafe_display_character) {
        return None;
    }
    Some(path.to_string())
}

fn is_unsafe_display_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{00ad}'
                | '\u{034f}'
                | '\u{061c}'
                | '\u{180e}'
                | '\u{200b}'..='\u{200f}'
                | '\u{2028}'..='\u{202e}'
                | '\u{2060}'..='\u{2069}'
                | '\u{feff}'
        )
}

fn validate_base_head(base_head: &str) -> Result<()> {
    if !matches!(base_head.len(), 40 | 64)
        || !base_head.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("recorded worktree base is not an exact commit id");
    }
    Ok(())
}

fn review_digest(
    lifecycle: &WorktreeLifecycle,
    changes: &[RawChange],
    untracked_output: &[u8],
) -> Result<String> {
    let mut digest = Sha256::new();
    digest.update(b"runhaven-worktree-review-v1\0");
    update_framed(&mut digest, lifecycle.run_id.as_bytes());
    update_framed(&mut digest, lifecycle.base_head.as_bytes());
    stream_git_stdout_into(
        &lifecycle.worktree_root,
        &[
            "diff",
            "--binary",
            "--full-index",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            &lifecycle.base_head,
            "--",
        ],
        "hash tracked worktree changes",
        &mut digest,
    )?;
    digest.update(b"\0untracked\0");
    let mut untracked = parse_untracked_paths(untracked_output)?;
    untracked.sort_by(|left, right| left.path.cmp(&right.path));
    for change in untracked {
        hash_untracked_leaf(&lifecycle.worktree_root, &change.path, &mut digest)?;
    }
    digest.update(b"\0summary\0");
    for change in changes {
        digest.update([change.kind.digest_tag()]);
        update_framed(&mut digest, &change.path);
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn hash_untracked_leaf(root: &Path, path: &[u8], digest: &mut Sha256) -> Result<()> {
    let root = root.canonicalize()?;
    let relative = PathBuf::from(OsString::from_vec(path.to_vec()));
    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    let parent = root.join(parent).canonicalize()?;
    if !parent.starts_with(&root) {
        bail!("untracked worktree path leaves the verified worktree");
    }
    let name = relative
        .file_name()
        .context("untracked worktree path has no leaf name")?;
    let leaf = parent.join(name);
    let before = fs::symlink_metadata(&leaf)?;
    digest.update(b"leaf\0");
    update_framed(digest, path);
    digest.update(before.mode().to_be_bytes());
    if before.file_type().is_symlink() {
        digest.update(b"symlink\0");
        let target = fs::read_link(&leaf)?;
        update_framed(digest, target.as_os_str().as_bytes());
        ensure_metadata_unchanged(&before, &fs::symlink_metadata(&leaf)?)?;
    } else if before.is_file() {
        digest.update(b"file\0");
        digest.update(before.len().to_be_bytes());
        let mut file = File::open(&leaf)?;
        ensure_metadata_unchanged(&before, &file.metadata()?)?;
        let mut read = 0_u64;
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            read = read
                .checked_add(count as u64)
                .context("untracked worktree file is too large")?;
            if read > before.len() {
                bail!("untracked worktree file changed while it was reviewed");
            }
            digest.update(&buffer[..count]);
        }
        if read != before.len() {
            bail!("untracked worktree file changed while it was reviewed");
        }
        ensure_metadata_unchanged(&before, &file.metadata()?)?;
    } else {
        bail!("untracked worktree path is not a regular file or symbolic link");
    }
    Ok(())
}

fn ensure_metadata_unchanged(before: &Metadata, after: &Metadata) -> Result<()> {
    if before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.mode() != after.mode()
        || before.len() != after.len()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
    {
        bail!("worktree file changed while it was reviewed");
    }
    Ok(())
}

fn update_framed(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

#[cfg(test)]
mod tests;

use std::io::{Read, sink};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use super::MAX_GIT_REVIEW_OUTPUT_BYTES;

pub(super) fn collect_git_stdout(cwd: &Path, args: &[&str], action: &str) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    run_git_stdout(cwd, args, action, |chunk| output.extend_from_slice(chunk))?;
    Ok(output)
}

pub(super) fn stream_git_stdout_into(
    cwd: &Path,
    args: &[&str],
    action: &str,
    digest: &mut Sha256,
) -> Result<()> {
    run_git_stdout(cwd, args, action, |chunk| digest.update(chunk))
}

fn run_git_stdout(
    cwd: &Path,
    args: &[&str],
    action: &str,
    mut consume: impl FnMut(&[u8]),
) -> Result<()> {
    let mut child = Command::new("git")
        .args(["-C"])
        .arg(cwd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("{action} failed: git is unavailable"))?;
    let mut stdout = child.stdout.take().context("git stdout missing")?;
    let stderr = child.stderr.take().context("git stderr missing")?;
    let stderr_reader = thread::spawn(move || read_capped(stderr));
    let mut total = 0_usize;
    let mut over_cap = false;
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let count = stdout.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total = total
            .checked_add(count)
            .context("git output is too large")?;
        if total <= MAX_GIT_REVIEW_OUTPUT_BYTES {
            consume(&buffer[..count]);
        } else {
            over_cap = true;
        }
    }
    let status = child.wait()?;
    let (stderr, stderr_truncated) = stderr_reader
        .join()
        .map_err(|_| anyhow::anyhow!("git stderr reader failed"))??;
    if over_cap {
        bail!("git worktree review output exceeded the safe limit");
    }
    if !status.success() {
        let mut detail = String::from_utf8_lossy(&stderr).trim().to_string();
        if stderr_truncated {
            detail.push_str(" [output truncated]");
        }
        bail!(
            "{action} failed: {}",
            if detail.is_empty() {
                status.to_string()
            } else {
                detail
            }
        );
    }
    Ok(())
}

fn read_capped(mut reader: impl Read) -> std::io::Result<(Vec<u8>, bool)> {
    let mut output = Vec::new();
    let mut limited = reader
        .by_ref()
        .take((MAX_GIT_REVIEW_OUTPUT_BYTES + 1) as u64);
    limited.read_to_end(&mut output)?;
    let truncated = output.len() > MAX_GIT_REVIEW_OUTPUT_BYTES;
    output.truncate(MAX_GIT_REVIEW_OUTPUT_BYTES);
    std::io::copy(&mut reader, &mut sink())?;
    Ok((output, truncated))
}

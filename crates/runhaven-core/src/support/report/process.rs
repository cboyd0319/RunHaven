use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(20);
const PIPE_DRAIN_TIMEOUT: Duration = Duration::from_millis(250);

pub(super) struct BoundedOutput {
    pub success: bool,
    pub timed_out: bool,
    pub stdout: Vec<u8>,
}

pub(super) fn run_bounded(
    program: &Path,
    args: &[&str],
    timeout: Duration,
    max_bytes: usize,
) -> std::io::Result<BoundedOutput> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child.stdout.take().expect("piped stdout");
    let (stdout_sender, stdout_reader) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = stdout_sender.send(read_bounded(stdout, max_bytes));
    });
    let deadline = Instant::now() + timeout;
    let (status, timed_out) = loop {
        if let Some(status) = child.try_wait()? {
            break (status, false);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            break (child.wait()?, true);
        }
        thread::sleep(POLL_INTERVAL);
    };
    // A broken CLI can leave a descendant holding the inherited stdout pipe
    // after the direct child exits. Never let that defeat the report deadline.
    let stdout = stdout_reader
        .recv_timeout(PIPE_DRAIN_TIMEOUT)
        .unwrap_or_default();
    Ok(BoundedOutput {
        success: status.success() && !timed_out,
        timed_out,
        stdout,
    })
}

fn read_bounded(reader: impl Read, max_bytes: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    let _ = reader
        .take(max_bytes.saturating_add(1) as u64)
        .read_to_end(&mut bytes);
    bytes.truncate(max_bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hanging_process_is_killed_at_the_deadline() {
        let started = Instant::now();
        let output = run_bounded(
            Path::new("/bin/sleep"),
            &["2"],
            Duration::from_millis(80),
            1024,
        )
        .expect("sleep process");

        assert!(output.timed_out);
        assert!(!output.success);
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[cfg(unix)]
    #[test]
    fn inherited_stdout_pipe_cannot_extend_the_deadline() {
        let started = Instant::now();
        let output = run_bounded(
            Path::new("/bin/sh"),
            &["-c", "sleep 2 &"],
            Duration::from_millis(200),
            1024,
        )
        .expect("shell process");

        assert!(output.success);
        assert!(!output.timed_out);
        assert!(started.elapsed() < Duration::from_secs(1));
    }
}

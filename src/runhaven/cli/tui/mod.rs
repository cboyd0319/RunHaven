use anyhow::Result;

#[allow(dead_code, unused_imports)]
pub(crate) mod pets;
#[allow(dead_code)]
pub(crate) mod terminal_detection;

#[allow(dead_code)]
#[path = "tui/frame_rate_limiter.rs"]
mod frame_rate_limiter;
#[allow(dead_code)]
#[path = "tui/frame_requester.rs"]
mod frame_requester;

pub use frame_requester::FrameRequester;

pub fn run() -> Result<i32> {
    eprintln!(
        "RunHaven TUI is being rebuilt from the Codex TUI source. Use a subcommand for now, such as `runhaven plan` or `runhaven run`."
    );
    Ok(2)
}

use anyhow::Result;

pub fn run() -> Result<i32> {
    eprintln!(
        "RunHaven TUI is being rebuilt from the Codex TUI source. Use a subcommand for now, such as `runhaven plan` or `runhaven run`."
    );
    Ok(2)
}

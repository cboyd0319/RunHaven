mod io;
pub mod run_history;

pub use io::read_jsonl;
pub use run_history::{
    RunRecordInput, find_run_record, format_git_summary, print_run_record, read_run_records,
    runs_diff, runs_list, runs_log, runs_show, summarize_auth_broker, summarize_provider_policy,
    write_run_record,
};

/// Compatibility facade for callers that still use the pre-facade module name.
pub mod history {
    pub use super::run_history::*;
}

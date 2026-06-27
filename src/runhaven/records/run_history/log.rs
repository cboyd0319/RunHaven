use anyhow::Result;
use serde_json::{Value, json};

use super::{find_run_record, print_run_record};

pub fn runs_log(
    run_id: &str,
    json_output: bool,
    provider_entries: Vec<Value>,
    auth_entries: Vec<Value>,
) -> Result<i32> {
    let record = find_run_record(run_id)?;
    let provider_entries = provider_entries
        .into_iter()
        .filter(|entry| entry.get("run_id").and_then(Value::as_str) == Some(run_id))
        .collect::<Vec<_>>();
    let auth_entries = auth_entries
        .into_iter()
        .filter(|entry| entry.get("run_id").and_then(Value::as_str) == Some(run_id))
        .collect::<Vec<_>>();
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &json!({"run": record, "provider_policy": provider_entries, "auth_broker": auth_entries})
            )?
        );
        return Ok(0);
    }
    print_run_record(&record);
    println!("Provider policy decisions:");
    if provider_entries.is_empty() {
        println!("  none");
    } else {
        for entry in provider_entries {
            println!(
                "  - {}  {}  {}:{}  count={}  reason={}  rule={}",
                entry
                    .get("timestamp")
                    .and_then(Value::as_str)
                    .unwrap_or("<unknown>"),
                entry
                    .get("decision")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                entry
                    .get("host")
                    .and_then(Value::as_str)
                    .unwrap_or("<unknown>"),
                entry
                    .get("port")
                    .map(Value::to_string)
                    .unwrap_or_else(|| "?".to_string()),
                entry
                    .get("count")
                    .map(Value::to_string)
                    .unwrap_or_else(|| "1".to_string()),
                entry
                    .get("reason")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                entry
                    .get("matched_rule")
                    .and_then(Value::as_str)
                    .unwrap_or("-"),
            );
        }
    }
    println!("Auth broker decisions:");
    if auth_entries.is_empty() {
        println!("  none");
    } else {
        for entry in auth_entries {
            println!(
                "  - {}  {}  {}  {} {}  status={}  count={}  reason={}",
                entry
                    .get("timestamp")
                    .and_then(Value::as_str)
                    .unwrap_or("<unknown>"),
                entry
                    .get("broker")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                entry
                    .get("decision")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                entry.get("method").and_then(Value::as_str).unwrap_or("-"),
                entry.get("path").and_then(Value::as_str).unwrap_or("-"),
                entry
                    .get("upstream_status")
                    .map(Value::to_string)
                    .unwrap_or_else(|| "-".to_string()),
                entry
                    .get("count")
                    .map(Value::to_string)
                    .unwrap_or_else(|| "1".to_string()),
                entry
                    .get("reason")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
            );
        }
    }
    Ok(0)
}

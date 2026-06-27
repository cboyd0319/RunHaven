use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::Value;

pub fn read_jsonl(path: &Path, limit: usize) -> Result<Vec<Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    for line in fs::read_to_string(path)?.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(payload) = serde_json::from_str::<Value>(line)
            && payload.is_object()
        {
            records.push(payload);
        }
    }
    if limit == 0 || records.len() <= limit {
        return Ok(records);
    }
    Ok(records[records.len() - limit..].to_vec())
}

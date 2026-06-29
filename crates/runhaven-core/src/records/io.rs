use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::Result;
use serde_json::Value;

pub fn read_jsonl(path: &Path, limit: usize) -> Result<Vec<Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    Ok(limit_records(
        parse_jsonl_lines(&fs::read_to_string(path)?),
        limit,
    ))
}

pub fn read_jsonl_tail_bounded(
    path: &Path,
    limit: usize,
    max_tail_bytes: u64,
) -> Result<Vec<Value>> {
    if limit == 0 {
        return read_jsonl(path, 0);
    }
    if !path.exists() || max_tail_bytes == 0 {
        return Ok(Vec::new());
    }

    let len = fs::metadata(path)?.len();
    if len == 0 {
        return Ok(Vec::new());
    }
    let start = len.saturating_sub(max_tail_bytes.min(len));
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let text = String::from_utf8_lossy(&bytes);
    let text = if start == 0 {
        text.as_ref()
    } else {
        text.split_once('\n').map_or("", |(_, rest)| rest)
    };

    Ok(limit_records(parse_jsonl_lines(text), limit))
}

fn parse_jsonl_lines(text: &str) -> Vec<Value> {
    let mut records = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(payload) = serde_json::from_str::<Value>(line)
            && payload.is_object()
        {
            records.push(payload);
        }
    }
    records
}

fn limit_records(records: Vec<Value>, limit: usize) -> Vec<Value> {
    if limit == 0 || records.len() <= limit {
        return records;
    }
    records[records.len() - limit..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn bounded_tail_reader_keeps_recent_jsonl_records() {
        let dir = tempfile::tempdir().expect("dir");
        let path = dir.path().join("events.jsonl");
        let mut file = std::fs::File::create(&path).expect("file");
        for idx in 0..20 {
            writeln!(file, "{{\"idx\":{idx}}}").expect("write");
        }

        let records = read_jsonl_tail_bounded(&path, 3, 512).expect("tail records");

        let indexes = records
            .iter()
            .map(|record| record.get("idx").and_then(Value::as_u64).expect("idx"))
            .collect::<Vec<_>>();
        assert_eq!(indexes, vec![17, 18, 19]);
    }
}

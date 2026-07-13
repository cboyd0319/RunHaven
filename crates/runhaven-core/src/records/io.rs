use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

use crate::support::paths::open_private_append;

const MALFORMED_JSONL_WARNING: &str =
    "RunHaven skipped a malformed nonempty JSONL record; record content was not logged";

pub(crate) fn append_private_jsonl<T: Serialize>(path: &Path, records: &[T]) -> Result<()> {
    if records.is_empty() {
        return Ok(());
    }
    let mut encoded = Vec::new();
    for record in records {
        serde_json::to_writer(&mut encoded, record)?;
        encoded.push(b'\n');
    }

    let mut file = open_private_append(path)?;
    file.lock()?;
    file.write_all(&encoded)?;
    Ok(())
}

pub fn read_jsonl(path: &Path, limit: usize) -> Result<Vec<Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(path)?;
    let lines = BufReader::new(file).lines();
    if limit == 0 {
        let mut records = Vec::new();
        for line in lines {
            if let Some(payload) = parse_jsonl_line(&line?) {
                records.push(payload);
            }
        }
        return Ok(records);
    }

    let mut records = VecDeque::with_capacity(limit);
    for line in lines {
        if let Some(payload) = parse_jsonl_line(&line?) {
            if records.len() == limit {
                records.pop_front();
            }
            records.push_back(payload);
        }
    }
    Ok(records.into_iter().collect())
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
        if let Some(payload) = parse_jsonl_line(line) {
            records.push(payload);
        }
    }
    records
}

fn parse_jsonl_line(line: &str) -> Option<Value> {
    parse_jsonl_line_with_warning(line, |message| log::warn!("{message}"))
}

fn parse_jsonl_line_with_warning(line: &str, warn: impl FnOnce(&str)) -> Option<Value> {
    if line.trim().is_empty() {
        return None;
    }
    match serde_json::from_str::<Value>(line) {
        Ok(payload) if payload.is_object() => Some(payload),
        _ => {
            warn(MALFORMED_JSONL_WARNING);
            None
        }
    }
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
    use std::collections::HashSet;
    use std::io::Write;
    use std::sync::{Arc, Barrier};

    #[test]
    fn concurrent_private_jsonl_appends_keep_every_record_intact() {
        const WRITERS: usize = 16;
        const RECORDS_PER_WRITER: usize = 64;

        let dir = tempfile::tempdir().expect("dir");
        let path = dir.path().join("events.jsonl");
        let barrier = Arc::new(Barrier::new(WRITERS));
        let handles = (0..WRITERS)
            .map(|writer| {
                let path = path.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    for record in 0..RECORDS_PER_WRITER {
                        append_private_jsonl(
                            &path,
                            &[serde_json::json!({
                                "id": format!("{writer}-{record}"),
                                "padding": "x".repeat(1024),
                            })],
                        )
                        .expect("append record");
                    }
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().expect("writer thread");
        }

        let records = read_jsonl(&path, 0).expect("records");
        let ids = records
            .iter()
            .map(|record| record["id"].as_str().expect("record id"))
            .collect::<HashSet<_>>();
        assert_eq!(records.len(), WRITERS * RECORDS_PER_WRITER);
        assert_eq!(ids.len(), records.len());
    }

    #[test]
    fn malformed_nonempty_jsonl_warns_without_record_content_and_is_skipped() {
        let malformed = r#"{"secret":"never-log-this""#;
        let mut warning = None;

        assert!(
            parse_jsonl_line_with_warning(malformed, |message| warning = Some(message.to_string()))
                .is_none()
        );
        assert_eq!(warning.as_deref(), Some(MALFORMED_JSONL_WARNING));
        assert!(!warning.expect("warning").contains("never-log-this"));

        let dir = tempfile::tempdir().expect("dir");
        let path = dir.path().join("recovery.jsonl");
        std::fs::write(
            &path,
            format!("{{\"idx\":1}}\n{malformed}\n{{\"idx\":2}}\n"),
        )
        .expect("fixture");
        let records = read_jsonl(&path, 0).expect("tolerant read");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["idx"], 1);
        assert_eq!(records[1]["idx"], 2);
    }

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

    #[test]
    fn read_jsonl_limit_keeps_recent_records_without_full_text_read() {
        let dir = tempfile::tempdir().expect("dir");
        let path = dir.path().join("events.jsonl");
        let mut file = std::fs::File::create(&path).expect("file");
        for idx in 0..20 {
            writeln!(file, "{{\"idx\":{idx}}}").expect("write");
        }

        let records = read_jsonl(&path, 4).expect("limited records");

        let indexes = records
            .iter()
            .map(|record| record.get("idx").and_then(Value::as_u64).expect("idx"))
            .collect::<Vec<_>>();
        assert_eq!(indexes, vec![16, 17, 18, 19]);
    }
}

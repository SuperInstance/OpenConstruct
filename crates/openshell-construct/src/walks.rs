// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! The walks file recorder — `walks/2` persistence (RFC 0004, §3).
//!
//! A room's walk log lives on disk as JSONL, append-only, byte-exact:
//!
//! - One **walk line** per arrival: the compact JSON encoding of a
//!   [`WalkRecord`] — `{ts, road, link_quality, arrival_meta}`, zero
//!   semantics, exactly the walks/2 schema.
//! - After each walk line, one **checkpoint line**:
//!   `{"chain_head":"<64 hex>","records":N}` — the persisted chain head
//!   (the `prev_chain`) covering exactly the first `N` walk lines. The
//!   checkpoint is how chain continuity survives restarts: a reloaded log
//!   continues from the checkpointed head, and any walk line edited,
//!   dropped, or corrupted since its checkpoint reads as a chain break.
//!
//! Loading is **tolerant**: blank lines are ignored, malformed lines are
//! skipped and counted (never a hard failure — a keeper reads a damaged
//! walks file the way a skipper reads choppy water), and continuity is
//! checked against every checkpoint. Legacy files (plain walk lines, no
//! checkpoints — the v0 / rd shape) load cleanly with nothing to check.
//!
//! Honest limits: a walk deleted *together with its checkpoint* leaves a
//! shorter but self-consistent file — the anchor for that audit is the
//! previous save's persisted head, held by the keeper, not the file.

use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::residency::{WalkLog, WalkRecord};

/// A chain checkpoint line: the persisted chain head ("prev_chain")
/// covering exactly the first `records` walk lines of the file.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckpointLine {
    chain_head: String,
    records: u64,
}

/// What the loader found while reading a walks file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadReport {
    /// Walk lines successfully loaded.
    pub records_loaded: usize,
    /// Lines that were neither walk lines, checkpoint lines, nor blank —
    /// skipped, and counted here for the keeper.
    pub skipped_malformed: usize,
    /// Checkpoints whose recomputed head matched (tamper-evidence).
    pub checkpoints_verified: usize,
    /// A checkpoint disagreed with the recomputed chain, or the file's
    /// tail is unpinned (walk lines after the last checkpoint). The log's
    /// [`WalkLog::verify`] will also read `false`.
    pub chain_broken: bool,
}

/// A loaded walks file: a ready-to-continue recorder plus its report.
#[derive(Debug, Clone)]
pub struct LoadOutcome {
    /// The recorder, positioned at the loaded head — appends continue the
    /// chain across the restart.
    pub recorder: WalkRecorder,
    /// What the loader saw.
    pub report: LoadReport,
}

/// Append-only `walks/2` recorder for one room.
///
/// [`WalkRecorder::record`] appends a walk line plus its checkpoint to the
/// file and to the in-memory [`WalkLog`] in lockstep; normal operation
/// never rewrites a byte. [`WalkRecorder::save`] is a full flush (atomic
/// temp-file rename) for creation, migration, or repair.
#[derive(Debug, Clone)]
pub struct WalkRecorder {
    path: PathBuf,
    log: WalkLog,
}

impl WalkRecorder {
    /// A recorder over `path` (not touched until the first write).
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            log: WalkLog::new(),
        }
    }

    /// The in-memory log (chain continues from whatever was loaded).
    pub fn log(&self) -> &WalkLog {
        &self.log
    }

    /// Record one arrival: append to the log and persist a walk line plus
    /// its checkpoint. Returns the new head.
    ///
    /// If the disk write fails the in-memory chain has already advanced;
    /// callers who care should [`WalkRecorder::save`] to reconcile.
    pub fn record(&mut self, record: WalkRecord) -> io::Result<[u8; 32]> {
        let walk_line = serde_json::to_vec(&record).expect("walk record serializes");
        let head = self.log.append(record);
        let mut file = OpenOptions::new().create(true).append(true).open(&self.path)?;
        file.write_all(&walk_line)?;
        file.write_all(b"\n")?;
        write_checkpoint(&mut file, &head, self.log.records().len())?;
        // The file now pins the new head; keep the in-memory pin in step.
        self.log.expect_chain_head(head);
        Ok(head)
    }

    /// Full flush: rewrite the file from the current log (atomic: temp
    /// file + rename), one walk line + checkpoint per record. For
    /// creation, migration, or repair — normal operation is append-only.
    pub fn save(&mut self) -> io::Result<()> {
        let tmp = self.path.with_extension("walks-tmp");
        {
            let mut file = fs::File::create(&tmp)?;
            let links = self.log.chain_links();
            for (i, record) in self.log.records().iter().enumerate() {
                file.write_all(&serde_json::to_vec(record).expect("walk record serializes"))?;
                file.write_all(b"\n")?;
                write_checkpoint(&mut file, &links[i], i + 1)?;
            }
            file.flush()?;
        }
        fs::rename(&tmp, &self.path)?;
        // The file now pins the current head.
        self.log.expect_chain_head(self.log.head());
        Ok(())
    }

    /// Load a walks file (tolerantly), positioned to continue its chain.
    ///
    /// Missing file → `io::Error` (`NotFound`); every other line-shape
    /// problem is tolerated and reported, never fatal.
    pub fn load(path: impl AsRef<Path>) -> io::Result<LoadOutcome> {
        let reader = BufReader::new(fs::File::open(path.as_ref())?);
        let mut log = WalkLog::new();
        let mut heads: Vec<[u8; 32]> = Vec::new();
        let mut skipped = 0_usize;
        let mut checkpoints: Vec<(usize, [u8; 32])> = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue; // trailing/blank lines are water, not damage
            }
            if let Ok(record) = serde_json::from_str::<WalkRecord>(&line) {
                heads.push(log.append(record));
            } else if let Ok(cp) = serde_json::from_str::<CheckpointLine>(&line) {
                match decode_hex32(&cp.chain_head) {
                    Some(head) => checkpoints.push((usize::try_from(cp.records).unwrap_or(usize::MAX), head)),
                    None => skipped += 1,
                }
            } else {
                skipped += 1;
            }
        }

        // Continuity: every checkpoint must pin the head of exactly its
        // prefix (genesis for zero records), and the last checkpoint must
        // cover the whole file — our writer checkpoints after every walk.
        let mut checkpoints_verified = 0_usize;
        let mut chain_broken = false;
        for (n, expected) in &checkpoints {
            let actual = if *n == 0 {
                Some(crate::residency::GENESIS_CHAIN)
            } else {
                heads.get(n - 1).copied()
            };
            match actual {
                Some(a) if a == *expected => checkpoints_verified += 1,
                _ => chain_broken = true,
            }
        }
        if let Some((n, expected)) = checkpoints.last() {
            if heads.len() != *n || heads.last().copied() != Some(*expected) {
                chain_broken = true; // unpinned tail
            }
            log.expect_chain_head(*expected);
        }

        Ok(LoadOutcome {
            recorder: WalkRecorder {
                path: path.as_ref().to_path_buf(),
                log,
            },
            report: LoadReport {
                records_loaded: heads.len(),
                skipped_malformed: skipped,
                checkpoints_verified,
                chain_broken,
            },
        })
    }
}

/// Write one checkpoint line for the head covering `records` walk lines.
fn write_checkpoint(file: &mut fs::File, head: &[u8; 32], records: usize) -> io::Result<()> {
    let line = CheckpointLine {
        chain_head: encode_hex(head),
        records: u64::try_from(records).expect("record count fits u64"),
    };
    file.write_all(&serde_json::to_vec(&line).expect("checkpoint serializes"))?;
    file.write_all(b"\n")
}

/// Lowercase hex encoding (shared with the growth record).
pub(crate) fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Decode a 64-char hex string into a 32-byte chain head.
pub(crate) fn decode_hex32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0_u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(s.get(2 * i..2 * i + 2)?, 16).ok()?;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn rec(ts: u64, road: &str, lq: f32) -> WalkRecord {
        WalkRecord {
            ts,
            road: road.into(),
            link_quality: lq,
            arrival_meta: None,
        }
    }

    fn tmp_path(tag: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "openshell-walks-{}-{}-{}.jsonl",
            std::process::id(),
            tag,
            n
        ))
    }

    #[test]
    fn save_then_load_verifies() {
        let path = tmp_path("roundtrip");
        let mut recorder = WalkRecorder::new(&path);
        recorder.record(rec(0, "local", 0.5)).unwrap();
        recorder.record(rec(60, "local", 0.5)).unwrap();
        let head = recorder.record(rec(120, "h-road-0", 0.8)).unwrap();
        recorder.save().unwrap();

        let outcome = WalkRecorder::load(&path).unwrap();
        assert_eq!(outcome.report.records_loaded, 3);
        assert_eq!(outcome.report.skipped_malformed, 0);
        assert_eq!(outcome.report.checkpoints_verified, 3);
        assert!(!outcome.report.chain_broken);
        assert_eq!(outcome.recorder.log().records().len(), 3);
        assert_eq!(outcome.recorder.log().head(), head);
        assert!(outcome.recorder.log().verify(), "loaded chain must be intact");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn corrupted_line_is_skipped_and_reported() {
        let path = tmp_path("corrupt");
        let mut recorder = WalkRecorder::new(&path);
        recorder.record(rec(0, "local", 0.5)).unwrap();
        recorder.record(rec(60, "local", 0.5)).unwrap();
        recorder.record(rec(120, "local", 0.5)).unwrap();
        recorder.save().unwrap();

        // Corrupt the first walk line (legacy partial write, editor damage).
        let text = fs::read_to_string(&path).unwrap();
        let mut lines: Vec<String> = text.lines().map(String::from).collect();
        lines[0] = "{\"ts\": \"half a walk".into();
        fs::write(&path, lines.join("\n") + "\n").unwrap();

        let outcome = WalkRecorder::load(&path).unwrap();
        assert_eq!(outcome.report.skipped_malformed, 1, "damage is counted, not fatal");
        assert_eq!(outcome.report.records_loaded, 2);
        assert!(outcome.report.chain_broken, "checkpoints pin 3 walks; only 2 survive");
        assert!(!outcome.recorder.log().verify());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn legacy_file_without_checkpoints_loads() {
        // The v0 / rd shape: plain walk lines, nothing else.
        let path = tmp_path("legacy");
        let mut text = String::new();
        for ts in [0_u64, 60, 120] {
            text.push_str(
                &String::from_utf8(serde_json::to_vec(&rec(ts, "local", 0.5)).unwrap()).unwrap(),
            );
            text.push('\n');
        }
        fs::write(&path, text).unwrap();

        let outcome = WalkRecorder::load(&path).unwrap();
        assert_eq!(outcome.report.records_loaded, 3);
        assert_eq!(outcome.report.skipped_malformed, 0);
        assert_eq!(outcome.report.checkpoints_verified, 0);
        assert!(!outcome.report.chain_broken, "nothing persisted, nothing to check");
        assert!(outcome.recorder.log().verify());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn chain_continues_across_restart() {
        let path = tmp_path("restart");
        let mut first = WalkRecorder::new(&path);
        first.record(rec(0, "local", 0.5)).unwrap();
        let head2 = first.record(rec(60, "local", 0.5)).unwrap();
        first.save().unwrap();

        // Restart: load, keep walking.
        let outcome = WalkRecorder::load(&path).unwrap();
        let mut resumed = outcome.recorder;
        assert_eq!(resumed.log().head(), head2);
        let head3 = resumed.record(rec(120, "h-road-3", 0.9)).unwrap();
        assert_ne!(head3, head2);
        assert!(resumed.log().verify(), "append continues the loaded chain");

        // Second restart: the continued chain is intact on disk.
        let outcome = WalkRecorder::load(&path).unwrap();
        assert_eq!(outcome.report.records_loaded, 3);
        assert!(!outcome.report.chain_broken);
        assert_eq!(outcome.recorder.log().head(), head3);
        assert!(outcome.recorder.log().verify());

        // And it is the same chain a fresh log over the same walks grows.
        let mut fresh = WalkLog::new();
        fresh.append(rec(0, "local", 0.5));
        fresh.append(rec(60, "local", 0.5));
        fresh.append(rec(120, "h-road-3", 0.9));
        assert_eq!(fresh.head(), head3);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn unpinned_tail_reads_as_chain_break() {
        let path = tmp_path("unpinned");
        let mut recorder = WalkRecorder::new(&path);
        recorder.record(rec(0, "local", 0.5)).unwrap();
        recorder.record(rec(60, "local", 0.5)).unwrap();
        recorder.save().unwrap();

        // Drop only the final checkpoint: the last walk is now unpinned.
        let text = fs::read_to_string(&path).unwrap();
        let mut lines: Vec<&str> = text.lines().collect();
        assert!(lines.pop().is_some_and(|l| l.contains("chain_head")));
        fs::write(&path, lines.join("\n") + "\n").unwrap();

        let outcome = WalkRecorder::load(&path).unwrap();
        assert_eq!(outcome.report.records_loaded, 2);
        assert!(outcome.report.chain_broken, "a walk without its checkpoint is a broken tail");
        assert!(!outcome.recorder.log().verify());

        let _ = fs::remove_file(&path);
    }
}

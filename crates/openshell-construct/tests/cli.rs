// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Integration test for the CLI verbs (`openshell-construct::cli`) — the
//! seam the `openconstruct room` subcommands call through.

use openshell_construct::cli::{grow_room, show_room, walk_room};
use openshell_construct::RoomError;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

fn tmp_room(tag: &str) -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("openshell-cli-room-{}-{}-{}.json", std::process::id(), tag, n))
}

#[test]
fn grow_walk_show_the_full_keeper_flow() {
    let path = tmp_room("flow");

    // grow: creates the file, prints the onboarding document.
    let doc = grow_room("demo", "hold the commission; read the water", 1_000, &path).unwrap();
    assert!(doc.contains("This room was grown, not configured."), "doc was:\n{doc}");
    assert!(path.exists(), "the room file must exist after grow");

    // walk: cold flatline arrivals, one per verb call.
    let mut ts = 0_u64;
    for i in 0..5_u64 {
        ts = i * 60;
        let out = walk_room(&path, "local", 0.5, ts).unwrap();
        assert!(out.contains("heat: cold"), "walk {i} output was:\n{out}");
        assert!(out.contains("prior: none"), "a quiet room reads no prior:\n{out}");
    }

    // walk: the 06:11 write — a novel road against flat sediment.
    let out = walk_room(&path, "h-road-9", 0.95, ts + 60).unwrap();
    assert!(out.contains("novel road"), "output was:\n{out}");
    assert!(out.contains("heat: cold"), "held without verdict:\n{out}");
    assert!(out.contains("prior: novel_road (0.90)"), "output was:\n{out}");

    // show: the growth record, told as the document — heat timeline now
    // carries the recorded temperature(s).
    let shown = show_room(&path).unwrap();
    assert!(shown.contains("This room was grown, not configured."));
    assert!(shown.contains("Heat timeline"), "shown was:\n{shown}");
    assert!(shown.contains("tick 0 — cold"), "the first tick is a temperature:\n{shown}");

    // Determinism of the seam: showing twice reads the same document.
    assert_eq!(shown, show_room(&path).unwrap());

    let _ = fs::remove_file(&path);
}

#[test]
fn walk_on_a_missing_or_tampered_room_refuses() {
    // Missing file: io error, not a panic.
    let missing = tmp_room("missing");
    assert!(matches!(walk_room(&missing, "local", 0.5, 0), Err(RoomError::Io(_))));

    // Tampered file: grow, damage the walks, walk refuses.
    let path = tmp_room("tampered");
    grow_room("demo", "test charter", 42, &path).unwrap();
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    value["walks"] = serde_json::json!([
        {"ts": 0, "road": "forged", "link_quality": 0.1},
    ]);
    fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    match walk_room(&path, "local", 0.5, 60) {
        Err(RoomError::Tamper(detail)) => {
            assert!(detail.contains("chain") || detail.contains("head"), "detail: {detail}");
        }
        other => panic!("expected Tamper, got {other:?}"),
    }

    let _ = fs::remove_file(&path);
}

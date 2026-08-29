// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Residency runtime (RFC 0004) — the registry that makes rooms live.
//!
//! [`Room`] and its pieces are pure; something has to hold them across
//! ticks and hand their answers to the runtime that calls the models.
//! [`RoomResidency`] is that holder, shaped by the two insertion points
//! documented in [`crate::hooks`]:
//!
//! - **grow-or-load** ([`RoomResidency::attach`]) — the supervisor calls
//!   this once per sandbox at spawn. The seed is derived from the sandbox
//!   id and its charter (`SandboxSpec` carries no charter field yet —
//!   v1 passes `None` and the derivation documents the default). If a
//!   room file already exists for the sandbox, it is **loaded**, so the
//!   walk chain and heat history continue across restarts; otherwise the
//!   room is grown fresh and saved best-effort.
//! - **tick** ([`RoomResidency::tick`]) — the supervision loop feeds one
//!   walk per tick; the resulting Ensign prior is fired through the
//!   [`HooksRegistry`] (the RFC dispatch mechanism — any registered
//!   [`RoomHooks`] hears it), and the room file is re-saved best-effort.
//! - **model params** ([`RoomResidency::model_params`],
//!   [`RoomResidency::apply_overrides`]) — the model-call path consults
//!   the registry; `Some(params)` means the room's residency (gravity
//!   modulated by heat) shapes the call. [`apply_overrides`] lands the
//!   temperature and token budget in a chat-completions-style JSON body.
//!
//! Honest limits, stated once:
//!
//! - **Charter gap.** The sandbox runtime has no charter field today;
//!   v1 wiring grows rooms under [`DEFAULT_SANDBOX_CHARTER`]. When a
//!   charter lands on the spec, pass it through and the seed (and thus
//!   the mask) changes — a re-seed, never an un-mask.
//! - **Prompt style is not a sampling knob.** `ModelParams::prompt_style`
//!   names how prompts should read; chat-completions bodies have no
//!   honest field for it, so [`apply_overrides`] sets only `temperature`
//!   and the token budget.
//! - **One resident room per process.** A sandbox process is one room in
//!   v1; `resident_room_id` is how the model-call path (which has no
//!   per-connection room identity) finds it.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::ensign::AttentionPrior;
use crate::gravity::{modulate, ModelParams};
use crate::hooks::{HooksRegistry, RoomHooks};
use crate::residency::{heat, HeatReading, WalkRecord};
use crate::room::{Room, RoomError, ROOM_WINDOW};

/// Domain-separation prefix for the sandbox seed derivation.
const ROOM_SEED_DOMAIN: &[u8] = b"openshell-construct/room-seed/v1";

/// The charter a sandbox-grown room holds until the spec carries a real
/// one (the charter gap, stated in the module docs).
pub const DEFAULT_SANDBOX_CHARTER: &str = "supervised sandbox — commission unstated";

/// What to do with an Ensign prior, as a function (so the host runtime
/// owns the policy — logging, events, surfacing — without the construct
/// crate depending on a logging framework).
pub type PriorSink = Arc<dyn Fn(&str, &AttentionPrior) + Send + Sync>;

/// One room held by the registry: the room itself plus where it persists.
struct RoomSlot {
    room: Arc<Mutex<Room>>,
    /// Where [`RoomResidency::tick`] re-saves the room, if anywhere.
    persist_path: Option<PathBuf>,
}

/// The shared registry of live rooms (RFC 0004 runtime).
///
/// Thread-safe by construction: rooms under `Arc<Mutex<…>>`, dispatch
/// through the [`HooksRegistry`]. The tick path locks one room at a time
/// and does no awaiting while holding a lock.
pub struct RoomResidency {
    rooms: Mutex<BTreeMap<String, RoomSlot>>,
    hooks: Mutex<HooksRegistry>,
    /// The room attached by *this* process (v1: one sandbox, one room).
    resident: Mutex<Option<String>>,
    gravity: f64,
    persist_dir: Option<PathBuf>,
    sink: Mutex<PriorSink>,
}

/// Derive a room seed from the sandbox identity.
///
/// The seed is `SHA-256("openshell-construct/room-seed/v1" ‖ 0x00 ‖
/// sandbox_id ‖ 0x00 ‖ charter)` with `charter` defaulting to
/// [`DEFAULT_SANDBOX_CHARTER`]. Deterministic: the same sandbox id and
/// charter always grow the same room (same id, same mask) — a lost room
/// file re-grows identically, its walks starting over.
pub fn seed_for_sandbox(sandbox_id: &str, charter: Option<&str>) -> [u8; 32] {
    let charter = charter.unwrap_or(DEFAULT_SANDBOX_CHARTER);
    let mut hasher = Sha256::new();
    hasher.update(ROOM_SEED_DOMAIN);
    hasher.update([0x00]);
    hasher.update(sandbox_id.as_bytes());
    hasher.update([0x00]);
    hasher.update(charter.as_bytes());
    hasher.finalize().into()
}

/// The room file name for a sandbox id (sandbox ids are server-generated
/// UUIDs; anything exotic is flattened to stay path-safe).
fn room_file_name(sandbox_id: &str) -> String {
    let safe: String = sandbox_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{safe}.room.json")
}

impl RoomResidency {
    /// An empty registry at gravity `gravity` (see [`modulate`]), not
    /// persisting anywhere.
    pub fn new(gravity: f64) -> Self {
        Self::with_persist_dir(gravity, None)
    }

    /// An empty registry that saves and loads rooms under `persist_dir`
    /// (created on first grow, best-effort).
    pub fn with_persist_dir(gravity: f64, persist_dir: Option<PathBuf>) -> Self {
        Self {
            rooms: Mutex::new(BTreeMap::new()),
            hooks: Mutex::new(HooksRegistry::new()),
            resident: Mutex::new(None),
            gravity,
            persist_dir,
            sink: Mutex::new(Arc::new(|_, _| {})),
        }
    }

    /// The gravity every attached room bends through (see [`modulate`]).
    pub fn gravity(&self) -> f64 {
        self.gravity
    }

    /// Install the sink every prior is handed to (default: drop quietly).
    /// The host runtime owns what a prior *does* — the hook only carries it.
    pub fn set_prior_sink(&self, sink: PriorSink) {
        *self.sink.lock().expect("residency sink poisoned") = sink;
    }

    /// Grow-or-load this sandbox's room and register it (the spawn-time
    /// insertion point).
    ///
    /// - If a room file for `sandbox_id` exists under the persist dir, it
    ///   is **loaded** (walks and heat continue; a tampered file refuses
    ///   with [`RoomError::Tamper`] rather than re-growing over evidence).
    /// - Otherwise the room is grown from [`seed_for_sandbox`] at
    ///   `now_tick` and saved best-effort (an unwritable persist dir
    ///   degrades to an in-memory room — the room lives, its file does
    ///   not; each tick retries the save).
    ///
    /// Returns the room id (also remembered as this process's resident
    /// room).
    pub fn attach(
        &self,
        sandbox_id: &str,
        charter: Option<&str>,
        now_tick: u64,
    ) -> Result<String, RoomError> {
        let charter = charter.unwrap_or(DEFAULT_SANDBOX_CHARTER);
        let persist_path = self
            .persist_dir
            .as_ref()
            .map(|dir| dir.join(room_file_name(sandbox_id)));

        let room = match &persist_path {
            Some(path) if path.exists() => Room::load(path)?,
            _ => {
                let (room, _onboarding_doc) =
                    Room::grow(&seed_for_sandbox(sandbox_id, Some(charter)), charter, now_tick);
                if let Some(path) = &persist_path {
                    // Best-effort first save: failure degrades to an
                    // in-memory room whose ticks keep retrying the save.
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = room.save(path);
                }
                room
            }
        };

        let room_id = room.id.clone();
        let slot = RoomSlot {
            room: Arc::new(Mutex::new(room)),
            persist_path,
        };
        let hook = ResidencyHook {
            room: Arc::clone(&slot.room),
            gravity: self.gravity,
            sink: self.sink.lock().expect("residency sink poisoned").clone(),
        };
        self.rooms
            .lock()
            .expect("residency rooms poisoned")
            .insert(room_id.clone(), slot);
        *self.resident.lock().expect("residency resident poisoned") = Some(room_id.clone());
        self.hooks
            .lock()
            .expect("residency hooks poisoned")
            .register(room_id.clone(), Box::new(hook));
        Ok(room_id)
    }

    /// The room this process attached, if any (v1: one sandbox, one room).
    pub fn resident_room_id(&self) -> Option<String> {
        self.resident
            .lock()
            .expect("residency resident poisoned")
            .clone()
    }

    /// One supervision tick: append the walk, read the heat, fire the
    /// Ensign prior through the [`HooksRegistry`], and re-save best-effort.
    /// Returns `None` if no room is registered under `room_id`.
    pub fn tick(&self, room_id: &str, walk: WalkRecord) -> Option<(HeatReading, AttentionPrior)> {
        let (room, persist_path) = {
            let rooms = self.rooms.lock().expect("residency rooms poisoned");
            let slot = rooms.get(room_id)?;
            (Arc::clone(&slot.room), slot.persist_path.clone())
        };
        let (reading, prior) = {
            let mut room = room.lock().expect("residency room poisoned");
            let ticked = room.tick(walk);
            // Best-effort: a failed save costs persistence, not the tick.
            if let Some(path) = &persist_path {
                let _ = room.save(path);
            }
            ticked
        };
        self.hooks
            .lock()
            .expect("residency hooks poisoned")
            .fire_prior(room_id, &prior);
        Some((reading, prior))
    }

    /// The room's residency-shaped model parameters, asked through the
    /// [`HooksRegistry`] (the model-call insertion point; `None` when the
    /// room has no hook or the hook declines).
    pub fn model_params(&self, room_id: &str) -> Option<ModelParams> {
        self.hooks
            .lock()
            .expect("residency hooks poisoned")
            .model_params(room_id)
    }

    /// [`Self::model_params`] for this process's resident room.
    pub fn resident_model_params(&self) -> Option<ModelParams> {
        self.model_params(&self.resident_room_id()?)
    }

    /// Land the room's model parameters in a chat-completions-style JSON
    /// request body: `temperature` is overridden, and the token budget
    /// lands in `max_tokens` — or `max_completion_tokens` when the caller
    /// already speaks the newer spelling. Returns `true` iff the body was
    /// shaped (a registered room with a hook); otherwise the body is left
    /// byte-identical.
    ///
    /// `prompt_style` deliberately does not land here: it is a prompt
    /// shape, not a sampling knob, and the wire body has no honest field
    /// for it (see the module's honest limits).
    pub fn apply_overrides(&self, room_id: &str, body: &mut Value) -> bool {
        let Some(params) = self.model_params(room_id) else {
            return false;
        };
        let Some(object) = body.as_object_mut() else {
            return false;
        };
        object.insert("temperature".into(), Value::from(params.temperature));
        let token_key = if object.contains_key("max_completion_tokens") {
            "max_completion_tokens"
        } else {
            "max_tokens"
        };
        object.insert(token_key.into(), Value::from(params.max_tokens));
        true
    }
}

/// The hook registered per room: answers model params from the room's
/// current heat, and hands priors to the sink.
struct ResidencyHook {
    room: Arc<Mutex<Room>>,
    gravity: f64,
    sink: PriorSink,
}

impl RoomHooks for ResidencyHook {
    fn on_prior(&self, room_id: &str, prior: &AttentionPrior) {
        (self.sink)(room_id, prior);
    }

    fn model_params(&self, _room_id: &str) -> Option<ModelParams> {
        let room = self.room.lock().expect("residency room poisoned");
        let state = heat(room.walklog.records(), ROOM_WINDOW).state;
        Some(modulate(self.gravity, state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ensign::AttentionReason;
    use crate::residency::HeatState;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "openshell-residency-{}-{}-{tag}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn rec(ts: u64, road: &str, lq: f32) -> WalkRecord {
        WalkRecord {
            ts,
            road: road.into(),
            link_quality: lq,
            arrival_meta: None,
        }
    }

    #[test]
    fn sandbox_seed_is_deterministic_and_charter_salted() {
        let a = seed_for_sandbox("sbx-1", None);
        assert_eq!(a, seed_for_sandbox("sbx-1", None), "same id, same seed");
        assert_ne!(a, seed_for_sandbox("sbx-2", None), "id is in the seed");
        assert_ne!(
            a,
            seed_for_sandbox("sbx-1", Some("a real charter")),
            "the charter is in the seed"
        );
        // The documented default charter is what `None` means.
        assert_eq!(a, seed_for_sandbox("sbx-1", Some(DEFAULT_SANDBOX_CHARTER)));
    }

    #[test]
    fn attach_grows_the_resident_room_and_answers_cold_lane() {
        // A hook that answers with a fixed value, proving dispatch is by
        // hook (the RFC mechanism), not by cached params.
        struct FixedHook;
        impl RoomHooks for FixedHook {
            fn on_prior(&self, _: &str, _: &AttentionPrior) {}
            fn model_params(&self, _: &str) -> Option<ModelParams> {
                Some(ModelParams { temperature: 9.9, max_tokens: 7, prompt_style: "test" })
            }
        }

        let residency = RoomResidency::new(0.5);
        let room_id = residency.attach("sbx-grow", None, 1_000).unwrap();

        assert_eq!(residency.resident_room_id().as_deref(), Some(room_id.as_str()));

        // An unread room is cold: the model-call path gets the cold lane.
        let params = residency.model_params(&room_id).unwrap();
        assert_eq!(params, modulate(0.5, HeatState::Cold));

        // And it is the registry's hook that answered: replacing the hook
        // changes the answer without touching the room.
        residency
            .hooks
            .lock()
            .unwrap()
            .register(room_id.clone(), Box::new(FixedHook));
        let answered = residency.model_params(&room_id).unwrap().temperature;
        assert!(
            (answered - 9.9).abs() < 1e-9,
            "dispatch is by hook, not by cached params (answered {answered})"
        );
    }

    #[test]
    fn attach_loads_an_existing_room_and_continues_its_chain() {
        let dir = temp_dir("reload");
        let residency = RoomResidency::with_persist_dir(0.5, Some(dir.clone()));
        let room_id = residency.attach("sbx-reload", None, 1_000).unwrap();
        residency.tick(&room_id, rec(60, "local", 0.5));
        residency.tick(&room_id, rec(120, "local", 0.5));

        // A new process (fresh registry, same persist dir) continues the room.
        let second = RoomResidency::with_persist_dir(0.5, Some(dir.clone()));
        let same_id = second.attach("sbx-reload", None, 999_999).unwrap();
        assert_eq!(same_id, room_id, "the loaded room keeps its identity");
        let (reading, prior) = second.tick(&room_id, rec(180, "h-road-0", 0.8)).expect("room registered");
        assert_ne!(prior.reason, AttentionReason::ChainBreak);
        // Two local flat walks then a varied road with a lift: cooling.
        assert_eq!(reading.state, HeatState::Cooling);
        // The chain carries all three walks, not a fresh genesis.
        let rooms = second.rooms.lock().unwrap();
        let slot = rooms.get(&room_id).unwrap();
        let room = slot.room.lock().unwrap();
        assert_eq!(room.walklog.records().len(), 3);
        assert!(room.walklog.verify(), "the chain continues across restarts");

        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn tampered_room_file_refuses_to_attach() {
        let dir = temp_dir("tamper");
        let residency = RoomResidency::with_persist_dir(0.5, Some(dir.clone()));
        let room_id = residency.attach("sbx-tamper", None, 1_000).unwrap();
        residency.tick(&room_id, rec(60, "local", 0.5));

        // The tamperer's pen: rewrite a walk in the saved file.
        let path = dir.join(room_file_name("sbx-tamper"));
        let mut value: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        value["walks"][0]["road"] = Value::from("h-road-9");
        std::fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

        let second = RoomResidency::with_persist_dir(0.5, Some(dir.clone()));
        match second.attach("sbx-tamper", None, 2_000) {
            Err(RoomError::Tamper(_)) => {}
            other => panic!("tampered room must refuse, got {other:?}"),
        }
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn tick_fires_priors_through_the_hooks_registry() {
        // Records every prior it hears, so the test can prove the registry
        // (not the tick) carries priors to the listener.
        struct Recorder(Arc<Mutex<Vec<(String, AttentionReason, f32)>>>);
        impl RoomHooks for Recorder {
            fn on_prior(&self, room_id: &str, prior: &AttentionPrior) {
                self.0
                    .lock()
                    .unwrap()
                    .push((room_id.into(), prior.reason, prior.urgency));
            }
            fn model_params(&self, _: &str) -> Option<ModelParams> {
                None
            }
        }

        let residency = RoomResidency::new(0.5);
        let room_id = residency.attach("sbx-prior", None, 0).unwrap();

        // Sustained cold, then the novel road — the Ensign must fire.
        let heard: Arc<Mutex<Vec<(String, AttentionReason, f32)>>> =
            Arc::new(Mutex::new(Vec::new()));
        residency
            .hooks
            .lock()
            .unwrap()
            .register(room_id.clone(), Box::new(Recorder(heard.clone())));

        for i in 0..8_u64 {
            residency.tick(&room_id, rec(i * 60, "local", 0.5));
        }
        let (_, prior) = residency.tick(&room_id, rec(8 * 60, "h-road-9", 0.95)).expect("room registered");

        assert_eq!(prior.reason, AttentionReason::NovelRoad);
        let heard = heard.lock().unwrap();
        assert_eq!(heard.len(), 9, "every tick fires its prior, quiet ones included");
        assert_eq!(heard.last().unwrap().1, AttentionReason::NovelRoad);
        assert_eq!(heard.last().unwrap().0, room_id);
    }

    #[test]
    fn model_params_track_the_room_heat() {
        let residency = RoomResidency::new(0.0); // gravity 0: lane endpoints
        let room_id = residency.attach("sbx-heat", None, 0).unwrap();

        // Unread room: cold lane, tight end of the budget.
        assert_eq!(residency.model_params(&room_id).unwrap().max_tokens, 2048);

        // Warm life; once the window warms, the lane widens.
        let roads = ["h-road-0", "north", "h-road-2", "rim"];
        let lqs = [0.20, 0.55, 0.90, 0.35, 0.75, 0.45, 0.95, 0.60];
        let gaps = [30_u64, 90, 15, 120, 45, 75, 10];
        let mut ts = 1_000_u64;
        for i in 0..8 {
            residency.tick(&room_id, rec(ts, roads[i % roads.len()], lqs[i]));
            ts += gaps[i % gaps.len()];
        }
        let params = residency.model_params(&room_id).unwrap();
        assert_eq!(params, modulate(0.0, HeatState::Warm));
        assert_eq!(params.max_tokens, 4096);
        assert_eq!(params.prompt_style, "expansive");
    }

    #[test]
    fn apply_overrides_shapes_chat_bodies_and_respects_newer_spelling() {
        let residency = RoomResidency::new(0.5);
        let room_id = residency.attach("sbx-override", None, 0).unwrap();

        // Cold lane at gravity 0.5: temperature 0.30, max_tokens 1280.
        let mut body: Value = serde_json::json!({
            "model": "ignored",
            "messages": [{"role": "user", "content": "hi"}],
            "temperature": 2.0,
            "max_tokens": 999_999
        });
        assert!(residency.apply_overrides(&room_id, &mut body));
        let cold = modulate(0.5, HeatState::Cold);
        assert!((body["temperature"].as_f64().unwrap() - cold.temperature).abs() < 1e-9);
        assert_eq!(body["max_tokens"], serde_json::json!(cold.max_tokens));
        // The caller's model and messages are untouched.
        assert_eq!(body["model"], serde_json::json!("ignored"));
        assert!(body["messages"].is_array());

        // Newer spelling wins when the caller already speaks it.
        let mut body: Value =
            serde_json::json!({"model": "m", "max_completion_tokens": 999_999});
        assert!(residency.apply_overrides(&room_id, &mut body));
        assert_eq!(
            body["max_completion_tokens"],
            serde_json::json!(cold.max_tokens)
        );
        assert!(body.get("max_tokens").is_none());

        // No room, no override — the body is left alone.
        let mut body: Value = serde_json::json!({"temperature": 1.5});
        assert!(!residency.apply_overrides("room-absent", &mut body));
        assert_eq!(body["temperature"], serde_json::json!(1.5));

        // Non-object bodies (bare prompts) pass through untouched.
        let mut body: Value = serde_json::json!("just a string");
        assert!(!residency.apply_overrides(&room_id, &mut body));
        assert_eq!(body, serde_json::json!("just a string"));
    }

    #[test]
    fn unwritable_persist_dir_degrades_to_in_memory_room() {
        // A file where the persist dir should be: create_dir_all fails,
        // the room still grows and lives.
        let blocker = temp_dir("blocker");
        std::fs::write(&blocker, b"not a dir").unwrap();
        let residency = RoomResidency::with_persist_dir(0.5, Some(blocker.join("rooms")));
        let room_id = residency.attach("sbx-degraded", None, 1_000).unwrap();
        let (_, prior) = residency.tick(&room_id, rec(60, "local", 0.5)).expect("room registered");
        assert_ne!(prior.reason, AttentionReason::ChainBreak);
        assert!(residency.model_params(&room_id).is_some());
        std::fs::remove_file(blocker).ok();
    }
}

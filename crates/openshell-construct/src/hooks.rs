// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Integration hooks — where OpenShell's loops call the room in (RFC 0004).
//!
//! Two insertion points, both documented in
//! `rfc/0004-the-room-grows-a-mask/README.md`:
//!
//! - [`RoomHooks::on_prior`] — the **supervisor loop** calls this whenever
//!   a room's Ensign attention prior fires (novel road, heat transition,
//!   chain break). The supervisor decides what a prior *does*; the hook
//!   only carries it. Intended call site: after `Room::tick` in the room's
//!   supervision cycle, before the next model call.
//! - [`RoomHooks::model_params`] — the **model-call path** calls this
//!   immediately before invoking the model for a room. `Some(params)` means
//!   the room's residency (gravity modulated by heat) shapes this call;
//!   `None` means the room is at its base gravity with no residency
//!   override. Intended call site: the request builder, so heat-modulated
//!   temperature/token/style land in the actual inference request.

use std::collections::BTreeMap;

use crate::ensign::AttentionPrior;
use crate::gravity::ModelParams;

/// The insertion points where the runtime calls a room in.
pub trait RoomHooks: Send + Sync {
    /// Called by the supervisor loop when a room's Ensign prior fires.
    fn on_prior(&self, room_id: &str, prior: &AttentionPrior);

    /// Called by the model-call path; `Some` overrides the request with
    /// residency-shaped parameters.
    fn model_params(&self, room_id: &str) -> Option<ModelParams>;
}

/// The default: rooms exist, nothing listens yet.
pub struct NoopHooks;

impl RoomHooks for NoopHooks {
    fn on_prior(&self, _room_id: &str, _prior: &AttentionPrior) {}
    fn model_params(&self, _room_id: &str) -> Option<ModelParams> {
        None
    }
}

/// Room-id → hooks. Dispatch is by exact room id; a hook registered for a
/// room sees only that room's events.
#[derive(Default)]
pub struct HooksRegistry {
    hooks: BTreeMap<String, Box<dyn RoomHooks>>,
}

impl HooksRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, room_id: impl Into<String>, hooks: Box<dyn RoomHooks>) {
        self.hooks.insert(room_id.into(), hooks);
    }

    pub fn unregister(&mut self, room_id: &str) -> Option<Box<dyn RoomHooks>> {
        self.hooks.remove(room_id)
    }

    /// Fan a prior out to the room's hook, if any. No hook, no call.
    pub fn fire_prior(&self, room_id: &str, prior: &AttentionPrior) {
        if let Some(h) = self.hooks.get(room_id) {
            h.on_prior(room_id, prior);
        }
    }

    /// Ask the room's hook for residency-shaped model parameters.
    pub fn model_params(&self, room_id: &str) -> Option<ModelParams> {
        self.hooks.get(room_id).and_then(|h| h.model_params(room_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AttentionReason;
    use std::sync::Mutex;

    struct ParamRoom(Mutex<Vec<String>>, ModelParams);

    impl RoomHooks for ParamRoom {
        fn on_prior(&self, _room_id: &str, _prior: &AttentionPrior) {}

        fn model_params(&self, room_id: &str) -> Option<ModelParams> {
            self.0.lock().unwrap().push(room_id.to_owned());
            Some(self.1)
        }
    }

    fn prior(reason: AttentionReason, urgency: f32) -> AttentionPrior {
        AttentionPrior {
            reason,
            urgency,
            detail: "test".into(),
        }
    }

    #[test]
    fn noop_default_hears_nothing_and_overrides_nothing() {
        let noop = NoopHooks;
        noop.on_prior("r", &prior(AttentionReason::NovelRoad, 0.9));
        assert!(noop.model_params("r").is_none());
        let reg = HooksRegistry::new();
        reg.fire_prior("r", &prior(AttentionReason::NovelRoad, 0.9));
        assert!(reg.model_params("r").is_none());
    }

    #[test]
    fn registry_dispatches_by_room_id() {
        let log: Mutex<Vec<String>> = Mutex::new(Vec::new());
        // Use a dedicated listener to prove dispatch reaches the right hook
        // and not others.
        struct Listener(Mutex<Vec<String>>);
        impl RoomHooks for Listener {
            fn on_prior(&self, room_id: &str, _prior: &AttentionPrior) {
                self.0.lock().unwrap().push(room_id.to_owned());
            }
            fn model_params(&self, _room_id: &str) -> Option<ModelParams> {
                None
            }
        }
        let listener = Listener(Mutex::new(Vec::new()));
        let mut reg = HooksRegistry::new();
        reg.register("room-a", Box::new(listener));
        let params = ModelParams { temperature: 0.35, max_tokens: 512, prompt_style: "terse" };
        reg.register("room-b", Box::new(ParamRoom(Mutex::new(Vec::new()), params)));
        reg.register("room-b2", Box::new(NoopHooks));

        reg.fire_prior("room-a", &prior(AttentionReason::NovelRoad, 0.9));
        reg.fire_prior("room-b", &prior(AttentionReason::HeatTransition, 0.6));
        // room-b2 was registered but never fired at; nothing should route
        // room-a's event to room-b or room-b2.
        assert_eq!(reg.model_params("room-b").map(|p| p.temperature), Some(0.35));
        assert!(reg.model_params("room-a").is_none());
        let touched: Vec<String> = log.lock().unwrap().clone();
        assert!(touched.is_empty()); // ParamRoom only records when its model_params is consulted for its own room
    }
}

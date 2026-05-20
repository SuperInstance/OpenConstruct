/// Core constraint check logic (hot path)
use crate::effects::{EffectHandler, Severity};
use crate::proof::ProofContext;
use crate::provenance::ProvenanceLog;
use crate::vector::VectorUnit;

/// A single constraint definition
#[derive(Debug, Clone)]
pub struct Constraint {
    pub lo: i32,
    pub hi: i32,
    pub tag: String,
}

impl Constraint {
    pub fn new(lo: i32, hi: i32, tag: &str) -> Self {
        Self {
            lo,
            hi,
            tag: tag.to_string(),
        }
    }

    /// Hot path: single value range check
    #[inline(always)]
    pub fn check(&self, value: i32) -> bool {
        value >= self.lo && value <= self.hi
    }
}

/// Check a single value against a constraint, with full instrumentation
pub fn check_value(
    value: i32,
    constraint: &Constraint,
    proof_ctx: &mut ProofContext,
    prov_log: &mut ProvenanceLog,
    handler: &mut EffectHandler,
    cycle: u64,
) -> bool {
    let pass = constraint.check(value);
    proof_ctx.prove_range(value, constraint.lo, constraint.hi, pass);
    prov_log.record(
        cycle,
        &format!("check:{}", constraint.tag),
        &value.to_le_bytes(),
    );
    if !pass {
        handler.emit(
            Severity::Fail,
            &format!(
                "{}: {} not in [{}, {}]",
                constraint.tag, value, constraint.lo, constraint.hi
            ),
            cycle,
        );
    }
    pass
}

/// Batch check: check multiple values against a single constraint
pub fn batch_check(
    values: &[i32],
    constraint: &Constraint,
    proof_ctx: &mut ProofContext,
    prov_log: &mut ProvenanceLog,
    handler: &mut EffectHandler,
    start_cycle: u64,
) -> (usize, usize) {
    let mut passed = 0;
    let mut failed = 0;
    for (i, &v) in values.iter().enumerate() {
        if check_value(v, constraint, proof_ctx, prov_log, handler, start_cycle + i as u64) {
            passed += 1;
        } else {
            failed += 1;
        }
    }
    (passed, failed)
}

/// Vector range check using SIMD unit
pub fn vector_check(
    vec_unit: &mut VectorUnit,
    reg: u8,
    lo: i8,
    hi: i8,
    proof_ctx: &mut ProofContext,
    handler: &mut EffectHandler,
    cycle: u64,
) -> u8 {
    let mask = vec_unit.range_check(reg, lo, hi).unwrap_or(0);
    proof_ctx.prove_vector(mask, lo, hi);
    handler.accumulate_mask(mask);
    if mask != 0xff {
        handler.emit(
            Severity::Warn,
            &format!("vector check: mask 0x{:02x} (expected 0xff)", mask),
            cycle,
        );
    }
    mask
}

/// Classify severity from a mask result
pub fn classify_mask(mask: u8) -> Severity {
    let bits = mask.count_ones() as usize;
    let total = 8;
    if bits == total {
        Severity::Ok
    } else if bits >= total * 3 / 4 {
        Severity::Warn
    } else if bits >= total / 2 {
        Severity::Fail
    } else {
        Severity::Critical
    }
}

/// Aviation preset constraints (example constraint set)
pub fn aviation_preset() -> Vec<Constraint> {
    vec![
        Constraint::new(0, 45000, "altitude"),      // feet
        Constraint::new(0, 600, "airspeed"),          // knots
        Constraint::new(-60, 60, "pitch"),            // degrees
        Constraint::new(-180, 180, "roll"),           // degrees
        Constraint::new(0, 5000, "vertical_speed"),   // ft/min
        Constraint::new(0, 100, "throttle"),           // percent
    ]
}

/// Temperature monitoring preset
pub fn temperature_preset() -> Vec<Constraint> {
    vec![
        Constraint::new(-40, 85, "sensor_temp"),    // °C
        Constraint::new(0, 100, "humidity"),         // percent
        Constraint::new(0, 500, "pressure"),         // hPa
    ]
}

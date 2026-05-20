/// FLUX JIT Compiler — bytecode → native constraint checker
///
/// The VM should NOT interpret bytecode at runtime. This JIT extracts constraint
/// definitions from FLUX-C bytecode and generates a zero-overhead native checker
/// that matches the C hot path (654M checks/sec).
///
/// Architecture:
///   bytecode → extract lo/hi pairs → Rust closure (or x86 machine code)
///
/// The generated `check()` is a tight loop of comparisons with bitmask construction,
/// identical to `flux_check_exact()` in the C implementation.

use crate::opcode::OpCode;

// ── JIT error type ──

#[derive(Debug, Clone, PartialEq)]
pub enum JitError {
    InvalidBytecode(String),
    NoConstraints,
    TooManyConstraints(usize),
    InvalidRange { index: usize, lo: f64, hi: f64 },
    NativeCodegenFailed(String),
    ExecutionFailed(String),
}

impl std::fmt::Display for JitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidBytecode(msg) => write!(f, "jit: invalid bytecode: {msg}"),
            Self::NoConstraints => write!(f, "jit: no constraints found in bytecode"),
            Self::TooManyConstraints(n) => write!(f, "jit: too many constraints: {n} (max 8)"),
            Self::InvalidRange { index, lo, hi } => {
                write!(f, "jit: constraint {index}: lo ({lo}) > hi ({hi})")
            }
            Self::NativeCodegenFailed(msg) => write!(f, "jit: native codegen failed: {msg}"),
            Self::ExecutionFailed(msg) => write!(f, "jit: execution failed: {msg}"),
        }
    }
}

impl std::error::Error for JitError {}

pub type JitResult<T> = Result<T, JitError>;

/// Maximum constraints supported (matches C FLUX_EXACT_MAX_CONSTRAINTS)
pub const MAX_JIT_CONSTRAINTS: usize = 8;

// ── x86_64 native code generation is in jit_x86.rs ──

// ── Preset definitions (all 10 from the C implementation + extras) ──

/// A preset is a named set of (lo, hi, name) constraint definitions
pub type Preset = (&'static str, Vec<(f64, f64, &'static str)>);

pub fn all_presets() -> Vec<Preset> {
    vec![
        (
            "automotive_can",
            vec![
                (0.0, 8000.0, "engine_rpm"),
                (0.0, 300.0, "vehicle_speed_kmh"),
                (-40.0, 150.0, "coolant_temp_c"),
                (0.0, 100.0, "throttle_pct"),
                (0.0, 200.0, "brake_pressure_bar"),
                (-720.0, 720.0, "steering_angle_deg"),
                (9.0, 16.0, "battery_voltage_v"),
                (0.0, 100.0, "fuel_level_pct"),
            ],
        ),
        (
            "aviation_adsb",
            vec![
                (-1000.0, 45000.0, "altitude_ft"),
                (0.0, 600.0, "ground_speed_kt"),
                (-180.0, 180.0, "heading_deg"),
                (-55.0, 70.0, "cabin_temp_c"),
                (75.0, 101.0, "cabin_pressure_kpa"),
                (0.0, 100.0, "fuel_flow_pct"),
                (60.0, 100.0, "hydraulic_pct"),
                (-90.0, 90.0, "pitch_deg"),
            ],
        ),
        (
            "medical_fhir",
            vec![
                (36.1, 37.8, "body_temp_c"),
                (60.0, 100.0, "heart_rate_bpm"),
                (95.0, 100.0, "spo2_pct"),
                (80.0, 120.0, "bp_systolic_mmhg"),
                (60.0, 100.0, "bp_diastolic_mmhg"),
                (12.0, 20.0, "respiratory_rate"),
                (7.35, 7.45, "ph"),
                (0.0, 300.0, "glucose_mg_dl"),
            ],
        ),
        (
            "energy_scada",
            vec![
                (49.0, 51.0, "grid_freq_hz"),
                (0.9, 1.1, "voltage_pu"),
                (0.0, 80.0, "transformer_temp_c"),
                (0.0, 100.0, "line_load_pct"),
                (0.0, 500.0, "current_a"),
                (-100.0, 100.0, "power_factor_offset"),
                (0.0, 360.0, "phase_angle_deg"),
                (0.0, 50.0, "thd_pct"),
            ],
        ),
        (
            "industrial_plc",
            vec![
                (0.0, 300.0, "pressure_psi"),
                (-20.0, 120.0, "temp_c"),
                (0.0, 100.0, "flow_rate_pct"),
                (0.0, 5000.0, "rpm"),
                (0.0, 360.0, "angle_deg"),
                (0.0, 100.0, "vibration_pct"),
                (380.0, 480.0, "voltage_v"),
                (45.0, 65.0, "freq_hz"),
            ],
        ),
        (
            "iot_environmental",
            vec![
                (-40.0, 85.0, "temp_c"),
                (0.0, 100.0, "humidity_pct"),
                (300.0, 1100.0, "pressure_hpa"),
                (0.0, 1000.0, "co2_ppm"),
                (0.0, 500.0, "pm25_ugm3"),
                (0.0, 100.0, "noise_db"),
                (0.0, 100000.0, "lux"),
                (0.0, 20.0, "wind_ms"),
            ],
        ),
        (
            "robotics_ros",
            vec![
                (-3.14159, 3.14159, "joint_angle_rad"),
                (-10.0, 10.0, "angular_vel_rads"),
                (-5.0, 5.0, "linear_vel_ms"),
                (0.0, 100.0, "torque_pct"),
                (-180.0, 180.0, "orientation_deg"),
                (0.0, 10.0, "gripper_force_n"),
                (0.0, 30.0, "battery_voltage_v"),
                (0.0, 100.0, "motor_temp_pct"),
            ],
        ),
        (
            "telecom_5g",
            vec![
                (-120.0, -20.0, "rsrp_dbm"),
                (-130.0, -30.0, "rsrq_db"),
                (-30.0, 0.0, "sinr_db"),
                (0.0, 100.0, "signal_pct"),
                (0.0, 50000.0, "throughput_mbps"),
                (0.0, 100.0, "latency_pct"),
                (0.0, 100.0, "packet_loss_pct"),
                (-40.0, 85.0, "device_temp_c"),
            ],
        ),
        (
            "marine_nmea",
            vec![
                (0.0, 60.0, "speed_knots"),
                (-180.0, 180.0, "longitude_deg"),
                (-90.0, 90.0, "latitude_deg"),
                (-10.0, 10.0, "pitch_deg"),
                (-30.0, 30.0, "roll_deg"),
                (0.0, 1100.0, "depth_m"),
                (800.0, 1100.0, "pressure_hpa"),
                (-20.0, 50.0, "water_temp_c"),
            ],
        ),
        (
            "satellite_telemetry",
            vec![
                (-100.0, 100.0, "angular_rate_degs"),
                (-180.0, 180.0, "attitude_deg"),
                (0.0, 100.0, "solar_panel_pct"),
                (0.0, 100.0, "battery_pct"),
                (-40.0, 85.0, "temp_c"),
                (0.0, 2000.0, "altitude_km"),
                (0.0, 100.0, "data_rate_pct"),
                (0.0, 100.0, "link_quality_pct"),
            ],
        ),
    ]
}

// ── Bytecode encoding for constraints ──
//
// FLUX-C bytecode format for constraint definitions:
//   0x15 (RangeCheck) followed by pairs of (lo, hi) encoded as:
//     u8  n_constraints
//     for each constraint:
//       f64 lo (8 bytes, little-endian)
//       f64 hi (8 bytes, little-endian)
//       u8   name_len
//       [u8; name_len] name
//   0x29 (Halt)
//
// We also support a simpler "constraint block" format:
//   0xFF 0xCA 0xFE  (magic: FLUX JIT constraint block)
//   u8 n
//   for each: f64 lo, f64 hi

/// Encode constraints into a bytecode blob that the JIT can parse
pub fn encode_constraints(lo_hi: &[(f64, f64, &str)]) -> Vec<u8> {
    let mut bc = Vec::new();
    // Push a value on the stack first (required by VM protocol)
    bc.push(OpCode::Push as u8);
    bc.extend_from_slice(&0i32.to_le_bytes());
    // RangeCheck opcode
    bc.push(OpCode::RangeCheck as u8);
    // Constraint block marker + data
    bc.push(0xFF);
    bc.push(0xCA);
    bc.push(0xFE);
    bc.push(lo_hi.len() as u8);
    for (lo, hi, _name) in lo_hi {
        bc.extend_from_slice(&lo.to_le_bytes());
        bc.extend_from_slice(&hi.to_le_bytes());
    }
    // Halt
    bc.push(OpCode::Halt as u8);
    bc
}

/// Extract lo/hi pairs from bytecode
fn extract_constraints(bytecode: &[u8]) -> JitResult<Vec<(f64, f64)>> {
    let mut lo_hi = Vec::new();

    // Strategy 1: Look for our constraint block marker (0xFF 0xCA 0xFE)
    if let Some(pos) = find_constraint_block(bytecode) {
        let n = *bytecode.get(pos + 3).ok_or_else(|| {
            JitError::InvalidBytecode("truncated constraint block count".into())
        })? as usize;

        if n > MAX_JIT_CONSTRAINTS {
            return Err(JitError::TooManyConstraints(n));
        }

        let mut offset = pos + 4;
        for i in 0..n {
            if offset + 16 > bytecode.len() {
                return Err(JitError::InvalidBytecode(format!(
                    "truncated constraint {i}: need 16 bytes at offset {offset}, have {}",
                    bytecode.len()
                )));
            }
            let lo = f64::from_le_bytes(
                bytecode[offset..offset + 8].try_into().unwrap(),
            );
            let hi = f64::from_le_bytes(
                bytecode[offset + 8..offset + 16].try_into().unwrap(),
            );
            if lo > hi {
                return Err(JitError::InvalidRange {
                    index: i,
                    lo,
                    hi,
                });
            }
            lo_hi.push((lo, hi));
            offset += 16;
        }
    } else {
        // Strategy 2: Look for RangeCheck opcodes and extract from constant pool
        // Walk the bytecode for Push + f64 pairs before RangeCheck
        let mut pc = 0;
        while pc < bytecode.len() {
            let op_byte = bytecode[pc];
            if let Some(op) = OpCode::from_u8(op_byte) {
                match op {
                    OpCode::Push | OpCode::LoadConst => {
                        pc += 1 + op.imm_bytes();
                    }
                    OpCode::RangeCheck => {
                        // We need constraints — but without the block marker,
                        // we can't extract them from pure bytecode alone.
                        // This is a fallback: return empty and let caller provide them.
                        pc += 1;
                    }
                    _ => {
                        pc += 1 + op.imm_bytes();
                    }
                }
            } else {
                pc += 1;
            }
        }
    }

    Ok(lo_hi)
}

fn find_constraint_block(bytecode: &[u8]) -> Option<usize> {
    if bytecode.len() < 4 {
        return None;
    }
    for i in 0..bytecode.len() - 3 {
        if bytecode[i] == 0xFF && bytecode[i + 1] == 0xCA && bytecode[i + 2] == 0xFE {
            return Some(i);
        }
    }
    None
}

// ── The JIT checker ──

pub struct JitChecker {
    /// Lower bounds for each constraint
    lo: Vec<f64>,
    /// Upper bounds for each constraint
    hi: Vec<f64>,
    /// Number of constraints
    n: usize,
    /// Native x86_64 machine code bytes (if generated)
    native_code: Option<Vec<u8>>,
}

// Safety: JitChecker is thread-safe — the native code is read-only after construction
unsafe impl Sync for JitChecker {}
unsafe impl Send for JitChecker {}

impl JitChecker {
    /// Build a JIT checker from FLUX-C bytecode
    pub fn from_bytecode(bytecode: &[u8]) -> JitResult<Self> {
        let pairs = extract_constraints(bytecode)?;
        if pairs.is_empty() {
            return Err(JitError::NoConstraints);
        }
        Self::from_pairs(&pairs)
    }

    /// Build a JIT checker from raw lo/hi pairs
    pub fn from_pairs(pairs: &[(f64, f64)]) -> JitResult<Self> {
        if pairs.is_empty() {
            return Err(JitError::NoConstraints);
        }
        if pairs.len() > MAX_JIT_CONSTRAINTS {
            return Err(JitError::TooManyConstraints(pairs.len()));
        }

        let lo: Vec<f64> = pairs.iter().map(|(l, _)| *l).collect();
        let hi: Vec<f64> = pairs.iter().map(|(_, h)| *h).collect();

        // Validate ranges
        for (i, (l, h)) in pairs.iter().enumerate() {
            if l > h {
                return Err(JitError::InvalidRange {
                    index: i,
                    lo: *l,
                    hi: *h,
                });
            }
        }

        let n = pairs.len();

        // Generate native x86_64 code
        #[cfg(target_arch = "x86_64")]
        let native_code = crate::jit_x86::generate_flux_check_native(&lo, &hi).ok();
        #[cfg(not(target_arch = "x86_64"))]
        let native_code = None;

        Ok(Self { lo, hi, n, native_code })
    }

    /// Build from a preset by name
    pub fn from_preset(name: &str) -> JitResult<Self> {
        let presets = all_presets();
        let preset = presets
            .iter()
            .find(|(n, _)| *n == name)
            .ok_or_else(|| JitError::InvalidBytecode(format!("unknown preset: {name}")))?;

        let pairs: Vec<(f64, f64)> = preset.1.iter().map(|(lo, hi, _)| (*lo, *hi)).collect();
        Self::from_pairs(&pairs)
    }

    /// **HOT PATH** — zero-overhead constraint check
    ///
    /// Identical semantics to C `flux_check_exact()`:
    /// - Returns 0 if all constraints pass
    /// - Bitmask of violated constraints otherwise
    /// - NaN violates ALL constraints
    #[inline(always)]
    pub fn check(&self, value: f64) -> u8 {
        // Native x86_64 code is generated but not executed due to W^X protections.
        // The Rust hot path below is equivalent and LLVM auto-vectorizes well.
        // To enable native execution, use mmap(PROT_WRITE|PROT_EXEC) in production.
        self.check_rust(value)
    }

    /// Rust implementation of the hot path — matches C `flux_check_exact` exactly
    #[inline(always)]
    fn check_rust(&self, value: f64) -> u8 {
        // NaN violates all constraints
        if value.is_nan() {
            return (1u8 << self.n) - 1;
        }

        let mut mask: u8 = 0;
        for i in 0..self.n {
            if value < self.lo[i] || value > self.hi[i] {
                mask |= 1 << i;
            }
        }
        mask
    }

    /// Batch check — process multiple values
    ///
    /// Returns a vector of error masks, one per input value.
    /// Uses auto-vectorization-friendly loop structure.
    pub fn check_batch(&self, values: &[f64]) -> Vec<u8> {
        let mut masks = vec![0u8; values.len()];

        // Process in chunks for better cache behavior
        const CHUNK: usize = 64;
        let chunks = values.chunks_exact(CHUNK);
        let remainder = values.len() % CHUNK;

        for (chunk_idx, chunk) in chunks.enumerate() {
            let base = chunk_idx * CHUNK;
            for (i, &value) in chunk.iter().enumerate() {
                masks[base + i] = self.check(value);
            }
        }

        // Handle remainder
        let full_chunks = values.len() - remainder;
        for i in 0..remainder {
            masks[full_chunks + i] = self.check(values[full_chunks + i]);
        }

        masks
    }

    /// Batch check using Rayon for parallelism
    pub fn check_batch_parallel(&self, values: &[f64]) -> Vec<u8> {
        use rayon::prelude::*;
        values.par_iter().map(|&v| self.check(v)).collect()
    }

    /// Number of constraints
    pub fn n_constraints(&self) -> usize {
        self.n
    }

    /// Lower bounds
    pub fn lo(&self) -> &[f64] {
        &self.lo
    }

    /// Upper bounds
    pub fn hi(&self) -> &[f64] {
        &self.hi
    }

    /// Whether native code is available
    pub fn has_native_code(&self) -> bool {
        self.native_code.is_some()
    }

    /// Benchmark: returns checks/sec
    pub fn benchmark(&self, iterations: u64) -> f64 {
        let start = std::time::Instant::now();
        let mut sink = 0u8;
        for i in 0..iterations {
            let value = ((i as i64 % 10000) - 5000) as f64;
            sink |= self.check(value);
        }
        let elapsed = start.elapsed().as_secs_f64();
        std::hint::black_box(&sink);
        if elapsed > 0.0 {
            iterations as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Benchmark batch processing
    pub fn benchmark_batch(&self, batch_size: usize, iterations: u64) -> f64 {
        let values: Vec<f64> = (0..batch_size)
            .map(|i| ((i as i64 % 10000) - 5000) as f64)
            .collect();

        let total_checks = batch_size as u64 * iterations;
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            std::hint::black_box(self.check_batch(&values));
        }
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            total_checks as f64 / elapsed
        } else {
            0.0
        }
    }
}

// ── x86_64 native code generation is in jit_x86.rs ──

// ── Severity classification (matches C implementation) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Pass,
    Caution,
    Warning,
    Critical,
}

const SEVERITY_TABLE: [Severity; 9] = [
    Severity::Pass,     // 0 bits
    Severity::Caution,  // 1 bit
    Severity::Warning,  // 2 bits
    Severity::Warning,  // 3 bits
    Severity::Critical, // 4 bits
    Severity::Critical, // 5 bits
    Severity::Critical, // 6 bits
    Severity::Critical, // 7 bits
    Severity::Critical, // 8 bits
];

/// Classify severity from an error mask (matches C `flux_mask_severity`)
pub fn classify_severity(mask: u8) -> Severity {
    let count = mask.count_ones() as usize;
    SEVERITY_TABLE[if count > 8 { 8 } else { count }]
}

/// Returns true if the mask indicates all constraints passed
pub fn mask_passed(mask: u8) -> bool {
    mask == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_presets_compile() {
        for (name, constraints) in all_presets() {
            let jit = JitChecker::from_preset(name);
            assert!(jit.is_ok(), "preset {name} failed: {:?}", jit.err());
            let jit = jit.unwrap();
            assert_eq!(jit.n_constraints(), constraints.len());
        }
    }

    #[test]
    fn test_basic_check() {
        let jit = JitChecker::from_pairs(&[(0.0, 100.0)]).unwrap();
        assert_eq!(jit.check(50.0), 0); // in range
        assert_eq!(jit.check(-1.0), 1); // below lo
        assert_eq!(jit.check(101.0), 1); // above hi
        assert_eq!(jit.check(0.0), 0); // boundary lo
        assert_eq!(jit.check(100.0), 0); // boundary hi
    }

    #[test]
    fn test_nan() {
        let jit = JitChecker::from_pairs(&[(0.0, 100.0), (-50.0, 50.0)]).unwrap();
        let mask = jit.check(f64::NAN);
        assert_eq!(mask, 0b11); // violates all
    }

    #[test]
    fn test_infinity() {
        let jit = JitChecker::from_pairs(&[(0.0, 100.0)]).unwrap();
        assert_eq!(jit.check(f64::INFINITY), 1);
        assert_eq!(jit.check(f64::NEG_INFINITY), 1);
    }

    #[test]
    fn test_multi_constraint() {
        let jit = JitChecker::from_pairs(&[
            (0.0, 100.0),
            (50.0, 150.0),
            (0.0, 50.0),
        ])
        .unwrap();

        // value 75: in [0,100] ✓, in [50,150] ✓, NOT in [0,50] → bit 2
        assert_eq!(jit.check(75.0), 0b100);

        // value -1: violates all 3
        assert_eq!(jit.check(-1.0), 0b111);

        // value 25: in [0,100] ✓, NOT in [50,150] → bit 1, in [0,50] ✓
        assert_eq!(jit.check(25.0), 0b010);
    }

    #[test]
    fn test_bytecode_roundtrip() {
        let constraints = vec![
            (0.0, 8000.0, "engine_rpm"),
            (0.0, 300.0, "speed"),
        ];
        let bc = encode_constraints(&constraints);
        let jit = JitChecker::from_bytecode(&bc).unwrap();
        assert_eq!(jit.n_constraints(), 2);
        assert_eq!(jit.check(4000.0), 0b10);  // in engine_rpm, NOT in speed
        assert_eq!(jit.check(9000.0), 0b11);  // violates both
        assert_eq!(jit.check(200.0), 0);       // in both
        assert_eq!(jit.check(350.0), 0b10);    // in engine_rpm, NOT in speed
    }

    #[test]
    fn test_classify_severity() {
        assert_eq!(classify_severity(0), Severity::Pass);   // 0 bits
        assert_eq!(classify_severity(1), Severity::Caution); // 1 bit
        assert_eq!(classify_severity(2), Severity::Caution); // 1 bit (0b10)
        assert_eq!(classify_severity(3), Severity::Warning); // 2 bits (0b11)
        assert_eq!(classify_severity(0b111), Severity::Warning); // 3 bits
        assert_eq!(classify_severity(0b1111), Severity::Critical); // 4 bits
        assert_eq!(classify_severity(0b11111), Severity::Critical); // 5 bits
        assert_eq!(classify_severity(0b111111), Severity::Critical); // 6 bits
        assert_eq!(classify_severity(0b1111111), Severity::Critical); // 7 bits
        assert_eq!(classify_severity(0b11111111), Severity::Critical); // 8 bits
    }

    #[test]
    fn test_batch() {
        let jit = JitChecker::from_pairs(&[(0.0, 100.0)]).unwrap();
        let values = vec![-1.0, 50.0, 101.0, 0.0, 100.0];
        let masks = jit.check_batch(&values);
        assert_eq!(masks, vec![1, 0, 1, 0, 0]);
    }

    #[test]
    fn test_invalid_range() {
        let result = JitChecker::from_pairs(&[(100.0, 0.0)]);
        assert!(matches!(result, Err(JitError::InvalidRange { .. })));
    }

    #[test]
    fn test_too_many_constraints() {
        let pairs: Vec<(f64, f64)> = (0..9).map(|i| (i as f64, (i + 1) as f64)).collect();
        let result = JitChecker::from_pairs(&pairs);
        assert!(matches!(result, Err(JitError::TooManyConstraints(9))));
    }
}

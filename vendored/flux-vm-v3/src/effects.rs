/// Effect handler system — silent/log/halt/broadcast

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectMode {
    Silent,  // Do nothing, just track
    Log,     // Log to provenance
    Halt,    // Stop execution on violation
    Broadcast, // Would send to fleet (stub)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Ok = 0,
    Warn = 1,
    Fail = 2,
    Critical = 3,
}

impl From<i32> for Severity {
    fn from(v: i32) -> Self {
        match v {
            0 => Self::Ok,
            1 => Self::Warn,
            2 => Self::Fail,
            _ => Self::Critical,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EffectEvent {
    pub severity: Severity,
    pub message: String,
    pub cycle: u64,
}

#[derive(Debug, Clone)]
pub struct EffectHandler {
    pub mode: EffectMode,
    events: Vec<EffectEvent>,
    halted: bool,
    mask_accumulator: u8,
}

impl Default for EffectHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectHandler {
    pub fn new() -> Self {
        Self {
            mode: EffectMode::Log,
            events: Vec::new(),
            halted: false,
            mask_accumulator: 0xff,
        }
    }

    pub fn with_mode(mode: EffectMode) -> Self {
        Self {
            mode,
            ..Self::new()
        }
    }

    /// Emit an effect event
    pub fn emit(&mut self, severity: Severity, msg: &str, cycle: u64) -> bool {
        let event = EffectEvent {
            severity,
            message: msg.to_string(),
            cycle,
        };
        self.events.push(event);
        match self.mode {
            EffectMode::Silent => true,
            EffectMode::Log => true,
            EffectMode::Halt => {
                if severity == Severity::Fail || severity == Severity::Critical {
                    self.halted = true;
                    false
                } else {
                    true
                }
            }
            EffectMode::Broadcast => {
                // Stub: would send to fleet
                true
            }
        }
    }

    /// Accumulate a mask bit
    pub fn accumulate_mask(&mut self, mask: u8) {
        self.mask_accumulator &= mask;
    }

    pub fn mask(&self) -> u8 {
        self.mask_accumulator
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    pub fn events(&self) -> &[EffectEvent] {
        &self.events
    }

    pub fn reset(&mut self) {
        self.events.clear();
        self.halted = false;
        self.mask_accumulator = 0xff;
    }
}

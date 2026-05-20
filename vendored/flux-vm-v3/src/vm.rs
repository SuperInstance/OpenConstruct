/// VM core — stack machine, execution loop
use crate::error::{FluxError, FluxResult};
use crate::opcode::OpCode;
use crate::memory::{BoundedMemory, STACK_LIMIT};
use crate::vector::VectorUnit;
use crate::proof::{ProofCertificate, ProofContext};
use crate::provenance::ProvenanceLog;
use crate::effects::EffectHandler;
use crate::streaming::StreamState;
use crate::check::{Constraint, aviation_preset};

pub const MAX_CYCLES: u64 = 4096;

#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub pc: usize,
    pub stack_len: usize,
    pub cycle: u64,
}

#[derive(Debug, Clone)]
pub struct VmResult {
    pub pass: bool,
    pub value: i32,
    pub cycles: u64,
}

impl Default for VmResult {
    fn default() -> Self {
        Self {
            pass: true,
            value: 0,
            cycles: 0,
        }
    }
}

pub struct FluxVM {
    stack: Vec<i32>,
    registers: [i32; 8],
    vector_unit: VectorUnit,
    bytecode: Vec<u8>,
    pc: usize,
    cycle_count: u64,
    max_cycles: u64,
    constraints: Vec<Constraint>,
    result: VmResult,
    handler: EffectHandler,
    provenance: ProvenanceLog,
    proof_ctx: ProofContext,
    memory: BoundedMemory,
    stream: StreamState,
    checkpoints: Vec<Checkpoint>,
    call_stack: Vec<usize>,
}

impl Default for FluxVM {
    fn default() -> Self {
        Self::new()
    }
}

impl FluxVM {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(STACK_LIMIT),
            registers: [0i32; 8],
            vector_unit: VectorUnit::new(),
            bytecode: Vec::new(),
            pc: 0,
            cycle_count: 0,
            max_cycles: MAX_CYCLES,
            constraints: Vec::new(),
            result: VmResult::default(),
            handler: EffectHandler::new(),
            provenance: ProvenanceLog::new(),
            proof_ctx: ProofContext::new(),
            memory: BoundedMemory::new(),
            stream: StreamState::new(),
            checkpoints: Vec::new(),
            call_stack: Vec::new(),
        }
    }

    // ── Configuration ──

    pub fn load_bytecode(&mut self, bc: &[u8]) {
        self.bytecode = bc.to_vec();
        self.pc = 0;
    }

    pub fn load_constraints(&mut self, constraints: Vec<Constraint>) {
        self.constraints = constraints;
    }

    pub fn set_handler(&mut self, handler: EffectHandler) {
        self.handler = handler;
    }

    pub fn set_max_cycles(&mut self, max: u64) {
        self.max_cycles = max;
    }

    pub fn push_value(&mut self, v: i32) {
        self.stack.push(v);
    }

    // ── Stack helpers ──

    #[inline]
    fn pop(&mut self) -> FluxResult<i32> {
        self.stack.pop().ok_or(FluxError::StackUnderflow)
    }

    #[inline]
    fn push(&mut self, v: i32) -> FluxResult<()> {
        if self.stack.len() >= STACK_LIMIT {
            return Err(FluxError::StackOverflow);
        }
        self.stack.push(v);
        Ok(())
    }

    #[inline]
    fn read_i32(&self, offset: usize) -> i32 {
        if offset + 4 <= self.bytecode.len() {
            i32::from_le_bytes([
                self.bytecode[offset],
                self.bytecode[offset + 1],
                self.bytecode[offset + 2],
                self.bytecode[offset + 3],
            ])
        } else {
            0
        }
    }

    #[inline]
    fn read_u8(&self, offset: usize) -> u8 {
        self.bytecode.get(offset).copied().unwrap_or(0)
    }

    #[inline]
    fn read_u16(&self, offset: usize) -> u16 {
        if offset + 2 <= self.bytecode.len() {
            u16::from_le_bytes([self.bytecode[offset], self.bytecode[offset + 1]])
        } else {
            0
        }
    }

    fn tick(&mut self) -> FluxResult<()> {
        self.cycle_count += 1;
        if self.cycle_count > self.max_cycles {
            return Err(FluxError::CycleLimitExceeded);
        }
        Ok(())
    }

    // ── Execution ──

    pub fn run(&mut self) -> FluxResult<VmResult> {
        self.cycle_count = 0;
        self.pc = 0;
        self.result = VmResult::default();

        while self.pc < self.bytecode.len() {
            if self.handler.is_halted() {
                self.result.pass = false;
                break;
            }

            self.tick()?;

            let op_byte = self.bytecode[self.pc];
            let op = OpCode::from_u8(op_byte).ok_or(FluxError::InvalidOpCode(op_byte))?;
            self.pc += 1;

            self.execute(op)?;

            if self.result.pass && matches!(op, OpCode::Halt) {
                break;
            }
            if !self.result.pass {
                break;
            }
        }

        self.result.cycles = self.cycle_count;
        Ok(self.result.clone())
    }

    fn execute(&mut self, op: OpCode) -> FluxResult<()> {
        match op {
            // ── Stack ops ──
            OpCode::Push => {
                let val = self.read_i32(self.pc);
                self.pc += 4;
                self.push(val)?;
            }
            OpCode::Pop => {
                self.pop()?;
            }
            OpCode::Dup => {
                let v = self.stack.last().copied().ok_or(FluxError::StackUnderflow)?;
                self.push(v)?;
            }
            OpCode::Swap => {
                let len = self.stack.len();
                if len < 2 {
                    return Err(FluxError::StackUnderflow);
                }
                self.stack.swap(len - 1, len - 2);
            }
            OpCode::Over => {
                let len = self.stack.len();
                if len < 2 {
                    return Err(FluxError::StackUnderflow);
                }
                let v = self.stack[len - 2];
                self.push(v)?;
            }
            OpCode::Drop => {
                self.pop()?;
            }
            OpCode::LoadConst => {
                let val = self.read_i32(self.pc);
                self.pc += 4;
                self.stack.clear();
                self.push(val)?;
            }
            OpCode::Nop => {}

            // ── Arithmetic ──
            OpCode::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a.wrapping_add(b))?;
            }
            OpCode::Sub => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a.wrapping_sub(b))?;
            }
            OpCode::Mul => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a.wrapping_mul(b))?;
            }
            OpCode::Div => {
                let b = self.pop()?;
                let a = self.pop()?;
                if b == 0 {
                    return Err(FluxError::DivisionByZero);
                }
                self.push(a / b)?;
            }
            OpCode::Saturate => {
                let hi = self.pop()?;
                let lo = self.pop()?;
                let v = self.pop()?;
                let clamped = v.max(lo).min(hi);
                self.push(clamped)?;
            }
            OpCode::Min => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a.min(b))?;
            }
            OpCode::Max => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a.max(b))?;
            }
            OpCode::Abs => {
                let v = self.pop()?;
                self.push(v.abs())?;
            }

            // ── Register ──
            OpCode::LoadReg => {
                let reg = self.read_u8(self.pc) as usize;
                self.pc += 1;
                if reg >= 8 {
                    return Err(FluxError::InvalidRegister(reg as u8));
                }
                self.push(self.registers[reg])?;
            }
            OpCode::StoreReg => {
                let reg = self.read_u8(self.pc) as usize;
                self.pc += 1;
                let v = self.pop()?;
                if reg >= 8 {
                    return Err(FluxError::InvalidRegister(reg as u8));
                }
                self.registers[reg] = v;
            }
            OpCode::LoadRegVec => {
                let reg = self.read_u8(self.pc) as usize;
                self.pc += 1;
                if reg >= 4 {
                    return Err(FluxError::InvalidRegister(reg as u8));
                }
                let vreg = self.vector_unit.store(reg as u8)?;
                for lane in &vreg {
                    self.push(*lane as i32)?;
                }
            }
            OpCode::StoreRegVec => {
                let reg = self.read_u8(self.pc) as usize;
                self.pc += 1;
                if reg >= 4 {
                    return Err(FluxError::InvalidRegister(reg as u8));
                }
                let mut data = [0i8; 8];
                for i in (0..8).rev() {
                    data[i] = self.pop()? as i8;
                }
                self.vector_unit.load(reg as u8, data)?;
            }

            // ── Constraint ──
            OpCode::RangeCheck => {
                if self.constraints.is_empty() {
                    return Err(FluxError::NoConstraint);
                }
                let (lo, hi) = {
                    let c = self.constraints.first().unwrap();
                    (c.lo, c.hi)
                };
                let value = self.pop()?;
                let pass = value >= lo && value <= hi;
                self.proof_ctx.prove_range(value, lo, hi, pass);
                self.push(if pass { 1 } else { 0 })?;
                if !pass {
                    self.result.pass = false;
                }
            }
            OpCode::BatchCheck => {
                if self.constraints.is_empty() {
                    return Err(FluxError::NoConstraint);
                }
                let count = self.pop()? as usize;
                let (lo, hi) = {
                    let c = self.constraints.first().unwrap();
                    (c.lo, c.hi)
                };
                let mut pass_count = 0u32;
                for _ in 0..count {
                    if let Ok(v) = self.pop() {
                        if v >= lo && v <= hi {
                            pass_count += 1;
                        }
                    }
                }
                self.push(pass_count as i32)?;
            }
            OpCode::AccumulateMask => {
                let mask = self.pop()? as u8;
                self.handler.accumulate_mask(mask);
                self.push(self.handler.mask() as i32)?;
            }
            OpCode::ClassifySeverity => {
                let mask = self.pop()? as u8;
                use crate::check::classify_mask;
                let sev = classify_mask(mask);
                self.push(sev as i32)?;
            }
            OpCode::Prove => {
                let v = self.pop()?;
                let hash = self.proof_ctx.prove_value(v);
                // Push first 4 bytes of hash as i32
                let val = i32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
                self.push(val)?;
            }
            OpCode::QueryBackward => {
                // Pop depth, return hash at that position
                let depth = self.pop()? as usize;
                if depth < self.proof_ctx.chain_len() {
                    self.push(depth as i32)?; // simplified
                } else {
                    self.push(-1)?;
                }
            }
            OpCode::Simplify => {
                // Pop and push — identity for now (simplification is constraint-dependent)
                let v = self.pop()?;
                self.push(v)?;
            }
            OpCode::Validate => {
                let v = self.pop()?;
                let pass = v != 0;
                self.push(if pass { 1 } else { 0 })?;
                if !pass {
                    self.result.pass = false;
                }
            }
            OpCode::HashCommit => {
                let root = self.proof_ctx.commit();
                let val = i32::from_le_bytes([root[0], root[1], root[2], root[3]]);
                self.push(val)?;
            }
            OpCode::Seal => {
                let root = self.proof_ctx.seal()?;
                let val = i32::from_le_bytes([root[0], root[1], root[2], root[3]]);
                self.push(val)?;
            }

            // ── Vector/SIMD ──
            OpCode::VecLoad => {
                // Load 8 values from stack into vector reg
                let reg = self.pop()? as u8;
                let mut data = [0i8; 8];
                for i in (0..8).rev() {
                    data[i] = self.pop()? as i8;
                }
                self.vector_unit.load(reg, data)?;
            }
            OpCode::VecStore => {
                let reg = self.pop()? as u8;
                let data = self.vector_unit.store(reg)?;
                for lane in &data {
                    self.push(*lane as i32)?;
                }
            }
            OpCode::VecRangeCheck => {
                let hi = self.pop()? as i8;
                let lo = self.pop()? as i8;
                let reg = self.pop()? as u8;
                let mask = self.vector_unit.range_check(reg, lo, hi)?;
                self.proof_ctx.prove_vector(mask, lo, hi);
                self.push(mask as i32)?;
            }
            OpCode::VecMaskMerge => {
                let b = self.pop()? as u8;
                let a = self.pop()? as u8;
                self.push((a & b) as i32)?;
            }
            OpCode::VecReduce => {
                let reg = self.pop()? as u8;
                let sum = self.vector_unit.reduce(reg)?;
                self.push(sum)?;
            }
            OpCode::VecGather => {
                // Pop indices, gather into vector reg
                let _dst = self.pop()? as u8;
                let _src = self.pop()? as u8;
                // Simplified: no-op for gather
            }

            // ── Control ──
            OpCode::FwdJump => {
                let offset = self.read_u16(self.pc) as usize;
                self.pc += 2;
                if self.pc + offset > self.bytecode.len() {
                    return Err(FluxError::InvalidJump(self.pc + offset));
                }
                self.pc += offset;
            }
            OpCode::CondJump => {
                let offset = self.read_u16(self.pc) as usize;
                self.pc += 2;
                let cond = self.pop()?;
                if cond != 0 {
                    if self.pc + offset > self.bytecode.len() {
                        return Err(FluxError::InvalidJump(self.pc + offset));
                    }
                    self.pc += offset;
                }
            }
            OpCode::CallBounded => {
                let target = self.read_u16(self.pc) as usize;
                self.pc += 2;
                if target > self.bytecode.len() {
                    return Err(FluxError::InvalidJump(target));
                }
                self.call_stack.push(self.pc);
                self.pc = target;
            }
            OpCode::Ret => {
                if let Some(ret_pc) = self.call_stack.pop() {
                    self.pc = ret_pc;
                }
            }
            OpCode::Halt => {
                self.result.cycles = self.cycle_count;
                // Don't break here — caller will check
            }
            OpCode::Checkpoint => {
                self.checkpoints.push(Checkpoint {
                    pc: self.pc,
                    stack_len: self.stack.len(),
                    cycle: self.cycle_count,
                });
                self.provenance.record(
                    self.cycle_count,
                    "checkpoint",
                    &self.cycle_count.to_le_bytes(),
                );
            }

            // ── Effects ──
            OpCode::SetHandler => {
                let mode = self.pop()?;
                self.handler.mode = match mode {
                    0 => crate::effects::EffectMode::Silent,
                    1 => crate::effects::EffectMode::Log,
                    2 => crate::effects::EffectMode::Halt,
                    3 => crate::effects::EffectMode::Broadcast,
                    _ => crate::effects::EffectMode::Log,
                };
            }
            OpCode::EmitEvent => {
                let severity = self.pop()?;
                self.handler.emit(
                    crate::effects::Severity::from(severity),
                    "user_event",
                    self.cycle_count,
                );
            }
            OpCode::Rollback => {
                if let Some(cp) = self.checkpoints.pop() {
                    self.stack.truncate(cp.stack_len);
                    self.pc = cp.pc;
                }
            }
            OpCode::GetResult => {
                self.push(if self.result.pass { 1 } else { 0 })?;
            }

            // ── Parallel (stubs — real parallel via ParallelBatch) ──
            OpCode::ParDispatch
            | OpCode::ParMerge
            | OpCode::ParBarrier
            | OpCode::ParReduce => {
                // These are handled externally via ParallelBatch
                // In-VM, they're identity ops
            }

            // ── Provenance ──
            OpCode::SnapRecord => {
                let tag_val = self.pop()?;
                self.provenance.record(
                    self.cycle_count,
                    &format!("snap_{}", tag_val),
                    &tag_val.to_le_bytes(),
                );
            }
            OpCode::SnapQuery => {
                // Push log length
                self.push(self.provenance.len() as i32)?;
            }
            OpCode::SnapHash => {
                let hash = self.provenance.hash();
                let val = i32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
                self.push(val)?;
            }
            OpCode::SnapVerify => {
                // Always passes in this implementation
                self.push(1)?;
            }

            // ── Streaming ──
            OpCode::StreamOpen => {
                let batch_size = self.pop().unwrap_or(64);
                self.stream.open(batch_size as usize)?;
            }
            OpCode::StreamCheck => {
                if self.stream.is_open() {
                    let value = self.pop()?;
                    self.stream.push(value)?;
                }
            }
            OpCode::StreamBatch => {
                if self.stream.is_open() {
                    let lo = self.pop()?;
                    let hi = self.pop()?;
                    let pass_count = self.stream.check_range(lo, hi);
                    self.push(pass_count as i32)?;
                }
            }
            OpCode::StreamClose => {
                self.stream.close()?;
            }
        }

        Ok(())
    }

    // ── Accessors ──

    pub fn result(&self) -> &VmResult {
        &self.result
    }

    pub fn proof_certificate(&self) -> Option<ProofCertificate> {
        ProofCertificate::from_context(&self.proof_ctx, self.cycle_count)
    }

    pub fn provenance_log(&self) -> &ProvenanceLog {
        &self.provenance
    }

    pub fn stack(&self) -> &[i32] {
        &self.stack
    }

    pub fn registers(&self) -> &[i32; 8] {
        &self.registers
    }

    pub fn cycles(&self) -> u64 {
        self.cycle_count
    }

    pub fn memory(&self) -> &BoundedMemory {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut BoundedMemory {
        &mut self.memory
    }

    pub fn vector_unit(&self) -> &VectorUnit {
        &self.vector_unit
    }

    pub fn vector_unit_mut(&mut self) -> &mut VectorUnit {
        &mut self.vector_unit
    }

    pub fn stream(&self) -> &StreamState {
        &self.stream
    }

    pub fn handler(&self) -> &EffectHandler {
        &self.handler
    }

    pub fn reset(&mut self) {
        self.stack.clear();
        self.registers = [0i32; 8];
        self.vector_unit = VectorUnit::new();
        self.bytecode.clear();
        self.pc = 0;
        self.cycle_count = 0;
        self.constraints.clear();
        self.result = VmResult::default();
        self.handler.reset();
        self.provenance.reset();
        self.proof_ctx.reset();
        self.memory.reset();
        self.stream.reset();
        self.checkpoints.clear();
        self.call_stack.clear();
    }

    /// Built-in benchmark: run constraint checks N times, return checks/sec
    pub fn benchmark(&mut self, iterations: u64) -> f64 {
        if self.constraints.is_empty() {
            self.load_constraints(aviation_preset());
        }
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            for c in &self.constraints {
                let _ = c.check(42);
            }
        }
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            (iterations as f64 * self.constraints.len() as f64) / elapsed
        } else {
            0.0
        }
    }
}

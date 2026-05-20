/// SIMD vector unit — 8-wide INT8 operations
use crate::error::{FluxError, FluxResult};

pub const VEC_WIDTH: usize = 8;
pub const VEC_REGS: usize = 4;

#[derive(Debug, Clone, Default)]
pub struct VectorUnit {
    pub regs: [[i8; VEC_WIDTH]; VEC_REGS],
}

impl VectorUnit {
    pub fn new() -> Self {
        Self {
            regs: [[0i8; VEC_WIDTH]; VEC_REGS],
        }
    }

    pub fn load(&mut self, reg: u8, data: [i8; VEC_WIDTH]) -> FluxResult<()> {
        let r = reg as usize;
        if r >= VEC_REGS {
            return Err(FluxError::InvalidRegister(reg));
        }
        self.regs[r] = data;
        Ok(())
    }

    pub fn store(&self, reg: u8) -> FluxResult<[i8; VEC_WIDTH]> {
        let r = reg as usize;
        if r >= VEC_REGS {
            return Err(FluxError::InvalidRegister(reg));
        }
        Ok(self.regs[r])
    }

    /// 8-wide INT8 range check: for each lane, check lo <= val <= hi
    /// Returns bitmask: bit i set if lane i passes
    pub fn range_check(&self, reg: u8, lo: i8, hi: i8) -> FluxResult<u8> {
        let data = self.store(reg)?;
        let mut mask: u8 = 0;
        for i in 0..VEC_WIDTH {
            if data[i] >= lo && data[i] <= hi {
                mask |= 1 << i;
            }
        }
        Ok(mask)
    }

    /// Merge masks with AND
    pub fn mask_merge(masks: &[u8]) -> u8 {
        masks.iter().fold(0xff, |acc, &m| acc & m)
    }

    /// Horizontal reduce: sum all lanes as i32
    pub fn reduce(&self, reg: u8) -> FluxResult<i32> {
        let data = self.store(reg)?;
        Ok(data.iter().map(|&v| v as i32).sum())
    }

    /// Gather: pick lanes from source using indices
    pub fn gather(&mut self, dst: u8, src: u8, indices: [u8; VEC_WIDTH]) -> FluxResult<()> {
        let src_data = self.store(src)?;
        let mut result = [0i8; VEC_WIDTH];
        for i in 0..VEC_WIDTH {
            let idx = indices[i] as usize;
            result[i] = if idx < VEC_WIDTH { src_data[idx] } else { 0 };
        }
        self.load(dst, result)
    }

    /// Batch range check across multiple registers
    pub fn batch_range_check(&self, lo: i8, hi: i8) -> FluxResult<u8> {
        let mut combined: u8 = 0xff;
        for r in 0..VEC_REGS {
            let mask = self.range_check(r as u8, lo, hi)?;
            combined &= mask;
        }
        Ok(combined)
    }
}

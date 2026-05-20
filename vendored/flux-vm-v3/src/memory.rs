/// Bounded memory — no dynamic allocation after init
use crate::error::{FluxError, FluxResult};

pub const HEAP_SIZE: usize = 4096; // 4KB heap
pub const STACK_LIMIT: usize = 256;

#[derive(Debug, Clone)]
pub struct BoundedMemory {
    heap: [u8; HEAP_SIZE],
    heap_used: usize,
}

impl Default for BoundedMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl BoundedMemory {
    pub fn new() -> Self {
        Self {
            heap: [0u8; HEAP_SIZE],
            heap_used: 0,
        }
    }

    pub fn write(&mut self, offset: usize, data: &[u8]) -> FluxResult<()> {
        if offset + data.len() > HEAP_SIZE {
            return Err(FluxError::MemoryExceeded);
        }
        self.heap[offset..offset + data.len()].copy_from_slice(data);
        self.heap_used = self.heap_used.max(offset + data.len());
        Ok(())
    }

    pub fn read(&self, offset: usize, len: usize) -> FluxResult<&[u8]> {
        if offset + len > HEAP_SIZE {
            return Err(FluxError::MemoryExceeded);
        }
        Ok(&self.heap[offset..offset + len])
    }

    pub fn read_i32(&self, offset: usize) -> FluxResult<i32> {
        let bytes = self.read(offset, 4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn write_i32(&mut self, offset: usize, val: i32) -> FluxResult<()> {
        self.write(offset, &val.to_le_bytes())
    }

    pub fn used(&self) -> usize {
        self.heap_used
    }

    pub fn available(&self) -> usize {
        HEAP_SIZE - self.heap_used
    }

    pub fn reset(&mut self) {
        self.heap = [0u8; HEAP_SIZE];
        self.heap_used = 0;
    }
}

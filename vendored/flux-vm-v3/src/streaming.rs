/// Streaming mode for data streams
use crate::error::{FluxError, FluxResult};

#[derive(Debug, Clone)]
pub struct StreamState {
    open: bool,
    buffer: Vec<i32>,
    batch_size: usize,
    results: Vec<bool>,
}

impl Default for StreamState {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamState {
    pub fn new() -> Self {
        Self {
            open: false,
            buffer: Vec::new(),
            batch_size: 64,
            results: Vec::new(),
        }
    }

    pub fn open(&mut self, batch_size: usize) -> FluxResult<()> {
        if self.open {
            return Err(FluxError::StreamAlreadyOpen);
        }
        self.open = true;
        self.batch_size = batch_size.max(1);
        self.buffer.clear();
        self.results.clear();
        Ok(())
    }

    pub fn push(&mut self, value: i32) -> FluxResult<()> {
        if !self.open {
            return Err(FluxError::StreamNotOpen);
        }
        self.buffer.push(value);
        Ok(())
    }

    pub fn close(&mut self) -> FluxResult<&[bool]> {
        if !self.open {
            return Err(FluxError::StreamNotOpen);
        }
        self.open = false;
        Ok(&self.results)
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn results(&self) -> &[bool] {
        &self.results
    }

    /// Check the buffer against a range, returns pass count
    pub fn check_range(&mut self, lo: i32, hi: i32) -> usize {
        let mut pass = 0;
        for &v in &self.buffer {
            let ok = v >= lo && v <= hi;
            self.results.push(ok);
            if ok {
                pass += 1;
            }
        }
        self.buffer.clear();
        pass
    }

    pub fn reset(&mut self) {
        self.open = false;
        self.buffer.clear();
        self.results.clear();
    }
}

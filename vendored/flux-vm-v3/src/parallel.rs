/// Parallel batch dispatch (rayon-based)
use rayon::prelude::*;

/// A single constraint check: value in [lo, hi]
#[derive(Debug, Clone, Copy)]
pub struct ConstraintCheck {
    pub value: i32,
    pub lo: i32,
    pub hi: i32,
}

impl ConstraintCheck {
    pub fn new(value: i32, lo: i32, hi: i32) -> Self {
        Self { value, lo, hi }
    }

    pub fn check(&self) -> bool {
        self.value >= self.lo && self.value <= self.hi
    }
}

/// Batch of constraint checks with parallel dispatch
#[derive(Debug, Clone)]
pub struct ParallelBatch {
    checks: Vec<ConstraintCheck>,
}

impl ParallelBatch {
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }

    pub fn add(&mut self, check: ConstraintCheck) {
        self.checks.push(check);
    }

    pub fn add_range(&mut self, values: &[i32], lo: i32, hi: i32) {
        for &v in values {
            self.checks.push(ConstraintCheck::new(v, lo, hi));
        }
    }

    /// Run all checks in parallel, return pass count
    pub fn dispatch(&self) -> ParallelResult {
        let results: Vec<bool> = self.checks.par_iter().map(|c| c.check()).collect();
        let passed = results.iter().filter(|&&r| r).count();
        ParallelResult {
            total: results.len(),
            passed,
            failed: results.len() - passed,
            pass_rate: if results.is_empty() {
                1.0
            } else {
                passed as f64 / results.len() as f64
            },
        }
    }

    /// Parallel reduce: sum all passing values
    pub fn reduce_sum(&self) -> i64 {
        self.checks
            .par_iter()
            .filter(|c| c.check())
            .map(|c| c.value as i64)
            .sum()
    }

    pub fn len(&self) -> usize {
        self.checks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.checks.is_empty()
    }

    pub fn clear(&mut self) {
        self.checks.clear();
    }
}

#[derive(Debug, Clone)]
pub struct ParallelResult {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f64,
}

impl ParallelResult {
    pub fn all_pass(&self) -> bool {
        self.failed == 0
    }
}

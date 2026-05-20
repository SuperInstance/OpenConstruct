/// Built-in benchmarking utilities
use crate::vm::FluxVM;
use crate::check::{aviation_preset, temperature_preset, Constraint};

pub struct BenchResult {
    pub checks_per_sec: f64,
    pub total_checks: u64,
    pub elapsed_secs: f64,
    pub avg_ns: f64,
}

impl std::fmt::Display for BenchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.0} checks/sec ({} total, {:.3}s, {:.1}ns/check)",
            self.checks_per_sec, self.total_checks, self.elapsed_secs, self.avg_ns
        )
    }
}

/// Benchmark raw constraint checking
pub fn bench_constraints(constraints: &[Constraint], iterations: u64) -> BenchResult {
    let start = std::time::Instant::now();
    let mut count = 0u64;
    for _ in 0..iterations {
        for c in constraints {
            let _ = c.check(42);
            count += 1;
        }
    }
    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64();
    BenchResult {
        checks_per_sec: if secs > 0.0 { count as f64 / secs } else { 0.0 },
        total_checks: count,
        elapsed_secs: secs,
        avg_ns: if count > 0 { elapsed.as_nanos() as f64 / count as f64 } else { 0.0 },
    }
}

/// Run full benchmark suite
pub fn run_benchmark_suite() -> String {
    let mut results = Vec::new();

    // Aviation
    let aviation = aviation_preset();
    let r = bench_constraints(&aviation, 10_000_000);
    results.push(format!("Aviation (6 constraints, 10M iters): {}", r));

    // Temperature
    let temp = temperature_preset();
    let r = bench_constraints(&temp, 10_000_000);
    results.push(format!("Temperature (3 constraints, 10M iters): {}", r));

    // Single constraint
    let single = vec![Constraint::new(0, 100, "bench")];
    let r = bench_constraints(&single, 100_000_000);
    results.push(format!("Single constraint (100M iters): {}", r));

    // VM-level benchmark
    let mut vm = FluxVM::new();
    vm.load_constraints(aviation_preset());
    let rate = vm.benchmark(10_000_000);
    results.push(format!("VM-level (10M iters): {:.0} checks/sec", rate));

    results.join("\n")
}

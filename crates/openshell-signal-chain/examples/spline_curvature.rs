// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Spline Curvature — fare curves of truth at hard ↔ soft boundary.
//!
//! Demonstrates how the dial transforms constraint evaluation:
//! - Hard (0.0): boundary crossing = failure, curvatures are existential
//! - Balanced (0.5): boundary crossing = cost, curvatures are informational  
//! - Soft (1.0): boundary crossing = guidance, curvatures are optional
//!
//! Run with: cargo run --example spline_curvature -p openshell-signal-chain

use openshell_signal_chain::{Dial, SplineConstraint, evaluate_spline, maritime_spline};

fn main() {
    println!("=== Spline Curvature: Fare Curves of Truth ===\n");

    // Create maritime spline constraints
    let constraints = maritime_spline();
    println!("Maritime constraints loaded:");
    for c in &constraints {
        println!(
            "  {}: [{}, {}] — lo_curvature={:.1}, hi_curvature={:.1}",
            c.name, c.lo, c.hi, c.lo_curvature, c.hi_curvature
        );
    }
    println!();

    // Normal passage values
    let normal_values = &[20, 60, 30, -10];
    println!("Normal passage: {:?}", normal_values);

    for dial_pos in [0.0, 0.3, 0.5, 0.7, 1.0] {
        let dial = Dial::new(dial_pos);
        match evaluate_spline(&constraints, normal_values, dial) {
            Ok(r) => println!(
                "  dial={:.1} ({}): PASS — curvature={:.3}, hard_pass={}",
                dial_pos, 
                if dial_pos < 0.25 { "hard" } else if dial_pos < 0.75 { "balanced" } else { "soft" },
                r.total_curvature,
                r.is_hard_pass
            ),
            Err(v) => println!(
                "  dial={:.1} ({}): FAIL — {} violation(s)",
                dial_pos,
                if dial_pos < 0.25 { "hard" } else if dial_pos < 0.75 { "balanced" } else { "soft" },
                v.len()
            ),
        }
    }
    println!();

    // Storm conditions — wave height critical, wind high
    let storm_values = &[18, 120, 85, -45];
    println!("Storm conditions: {:?}", storm_values);

    for dial_pos in [0.0, 0.5, 1.0] {
        let dial = Dial::new(dial_pos);
        match evaluate_spline(&constraints, storm_values, dial) {
            Ok(r) => println!(
                "  dial={:.1}: PASS — curvature={:.3}",
                dial_pos, r.total_curvature
            ),
            Err(v) => {
                for violation in &v {
                    println!(
                        "  dial={:.1}: VIOLATION — {} (value={}, curvature={:.2}, severity={:?})",
                        dial_pos,
                        violation.constraint,
                        violation.value,
                        violation.curvature,
                        violation.severity
                    );
                }
            }
        }
    }
    println!();

    // Custom spline: depth sounding
    let depth_spline = SplineConstraint::new(
        0,     // 0 fathoms (surface)
        120,   // 120 fathoms (safe working depth)
        "depth_fathoms",
        1.3,   // cost increases sharply near surface (wave action)
        1.0,   // cost near bottom is moderate
        0.4,   // neutral zone is deeper (working depth)
    );

    println!("Depth spline (neutral zone at 40% of range):");
    for value in [20, 40, 60, 80, 100, 120, 130] {
        let curvature = depth_spline.curvature_distance(value);
        let hard_result = depth_spline.evaluate(value, Dial::hard());
        let soft_result = depth_spline.evaluate(value, Dial::soft());

        print!(
            "  depth={:4} — curvature={:.2} — hard={} soft={}",
            value,
            curvature,
            if hard_result.is_ok() { "PASS" } else { "FAIL" },
            if soft_result.is_ok() { "ok" } else { "advisory" }
        );
        if let Err(v) = hard_result {
            print!(" (severity={:?})", v.severity);
        }
        println!();
    }
}

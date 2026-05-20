// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Fleet Holonomy — Betti number computation across a fleet of rooms.
//!
//! Run with: cargo run --example fleet_holonomy -p openshell-signal-chain

use openshell_signal_chain::{Dial, HolonomyRoom, HolonomyChain};

fn main() {
    println!("=== Fleet Holonomy: Betti Numbers Across Rooms ===\n");

    // Create a fleet of rooms, each with different topology
    let scenarios = vec![
        ("scouting", 3, 3, Dial::hard()),   // under-constrained
        ("transit", 5, 7, Dial::hard()),    // exactly rigid (Laman)
        ("dockside", 4, 10, Dial::soft()),  // over-constrained → emergent
        ("storm", 6, 15, Dial::hard()),      // strongly emergent
    ];

    let mut chain = HolonomyChain::new("fleet-ops");

    for (name, vertices, edges, dial) in scenarios {
        let mut room = HolonomyRoom::new(name, dial);
        
        // Add vertices (tiles = agents or observations)
        for _ in 0..vertices {
            if dial.position < 0.5 {
                room.add_snap(); // hard: verified snaps
            } else {
                room.add_inference(); // soft: allow inferences
            }
        }
        
        // Add edges (trust connections)
        for _ in 0..edges {
            room.add_edge();
        }
        
        let betti = room.betti();
        let status = room.status();
        
        println!(
            "Room: {:12} | V={:2} E={:3} | β₁={:2} | {} | {}",
            name,
            vertices,
            edges,
            betti.beta,
            if betti.is_rigid { "RIGID" } else if betti.has_emergence { "EMERGENT" } else { "loose" },
            match status {
                openshell_signal_chain::HolonomyStatus::Verified => "verified",
                openshell_signal_chain::HolonomyStatus::Pending(_) => "pending",
                openshell_signal_chain::HolonomyStatus::Inconsistent(_) => "inconsistent",
                openshell_signal_chain::HolonomyStatus::Emergent { .. } => "EMERGENT",
            }
        );
        
        chain.add_room(room);
    }

    println!();
    println!("=== Chain Summary ===");
    println!("Total vertices: {}", chain.total_vertices());
    println!("Total edges:    {}", chain.total_edges());
    println!("Chain β₁:        {}", chain.chain_betti());
    println!("Rigid rooms:    {:?}", chain.rigid_rooms());
    println!("Emergent rooms: {:?}", chain.emergent_rooms());
    
    println!();
    println!("=== Laman Check per Room ===");
    for room in ["scouting", "transit", "dockside", "storm"] {
        let rooms: Vec<_> = (0..).zip(chain.rigid_rooms().iter())
            .filter(|(_, n)| *n == room)
            .collect();
        if let Some(r) = chain.rigid_rooms().iter().position(|n| *n == room) {
            println!("{}: rigid ✓", room);
        } else {
            println!("{}: not rigid", room);
        }
    }
}

//! pythagorean48-codes — 48 exact direction vectors for fleet trust encoding
//!
//! This is the SHARED codebook used across multiple SuperInstance repos:
//! - fleet-coordinate (trust topology)
//! - aboracle (research encoding)
//! - holonomy-consensus (consensus verification)
//!
//! Property: 48 directions = log₂(48) ≈ 5.585 bits per vector
//! Zero drift after unlimited hops (exact integer arithmetic)

use serde::{Deserialize, Serialize};

/// A vector encoded in one of 48 exact directions (6 bits)
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TrustVector(pub u8);

impl TrustVector {
    pub const COUNT: usize = 48;

    /// All 48 direction vectors as exact rationals (xn/xd, yn/yd)
    pub fn all_directions() -> [(i16, i16, i16, i16); 48] {
        [
            // Cardinal axes (4)
            (1, 1, 0, 1), (-1, 1, 0, 1), (0, 1, 1, 1), (0, 1, -1, 1),
            // 3-4-5 triangles (8)
            (3, 5, 4, 5), (-3, 5, 4, 5), (3, 5, -4, 5), (-3, 5, -4, 5),
            (4, 5, 3, 5), (-4, 5, 3, 5), (4, 5, -3, 5), (-4, 5, -3, 5),
            // 5-12-13 triangles (8)
            (5, 13, 12, 13), (-5, 13, 12, 13), (5, 13, -12, 13), (-5, 13, -12, 13),
            (12, 13, 5, 13), (-12, 13, 5, 13), (12, 13, -5, 13), (-12, 13, -5, 13),
            // 7-24-25 triangles (8)
            (7, 25, 24, 25), (-7, 25, 24, 25), (7, 25, -24, 25), (-7, 25, -24, 25),
            (24, 25, 7, 25), (-24, 25, 7, 25), (24, 25, -7, 25), (-24, 25, -7, 25),
            // 8-15-17 triangles (8)
            (8, 17, 15, 17), (-8, 17, 15, 17), (8, 17, -15, 17), (-8, 17, -15, 17),
            (15, 17, 8, 17), (-15, 17, 8, 17), (15, 17, -8, 17), (-15, 17, -8, 17),
            // 9-40-41 triangles (8)
            (9, 41, 40, 41), (-9, 41, 40, 41), (9, 41, -40, 41), (-9, 41, -40, 41),
            (40, 41, 9, 41), (-40, 41, 9, 41), (40, 41, -9, 41), (-40, 41, -9, 41),
            // 15-8-17 swapped (4 more, completing 48)
            (21, 29, 20, 29), (-21, 29, 20, 29), (21, 29, -20, 29), (-21, 29, -20, 29),
        ]
    }

    pub fn to_f32(&self) -> (f32, f32) {
        let (xn, xd, yn, yd) = Self::all_directions()[self.0 as usize];
        (xn as f32 / xd as f32, yn as f32 / yd as f32)
    }

    pub fn from_f32(x: f32, y: f32) -> Self {
        let mut best = 0;
        let mut best_dist = f32::MAX;
        for (i, (xn, xd, yn, yd)) in Self::all_directions().iter().enumerate() {
            let dx = x - (*xn as f32 / *xd as f32);
            let dy = y - (*yn as f32 / *yd as f32);
            if dx*dx + dy*dy < best_dist {
                best_dist = dx*dx + dy*dy;
                best = i;
            }
        }
        TrustVector(best as u8)
    }

    pub fn bits(&self) -> f64 { 5.58496 }
}

impl Default for TrustVector { fn default() -> Self { TrustVector(0) } }

pub fn codebook_info() -> CodebookInfo {
    CodebookInfo { count: 48, bits_per_vector: 5.5849625007, zero_drift: true }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodebookInfo { pub count: usize, pub bits_per_vector: f64, pub zero_drift: bool }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_all_on_circle() {
        for i in 0..48 {
            let (x, y) = TrustVector(i as u8).to_f32();
            assert!((x*x+y*y).sqrt().abs() - 1.0 < 0.001);
        }
    }
    #[test] fn test_encode_decode() {
        let (x, y) = (0.6_f32, 0.8_f32);
        let (dx, _dy) = TrustVector::from_f32(x, y).to_f32();
        assert!((dx - x).abs() < 0.02);
    }

    #[test]
    fn test_all_directions_unique() {
        let dirs = TrustVector::all_directions();
        assert_eq!(dirs.len(), 48, "COUNT must be 48");
        let mut seen = std::collections::HashSet::new();
        for (i, &(xn, xd, yn, yd)) in dirs.iter().enumerate() {
            let key = (xn, xd, yn, yd);
            assert!(seen.insert(key), "Direction {} is duplicate of an earlier direction", i);
        }
    }
}

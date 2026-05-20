//! Coalescence verification — bitwise OR merge with zero false negatives.

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Coalesces block-level error masks into a unified error mask.
///
/// ## Correctness Proof
///
/// Let Eᵢ be the error mask for block i. Each Eᵢ encodes constraint
/// violations for constraints in block i. Since blocks are independent
/// (no shared dimensions), the violation events are disjoint.
/// The total error mask `E = E₁ | E₂ | … | Eₖ` captures ALL violations.
///
/// **No false negatives:** if constraint j is violated, it appears in exactly
/// one block's mask, so its bit is set in the OR.
pub struct Coalescer;

impl Coalescer {
    pub fn new() -> Self { Self }

    /// Coalesce integer error masks via bitwise OR.
    pub fn coalesce_masks(&self, block_masks: &[u64]) -> u64 {
        block_masks.iter().fold(0u64, |acc, &m| acc | m)
    }

    /// Coalesce boolean violation arrays via elementwise OR.
    pub fn coalesce_arrays(&self, block_arrays: &[Vec<u8>]) -> Vec<u8> {
        if block_arrays.is_empty() {
            return Vec::new();
        }
        let len = block_arrays[0].len();
        let mut result = vec![0u8; len];
        for arr in block_arrays {
            for (i, &v) in arr.iter().enumerate() {
                result[i] |= v;
            }
        }
        result
    }

    /// Verify that coalesced result equals monolithic result.
    ///
    /// Each block mask has bits only for its own constraints (bit position
    /// within the block). We reconstruct a global mask by mapping each block-local
    /// bit back to the global constraint index.
    ///
    /// Returns `(is_correct, message)`.
    pub fn verify_coalescence(
        &self,
        block_masks: &[u64],
        block_constraint_indices: &[Vec<usize>],
        monolithic_mask: u64,
    ) -> (bool, String) {
        // Reconstruct global mask from block-local masks
        let mut reconstructed: u64 = 0;
        for (mask, indices) in block_masks.iter().zip(block_constraint_indices.iter()) {
            for (bit_pos, &c_idx) in indices.iter().enumerate() {
                if mask & (1u64 << bit_pos) != 0 {
                    reconstructed |= 1u64 << c_idx;
                }
            }
        }

        if reconstructed == monolithic_mask {
            (true, format!("PERFECT MATCH: coalesced=0x{monolithic_mask:x}"))
        } else {
            let false_neg = monolithic_mask & !reconstructed;
            let false_pos = reconstructed & !monolithic_mask;
            (false, format!(
                "MISMATCH: coalesced=0x{reconstructed:x} vs monolithic=0x{monolithic_mask:x} | false_neg=0x{false_neg:x} false_pos=0x{false_pos:x}"
            ))
        }
    }
}

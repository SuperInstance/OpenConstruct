/// Proof certificate generation — SHA-256 hash chain
use sha2::{Sha256, Digest};
use crate::error::{FluxError, FluxResult};

#[derive(Debug, Clone)]
pub struct ProofContext {
    chain: Vec<[u8; 32]>,
    committed: bool,
    sealed: bool,
}

impl Default for ProofContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ProofContext {
    pub fn new() -> Self {
        Self {
            chain: vec![],
            committed: false,
            sealed: false,
        }
    }

    /// Hash a single value into the chain
    pub fn prove_value(&mut self, value: i32) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"FLUX-V3::prove");
        let prev = self.chain.last().copied().unwrap_or([0u8; 32]);
        hasher.update(prev);
        hasher.update(value.to_le_bytes());
        let hash: [u8; 32] = hasher.finalize().into();
        self.chain.push(hash);
        hash
    }

    /// Prove a range check: value in [lo, hi]
    pub fn prove_range(&mut self, value: i32, lo: i32, hi: i32, pass: bool) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"FLUX-V3::range");
        let prev = self.chain.last().copied().unwrap_or([0u8; 32]);
        hasher.update(prev);
        hasher.update(value.to_le_bytes());
        hasher.update(lo.to_le_bytes());
        hasher.update(hi.to_le_bytes());
        hasher.update(if pass { &[1u8] } else { &[0u8] });
        let hash: [u8; 32] = hasher.finalize().into();
        self.chain.push(hash);
        hash
    }

    /// Prove a vector mask result
    pub fn prove_vector(&mut self, mask: u8, lo: i8, hi: i8) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"FLUX-V3::vector");
        let prev = self.chain.last().copied().unwrap_or([0u8; 32]);
        hasher.update(prev);
        hasher.update(&[mask]);
        hasher.update(&[lo as u8, hi as u8]);
        let hash: [u8; 32] = hasher.finalize().into();
        self.chain.push(hash);
        hash
    }

    /// Commit current chain state
    pub fn commit(&mut self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"FLUX-V3::commit");
        for h in &self.chain {
            hasher.update(h);
        }
        self.committed = true;
        let hash: [u8; 32] = hasher.finalize().into();
        self.chain.push(hash);
        hash
    }

    /// Seal the proof — no more additions
    pub fn seal(&mut self) -> FluxResult<[u8; 32]> {
        if self.sealed {
            return Err(FluxError::ProofMismatch);
        }
        let mut hasher = Sha256::new();
        hasher.update(b"FLUX-V3::seal");
        for h in &self.chain {
            hasher.update(h);
        }
        let hash: [u8; 32] = hasher.finalize().into();
        self.chain.push(hash);
        self.sealed = true;
        Ok(hash)
    }

    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    pub fn chain_len(&self) -> usize {
        self.chain.len()
    }

    pub fn root_hash(&self) -> Option<[u8; 32]> {
        self.chain.last().copied()
    }

    /// Verify against expected root
    pub fn verify(&self, expected: &[u8; 32]) -> bool {
        match self.chain.last() {
            Some(h) => h == expected,
            None => false,
        }
    }

    pub fn reset(&mut self) {
        self.chain.clear();
        self.committed = false;
        self.sealed = false;
    }
}

/// A complete proof certificate
#[derive(Debug, Clone)]
pub struct ProofCertificate {
    pub root_hash: [u8; 32],
    pub chain_length: usize,
    pub cycle_count: u64,
}

impl ProofCertificate {
    pub fn from_context(ctx: &ProofContext, cycles: u64) -> Option<Self> {
        ctx.root_hash().map(|root| Self {
            root_hash: root,
            chain_length: ctx.chain_len(),
            cycle_count: cycles,
        })
    }
}

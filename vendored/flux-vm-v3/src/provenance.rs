/// SNAP-style audit trail / provenance
use sha2::{Sha256, Digest};

pub const PROVENANCE_LOG_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub struct ProvenanceEntry {
    pub cycle: u64,
    pub tag: String,
    pub data_hash: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct ProvenanceLog {
    entries: Vec<ProvenanceEntry>,
    ring_offset: usize,
    total_recorded: u64,
}

impl Default for ProvenanceLog {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(PROVENANCE_LOG_SIZE),
            ring_offset: 0,
            total_recorded: 0,
        }
    }

    /// Record an event with tag and data
    pub fn record(&mut self, cycle: u64, tag: &str, data: &[u8]) {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let data_hash: [u8; 32] = hasher.finalize().into();

        let entry = ProvenanceEntry {
            cycle,
            tag: tag.to_string(),
            data_hash,
        };

        if self.entries.len() < PROVENANCE_LOG_SIZE {
            self.entries.push(entry);
        } else {
            // Ring buffer behavior
            self.entries[self.ring_offset] = entry;
            self.ring_offset = (self.ring_offset + 1) % PROVENANCE_LOG_SIZE;
        }
        self.total_recorded += 1;
    }

    /// Query entries by tag
    pub fn query(&self, tag: &str) -> Vec<&ProvenanceEntry> {
        self.entries.iter().filter(|e| e.tag == tag).collect()
    }

    /// Hash the entire log
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"FLUX-V3::provenance");
        for entry in &self.entries {
            hasher.update(entry.cycle.to_le_bytes());
            hasher.update(entry.data_hash);
        }
        hasher.finalize().into()
    }

    /// Verify integrity of a specific entry
    pub fn verify(&self, idx: usize, expected_hash: &[u8; 32]) -> bool {
        self.entries
            .get(idx)
            .map(|e| &e.data_hash == expected_hash)
            .unwrap_or(false)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn total_recorded(&self) -> u64 {
        self.total_recorded
    }

    pub fn reset(&mut self) {
        self.entries.clear();
        self.ring_offset = 0;
        self.total_recorded = 0;
    }
}

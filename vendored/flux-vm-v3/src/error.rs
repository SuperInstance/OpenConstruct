/// FLUX-C v3 error types
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum FluxError {
    StackUnderflow,
    StackOverflow,
    InvalidOpCode(u8),
    DivisionByZero,
    CycleLimitExceeded,
    MemoryExceeded,
    InvalidRegister(u8),
    InvalidJump(usize),
    NoConstraint,
    ProofMismatch,
    StreamNotOpen,
    StreamAlreadyOpen,
    ProvenanceError(String),
    NotImplemented(&'static str),
}

impl fmt::Display for FluxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StackUnderflow => write!(f, "stack underflow"),
            Self::StackOverflow => write!(f, "stack overflow (max 256)"),
            Self::InvalidOpCode(c) => write!(f, "invalid opcode: 0x{c:02x}"),
            Self::DivisionByZero => write!(f, "division by zero"),
            Self::CycleLimitExceeded => write!(f, "cycle limit exceeded"),
            Self::MemoryExceeded => write!(f, "memory exceeded"),
            Self::InvalidRegister(r) => write!(f, "invalid register: {r}"),
            Self::InvalidJump(pc) => write!(f, "invalid jump target: {pc}"),
            Self::NoConstraint => write!(f, "no constraint loaded"),
            Self::ProofMismatch => write!(f, "proof certificate mismatch"),
            Self::StreamNotOpen => write!(f, "stream not open"),
            Self::StreamAlreadyOpen => write!(f, "stream already open"),
            Self::ProvenanceError(s) => write!(f, "provenance: {s}"),
            Self::NotImplemented(s) => write!(f, "not implemented: {s}"),
        }
    }
}

impl std::error::Error for FluxError {}

pub type FluxResult<T> = Result<T, FluxError>;

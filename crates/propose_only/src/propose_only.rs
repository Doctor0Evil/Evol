#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// What a propose-only terminal is allowed to emit.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProposedUpgrade {
    pub session_id: String,
    pub raw_aln: String,          // SESSION/INTENT/SAFETY/EVIDENCE/TERMINAL
    pub parsed_intent: String,    // human-checked label
}

/// Policy errors for malformed or out-of-scope proposals.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PolicyError {
    InvalidGrammar,
    UnsafeTerminalClass,
    MissingEvidenceBundle,
}

/// Terminals that implement this trait *cannot* actuate.
pub trait ProposeOnlyTerminal {
    fn submit_proposal_aln(&self, raw_aln: &str) -> Result<ProposedUpgrade, PolicyError>;
}

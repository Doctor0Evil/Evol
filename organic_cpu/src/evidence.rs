//! Organic CPU biophysical-proof artifacts and emission for AugDoctor.
//!
//! This module defines:
//! - `BiophysicalProofArtifact`: a machine-readable, non-financial,
//!   host-sealed attestation object per accepted runtime event.
//! - `ProofEmitter`: a trait for host-local emitters that append
//!   these artifacts to an immutable audit stream (e.g. JSONL file).
//!
//! Design guarantees:
//! - No transfer / stake / financial semantics.
//! - No direct exposure of raw BRAIN/BLOOD/OXYGEN/NANO balances.
//! - No EEG payloads, souls, or consciousness fields.
//! - Artifacts are derived only from hashes, bands, eco metrics,
//!   and consent/provenance hashes already enforced in the runtime.
//!
//! Intended integration:
//! - `BiophysicalRuntime::execute_event` or the host node calls
//!   `emit_proof_artifact` ONLY after a state transition has been
//!   fully validated and committed.
//! - External AI-chats (Perplexity, Gemini, Copilot, Grok, etc.)
//!   may read artifacts (or relayed summaries) but can never write
//!   or modify them.

use std::fs::{OpenOptions};
use std::io::{Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Minimal ALN/Bostrom host identity view for evidence.
/// This carries no roles or privileges: it is just an anchor.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HostIdentity {
    /// Canonical ALN/Bostrom-style DID or address.
    /// Example: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".
    pub did: String,
    /// Host shard or locality label, e.g. "phx-main".
    pub shard: String,
}

/// Lorentz-consistent timestamp snapshot (copied from runtime types).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LorentzTimestamp {
    /// Proper time component (nanoseconds or similar monotone unit).
    pub proper_time_ns: i128,
    /// Frame offset (e.g. picoseconds or implementation-specific).
    pub frame_offset_ps: i64,
}

/// Lifeforce safety band at the time of the event.
/// This is a coarse, non-sensitive label, not raw physiology.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LifeforceBand {
    Safe,
    SoftWarn,
    HardStop,
}

/// Eco band at the time of the event.
/// This encodes environmental impact in abstract bands only.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EcoBand {
    Low,
    Medium,
    High,
}

/// Pain corridor classification at the time of the event.
/// This is derived from ethics/EEG signals, not raw traces.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PainBand {
    None,
    Mild,
    Strong,
    HardStopEquivalent,
}

/// High-level event domain at the runtime boundary.
/// Purely descriptive: no mechanics or token semantics here.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventDomain {
    WaveLoad,
    SmartAutonomy,
    EvolutionUpgrade,
    /// Micro defensive evolution (teeth/claw, etc.), still bounded
    /// by lifeforce/eco/SCALE and consent invariants.
    DefensiveMicro,
    /// Other domains can be added without changing core invariants.
    Other(String),
}

/// Metabolic consent mode snapshot when the event was applied.
/// This is a runtime configuration label, not a token.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetabolicConsentMode {
    ManualOnly,
    AutoMicro,
    AutoMicroPlusWave,
    Unknown,
}

/// Minimal view of the biophysical event that was accepted.
/// This is intentionally small and non-sensitive.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventSummary {
    /// Host-generated event identifier (could be a hash).
    pub event_id: String,
    /// High-level domain label.
    pub domain: EventDomain,
    /// Free-form, non-sensitive reason/label, if any.
    pub reason: Option<String>,
}

/// A single, host-local, organic_cpu biophysical proof artifact.
///
/// This object is designed for machine readability, discovery,
/// and biophysical proof, without exposing raw internal state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BiophysicalProofArtifact {
    /// Host identity that owned and executed the event.
    pub host: HostIdentity,

    /// Lorentz-consistent timestamp from the host runtime.
    pub lorentz_ts: LorentzTimestamp,

    /// Wall-clock UTC milliseconds since Unix epoch (for humans/tools).
    pub utc_ms: i64,

    /// Origin plane helps distinguish organic_cpu from other planes
    /// (e.g. "bioscale-metabolic", "bci-hci-eeg", "software-only").
    pub origin_plane: String,

    /// Event summary and domain.
    pub event: EventSummary,

    /// Consensus hash of the pre-state (BioTokenState::consensus_attest).
    pub pre_state_hash_hex: String,

    /// Consensus hash of the post-state.
    pub post_state_hash_hex: String,

    /// Sequence number of the consensus frame (monotone per host).
    pub consensus_seqno: u64,

    /// Lifeforce band classification (e.g. Safe/SoftWarn/HardStop).
    pub lifeforce_band: LifeforceBand,

    /// Whether lifeforce guards accepted the event.
    pub lifeforce_ok: bool,

    /// Eco band classification (Low/Medium/High).
    pub eco_band: EcoBand,

    /// Approximate ecocost used for banding only (no FLOP counts).
    pub eco_cost_hint: f64,

    /// Pain corridor band at the time of application, if any.
    pub pain_band: Option[PainBand],

    /// Metabolic consent mode snapshot.
    pub metabolic_mode: MetabolicConsentMode,

    /// Hash of the consent shard / proof that authorized this event.
    /// This is a digest only, NOT the full consent content.
    pub consent_proof_hash_hex: Option<String>,

    /// Optional hash of a mutation-provenance shard (e.g. for
    /// evolution or defensive micro-evolution events).
    pub provenance_hash_hex: Option<String>,

    /// Optional host-local civic/audit correlation ID.
    pub civic_audit_id: Option<String>,
}

/// Result type for proof emission.
pub type ProofResult<T> = Result<T, ProofError>;

/// Errors that can occur while emitting artifacts.
#[derive(thiserror::Error, Debug)]
pub enum ProofError {
    #[error("I/O error while writing proof artifact: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error while encoding proof artifact: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Trait implemented by host-local emitters that know how to persist
/// biophysical proof artifacts in an append-only way.
pub trait ProofEmitter: Send + Sync + 'static {
    /// Append a single artifact to the emitter's backing store.
    ///
    /// Implementations MUST be:
    /// - append-only (no rewrites),
    /// - host-local (no remote writes),
    /// - non-financial (no token semantics).
    fn emit_proof_artifact(&self, artifact: &BiophysicalProofArtifact) -> ProofResult<()>;
}

/// JSONL file-based proof emitter.
/// Each artifact is written as one JSON line, append-only.
#[derive(Clone, Debug)]
pub struct JsonlFileProofEmitter {
    path: PathBuf,
}

impl JsonlFileProofEmitter {
    /// Create a new emitter targeting a JSONL file.
    /// The file is created if it does not exist.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    fn ensure_parent_dirs(&self) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}

impl ProofEmitter for JsonlFileProofEmitter {
    fn emit_proof_artifact(&self, artifact: &BiophysicalProofArtifact) -> ProofResult<()> {
        self.ensure_parent_dirs()?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let line = serde_json::to_string(artifact)?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }
}

/// Helper: get current UTC ms for human/forensic use.
/// Runtime Lorentz timestamp still comes from the runtime itself.
pub fn current_utc_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

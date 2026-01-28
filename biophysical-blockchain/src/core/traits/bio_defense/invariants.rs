//! Invariants and types for the Bio-Defense TRAIT, enforcing doctrine-level rules.
//!
//! - Non-financial, host-local, per-domain defensive evolution regulator.
//! - Allocates existing EVOLVE capacity, never mints new capacity.
//! - Bound by BRAIN, BLOOD, OXYGEN, NANO, SCALE, eco ceilings, and lifeforce bands.
//! - Strictly consent-gated and reversible via DemonstratedConsentShard / MetabolicConsent.
//! - May be damped by KarmaAura, never amplified (decay ∈ [0.0, 1.0]).
//! - Cannot encode or touch souls/consciousness; it only shapes mutation *paths*.

use crate::types::{
    HostId,
    EvolutionDomainId,
    LifeforceBand,
    LifeforceBandSeries,
    EcoBandProfile,
    SystemAdjustment,
};
use crate::innerledger::InnerLedger;
use crate::lifeforce::LifeforceError;
use crate::mutation::LedgerMutator; // sealed trait
use crate::access::AccessError;

/// Defensive evolution domain (trait) definition: immutable, host-agnostic.
#[derive(Clone, Debug)]
pub struct DefensiveEvolutionDomain {
    pub id: EvolutionDomainId,
    /// Human-readable label, e.g. "ThermoregulationControl" / "ColdResilience".
    pub name: String,
    /// Minimum normalized lifeforce (0.0–1.0) that must remain after a step.
    pub lifeforce_floor: f32,
    /// Eco ceiling (normalized or FLOPs-derived scalar) this domain may consume per epoch.
    pub eco_ceiling_per_epoch: f64,
    /// Maximum SCALE budget this domain may consume per epoch (0.0–1.0 of host SCALE).
    pub scale_limit_per_epoch: f32,
    /// Whether this domain may be temporarily denied for safety, but never permanently banned.
    pub allow_temporary_denial_only: bool,
}

/// Shard of host-authored, revocable consent for a specific defensive domain.
#[derive(Clone, Debug)]
pub struct DemonstratedConsentShard {
    pub host_id: HostId,
    pub domain_id: EvolutionDomainId,
    /// Unix millis when consent was granted.
    pub granted_at_ms_utc: i64,
    /// Whether the host explicitly marked this consent as revocable (it MUST be true for TRAIT).
    pub is_revocable: bool,
    /// Opaque zk signature or proof bytes; verified by outer consent-governance crate.
    pub zk_proof: Vec<u8>,
}

/// Live runtime view of a Bio-Defense TRAIT on a host.
#[derive(Clone, Debug)]
pub struct ActiveBioDefenseTrait {
    pub shard: DemonstratedConsentShard,
    pub domain: DefensiveEvolutionDomain,
    /// Whether the trait is currently allowed to apply micro-steps.
    pub is_active: bool,
    /// Last time a defensive micro-step was successfully applied (Unix millis).
    pub last_activation_ms_utc: Option<i64>,
}

/// Domain-local SCALE / eco consumption window for audit and throttling.
#[derive(Clone, Debug, Default)]
pub struct TraitEpochUsage {
    pub epoch_id: String,      // e.g. "2026-01-28T08"
    pub scale_used: f32,       // fraction of host SCALE consumed this epoch
    pub eco_cost_used: f64,    // FLOPs/nJ consumed by this domain this epoch
}

/// Errors specific to Bio-Defense TRAIT doctrine and activation.
#[derive(Debug, thiserror::Error)]
pub enum BioDefenseDoctrineError {
    #[error("TRAIT activation violates BRAIN / SCALE / eco capacity limits.")]
    CapacityExceeded,

    #[error("Attempt to activate TRAIT during HardStop lifeforce band.")]
    LifeforceHardStop,

    #[error("TRAIT activation would deplete lifeforce below the domain floor.")]
    LifeforceBelowFloor,

    #[error("KarmaAura attempted to amplify risk (decay multiplier > 1.0).")]
    KarmaAmplificationForbidden,

    #[error("Governance policy attempts a permanent ban; only temporary denial is allowed.")]
    StructuralDomainBanAttempted,

    #[error("No valid, revocable MetabolicConsent / DemonstratedConsentShard for this domain.")]
    NoValidConsent,

    #[error("Environment unsafe for SMART automation; manual evolution required.")]
    EnvironmentUnsafe,

    #[error("Inner-ledger access denied: {0}")]
    AccessDenied(#[from] AccessError),

    #[error("Lifeforce-guarded adjustment failed: {0}")]
    LifeforceViolation(#[from] LifeforceError),
}

/// Auditor-facing assertion: a defensive domain policy must not encode structural bans.
pub fn assert_policy_respects_doctrine(
    domain: &DefensiveEvolutionDomain,
    declares_structural_bans: bool,
) -> Result<(), BioDefenseDoctrineError> {
    if declares_structural_bans {
        return Err(BioDefenseDoctrineError::StructuralDomainBanAttempted);
    }
    if !domain.allow_temporary_denial_only {
        return Err(BioDefenseDoctrineError::StructuralDomainBanAttempted);
    }
    Ok(())
}

/// Clamp Karma / aura decay multiplier into [0.0, 1.0], rejecting amplification.
pub fn enforce_decay_multiplier_bounds(raw: f32) -> Result<f32, BioDefenseDoctrineError> {
    if raw > 1.0 {
        return Err(BioDefenseDoctrineError::KarmaAmplificationForbidden);
    }
    let clamped = if raw < 0.0 { 0.0 } else { raw };
    Ok(clamped)
}

/// Check lifeforce series and domain floor before applying any defensive step.
fn assert_lifeforce_allows_trait(
    lifeforce: &LifeforceBandSeries,
    domain: &DefensiveEvolutionDomain,
) -> Result<(), BioDefenseDoctrineError> {
    // Most recent sample is the operational state.
    let Some(last) = lifeforce.samples.last() else {
        return Err(BioDefenseDoctrineError::LifeforceBelowFloor);
    };

    // HardStop: absolutely no mutation or TRAIT micro-step allowed.
    if last.band == LifeforceBand::HardStop {
        return Err(BioDefenseDoctrineError::LifeforceHardStop);
    }

    // Ensure lifeforce will not be driven below the domain floor.
    if last.lifeforcel < domain.lifeforce_floor {
        return Err(BioDefenseDoctrineError::LifeforceBelowFloor);
    }

    Ok(())
}

/// Check that trait usage stays under SCALE and eco ceilings for the current epoch.
pub fn assert_epoch_usage_within_bounds(
    domain: &DefensiveEvolutionDomain,
    usage: &TraitEpochUsage,
) -> Result<(), BioDefenseDoctrineError> {
    if usage.scale_used > domain.scale_limit_per_epoch {
        return Err(BioDefenseDoctrineError::CapacityExceeded);
    }
    if usage.eco_cost_used > domain.eco_ceiling_per_epoch {
        return Err(BioDefenseDoctrineError::CapacityExceeded);
    }
    Ok(())
}

/// Minimal trait for querying host-local MetabolicConsent scope.
pub trait MetabolicConsentScope {
    /// Returns true if this host has active, revocable consent for the given domain.
    fn is_domain_allowed(&self, host: &HostId, domain_id: &EvolutionDomainId) -> bool;
}

/// Minimal trait for runtime SMART state / environment safety.
pub trait SmartRuntimeView {
    /// Returns true if SMART is permitted to apply automatic micro-steps right now.
    fn is_environment_safe_for_automation(&self) -> bool;
}

/// High-level orchestrator: try to activate or maintain an ActiveBioDefenseTrait.
///
/// This does *not* expose inner-ledger mechanics; it simply:
/// - checks consent,
/// - checks doctrine-level invariants,
/// - returns an updated ActiveBioDefenseTrait that higher layers may use to mint
///   a tiny SystemAdjustment and call InnerLedger::systemapply().
pub fn try_activate_bio_defense_trait(
    host_id: HostId,
    domain: DefensiveEvolutionDomain,
    lifeforce: &LifeforceBandSeries,
    eco_profile: &EcoBandProfile,
    epoch_usage: &TraitEpochUsage,
    metabolic_consent: &dyn MetabolicConsentScope,
    smart_view: &dyn SmartRuntimeView,
    karma_decay_raw: f32,
    now_ms_utc: i64,
) -> Result<ActiveBioDefenseTrait, BioDefenseDoctrineError> {
    // 1. Consent: domain must be allowed under current MetabolicConsent.
    if !metabolic_consent.is_domain_allowed(&host_id, &domain.id) {
        return Err(BioDefenseDoctrineError::NoValidConsent);
    }

    // 2. Lifeforce gates: HardStop and domain lifeforce floor.
    assert_lifeforce_allows_trait(lifeforce, &domain)?;

    // 3. Epoch SCALE / eco ceilings for this domain.
    assert_epoch_usage_within_bounds(&domain, epoch_usage)?;

    // 4. KarmaAura: decay must be ∈ [0.0, 1.0] (damping only).
    let _decay = enforce_decay_multiplier_bounds(karma_decay_raw)?;

    // 5. SMART environment safety: if unsafe, require manual evolution instead.
    if !smart_view.is_environment_safe_for_automation() {
        return Err(BioDefenseDoctrineError::EnvironmentUnsafe);
    }

    // 6. Construct a fresh consent shard (actual zk proof comes from outer crate).
    let shard = DemonstratedConsentShard {
        host_id,
        domain_id: domain.id.clone(),
        granted_at_ms_utc: now_ms_utc,
        is_revocable: true,
        zk_proof: Vec::new(), // placeholder; real proof is injected by consent-governance.
    };

    Ok(ActiveBioDefenseTrait {
        shard,
        domain,
        is_active: true,
        last_activation_ms_utc: Some(now_ms_utc),
    })
}

/// Canonical helper for applying a *single* defensive micro-step via the sealed inner ledger.
///
/// - Takes a tiny SystemAdjustment precomputed by higher layers (no BLOOD/OXYGEN spend, small deltas).
/// - Delegates to InnerLedger::systemapply (via LedgerMutator), which already enforces
///   BRAIN/BLOOD/OXYGEN/NANO/SMART invariants, lifeforce, eco, and identity gating.[file:6][file:5]
///
/// This function exists so auditors can see clearly: Bio-Defense TRAIT never bypasses
/// inner-ledger invariants; it only shapes the *reason* and magnitude of SystemAdjustment.
pub fn apply_defensive_micro_step(
    ledger: &mut InnerLedger,
    id_header: crate::types::IdentityHeader,
    required_k: f32,
    adj: SystemAdjustment,
    lifeforce_series: LifeforceBandSeries,
    eco_profile: EcoBandProfile,
    wave_curve: crate::types::SafetyCurveWave,
    timestamputc: &str,
) -> Result<crate::consensus::LedgerEvent, BioDefenseDoctrineError> {
    let event = LedgerMutator::systemapplyguarded(
        ledger,
        id_header,
        required_k,
        adj,
        timestamputc,
        lifeforce_series,
        eco_profile,
        wave_curve,
    )?;
    Ok(event)
}

//! Invariants and types for the Bio-Defense TRAIT, enforcing the core doctrine.

use crate::types::{HostId, EvolutionDomainId};
use super::domain::DefensiveEvolutionDomain;

/// An enumeration of doctrinal violations specific to Bio-Defense TRAIT operations.
#[derive(Debug, thiserror::Error)]
pub enum BioDefenseDoctrineError {
    #[error("TRAIT activation violates BRAIN's sovereign capacity limits.")]
    BRAINInsufficientCapacity,

    #[error("Attempt to activate TRAIT failed due to unsafe real-time physiological conditions.")]
    EnvironmentUnsafe,

    #[error("TRAIT activation would deplete lifeforce below the domain's mandatory floor.")]
    LifeforceBelowFloor,

    #[error("KarmaAura attempted to amplify evolution risk by producing a decay multiplier > 1.0.")]
    KarmaAmplificationForbidden,

    #[error("Attempt to permanently ban a defensive evolution domain; only temporary denial is allowed.")]
    StructuralDomainBanAttempted,
}

/// A compile-time style helper to ensure a governance policy for a defensive domain does not violate core doctrine.
pub fn assert_policy_respects_doctrine(
    domain: &DefensiveEvolutionDomain,
    declares_structural_bans: bool,
) -> Result<(), BioDefenseDoctrineError> {
    if declares_structural_bans {
        return Err(BioDefenseDoctrineError::StructuralDomainBanAttempted);
    }
    // Additional checks can be added here as the doctrine evolves.
    Ok(())
}

/// Runtime guard for BRAIN-level validation of a proposed evolution step for a Bio-Defense TRAIT.
pub fn validate_evolution_step_against_brain_capacity(
    evolution_delta: f32,
    brain_capacity: f32,
    lifeforce_consumption: f32,
    lifeforce_floor: f32,
) -> Result<(), BioDefenseDoctrineError> {
    if evolution_delta > brain_capacity {
        return Err(BioDefenseDoctrineError::BRAINInsufficientCapacity);
    }
    if lifeforce_consumption > lifeforce_floor {
        return Err(BioDefenseDoctrineError::LifeforceBelowFloor);
    }
    Ok(())
}

/// Runtime guard for Karma/Aura decay multipliers, enforcing the dampening-only rule.
pub fn enforce_decay_multiplier_bounds(raw: f32) -> Result<f32, BioDefenseDoctrineError> {
    if raw > 1.0 {
        return Err(BioDefenseDoctrineError::KarmaAmplificationForbidden);
    }
    Ok(raw.clamp(0.0, 1.0))
}

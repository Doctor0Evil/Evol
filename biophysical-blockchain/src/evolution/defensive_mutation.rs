use crate::doctrine::invariants_evolution_freedom::{validate_automated_evolution_path, EvolutionFreedomError};
use crate::evolution::karma_decay::{BiophysicalAura, EvolutionDomain, apply_aura_shaped_adjustment, KarmaClass};
use crate::lifeforce::{LifeforceBandSeries, LifeforceMutator, LifeforceError};
use crate::types::{BioTokenState, HostEnvelope, SystemAdjustment, EvolutionDomainId};

#[derive(Clone, Debug)]
pub struct DefensiveMutationConfig {
    pub domain_id: EvolutionDomainId,
    pub has_consent: bool,
    pub provenance_verified: bool,
    pub max_daily_fraction: f32,  // e.g., 0.3 for teeth/claws
}

pub fn apply_defensive_mutation<M>(
    state: &mut BioTokenState,
    env: &HostEnvelope,
    adj: SystemAdjustment,
    aura: &BiophysicalAura,
    config: &DefensiveMutationConfig,
    lifeforce_mut: &M,
    lifeforce_bands: &LifeforceBandSeries,
) -> Result<(), LifeforceError>
where
    M: LifeforceMutator,
{
    if lifeforce_bands.is_hard_stop() {
        return Err(LifeforceError::HardStopBand);
    }
    if !config.has_consent || !config.provenance_verified {
        return Err(LifeforceError::MissingConsentOrProvenance);
    }
    let domain = EvolutionDomain::TeethClawsDefense;
    if let Err(e) = validate_automated_evolution_path(&env.evolution_config, &config.domain_id, false) {
        return Err(LifeforceError::DoctrinalViolation(e));
    }
    if classify_karma(aura.karma_score) != KarmaClass::Benevolent {
        return Err(LifeforceError::InsufficientKarmaForDefense);
    }
    let scaled_adj = SystemAdjustment {
        deltabrain: adj.deltabrain * config.max_daily_fraction as f64,
        ..adj
    };
    apply_aura_shaped_adjustment(state, env, scaled_adj, aura, domain, lifeforce_mut)
}

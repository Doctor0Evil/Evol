//! Neuromorphic reflex/sense evolution eligibility gated by SMART.
//!
//! This module defines three neuromorphic evolution domains and a
//! `ReflexSenseEligibility` helper that your Evolution Eligibility Filter
//! can call before allowing any EvolutionUpgrade for these domains.
//!
//! Core properties:
//! - Software-only, non-somatic neuromorphic evolution paths.
//! - SMART-governed autonomy: auto-micro evolution only when SMART is high
//!   enough and within BRAIN- and lifeforce-safe corridors.
//! - DECAY-compatible: returns a comfort/safety weight you can feed into
//!   existing DECAY / safedecaymultiplier logic.
//! - Non-financial, per-host, no transfer/bridge semantics.

use crate::types::BioTokenState;
use crate::types::HostEnvelope;
use crate::lifeforce::LifeforceError;

/// Neuromorphic micro-evolution domains for heightened senses/reflexes.
///
/// These are explicitly *software-only* and non-somatic: they adjust decoding,
/// routing, and scheduling, not tissue or morphology.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NeuromorphDomain {
    /// Reflex safety controllers (auto-lockdown, safe-mode, overload handling).
    ReflexSafety,          // "neuromorph-reflex-micro"
    /// Sensory clarity (denoising, better decoding of EMG/EEG/motion).
    SensoryClarity,        // "neuromorph-sense-micro"
    /// Attention / load-balancing (WAVE/NANO scheduling across senses).
    AttentionRouting,      // "neuromorph-attention-micro"
}

/// High-level host comfort and load context used for neuromorphic decisions.
///
/// This is designed to be populated from LifeforceBandSeries + NeuralRope
/// summaries (fatigue, comfort bands, overload history).
#[derive(Clone, Debug)]
pub struct NeuromorphContext {
    /// 0.0–1.0, where 0.0 = fully comfortable, 1.0 = severe discomfort or pain.
    pub discomfort_index: f32,
    /// 0.0–1.0, fraction of recent epochs that were overload/near-HardStop.
    pub overload_ratio: f32,
    /// 0.0–1.0, how noisy / unstable the host interface has been recently.
    /// (e.g., EMG/EEG/motion decoding errors or unstable control).
    pub interface_instability: f32,
    /// 0.0–1.0, fraction of SCALE budget already consumed by
    /// *neuromorphic* domains in this epoch (advisory – not a hard cap).
    pub neuromorph_scale_usage: f32,
}

/// Result of an eligibility check for a proposed neuromorphic evolution step.
#[derive(Clone, Debug)]
pub struct ReflexSenseEligibility {
    /// True if an EvolutionUpgrade in this domain is allowed to be proposed.
    pub allowed: bool,
    /// If allowed, this is a 0.0–1.0 weight that can be fed into DECAY
    /// or into your safedecaymultiplier as an extra scaling factor.
    /// 1.0 = fully trusted micro-step; 0.0 = no step.
    pub decay_weight: f32,
    /// Whether SMART-governed *automatic* micro-evolution is permitted.
    /// If false, the runtime should require explicit host consent even
    /// for tiny neuromorphic changes in this domain.
    pub auto_allowed: bool,
    /// Optional human-readable reason or diagnostic tag.
    pub reason: &'static str,
}

impl ReflexSenseEligibility {
    pub fn denied(reason: &'static str) -> Self {
        Self {
            allowed: false,
            decay_weight: 0.0,
            auto_allowed: false,
            reason,
        }
    }

    pub fn allowed(decay_weight: f32, auto_allowed: bool, reason: &'static str) -> Self {
        Self {
            allowed: true,
            decay_weight,
            auto_allowed,
            reason,
        }
    }
}

/// SMART-governed neuromorphic eligibility function.
///
/// This function is *pure policy* over current lifeforce state, SMART, and
/// neuromorphic context. It does not mutate any state. Call it from your
/// Evolution Eligibility Filter before constructing an EvolutionUpgrade.
///
/// Semantics:
/// - Enforces basic lifeforce floors via HostEnvelope.
/// - Uses SMART as automation allowance: auto-micro evolution is only
///   permitted when SMART is above a host-defined threshold and still
///   well below its maximum, and BRAIN/BLOOD/OXYGEN are safely above floors.
/// - Dampens decay_weight when discomfort_index, overload_ratio, or
///   interface_instability are high.
/// - For very high discomfort or overload, returns `allowed = false`.
pub fn check_neuromorph_eligibility(
    domain: NeuromorphDomain,
    state: &BioTokenState,
    env: &HostEnvelope,
    ctx: &NeuromorphContext,
) -> Result<ReflexSenseEligibility, LifeforceError> {
    // 1. Hard lifeforce floors from HostEnvelope.
    if state.brain < env.brainmin {
        return Ok(ReflexSenseEligibility::denied("brain_below_min"));
    }
    if state.blood <= env.bloodmin {
        return Err(LifeforceError::BloodDepletion);
    }
    if state.oxygen <= env.oxygenmin {
        return Err(LifeforceError::OxygenDepletion);
    }

    // 2. SMART bounds (automation allowance) from HostEnvelope.
    //
    // SMART must remain under both smartmax and BRAIN (doctrine).
    if state.smart > env.smartmax || state.smart > state.brain {
        return Err(LifeforceError::SmartOverMax);
    }

    // Treat this as an "autonomy gating" SMART threshold: below this,
    // neuromorphic evolution may still be allowed, but not automatic.
    let smart_frac_of_brain = if state.brain > 0.0 {
        (state.smart / state.brain).max(0.0)
    } else {
        0.0
    };

    // Host-tunable thresholds (could later be moved to a config shard).
    let smart_auto_threshold: f64 = 0.10; // 10% of BRAIN for auto-micro
    let smart_hard_floor: f64 = 0.02;     // 2% of BRAIN for any neuromorph step

    if smart_frac_of_brain < smart_hard_floor {
        // SMART too low: host wants near-manual mode.
        return Ok(ReflexSenseEligibility::denied("smart_too_low_for_neuromorph"));
    }

    let auto_allowed = smart_frac_of_brain >= smart_auto_threshold;

    // 3. Discomfort and overload rules.
    //
    // If discomfort_index or overload_ratio are very high, we deny
    // new neuromorphic mutations and let existing controllers handle
    // protection with current capabilities only.
    let discomfort = ctx.discomfort_index.clamp(0.0, 1.0);
    let overload = ctx.overload_ratio.clamp(0.0, 1.0);
    let instability = ctx.interface_instability.clamp(0.0, 1.0);
    let scale_use = ctx.neuromorph_scale_usage.clamp(0.0, 1.0);

    // Hard stop thresholds (host-tunable):
    if discomfort > 0.85 {
        return Ok(ReflexSenseEligibility::denied("discomfort_hard_stop"));
    }
    if overload > 0.75 {
        return Ok(ReflexSenseEligibility::denied("overload_hard_stop"));
    }

    // 4. Compute a base decay_weight from comfort and stability.
    //
    // Start from 1.0 and downscale with discomfort, overload, instability,
    // and current neuromorphic SCALE usage (soft cap).
    let mut weight: f32 = 1.0;

    // Each factor reduces weight; coefficients can be tuned.
    weight -= 0.6 * discomfort;   // pain/discomfort dominates
    weight -= 0.4 * overload;     // recent overload important
    weight -= 0.3 * instability;  // noisy interface => smaller changes
    weight -= 0.3 * scale_use;    // if neuromorphic already used a lot of SCALE, slow down

    // Domain-specific shaping.
    match domain {
        NeuromorphDomain::ReflexSafety => {
            // Reflex safety can still be useful under mild discomfort,
            // but we must avoid over-reactive behavior. Clamp minimum higher.
            weight = weight.max(0.25);
        }
        NeuromorphDomain::SensoryClarity => {
            // Sensory clarity is often low risk; allow a bit more even under
            // moderate discomfort, but never exceed 0.9.
            weight = (weight + 0.1).min(0.9).max(0.2);
        }
        NeuromorphDomain::AttentionRouting => {
            // Attention routing can destabilize if overused; be conservative.
            weight = weight.min(0.7).max(0.2);
        }
    }

    if weight <= 0.0 {
        return Ok(ReflexSenseEligibility::denied("decay_weight_zero"));
    }

    // 5. Final clamp.
    let decay_weight = weight.clamp(0.0, 1.0);

    // 6. Reason tags per domain.
    let reason = match domain {
        NeuromorphDomain::ReflexSafety => {
            if auto_allowed {
                "allowed_reflex_safety_auto_micro"
            } else {
                "allowed_reflex_safety_manual_only"
            }
        }
        NeuromorphDomain::SensoryClarity => {
            if auto_allowed {
                "allowed_sensory_clarity_auto_micro"
            } else {
                "allowed_sensory_clarity_manual_only"
            }
        }
        NeuromorphDomain::AttentionRouting => {
            if auto_allowed {
                "allowed_attention_routing_auto_micro"
            } else {
                "allowed_attention_routing_manual_only"
            }
        }
    };

    Ok(ReflexSenseEligibility::allowed(decay_weight, auto_allowed, reason))
}

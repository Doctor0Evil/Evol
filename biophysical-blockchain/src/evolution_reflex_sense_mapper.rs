//! Mapping layer for neuromorphic reflex/sense evolution.
//!
//! - Maps domain strings + RuntimeEventKind into `NeuromorphDomain`.
//! - Builds `NeuromorphContext` from Lifeforce bands + NeuralRope metrics.
//!
//! Intended to sit between your host daemon / boundary services and
//! `check_neuromorph_eligibility` in `evolution_reflex_sense.rs`.

use crate::evolution_reflex_sense::{NeuromorphDomain, NeuromorphContext};
use crate::types::BioTokenState;
use crate::lifeforce_bands::{LifeforceBand, LifeforceBandSeries};
use crate::neuralrope_metrics::NeuralInterfaceSnapshot;
use crate::runtime::RuntimeEventKind;

/// Lightweight abstraction of your existing lifeforce bands crate.
/// Example enums (you likely already have these in LifeforceBandSeries).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComfortBand {
    ComfortSafe,
    DiscomfortSoftWarn,
    DiscomfortHardStop,
}

/// Map a domain string (e.g. ALN / template label) into a NeuromorphDomain.
///
/// This is intentionally small and explicit; you can extend with
/// more aliases as you add templates.
pub fn map_domain_str(domain: &str) -> Option<NeuromorphDomain> {
    match domain {
        // Reflex safety controllers.
        "neuromorph-reflex-micro"
        | "evo.reflex.safety"
        | "evolution.neuromorph.reflex" => Some(NeuromorphDomain::ReflexSafety),

        // Sensory clarity (denoising / decoding).
        "neuromorph-sense-micro"
        | "evo.sense.clarity"
        | "evolution.neuromorph.sense" => Some(NeuromorphDomain::SensoryClarity),

        // Attention / routing across senses.
        "neuromorph-attention-micro"
        | "evo.attention.routing"
        | "evolution.neuromorph.attention" => Some(NeuromorphDomain::AttentionRouting),

        _ => None,
    }
}

/// Map a RuntimeEventKind into a domain + flag when it is clearly neuromorphic.
///
/// This keeps neuromorph classification out of the core runtime and
/// lets boundary services decide which events should be treated as
/// neuromorphic micro-evolution.
pub fn map_event_kind_to_neuromorph(
    kind: &RuntimeEventKind,
    domain_hint: Option<&str>,
) -> Option<NeuromorphDomain> {
    match kind {
        // EvolutionUpgrade is the only mutating path; use the evolution ID
        // or explicit domain hints to route into a neuromorphic domain.
        RuntimeEventKind::EvolutionUpgrade { evolutionid } => {
            if let Some(hint) = domain_hint {
                if let Some(d) = map_domain_str(hint) {
                    return Some(d);
                }
            }
            // Fallback: classify by evolutionid prefix.
            if evolutionid.starts_with("neuromorph-reflex-") {
                Some(NeuromorphDomain::ReflexSafety)
            } else if evolutionid.starts_with("neuromorph-sense-") {
                Some(NeuromorphDomain::SensoryClarity)
            } else if evolutionid.starts_with("neuromorph-attn-")
                || evolutionid.starts_with("neuromorph-attention-")
            {
                Some(NeuromorphDomain::AttentionRouting)
            } else {
                None
            }
        }

        // You can optionally treat small SmartAutonomy events tagged
        // as neuromorphic as well, but default is None here for safety.
        _ => None,
    }
}

/// Build a NeuromorphContext from lifeforce bands and NeuralRope / EEG metrics.
///
/// Inputs are designed to be thin adapters over your existing types:
/// - `bands`: recent LifeforceBandSeries (safe/soft-warn/hard-stop).
/// - `iface`: snapshot of interface clarity / instability from NeuralRope.
pub fn build_neuromorph_context(
    bands: &LifeforceBandSeries,
    iface: &NeuralInterfaceSnapshot,
) -> NeuromorphContext {
    // 1. Discomfort index from comfort band and pain corridor.
    let comfort_band = map_to_comfort_band(bands);
    let mut discomfort_index: f32 = match comfort_band {
        ComfortBand::ComfortSafe => 0.1,
        ComfortBand::DiscomfortSoftWarn => 0.5,
        ComfortBand::DiscomfortHardStop => 0.9,
    };

    // PainCorridorSignal equivalent from iface (0.0–1.0).
    let pain_corridor = iface.pain_corridor_index.clamp(0.0, 1.0);
    // Let pain add directly to discomfort, but clamp to 1.0.
    discomfort_index = (discomfort_index + 0.6 * pain_corridor).clamp(0.0, 1.0);

    // 2. Overload ratio from recent HardStop/SoftWarn lifeforce bands.
    let overload_ratio = bands.overload_ratio.clamp(0.0, 1.0);

    // 3. Interface instability from clarity and dropouts.
    //
    // Assume NeuralInterfaceSnapshot provides:
    // - clarity_index: 0.0–1.0 (1.0 = very clear, low noise).
    // - dropout_ratio: 0.0–1.0 fraction of frames with poor signal.
    let clarity = iface.clarity_index.clamp(0.0, 1.0);
    let dropout = iface.dropout_ratio.clamp(0.0, 1.0);
    let interface_instability = ((1.0 - clarity) * 0.7 + dropout * 0.6).clamp(0.0, 1.0);

    // 4. Neuromorphic SCALE usage (soft, per-domain advisory).
    //
    // This can be derived from your SCALE accounting per domain; here we
    // assume iface carries a normalized 0.0–1.0 usage for neuromorph.
    let neuromorph_scale_usage = iface.neuromorph_scale_usage.clamp(0.0, 1.0);

    NeuromorphContext {
        discomfort_index,
        overload_ratio,
        interface_instability,
        neuromorph_scale_usage,
    }
}

/// Thin adapter from LifeforceBandSeries to a ComfortBand.
///
/// LifeforceBandSeries already has SoftWarn / HardStop semantics in
/// your doctrine; we collapse them into a single comfort index band.
fn map_to_comfort_band(bands: &LifeforceBandSeries) -> ComfortBand {
    match bands.current {
        LifeforceBand::Safe => ComfortBand::ComfortSafe,
        LifeforceBand::SoftWarn => ComfortBand::DiscomfortSoftWarn,
        LifeforceBand::HardStop => ComfortBand::DiscomfortHardStop,
    }
}

/// Example traits for the external types we depend on, so this module is
/// self-contained. In your tree, you already have real definitions for
/// these in lifeforce / neuralrope crates.
pub mod lifeforce_bands {
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum LifeforceBand {
        Safe,
        SoftWarn,
        HardStop,
    }

    #[derive(Clone, Debug)]
    pub struct LifeforceBandSeries {
        pub current: LifeforceBand,
        /// 0.0–1.0 fraction of recent epochs that were SoftWarn or HardStop.
        pub overload_ratio: f32,
    }
}

pub mod neuralrope_metrics {
    #[derive(Clone, Debug)]
    pub struct NeuralInterfaceSnapshot {
        /// 0.0–1.0, 1.0 = very clear, low-noise interface.
        pub clarity_index: f32,
        /// 0.0–1.0, fraction of frames with bad/unstable decoding.
        pub dropout_ratio: f32,
        /// 0.0–1.0, summary of pain/aversion markers (PainCorridorSignal).
        pub pain_corridor_index: f32,
        /// 0.0–1.0, fraction of daily SCALE used by neuromorphic domains.
        pub neuromorph_scale_usage: f32,
    }
}

/// Runtime alias, so we can stay in sync with your existing runtime crate.
pub mod runtime {
    #[derive(Clone, Debug)]
    pub enum RuntimeEventKind {
        EvolutionUpgrade { evolutionid: String },
        WaveLoad { taskid: String, requestedwave: f64 },
        SmartAutonomy { agentid: String, requestedsmart: f64 },
    }
}

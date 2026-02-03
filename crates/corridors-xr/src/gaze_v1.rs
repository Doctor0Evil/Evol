#![forbid(unsafe_code)]

use std::time::Duration;

use serde::{Deserialize, Serialize};

use bioscale_core::{BrainSpecs, EvolutionDecision, EvolutionDecisionKind, HostBudget};
use bioscale_metrics::CorridorMetricsSink;
use bioscale_neuro::BciHostSnapshot;

/// Stable identifier for this corridor.
pub const XR_GAZE_CORRIDOR_ID: &str = "bio.corridor.xr.gaze.v1";

/// Evidence bundle for this corridor (exactly 10 tags, aligned with ALN shard).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XrGazeEvidenceBundleV1 {
    pub tags: [String; 10],
}

impl Default for XrGazeEvidenceBundleV1 {
    fn default() -> Self {
        Self {
            tags: [
                "a1f3c9b2".to_string(),
                "4be79d01".to_string(),
                "9cd4a7e8".to_string(),
                "2f8c6b44".to_string(),
                "7e1da2ff".to_string(),
                "5b93e0c3".to_string(),
                "d0174aac".to_string(),
                "6ac2f9d9".to_string(),
                "c4e61b20".to_string(),
                "8f09d5ee".to_string(),
            ],
        }
    }
}

/// Static envelope parameters derived from the ALN corridor shard.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XrGazeCorridorEnvelopeV1 {
    pub max_spatial_error_cm: f32,
    pub per_event_energy_j: f32,
    pub per_session_energy_j: f32,
    pub daily_energy_j: f32,
    pub sbio_load_index_max: f32,
    pub local_thermal_delta_c_max: f32,
    pub global_thermal_delta_c_max: f32,
    pub max_duty_fraction_session: f32,
    pub min_inter_event_ms: u64,
    pub max_continuous_burst_ms: u64,
    pub min_cooldown_between_bursts_ms: u64,
    pub hrv_drop_allowed_ratio: f32,
    pub eeg_beta_gamma_ceiling: f32,
    pub roh_ceiling: f32,
    pub roh_target: f32,
}

impl Default for XrGazeCorridorEnvelopeV1 {
    fn default() -> Self {
        Self {
            max_spatial_error_cm: 0.20,
            per_event_energy_j: 0.05,
            per_session_energy_j: 5.0,
            daily_energy_j: 25.0,
            sbio_load_index_max: 0.30,
            local_thermal_delta_c_max: 0.8,
            global_thermal_delta_c_max: 0.4,
            max_duty_fraction_session: 0.35,
            min_inter_event_ms: 25,
            max_continuous_burst_ms: 500,
            min_cooldown_between_bursts_ms: 250,
            hrv_drop_allowed_ratio: 0.85,
            eeg_beta_gamma_ceiling: 0.80,
            roh_ceiling: 0.30,
            roh_target: 0.10,
        }
    }
}

/// Observables projected into the corridor state space for one decision step.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XrGazeCorridorStateV1 {
    pub spatial_error_cm: f32,
    pub event_energy_j: f32,
    pub session_energy_j: f32,
    pub daily_energy_j: f32,
    pub sbio_load_index: f32,
    pub local_thermal_delta_c: f32,
    pub global_thermal_delta_c: f32,
    pub session_duty_fraction: f32,
    pub inter_event: Duration,
    pub continuous_burst: Duration,
    pub cooldown_since_last_burst: Duration,
    pub hrv_ratio: f32,
    pub eeg_beta_gamma_load: f32,
    pub roh_estimate_window: f32,
}

/// Trait for XR corridor guard kernels.
pub trait XrCorridorGuardKernel {
    fn corridor_id(&self) -> &'static str;

    fn check_and_decide(
        &self,
        brain: &BrainSpecs,
        budget: &HostBudget,
        bci: &BciHostSnapshot,
        state: &XrGazeCorridorStateV1,
        metrics: &dyn CorridorMetricsSink,
    ) -> EvolutionDecision;
}

/// Minimal guard skeleton for bio.corridor.xr.gaze.v1.
#[derive(Clone, Debug)]
pub struct XrGazeCorridorGuardV1 {
    pub envelope: XrGazeCorridorEnvelopeV1,
    pub evidence: XrGazeEvidenceBundleV1,
}

impl Default for XrGazeCorridorGuardV1 {
    fn default() -> Self {
        Self {
            envelope: XrGazeCorridorEnvelopeV1::default(),
            evidence: XrGazeEvidenceBundleV1::default(),
        }
    }
}

impl XrGazeCorridorGuardV1 {
    fn within_spatial(&self, s: &XrGazeCorridorStateV1) -> bool {
        s.spatial_error_cm <= self.envelope.max_spatial_error_cm
    }

    fn within_energy(&self, s: &XrGazeCorridorStateV1) -> bool {
        s.event_energy_j <= self.envelope.per_event_energy_j
            && s.session_energy_j <= self.envelope.per_session_energy_j
            && s.daily_energy_j <= self.envelope.daily_energy_j
    }

    fn within_sbio_and_thermal(&self, s: &XrGazeCorridorStateV1) -> bool {
        s.sbio_load_index <= self.envelope.sbio_load_index_max
            && s.local_thermal_delta_c <= self.envelope.local_thermal_delta_c_max
            && s.global_thermal_delta_c <= self.envelope.global_thermal_delta_c_max
    }

    fn within_duty_and_timing(&self, s: &XrGazeCorridorStateV1) -> bool {
        s.session_duty_fraction <= self.envelope.max_duty_fraction_session
            && s.inter_event.as_millis() as u64 >= self.envelope.min_inter_event_ms
            && s.continuous_burst.as_millis() as u64 <= self.envelope.max_continuous_burst_ms
            && s.cooldown_since_last_burst.as_millis() as u64
                >= self.envelope.min_cooldown_between_bursts_ms
    }

    fn within_hrv_eeg_and_roh(&self, s: &XrGazeCorridorStateV1) -> bool {
        s.hrv_ratio >= self.envelope.hrv_drop_allowed_ratio
            && s.eeg_beta_gamma_load <= self.envelope.eeg_beta_gamma_ceiling
            && s.roh_estimate_window <= self.envelope.roh_ceiling
    }
}

impl XrCorridorGuardKernel for XrGazeCorridorGuardV1 {
    fn corridor_id(&self) -> &'static str {
        XR_GAZE_CORRIDOR_ID
    }

    fn check_and_decide(
        &self,
        _brain: &BrainSpecs,
        _budget: &HostBudget,
        _bci: &BciHostSnapshot,
        state: &XrGazeCorridorStateV1,
        metrics: &dyn CorridorMetricsSink,
    ) -> EvolutionDecision {
        let mut allowed = true;
        let mut reasons = Vec::new();

        if !self.within_spatial(state) {
            allowed = false;
            reasons.push("spatial_error_exceeds_corridor".to_string());
            metrics.inc_corridor_breach(
                XR_GAZE_CORRIDOR_ID,
                "spatial",
                state.spatial_error_cm as f64,
            );
        }

        if !self.within_energy(state) {
            allowed = false;
            reasons.push("energy_envelope_exceeded".to_string());
            metrics.inc_corridor_breach(
                XR_GAZE_CORRIDOR_ID,
                "energy",
                state.event_energy_j as f64,
            );
        }

        if !self.within_sbio_and_thermal(state) {
            allowed = false;
            reasons.push("sbio_or_thermal_envelope_exceeded".to_string());
            metrics.inc_corridor_breach(
                XR_GAZE_CORRIDOR_ID,
                "thermal",
                state.local_thermal_delta_c as f64,
            );
        }

        if !self.within_duty_and_timing(state) {
            allowed = false;
            reasons.push("duty_or_timing_violation".to_string());
            metrics.inc_corridor_breach(
                XR_GAZE_CORRIDOR_ID,
                "duty_timing",
                state.session_duty_fraction as f64,
            );
        }

        if !self.within_hrv_eeg_and_roh(state) {
            allowed = false;
            reasons.push("hrv_eeg_or_roh_violation".to_string());
            metrics.inc_corridor_breach(
                XR_GAZE_CORRIDOR_ID,
                "hrv_eeg_roh",
                state.roh_estimate_window as f64,
            );
        }

        metrics.observe_corridor_kernel_distance(
            XR_GAZE_CORRIDOR_ID,
            state.spatial_error_cm as f64,
        );

        metrics.observe_corridor_knowledge_factor(
            XR_GAZE_CORRIDOR_ID,
            compute_knowledge_factor(state),
        );

        if allowed {
            EvolutionDecision {
                kind: EvolutionDecisionKind::Allow,
                corridor_id: XR_GAZE_CORRIDOR_ID.to_string(),
                reasons,
            }
        } else {
            EvolutionDecision {
                kind: EvolutionDecisionKind::Deny,
                corridor_id: XR_GAZE_CORRIDOR_ID.to_string(),
                reasons,
            }
        }
    }
}

fn compute_knowledge_factor(state: &XrGazeCorridorStateV1) -> f64 {
    let spatial_term = (1.0_f64 - (state.spatial_error_cm as f64 / 0.20).min(1.0)).max(0.0);
    let roh_term = (1.0_f64 - (state.roh_estimate_window as f64 / 0.30).min(1.0)).max(0.0);
    (0.6 * spatial_term + 0.4 * roh_term).clamp(0.0, 1.0)
}

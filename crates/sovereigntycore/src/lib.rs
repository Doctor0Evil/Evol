use autonomysafety::{polytope::CorridorPolytope, roh::RiskOfHarm};
use governance::{neurorights::NeurorightsProfile, stake::StakePolicy, token::EvolveToken};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NormalizedBioState {
    pub metrics: autonomysafety::polytope::NormalizedMetrics,
    pub roh: RiskOfHarm,
    pub pain_vas: f32,
    pub cognitive_load: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProposalKind {
    ParamNudge,
    ThresholdShift,
    ModeShift,
    PolicyUpdate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub subjectid: String,
    pub kind: ProposalKind,
    pub roh_delta: f32,
    pub projected_metrics: autonomysafety::polytope::NormalizedMetrics,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Decision {
    Allowed,
    Denied(String),
}

pub struct SovereigntyCore {
    pub neurorights: NeurorightsProfile,
    pub stake: StakePolicy,
    pub corridor: CorridorPolytope,
}

impl SovereigntyCore {
    fn check_neurorights(&self, state: &NormalizedBioState) -> Result<(), String> {
        if state.pain_vas > self.neurorights.max_pain_vas {
            return Err("Pain envelope exceeded".into());
        }
        if state.cognitive_load > self.neurorights.max_cognitive_load {
            return Err("Cognitive load envelope exceeded".into());
        }
        Ok(())
    }

    fn check_physioguard(
        guard: &governance::token::PhysioGuard,
        metrics: &autonomysafety::polytope::NormalizedMetrics,
    ) -> Result<(), String> {
        if metrics.hrv_sdnn < guard.min_hrv_sdnn {
            return Err("HRV below physioguard baseline".into());
        }
        if metrics.emg_tension > guard.max_emg_tension {
            return Err("EMG tension above physioguard limit".into());
        }
        if metrics.fatigue_index > guard.max_fatigue_index {
            return Err("Fatigue index above physioguard limit".into());
        }
        Ok(())
    }

    pub fn evaluate(
        &self,
        state: &NormalizedBioState,
        proposal: &Proposal,
        token: Option<&EvolveToken>,
        now_unix: i64,
    ) -> Decision {
        if proposal.subjectid != self.stake.subjectid {
            return Decision::Denied("Subject is not host".into());
        }

        if let Err(e) = self.check_neurorights(state) {
            return Decision::Denied(e);
        }

        if !self.corridor.is_safe(&proposal.projected_metrics) {
            return Decision::Denied("Projected state leaves safety corridor".into());
        }

        // strict band
        let strict_candidate = state.roh.evaluate_strict(proposal.roh_delta);
        if let Ok(new_roh) = strict_candidate {
            return Decision::Allowed;
        }

        // research band path requires EVOLVE token
        let t = match token {
            None => {
                return Decision::Denied("RoH override requires EVOLVE token".into());
            }
            Some(tok) => tok,
        };

        if t.subjectid != proposal.subjectid {
            return Decision::Denied("Token subject mismatch".into());
        }
        if t.roh_band != "research" {
            return Decision::Denied("Token roh_band must be 'research'".into());
        }
        if !t.scope.iter().any(|s| s == "highrisk_research") {
            return Decision::Denied("Token missing highrisk_research scope".into());
        }
        if now_unix < t.valid_from || now_unix > t.valid_until {
            return Decision::Denied("Token expired or not yet valid".into());
        }

        if let Err(e) = Self::check_physioguard(&t.physioguard, &proposal.projected_metrics) {
            return Decision::Denied(e);
        }

        match proposal.kind {
            ProposalKind::ParamNudge | ProposalKind::ThresholdShift => {}
            _ => {
                return Decision::Denied(
                    "High-risk research allows only ParamNudge or ThresholdShift".into(),
                )
            }
        }

        match state.roh.evaluate_research(proposal.roh_delta, t.max_effectsize) {
            Ok(_new_roh) => Decision::Allowed,
            Err(e) => Decision::Denied(e),
        }
    }
}

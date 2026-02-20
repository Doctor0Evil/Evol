#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use bioscale_upgrade_store::{UpgradeDescriptor, HostBudget};
use cyberswarm_neurostack::alnparticles::ALNComplianceParticle;
use cyberswarm_neurostack::telemetry::BciHostSnapshot;
use neurorights_types::ConsentEnvelopes;
use evidence_types::EvidenceBundle10;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EvoMode {
    Conservative,
    Copilot,
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvoConsent {
    pub host_did: String,
    pub policy_id: String,
    pub version: String,
    pub mode: EvoMode,
    pub aln_particles: Vec<ALNComplianceParticle>,
    pub envelopes: ConsentEnvelopes,        // HostBudget + Thermo + Cognitive + Rope
    pub evidence_bundle: EvidenceBundle10,  // fixed 10-tag bundle
}

#[derive(Clone, Debug)]
pub enum PolicyError {
    MissingRollbackAnytime,
    MissingMentalPrivacyFlag,
    BudgetViolation,
    EnvelopeViolation,
    EvidenceMissing,
}

impl EvoConsent {
    /// Hard gate: if this returns Err, the upgrade is not admissible.
    pub fn enforce_on(
        &self,
        desc: &UpgradeDescriptor,
        host: &HostBudget,
        snap: &BciHostSnapshot,
    ) -> Result<(), PolicyError> {
        // 1. Neurorights: rollbackanytime, no non-consensual modulation, no raw EEG export.
        if !self
            .aln_particles
            .iter()
            .any(|p| p.neurorights_rollback_anytime)
        {
            return Err(PolicyError::MissingRollbackAnytime);
        }
        if !self
            .aln_particles
            .iter()
            .all(|p| p.neurorights_no_nonconsensual_modulation)
        {
            return Err(PolicyError::MissingMentalPrivacyFlag);
        }
        if !self
            .aln_particles
            .iter()
            .all(|p| p.neurorights_no_raw_eeg_export)
        {
            return Err(PolicyError::MissingMentalPrivacyFlag);
        }

        // 2. HostBudget + thermo + cognitive envelopes from ConsentEnvelopes.
        if !self.envelopes.fits_host_budget(host, desc) {
            return Err(PolicyError::BudgetViolation);
        }
        if !self.envelopes.fits_thermo_and_cognitive(host, snap, desc) {
            return Err(PolicyError::EnvelopeViolation);
        }

        // 3. Ropes & identity: topology + plasticity bounds.
        if !self.envelopes.fits_neural_ropes(desc) {
            return Err(PolicyError::EnvelopeViolation);
        }

        // 4. Evidence: require exactly 10 tags matching registry.
        if !self.evidence_bundle.is_complete_for(desc) {
            return Err(PolicyError::EvidenceMissing);
        }

        Ok(())
    }
}

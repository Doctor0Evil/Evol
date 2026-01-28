use std::sync::Arc;
use organic_cpu::evidence::{
    BiophysicalProofArtifact,
    EcoBand,
    EventDomain,
    EventSummary,
    HostIdentity,
    LifeforceBand,
    MetabolicConsentMode,
    PainBand,
    ProofEmitter,
    current_utc_ms,
};
use crate::chain::biophysicalruntime::{
    BioTokenState,
    LorentzTimestamp,
    RuntimeEventKind,
    // ... other imports
};

// Extend HostNode to hold a proof emitter.
pub struct HostNode<D, C, HC>
where
    D: alndid::DIDDirectory + Send + Sync + 'static,
    C: alndid::ConsentVerifier + Send + Sync + 'static,
    HC: biospectreconsensus::HostConsensus + Send + Sync + 'static,
{
    // existing fields...
    hostid: alndid::ALNDID,
    runtime: Arc<BiophysicalRuntime<D, C, HC>>,
    storage: HostStorage,
    // NEW:
    proof_emitter: Arc<dyn ProofEmitter>,
    // ...
}

// Example constructor update:
impl<D, C, HC> HostNode<D, C, HC>
where
    D: alndid::DIDDirectory + Send + Sync + 'static,
    C: alndid::ConsentVerifier + Send + Sync + 'static,
    HC: biospectreconsensus::HostConsensus + Send + Sync + 'static,
{
    pub fn new(
        hostid: alndid::ALNDID,
        runtime: BiophysicalRuntime<D, C, HC>,
        initial_state: BioTokenState,
        proof_emitter: Arc<dyn ProofEmitter>,
    ) -> (Self, tokio::sync::mpsc::Receiver<GossipFrame>) {
        // existing setup...
        let storage = HostStorage::new(initial_state);
        let (gossip_tx, gossip_rx) = tokio::sync::mpsc::channel(128);

        let node = HostNode {
            hostid,
            runtime: Arc::new(runtime),
            storage,
            proof_emitter,
            // ...
        };

        (node, gossip_rx)
    }
}

// In the RPC handler, right after execute_event returns Ok(frame):

async fn handle_rpc(&self, req: RpcRequest) -> RpcResponse {
    match req {
        RpcRequest::SubmitEvent { header, event } => {
            // existing security header checks...
            // load pre-state:
            let mut state = self.storage.read_state();
            let pre_state = state.clone();
            let host_frame = self.build_host_frame();
            let previous = self.storage.read_last_frame();
            let previous_ref = previous.as_ref();

            // convert RPC event -> runtime event:
            let runtime_event = self.convert_rpc_event(event);

            // call runtime:
            let result = self.runtime.execute_event(
                &mut state,
                previous_ref,
                host_frame.clone(),
                runtime_event,
            );

            match result {
                Ok(frame) => {
                    // persist new state + frame:
                    self.storage.apply_state_and_frame(state.clone(), frame.clone());

                    // === NEW: build and emit proof artifact ===
                    if let Err(e) = self.emit_proof_for_transition(
                        &pre_state,
                        &state,
                        &frame,
                        &host_frame,
                        &header,
                    ) {
                        eprintln!("Warning: failed to emit proof artifact: {:?}", e);
                    }

                    // existing gossip + response...
                    RpcResponse::OkEventApplied {
                        seqno: frame.seqno,
                        statehash: hex::encode(frame.statehash.0),
                    }
                }
                Err(e) => {
                    RpcResponse::Error {
                        error: format!("runtime error: {:?}", e),
                    }
                }
            }
        }
        // ...
    }
}

// Helper method on HostNode to build and emit a BiophysicalProofArtifact.
impl<D, C, HC> HostNode<D, C, HC>
where
    D: alndid::DIDDirectory + Send + Sync + 'static,
    C: alndid::ConsentVerifier + Send + Sync + 'static,
    HC: biospectreconsensus::HostConsensus + Send + Sync + 'static,
{
    fn emit_proof_for_transition(
        &self,
        pre_state: &BioTokenState,
        post_state: &BioTokenState,
        frame: &biospectreconsensus::ConsensusFrame,
        host_frame: &ALNHostFrame,
        header: &RpcSecurityHeader,
    ) -> ProofResult<()> {
        // Map runtime types to evidence types.

        let host = HostIdentity {
            did: self.hostid.id.clone(),
            shard: self.hostid.shard.clone(),
        };

        let lorentz_ts = LorentzTimestamp {
            proper_time_ns: post_state.lorentzts.0,
            frame_offset_ps: post_state.lorentzts.1,
        };

        let origin_plane = "organic_cpu".to_string();

        let event_domain = match header.event_kind_string.as_str() {
            "evolution_upgrade" => EventDomain::EvolutionUpgrade,
            "wave_load" => EventDomain::WaveLoad,
            "smart_autonomy" => EventDomain::SmartAutonomy,
            "defensive_micro" => EventDomain::DefensiveMicro,
            other => EventDomain::Other(other.to_string()),
        };

        let event = EventSummary {
            event_id: frame.statehash.to_hex(), // or a separate event hash
            domain: event_domain,
            reason: header.reason.clone(),
        };

        // These helpers are placeholders; wire them to your existing
        // lifeforce/eco/pain/consent modules as appropriate:
        let lifeforce_band = self.map_lifeforce_band(post_state);
        let lifeforce_ok = lifeforce_band != LifeforceBand::HardStop;

        let eco_band = self.map_eco_band(post_state);
        let eco_cost_hint = self.estimate_ecocost_hint(post_state);

        let pain_band = self.map_pain_band_if_any();

        let metabolic_mode = self.current_metabolic_mode();

        let consent_proof_hash_hex = self.lookup_consent_hash_for_frame(frame);
        let provenance_hash_hex = self.lookup_provenance_hash_for_frame(frame);
        let civic_audit_id = self.lookup_civic_audit_id_for_frame(frame);

        let artifact = BiophysicalProofArtifact {
            host,
            lorentz_ts,
            utc_ms: current_utc_ms(),
            origin_plane,
            event,
            pre_state_hash_hex: pre_state.consensusattest().to_hex(),
            post_state_hash_hex: post_state.consensusattest().to_hex(),
            consensus_seqno: frame.seqno,
            lifeforce_band,
            lifeforce_ok,
            eco_band,
            eco_cost_hint,
            pain_band,
            metabolic_mode,
            consent_proof_hash_hex,
            provenance_hash_hex,
            civic_audit_id,
        };

        self.proof_emitter.emit_proof_artifact(&artifact)
    }

    // Stub mappings; implement using your existing lifeforce/eco modules.
    fn map_lifeforce_band(&self, _state: &BioTokenState) -> LifeforceBand {
        // Example: derive from LifeforceState / metabolic bands.
        LifeforceBand::Safe
    }

    fn map_eco_band(&self, _state: &BioTokenState) -> EcoBand {
        EcoBand::Low
    }

    fn estimate_ecocost_hint(&self, _state: &BioTokenState) -> f64 {
        0.0
    }

    fn map_pain_band_if_any(&self) -> Option<PainBand> {
        None
    }

    fn current_metabolic_mode(&self) -> MetabolicConsentMode {
        MetabolicConsentMode::Unknown
    }

    fn lookup_consent_hash_for_frame(
        &self,
        _frame: &biospectreconsensus::ConsensusFrame,
    ) -> Option<String> {
        None
    }

    fn lookup_provenance_hash_for_frame(
        &self,
        _frame: &biospectreconsensus::ConsensusFrame,
    ) -> Option<String> {
        None
    }

    fn lookup_civic_audit_id_for_frame(
        &self,
        _frame: &biospectreconsensus::ConsensusFrame,
    ) -> Option<String> {
        None
    }
}

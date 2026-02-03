#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RohBands {
    pub rohceiling_strict: f32,
    pub rohceiling_research: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RohModelCore {
    pub id: String,
    pub weights: RohWeights,
    pub bands: RohBands,
    pub notes: Option<String>,
}

impl RohModelShard {
    pub fn roh_ceiling_strict(&self) -> f32 {
        self.model.bands.rohceiling_strict
    }
    pub fn roh_ceiling_research(&self) -> f32 {
        self.model.bands.rohceiling_research
    }
    pub fn validate_invariants(&self) -> Result<(), String> {
        // existing weight checks…
        if self.model.bands.rohceiling_research < self.model.bands.rohceiling_strict {
            return Err("research ceiling < strict ceiling".into());
        }
        if self.model.bands.rohceiling_research > 0.45 {
            return Err("research ceiling > 0.45".into());
        }
        Ok(())
    }
}

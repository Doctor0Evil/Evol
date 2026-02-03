pub struct RiskOfHarm {
    model: RohModelShard,
}

impl RiskOfHarm {
    pub fn ceiling_strict(&self) -> f32 {
        self.model.roh_ceiling_strict()
    }
    pub fn ceiling_research(&self) -> f32 {
        self.model.roh_ceiling_research()
    }

    pub fn check_normal(&self, before: RohInputs, after: RohInputs) -> bool {
        let roh_before = self.model.compute_roh(before);
        let roh_after  = self.model.compute_roh(after);
        roh_after <= roh_before && roh_after <= self.ceiling_strict()
    }

    pub fn check_research(&self, before: RohInputs, after: RohInputs) -> bool {
        let roh_before = self.model.compute_roh(before);
        let roh_after  = self.model.compute_roh(after);
        roh_after <= self.ceiling_research()
    }
}

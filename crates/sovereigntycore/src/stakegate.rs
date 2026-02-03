impl StakeGate {
    pub fn requires_scope(&self, scope: &str) -> Result<(), String> {
        // ensure Host + OrganicCPU both have EVOLVE scope including `scope`
        // and signatures are present for this proposal
        // …
    }
}

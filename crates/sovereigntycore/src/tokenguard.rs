pub fn enforce_research_band(
    token: &EvolveToken,
    effect_l2: f32,
    roh_after: f32,
    roh_ceiling_research: f32,
    biostate: &BioState,
) -> Result<(), String> {
    if token.roh_band != RohBand::Research {
        return Err("RoH override requires roh_band=research".into());
    }
    if !token.scope.contains(&"highrisk_research".to_string()) {
        return Err("Token scope missing highrisk_research".into());
    }
    if effect_l2 > token.maxeffectsize {
        return Err("effect size exceeds token.maxeffectsize".into());
    }
    if roh_after > roh_ceiling_research {
        return Err("roh_after exceeds research ceiling".into());
    }
    if biostate.hrv.sdnn_ms < token.physioguard.min_hrv_sdnn_ms
        || biostate.fatigueindex > token.physioguard.max_fatigueindex
        || biostate.pain_vas > token.physioguard.max_pain_vas
    {
        return Err("physioguard thresholds violated".into());
    }
    Ok(())
}

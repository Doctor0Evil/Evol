#![forbid(unsafe_code)]

/// Abstract sink that your Prometheus-using binary implements.
pub trait CorridorMetricsSink: Send + Sync {
    /// Count a corridor envelope breach, labeled by corridor_id and breach_type.
    /// `value` can be the offending magnitude (e.g., cm error, J, duty fraction).
    fn inc_corridor_breach(&self, corridor_id: &str, breach_type: &str, value: f64);

    /// Observe kernel distance (e.g., spatial error or composite distance).
    fn observe_corridor_kernel_distance(&self, corridor_id: &str, distance: f64);

    /// Observe Knowledge-Factor in [0,1] for this corridor and window.
    fn observe_corridor_knowledge_factor(&self, corridor_id: &str, kf: f64);
}

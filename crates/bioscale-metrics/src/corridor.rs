#![forbid(unsafe_code)]

pub trait CorridorMetricsSink: Send + Sync {
    fn inc_corridor_breach(&self, corridor_id: &str, breach_type: &str, value: f64);
    fn observe_corridor_kernel_distance(&self, corridor_id: &str, distance: f64);
    fn observe_corridor_knowledge_factor(&self, corridor_id: &str, kf: f64);
}

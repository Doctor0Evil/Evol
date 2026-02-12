use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsSchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricRegistrySnapshot {
    pub schema_version: MetricsSchemaVersion,
    pub metrics: Vec<MetricDescriptor>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricDescriptor {
    pub name: String,
    pub labels: Vec<String>,
    pub aln_clause_ids: Vec<String>,
}

pub fn validate_metrics_schema(snapshot: &MetricRegistrySnapshot) -> anyhow::Result<()> {
    // Example: enforce alignment between metric names and ALN clause IDs.
    for m in &snapshot.metrics {
        if m.name.starts_with("aln_non_compliant_events_total")
            && !m.aln_clause_ids.iter().all(|cid| cid.starts_with("ALN-"))
        {
            anyhow::bail!(
                "Metric {} has invalid ALN clause IDs: {:?}",
                m.name,
                m.aln_clause_ids
            );
        }
    }
    Ok(())
}

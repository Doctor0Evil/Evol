use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct SensorSnapshot {
    pub sensor_id: String,
    pub at: SystemTime,
    pub values: Vec<f64>,
    pub labels: Vec<String>,
}

pub trait SensorAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn sample(&self) -> Option<SensorSnapshot>;
}

pub struct DummyBiophysicalSensor {
    id: String,
    labels: Vec<String>,
}

impl DummyBiophysicalSensor {
    pub fn new(id: impl Into<String>, labels: Vec<String>) -> Self {
        Self {
            id: id.into(),
            labels,
        }
    }
}

impl SensorAdapter for DummyBiophysicalSensor {
    fn id(&self) -> &str {
        &self.id
    }

    fn sample(&self) -> Option<SensorSnapshot> {
        let now = SystemTime::now();
        let values = vec![0.0; self.labels.len()];
        Some(SensorSnapshot {
            sensor_id: self.id.clone(),
            at: now,
            values,
            labels: self.labels.clone(),
        })
    }
}

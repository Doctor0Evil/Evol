use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct EffectorCommand {
    pub effector_id: String,
    pub at: SystemTime,
    pub payload: Vec<f64>,
    pub labels: Vec<String>,
}

pub trait EffectorAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn apply(&self, cmd: EffectorCommand) -> bool;
}

pub struct DummyEffector {
    id: String,
}

impl DummyEffector {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl EffectorAdapter for DummyEffector {
    fn id(&self) -> &str {
        &self.id
    }

    fn apply(&self, _cmd: EffectorCommand) -> bool {
        true
    }
}

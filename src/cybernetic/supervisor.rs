use super::{
    feedback_bus::{CommandEnvelope, FeedbackBus, SensorEnvelope},
    EffectorAdapter, EffectorCommand, SensorAdapter,
};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub struct SupervisorConfig {
    pub queue_len: usize,
}

pub struct Supervisor {
    bus: Arc<Mutex<FeedbackBus>>,
    sensors: Vec<Arc<dyn SensorAdapter>>,
    effectors: Vec<Arc<dyn EffectorAdapter>>,
}

impl Supervisor {
    pub fn new(config: SupervisorConfig) -> Self {
        let bus = FeedbackBus::new(config.queue_len);
        Self {
            bus: Arc::new(Mutex::new(bus)),
            sensors: Vec::new(),
            effectors: Vec::new(),
        }
    }

    pub fn bus(&self) -> Arc<Mutex<FeedbackBus>> {
        Arc::clone(&self.bus)
    }

    pub fn register_sensor(&mut self, sensor: Arc<dyn SensorAdapter>) {
        self.sensors.push(sensor);
    }

    pub fn register_effector(&mut self, effector: Arc<dyn EffectorAdapter>) {
        self.effectors.push(effector);
    }

    pub fn poll_sensors(&self) {
        if let Ok(mut bus) = self.bus.lock() {
            for sensor in &self.sensors {
                if let Some(snapshot) = sensor.sample() {
                    let payload = snapshot
                        .values
                        .iter()
                        .flat_map(|v| v.to_le_bytes())
                        .collect::<Vec<u8>>();
                    let env = SensorEnvelope {
                        id: snapshot.sensor_id.clone(),
                        at: snapshot.at,
                        payload,
                    };
                    bus.push_sensor(env);
                }
            }
        }
    }

    pub fn drive_effectors(&self) {
        if let Ok(mut bus) = self.bus.lock() {
            while let Some(env) = bus.pull_command() {
                for eff in &self.effectors {
                    if eff.id() == env.id {
                        let mut values = Vec::new();
                        let mut chunk = [0u8; 8];
                        for c in env.payload.chunks(8) {
                            if c.len() == 8 {
                                chunk.copy_from_slice(c);
                                let v = f64::from_le_bytes(chunk);
                                values.push(v);
                            }
                        }
                        let cmd = EffectorCommand {
                            effector_id: env.id.clone(),
                            at: env.at,
                            payload: values,
                            labels: Vec::new(),
                        };
                        eff.apply(cmd);
                    }
                }
            }
        }
    }

    pub fn enqueue_effector_command(
        &self,
        effector_id: String,
        payload_values: Vec<f64>,
    ) {
        let payload = payload_values
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<u8>>();
        let env = CommandEnvelope {
            id: effector_id,
            at: SystemTime::now(),
            payload,
        };
        if let Ok(mut bus) = self.bus.lock() {
            bus.push_command(env);
        }
    }
}

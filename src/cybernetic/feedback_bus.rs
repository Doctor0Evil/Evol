use std::collections::VecDeque;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub enum FeedbackEvent {
    Sensor(SensorEnvelope),
    Command(CommandEnvelope),
}

#[derive(Debug, Clone)]
pub struct SensorEnvelope {
    pub id: String,
    pub at: SystemTime,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct CommandEnvelope {
    pub id: String,
    pub at: SystemTime,
    pub payload: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct FeedbackBus {
    sensor_queue: VecDeque<SensorEnvelope>,
    command_queue: VecDeque<CommandEnvelope>,
    max_len: usize,
}

impl FeedbackBus {
    pub fn new(max_len: usize) -> Self {
        Self {
            sensor_queue: VecDeque::with_capacity(max_len),
            command_queue: VecDeque::with_capacity(max_len),
            max_len,
        }
    }

    pub fn push_sensor(&mut self, env: SensorEnvelope) {
        if self.sensor_queue.len() >= self.max_len {
            self.sensor_queue.pop_front();
        }
        self.sensor_queue.push_back(env);
    }

    pub fn push_command(&mut self, env: CommandEnvelope) {
        if self.command_queue.len() >= self.max_len {
            self.command_queue.pop_front();
        }
        self.command_queue.push_back(env);
    }

    pub fn pull_sensor(&mut self) -> Option<SensorEnvelope> {
        self.sensor_queue.pop_front()
    }

    pub fn pull_command(&mut self) -> Option<CommandEnvelope> {
        self.command_queue.pop_front()
    }

    pub fn purge_older_than(&mut self, cutoff: SystemTime) {
        while let Some(front) = self.sensor_queue.front() {
            if front.at < cutoff {
                self.sensor_queue.pop_front();
            } else {
                break;
            }
        }
        while let Some(front) = self.command_queue.front() {
            if front.at < cutoff {
                self.command_queue.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn purge_older_than_duration(&mut self, d: Duration) {
        if let Ok(now) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            let cutoff = SystemTime::UNIX_EPOCH + now.saturating_sub(d);
            self.purge_older_than(cutoff);
        }
    }
}

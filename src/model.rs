use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

pub const CPU_HISTORY_LEN: usize = 60; // 60 samples ~ 1 minute at 1s ticks

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub completed: bool,
    pub created_at: u64,
    pub completed_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String,
    pub cmdline: String,
    pub exe: String,
    pub cpu_percent: f32,
    pub mem_bytes: u64,
    pub status: String,
    pub group: ProcessGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProcessGroup {
    Application,
    #[default]
    Background,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemStats {
    pub cpu_percent: f32,
    pub cpu_per_core: Vec<f32>,
    pub mem_used: u64,
    pub mem_total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
    pub load_1m: f64,
    pub load_5m: f64,
    pub load_15m: f64,
    pub process_count: usize,
    pub uptime_secs: u64,
    pub cpu_history: VecDeque<f32>,
    #[serde(skip)]
    pub per_core_history: Vec<VecDeque<f32>>,
}

impl SystemStats {
    pub fn new(core_count: usize) -> Self {
        Self {
            per_core_history: vec![VecDeque::new(); core_count],
            ..Default::default()
        }
    }

    pub fn mem_percent(&self) -> f32 {
        if self.mem_total == 0 {
            0.0
        } else {
            (self.mem_used as f32 / self.mem_total as f32) * 100.0
        }
    }

    pub fn swap_percent(&self) -> f32 {
        if self.swap_total == 0 {
            0.0
        } else {
            (self.swap_used as f32 / self.swap_total as f32) * 100.0
        }
    }

    pub fn format_uptime(&self) -> String {
        let total = self.uptime_secs;
        let h = total / 3600;
        let m = (total % 3600) / 60;
        let s = total % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    }

    pub fn format_load(&self) -> String {
        format!(
            "{:.2} {:.2} {:.2}",
            self.load_1m, self.load_5m, self.load_15m
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub timestamp: u64,
    pub processes: Vec<ProcessEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppData {
    pub tasks: Vec<Task>,
    pub next_task_id: u64,
    #[serde(default)]
    pub snapshots: Vec<ProcessSnapshot>,
}

impl AppData {
    pub fn add_task(&mut self, title: String) {
        let id = self.next_task_id;
        self.next_task_id += 1;
        self.tasks.push(Task {
            id,
            title,
            completed: false,
            created_at: now_secs(),
            completed_at: None,
        });
    }

    pub fn toggle_complete(&mut self, id: u64) -> bool {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            if !t.completed {
                t.completed = true;
                t.completed_at = Some(now_secs());
            } else {
                t.completed = false;
                t.completed_at = None;
            }
            return true;
        }
        false
    }

    pub fn delete_task(&mut self, id: u64) -> bool {
        let len = self.tasks.len();
        self.tasks.retain(|t| t.id != id);
        self.tasks.len() != len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_task_increments_id() {
        let mut data = AppData::default();
        data.add_task("first".to_string());
        data.add_task("second".to_string());
        assert_eq!(data.tasks.len(), 2);
        assert_eq!(data.tasks[0].id, 0);
        assert_eq!(data.tasks[1].id, 1);
        assert_eq!(data.next_task_id, 2);
    }

    #[test]
    fn toggle_complete_flips() {
        let mut data = AppData::default();
        data.add_task("t".to_string());
        assert!(!data.tasks[0].completed);
        data.toggle_complete(0);
        assert!(data.tasks[0].completed);
        assert!(data.tasks[0].completed_at.is_some());
        data.toggle_complete(0);
        assert!(!data.tasks[0].completed);
        assert!(data.tasks[0].completed_at.is_none());
    }

    #[test]
    fn delete_task_removes() {
        let mut data = AppData::default();
        data.add_task("a".to_string());
        data.add_task("b".to_string());
        assert!(data.delete_task(0));
        assert_eq!(data.tasks.len(), 1);
        assert_eq!(data.tasks[0].id, 1);
        assert!(!data.delete_task(999));
    }

    #[test]
    fn mem_percent_handles_zero_total() {
        let stats = SystemStats::default();
        assert_eq!(stats.mem_percent(), 0.0);
    }

    #[test]
    fn format_uptime_pads() {
        let stats = SystemStats {
            uptime_secs: 5,
            ..Default::default()
        };
        assert_eq!(stats.format_uptime(), "00:00:05");
    }
}

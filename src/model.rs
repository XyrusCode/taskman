use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

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
}

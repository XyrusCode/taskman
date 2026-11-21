use crate::model::{AppData, ProcessEntry, ProcessSnapshot, Task};
use crate::storage::{load, save};
use crossterm::event::{KeyCode, KeyEvent};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Tasks,
    Processes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Cpu,
    Mem,
    Name,
}

pub struct App {
    pub data: AppData,
    pub view: View,
    pub task_selected: usize,
    pub proc_selected: usize,
    pub sort_by: SortBy,
    pub processes: Vec<ProcessEntry>,
    pub status: String,
    pub input: String,
    pub input_mode: bool,
    pub sys: sysinfo::System,
}

impl App {
    pub fn new() -> std::io::Result<Self> {
        let data = load()?;
        let mut sys = sysinfo::System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        Ok(App {
            data,
            view: View::Tasks,
            task_selected: 0,
            proc_selected: 0,
            sort_by: SortBy::Cpu,
            processes: Vec::new(),
            status: String::new(),
            input: String::new(),
            input_mode: false,
            sys,
        })
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status = msg.into();
    }

    // ---- Tasks ----

    pub fn selected_task(&self) -> Option<&Task> {
        self.data.tasks.get(self.task_selected)
    }

    pub fn begin_add_task(&mut self) {
        self.input_mode = true;
        self.input.clear();
    }

    pub fn add_task(&mut self) {
        if self.input.trim().is_empty() {
            self.set_status("Task title cannot be empty");
            return;
        }
        self.data.add_task(self.input.trim().to_string());
        self.input.clear();
        self.input_mode = false;
        self.task_selected = self.data.tasks.len().saturating_sub(1);
        if let Err(e) = save(&self.data) {
            self.set_status(format!("Save failed: {}", e));
        } else {
            self.set_status("Task added");
        }
    }

    pub fn cancel_add_task(&mut self) {
        self.input.clear();
        self.input_mode = false;
    }

    pub fn toggle_complete(&mut self) {
        if let Some(id) = self.selected_task().map(|t| t.id) {
            self.data.toggle_complete(id);
            if let Err(e) = save(&self.data) {
                self.set_status(format!("Save failed: {}", e));
            } else {
                self.set_status("Task toggled");
            }
        }
    }

    pub fn delete_selected_task(&mut self) {
        if let Some(id) = self.selected_task().map(|t| t.id) {
            self.data.delete_task(id);
            if self.task_selected >= self.data.tasks.len() && !self.data.tasks.is_empty() {
                self.task_selected = self.data.tasks.len() - 1;
            }
            if let Err(e) = save(&self.data) {
                self.set_status(format!("Save failed: {}", e));
            } else {
                self.set_status("Task deleted");
            }
        }
    }

    // ---- Processes ----

    pub fn refresh_processes(&mut self) -> std::io::Result<()> {
        self.sys
            .refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let mut entries: Vec<ProcessEntry> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, proc)| ProcessEntry {
                pid: pid.as_u32(),
                name: proc.name().to_string_lossy().to_string(),
                cpu_percent: proc.cpu_usage(),
                mem_bytes: proc.memory(),
                status: format!("{:?}", proc.status()),
            })
            .collect();

        match self.sort_by {
            SortBy::Cpu => entries.sort_by(|a, b| {
                b.cpu_percent
                    .partial_cmp(&a.cpu_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            SortBy::Mem => entries.sort_by_key(|a| std::cmp::Reverse(a.mem_bytes)),
            SortBy::Name => entries.sort_by(|a, b| a.name.cmp(&b.name)),
        }

        self.processes = entries;

        // Keep a rolling snapshot history (cap at 60 samples ~ 1 minute at 1s ticks).
        let snapshot = ProcessSnapshot {
            timestamp: now_secs(),
            processes: self.processes.clone(),
        };
        self.data.snapshots.push(snapshot);
        if self.data.snapshots.len() > 60 {
            let drop = self.data.snapshots.len() - 60;
            self.data.snapshots.drain(0..drop);
        }

        if let Err(e) = save(&self.data) {
            self.set_status(format!("Save failed: {}", e));
        }
        Ok(())
    }

    pub fn selected_process(&self) -> Option<&ProcessEntry> {
        self.processes.get(self.proc_selected)
    }

    pub fn kill_selected(&mut self) {
        if let Some(proc) = self.selected_process() {
            let pid = proc.pid;
            let killed = match self.sys.process(sysinfo::Pid::from_u32(proc.pid)) {
                Some(p) => p.kill(),
                None => false,
            };
            if killed {
                self.set_status(format!("Killed PID {}", pid));
            } else {
                self.set_status(format!("Failed to kill PID {}", pid));
            }
            let _ = self.refresh_processes();
        }
    }

    pub fn cycle_sort(&mut self) {
        self.sort_by = match self.sort_by {
            SortBy::Cpu => SortBy::Mem,
            SortBy::Mem => SortBy::Name,
            SortBy::Name => SortBy::Cpu,
        };
        let _ = self.refresh_processes();
    }

    // ---- Input ----

    pub fn handle_key(&mut self, key: KeyEvent) -> std::io::Result<bool> {
        // Returns true when the app should quit.

        if self.input_mode {
            match key.code {
                KeyCode::Enter => {
                    self.add_task();
                }
                KeyCode::Esc => {
                    self.cancel_add_task();
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('1') => self.view = View::Tasks,
            KeyCode::Char('2') => self.view = View::Processes,
            KeyCode::Tab => {
                self.view = match self.view {
                    View::Tasks => View::Processes,
                    View::Processes => View::Tasks,
                };
            }
            KeyCode::Char('j') | KeyCode::Down => match self.view {
                View::Tasks => {
                    if !self.data.tasks.is_empty() {
                        self.task_selected =
                            (self.task_selected + 1).min(self.data.tasks.len() - 1);
                    }
                }
                View::Processes => {
                    if !self.processes.is_empty() {
                        self.proc_selected = (self.proc_selected + 1).min(self.processes.len() - 1);
                    }
                }
            },
            KeyCode::Char('k') | KeyCode::Up => {
                self.task_selected = self.task_selected.saturating_sub(1);
                self.proc_selected = self.proc_selected.saturating_sub(1);
            }
            KeyCode::Char('a') => {
                self.begin_add_task();
            }
            KeyCode::Char('c') | KeyCode::Enter => {
                if self.view == View::Tasks {
                    self.toggle_complete();
                }
            }
            KeyCode::Char('d') | KeyCode::Char('x') => {
                if self.view == View::Tasks {
                    self.delete_selected_task();
                } else {
                    self.kill_selected();
                }
            }
            KeyCode::Char('s') => {
                if self.view == View::Processes {
                    self.cycle_sort();
                }
            }
            KeyCode::Char('r') if self.view == View::Processes => {
                self.refresh_processes()?;
            }
            _ => {}
        }
        Ok(false)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("failed to initialise app")
    }
}

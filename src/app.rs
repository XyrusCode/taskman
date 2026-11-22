use crate::model::{
    AppData, ProcessEntry, ProcessGroup, ProcessSnapshot, SystemStats, Task, CPU_HISTORY_LEN,
};
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
    Summary,
    Cpu,
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
    pub stats: SystemStats,
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
        sys.refresh_all();

        let core_count = sys.cpus().len();
        let mut stats = SystemStats::new(core_count);
        Self::gather_stats(&mut sys, &mut stats);

        Ok(App {
            data,
            stats,
            view: View::Summary,
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

    // ---- System stats ----

    fn gather_stats(sys: &mut sysinfo::System, stats: &mut SystemStats) {
        sys.refresh_cpu_all();
        sys.refresh_memory();

        stats.cpu_percent = sys.global_cpu_usage();
        stats.cpu_per_core = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
        stats.mem_used = sys.used_memory();
        stats.mem_total = sys.total_memory();
        stats.swap_used = sys.used_swap();
        stats.swap_total = sys.total_swap();
        let load = sysinfo::System::load_average();
        stats.load_1m = load.one;
        stats.load_5m = load.five;
        stats.load_15m = load.fifteen;
        stats.process_count = sys.processes().len();
        stats.uptime_secs = sysinfo::System::uptime();

        // Roll CPU history.
        stats.cpu_history.push_back(stats.cpu_percent);
        if stats.cpu_history.len() > CPU_HISTORY_LEN {
            stats.cpu_history.pop_front();
        }

        // Roll per-core history.
        for (i, core) in sys.cpus().iter().enumerate() {
            if i < stats.per_core_history.len() {
                stats.per_core_history[i].push_back(core.cpu_usage());
                if stats.per_core_history[i].len() > CPU_HISTORY_LEN {
                    stats.per_core_history[i].pop_front();
                }
            }
        }
    }

    pub fn refresh_all(&mut self) -> std::io::Result<()> {
        Self::gather_stats(&mut self.sys, &mut self.stats);

        // Refresh processes (grouped apps-first).
        self.sys
            .refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let entries: Vec<ProcessEntry> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, proc)| {
                let cmdline = Self::cmdline_str(proc);
                let exe = Self::exe_str(proc);
                let group =
                    Self::classify_group(proc.name().to_string_lossy().as_ref(), &cmdline, &exe);
                ProcessEntry {
                    pid: pid.as_u32(),
                    name: proc.name().to_string_lossy().to_string(),
                    cmdline,
                    exe,
                    cpu_percent: proc.cpu_usage(),
                    mem_bytes: proc.memory(),
                    status: format!("{:?}", proc.status()),
                    group,
                }
            })
            .collect();

        let mut apps: Vec<ProcessEntry> = Vec::new();
        let mut background: Vec<ProcessEntry> = Vec::new();
        for e in entries {
            match e.group {
                ProcessGroup::Application => apps.push(e),
                ProcessGroup::Background => background.push(e),
            }
        }
        let sort = |v: &mut Vec<ProcessEntry>| match self.sort_by {
            SortBy::Cpu => v.sort_by(|a, b| {
                b.cpu_percent
                    .partial_cmp(&a.cpu_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            SortBy::Mem => v.sort_by_key(|a| std::cmp::Reverse(a.mem_bytes)),
            SortBy::Name => v.sort_by(|a, b| a.name.cmp(&b.name)),
        };
        sort(&mut apps);
        sort(&mut background);

        self.processes = apps;
        self.processes.append(&mut background);

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

    fn classify_group(name: &str, cmdline: &str, exe: &str) -> ProcessGroup {
        let lower = name.to_lowercase();
        let cmd_lower = cmdline.to_lowercase();
        let exe_lower = exe.to_lowercase();

        let background_markers = [
            "kthreadd",
            "ksoftirqd",
            "migration",
            "rcu",
            "watchdog",
            "systemd",
            "dbus",
            "udisks",
            "accounts",
            "polkit",
            "chrony",
            "irqbalance",
            "vmstat",
            "snapd",
        ];
        if background_markers
            .iter()
            .any(|m| lower.contains(m) || cmd_lower.contains(m))
        {
            return ProcessGroup::Background;
        }

        let app_markers = [
            "code",
            "opencode",
            "ai.opencode",
            "vim",
            "nvim",
            "emacs",
            "nano",
            "firefox",
            "chrome",
            "chromium",
            "gnome-terminal",
            "konsole",
            "kitty",
            "alacritty",
            "wezterm",
            "tilix",
            "nautilus",
            "dolphin",
            "thunderbird",
            "spotify",
            "discord",
            "telegram",
        ];
        if app_markers
            .iter()
            .any(|m| lower.contains(m) || cmd_lower.contains(m) || exe_lower.contains(m))
        {
            return ProcessGroup::Application;
        }

        if !cmdline.is_empty() && !lower.starts_with('[') {
            return ProcessGroup::Application;
        }

        ProcessGroup::Background
    }

    fn cmdline_str(proc: &sysinfo::Process) -> String {
        proc.cmd()
            .iter()
            .map(|a| a.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn exe_str(proc: &sysinfo::Process) -> String {
        proc.exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
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
            let _ = self.refresh_all();
        }
    }

    pub fn cycle_sort(&mut self) {
        self.sort_by = match self.sort_by {
            SortBy::Cpu => SortBy::Mem,
            SortBy::Mem => SortBy::Name,
            SortBy::Name => SortBy::Cpu,
        };
        let _ = self.refresh_all();
    }

    pub fn cycle_view(&mut self) {
        self.view = match self.view {
            View::Summary => View::Cpu,
            View::Cpu => View::Processes,
            View::Processes => View::Summary,
        };
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
            KeyCode::Char('1') => self.view = View::Summary,
            KeyCode::Char('2') => self.view = View::Cpu,
            KeyCode::Char('3') => self.view = View::Processes,
            KeyCode::Tab => self.cycle_view(),
            KeyCode::Char('j') | KeyCode::Down => {
                if self.view == View::Processes && !self.processes.is_empty() {
                    self.proc_selected = (self.proc_selected + 1).min(self.processes.len() - 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.task_selected = self.task_selected.saturating_sub(1);
                self.proc_selected = self.proc_selected.saturating_sub(1);
            }
            KeyCode::Char('a') => {
                self.begin_add_task();
            }
            KeyCode::Char('c') | KeyCode::Enter => {
                if self.view == View::Summary {
                    self.toggle_complete();
                }
            }
            KeyCode::Char('d') | KeyCode::Char('x') => match self.view {
                View::Summary => self.delete_selected_task(),
                View::Processes => self.kill_selected(),
                View::Cpu => {}
            },
            KeyCode::Char('s') => {
                if self.view == View::Processes {
                    self.cycle_sort();
                }
            }
            KeyCode::Char('r') => {
                self.refresh_all()?;
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

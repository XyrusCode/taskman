use crate::app::{App, SortBy, View};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Sparkline};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tab bar
            Constraint::Min(0),    // body
            Constraint::Length(3), // help line
            Constraint::Length(1), // status bar
        ])
        .split(f.area());

    draw_tab_bar(f, app, chunks[0]);
    match app.view {
        View::Summary => draw_summary(f, app, chunks[1]),
        View::Cpu => draw_cpu(f, app, chunks[1]),
        View::Processes => draw_processes(f, app, chunks[1]),
    }
    draw_help(f, app, chunks[2]);
    draw_status(f, app, chunks[3]);
}

fn draw_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let summary_style = if app.view == View::Summary {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let cpu_style = if app.view == View::Cpu {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let proc_style = if app.view == View::Processes {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let items = [
        ListItem::new(" 1 Summary ").style(summary_style),
        ListItem::new(" 2 CPU ").style(cpu_style),
        ListItem::new(" 3 Processes ").style(proc_style),
    ];
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("taskman"));
    f.render_widget(list, area);
}

fn draw_summary(f: &mut Frame, app: &App, area: Rect) {
    let s = &app.stats;

    let cpu_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("CPU  {:.1}%", s.cpu_percent)),
        )
        .gauge_style(Style::default().fg(Color::Green))
        .ratio((s.cpu_percent / 100.0) as f64);

    let mem_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(format!(
            "MEM  {:.1}%  ({} / {} MB)",
            s.mem_percent(),
            s.mem_used / 1_048_576,
            s.mem_total / 1_048_576
        )))
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio((s.mem_percent() / 100.0) as f64);

    let swap_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(format!(
            "SWAP {:.1}%  ({} / {} MB)",
            s.swap_percent(),
            s.swap_used / 1_048_576,
            s.swap_total / 1_048_576
        )))
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio((s.swap_percent() / 100.0) as f64);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    f.render_widget(cpu_gauge, top_chunks[0]);
    f.render_widget(mem_gauge, top_chunks[1]);
    f.render_widget(swap_gauge, top_chunks[2]);

    // Info block below gauges.
    let info = [
        format!("Load avg:        {}", s.format_load()),
        format!("Processes:       {}", s.process_count),
        format!("Uptime:          {}", s.format_uptime()),
        format!(
            "Tasks: {} ({} done)",
            app.data.tasks.len(),
            app.data.tasks.iter().filter(|t| t.completed).count()
        ),
        "Version:         v0.4.0".to_string(),
    ];
    let info_text = info.join("\n");
    let info_block =
        Paragraph::new(info_text).block(Block::default().borders(Borders::ALL).title("System"));
    let info_area = Rect {
        x: area.x,
        y: area.y + 6,
        width: area.width,
        height: area.height.saturating_sub(6),
    };
    f.render_widget(info_block, info_area);
}

fn draw_cpu(f: &mut Frame, app: &App, area: Rect) {
    let s = &app.stats;

    // Overall CPU sparkline.
    let cpu_data: Vec<u64> = s.cpu_history.iter().map(|v| *v as u64).collect();
    let cpu_spark = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Overall CPU  {:.1}%", s.cpu_percent)),
        )
        .data(&cpu_data)
        .style(Style::default().fg(Color::Green));

    // Per-core sparklines.
    let core_count = s.cpu_per_core.len();
    let mut core_chunks: Vec<Rect> = Vec::new();
    if core_count <= 4 {
        core_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                (0..core_count)
                    .map(|_| Constraint::Length(3))
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
            .split(area)
            .to_vec();
    } else {
        let half = core_count.div_ceil(2);
        let left: Vec<Rect> = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                (0..half)
                    .map(|_| Constraint::Length(3))
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
            .split(area)
            .to_vec();
        let right: Vec<Rect> = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                (half..core_count)
                    .map(|_| Constraint::Length(3))
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
            .split(area)
            .to_vec();
        core_chunks.extend(left);
        core_chunks.extend(right);
    }

    let top_height = 6;
    let cpu_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: top_height,
    };
    let cores_area = Rect {
        x: area.x,
        y: area.y + top_height,
        width: area.width,
        height: area.height.saturating_sub(top_height),
    };

    f.render_widget(cpu_spark, cpu_area);

    // Render per-core sparklines in a grid.
    if core_count <= 4 {
        for (i, core_area) in core_chunks.iter().enumerate() {
            if i < s.cpu_per_core.len() {
                let usage = s.cpu_per_core[i];
                let data: Vec<u64> = s.per_core_history[i].iter().map(|v| *v as u64).collect();
                let spark = Sparkline::default()
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("CPU {}  {:.1}%", i, usage)),
                    )
                    .data(&data)
                    .style(Style::default().fg(Color::Cyan));
                f.render_widget(spark, *core_area);
            }
        }
    } else {
        // Two columns.
        let half = core_count.div_ceil(2);
        let mid = cores_area.width / 2;
        for (i, core_area) in core_chunks.iter().enumerate() {
            if i < s.cpu_per_core.len() {
                let usage = s.cpu_per_core[i];
                let data: Vec<u64> = s.per_core_history[i].iter().map(|v| *v as u64).collect();
                let spark = Sparkline::default()
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("CPU {}  {:.1}%", i, usage)),
                    )
                    .data(&data)
                    .style(Style::default().fg(Color::Cyan));
                // Adjust x for right column.
                let adjusted = if i >= half {
                    Rect {
                        x: cores_area.x + mid,
                        y: core_area.y,
                        width: cores_area.width - mid,
                        height: core_area.height,
                    }
                } else {
                    *core_area
                };
                f.render_widget(spark, adjusted);
            }
        }
    }

    // Legend.
    let legend = Paragraph::new("1 Summary  2 CPU  3 Processes  j/k: navigate (Processes)  x: kill  s: sort  r: refresh  q: quit")
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(
        legend,
        Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(2),
            width: area.width,
            height: 1,
        },
    );
}

fn draw_processes(f: &mut Frame, app: &App, area: Rect) {
    let app_count = app
        .processes
        .iter()
        .filter(|p| p.group == crate::model::ProcessGroup::Application)
        .count();

    let mut items: Vec<ListItem> = Vec::new();
    let mut proc_to_display: Vec<usize> = Vec::with_capacity(app.processes.len());

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let mut last_group: Option<crate::model::ProcessGroup> = None;
    for (i, p) in app.processes.iter().enumerate() {
        if last_group != Some(p.group) {
            let label = match p.group {
                crate::model::ProcessGroup::Application => "Applications",
                crate::model::ProcessGroup::Background => "Background",
            };
            items.push(
                ListItem::new(format!("{} ({})", label, section_count(app, p.group)))
                    .style(header_style),
            );
            last_group = Some(p.group);
        }
        proc_to_display.push(items.len());
        items.push(process_row(p, i == app.proc_selected));
    }

    let selected_display = proc_to_display.get(app.proc_selected).copied();

    let sort_label = match app.sort_by {
        SortBy::Cpu => "CPU",
        SortBy::Mem => "MEM",
        SortBy::Name => "NAME",
    };

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Processes ({} total, {} apps) | sort: {}",
            app.processes.len(),
            app_count,
            sort_label
        )));

    let mut state = ListState::default();
    state.select(selected_display);
    f.render_stateful_widget(list, area, &mut state);
}

fn section_count(app: &App, group: crate::model::ProcessGroup) -> usize {
    app.processes.iter().filter(|p| p.group == group).count()
}

fn process_row<'a>(p: &'a crate::model::ProcessEntry, selected: bool) -> ListItem<'a> {
    let style = if selected {
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let group_mark = if p.group == crate::model::ProcessGroup::Application {
        "●"
    } else {
        "○"
    };
    let mem_mb = p.mem_bytes / 1_048_576;
    let cmd = if p.cmdline.len() > 38 {
        format!("{}…", &p.cmdline[..38])
    } else {
        p.cmdline.clone()
    };
    ListItem::new(format!(
        "{} {:>5} {:<22} {:<40} {:>5.1} {:>6} MB",
        group_mark, p.pid, p.name, cmd, p.cpu_percent, mem_mb
    ))
    .style(style)
}

fn draw_help(f: &mut Frame, app: &App, area: Rect) {
    let help: String = if app.input_mode {
        format!("> {}  (Enter: add, Esc: cancel)", app.input)
    } else {
        match app.view {
            View::Summary => {
                "j/k: navigate  a: add  c/Enter: complete  d/x: delete  1/2/3: view  q: quit"
                    .to_string()
            }
            View::Cpu => "j/k: navigate  1/2/3: view  r: refresh  q: quit".to_string(),
            View::Processes => {
                "j/k: navigate  x: kill  s: sort  r: refresh  1/2/3: view  q: quit".to_string()
            }
        }
    };
    let p = Paragraph::new(help).block(Block::default().borders(Borders::NONE));
    f.render_widget(p, area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let text = match app.view {
        View::Summary => format!(
            "Tasks: {} ({} done)  |  CPU {:.1}%  |  MEM {:.1}%  |  v0.4.0  |  {}",
            app.data.tasks.len(),
            app.data.tasks.iter().filter(|t| t.completed).count(),
            app.stats.cpu_percent,
            app.stats.mem_percent(),
            app.status
        ),
        View::Cpu => format!(
            "CPU {:.1}%  |  {} cores  |  v0.4.0  |  {}",
            app.stats.cpu_percent,
            app.stats.cpu_per_core.len(),
            app.status
        ),
        View::Processes => format!(
            "Processes: {}  |  CPU {:.1}%  |  MEM {:.1}%  |  v0.4.0  |  {}",
            app.processes.len(),
            app.stats.cpu_percent,
            app.stats.mem_percent(),
            app.status
        ),
    };
    let p = Paragraph::new(text).block(Block::default().borders(Borders::NONE));
    f.render_widget(p, area);
}

use crate::app::{App, SortBy, View};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
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
        View::Tasks => draw_tasks(f, app, chunks[1]),
        View::Processes => draw_processes(f, app, chunks[1]),
    }
    draw_help(f, app, chunks[2]);
    draw_status(f, app, chunks[3]);
}

fn draw_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let tasks_style = if app.view == View::Tasks {
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
        ListItem::new(" 1 Tasks ").style(tasks_style),
        ListItem::new(" 2 Processes ").style(proc_style),
    ];
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("taskman"));
    f.render_widget(list, area);
}

fn draw_tasks(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .data
        .tasks
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let marker = if t.completed { "[x]" } else { "[ ]" };
            let selected = i == app.task_selected;
            let style = if selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else if t.completed {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            ListItem::new(format!("{} {}", marker, t.title)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Tasks"));
    f.render_widget(list, area);
}

fn draw_processes(f: &mut Frame, app: &App, area: Rect) {
    let app_count = app
        .processes
        .iter()
        .filter(|p| p.group == crate::model::ProcessGroup::Application)
        .count();

    // Build a single list with group headers inserted on group transition.
    // proc_to_display maps process index (in app.processes) -> display index.
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

/// Count processes of a given group.
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
            View::Tasks => {
                "j/k: navigate  a: add  c/Enter: complete  d/x: delete  1/2/Tab: view  q: quit"
                    .to_string()
            }
            View::Processes => {
                "j/k: navigate  x: kill  s: sort  r: refresh  1/2/Tab: view  q: quit".to_string()
            }
        }
    };
    let p = Paragraph::new(help).block(Block::default().borders(Borders::NONE));
    f.render_widget(p, area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let text = match app.view {
        View::Tasks => format!(
            "Tasks: {} ({} done)  |  v0.3.0  |  {}",
            app.data.tasks.len(),
            app.data.tasks.iter().filter(|t| t.completed).count(),
            app.status
        ),
        View::Processes => format!(
            "Processes: {}  |  v0.3.0  |  {}",
            app.processes.len(),
            app.status
        ),
    };
    let p = Paragraph::new(text).block(Block::default().borders(Borders::NONE));
    f.render_widget(p, area);
}

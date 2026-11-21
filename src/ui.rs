use crate::app::{App, SortBy, View};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table};
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
    let header = ["PID", "NAME", "CPU%", "MEM", "STATUS"];
    let rows: Vec<Row> = app
        .processes
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let selected = i == app.proc_selected;
            let style = if selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let mem_mb = p.mem_bytes / 1_048_576;
            Row::new([
                Cell::from(p.pid.to_string()).style(style),
                Cell::from(p.name.clone()).style(style),
                Cell::from(format!("{:.1}", p.cpu_percent)).style(style),
                Cell::from(format!("{} MB", mem_mb)).style(style),
                Cell::from(p.status.clone()).style(style),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(6),
        Constraint::Length(24),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(12),
    ];

    let sort_label = match app.sort_by {
        SortBy::Cpu => "CPU",
        SortBy::Mem => "MEM",
        SortBy::Name => "NAME",
    };

    let table = Table::new(rows, widths)
        .header(
            Row::new(header.iter().map(|h| Cell::from(*h)))
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Processes (sort: {})", sort_label)),
        );

    f.render_widget(table, area);
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
            "Tasks: {} ({} done)  |  v0.1.0  |  {}",
            app.data.tasks.len(),
            app.data.tasks.iter().filter(|t| t.completed).count(),
            app.status
        ),
        View::Processes => format!(
            "Processes: {}  |  v0.1.0  |  {}",
            app.processes.len(),
            app.status
        ),
    };
    let p = Paragraph::new(text).block(Block::default().borders(Borders::NONE));
    f.render_widget(p, area);
}

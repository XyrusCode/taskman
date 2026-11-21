mod app;
mod model;
mod storage;
mod ui;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if let Err(e) = result {
        eprintln!("taskman: {}", e);
        std::process::exit(1);
    }
    Ok(())
}

use crate::app::View;

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::event::Event;
    use std::time::Duration;

    let mut app = app::App::new()?;
    let mut running = true;

    while running {
        terminal.draw(|f| ui::draw(f, &app))?;

        if crossterm::event::poll(Duration::from_millis(1000))? {
            if let Event::Key(key) = crossterm::event::read()? {
                if app.handle_key(key)? {
                    running = false;
                }
            }
        } else {
            // Timer tick: refresh process data (no input pending).
            // Like the original Task Manager, we only do work when there is
            // something to show, never busy-poll.
            if app.view == View::Processes {
                app.refresh_processes()?;
            }
        }
    }

    Ok(())
}

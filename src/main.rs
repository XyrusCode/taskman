mod app;
mod model;
mod storage;
mod ui;

use clap::{Parser, Subcommand};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};

#[derive(Parser)]
#[command(name = "taskman")]
#[command(about = "Lightweight terminal task manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add { title: String },
    List,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Add { title }) => {
            let mut data = storage::load()?;
            data.add_task(title);
            storage::save(&data)?;
            println!("Task added");
        }
        Some(Commands::List) => {
            let data = storage::load()?;
            if data.tasks.is_empty() {
                println!("No tasks yet. Add one with: taskman add <title>");
            } else {
                for t in &data.tasks {
                    let marker = if t.completed { "[x]" } else { "[ ]" };
                    println!("{} {} {}", marker, t.id, t.title);
                }
            }
        }
        None => run_tui()?,
    }
    Ok(())
}

fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
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
            // Timer tick: refresh system stats and processes (no input pending).
            app.refresh_all()?;
        }
    }

    Ok(())
}

mod app;
mod handler;
mod queries;
mod theme;
mod views;

pub use app::App;

use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;

/// Launch the TUI. Blocks until the user quits.
pub fn run(db_path: &Path) -> io::Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(db_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let result = run_loop(&mut terminal, &mut app);

    // Terminal teardown — always runs even if the loop errored.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);
    let refresh_interval = Duration::from_secs(2);
    let mut last_refresh = Instant::now();

    loop {
        terminal.draw(|frame| views::draw(frame, app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handler::handle_key(app, key);
                }
            }
        }

        // Auto-refresh from DB to pick up changes from concurrent processes.
        if last_refresh.elapsed() >= refresh_interval {
            app.refresh();
            last_refresh = Instant::now();
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}

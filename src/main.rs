use std::io;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use chattui::ChatApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = ChatApp::new();

    loop {
        terminal.draw(|f| {
            app.render(f);
        })?;

        if let Some((x, y)) = app.get_cursor_pos() {
            execute!(terminal.backend_mut(), MoveTo(x, y), Show)?;
        } else {
            execute!(terminal.backend_mut(), Hide)?;
        }

        if let Event::Key(key) = event::read()? {
            app.on_key(key);
        }

        if app.should_quit() {
            break;
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        Show
    )?;
    terminal.show_cursor()?;

    Ok(())
}

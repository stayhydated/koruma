use std::io;

use crossterm::event::{self, Event, KeyCode as CrosstermKeyCode, KeyEvent, KeyEventKind};
use koruma_collection_example_core::{App, KeyCode, init_i18n};
use ratatui::DefaultTerminal;

fn main() -> io::Result<()> {
    init_i18n();
    let mut terminal = ratatui::init();
    let result = run_native(&mut terminal);
    ratatui::restore();
    result
}

fn run_native(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();

    while !app.should_exit() {
        terminal.draw(|frame| app.render(frame))?;
        if let Event::Key(key) = event::read()? {
            handle_key_event(&mut app, key);
        }
    }
    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }
    let code = match key.code {
        CrosstermKeyCode::Char(c) => KeyCode::Char(c),
        CrosstermKeyCode::Backspace => KeyCode::Backspace,
        CrosstermKeyCode::Delete => KeyCode::Delete,
        CrosstermKeyCode::Home => KeyCode::Home,
        CrosstermKeyCode::End => KeyCode::End,
        CrosstermKeyCode::PageUp => KeyCode::PageUp,
        CrosstermKeyCode::PageDown => KeyCode::PageDown,
        CrosstermKeyCode::Up => KeyCode::Up,
        CrosstermKeyCode::Down => KeyCode::Down,
        CrosstermKeyCode::Esc => KeyCode::Esc,
        CrosstermKeyCode::Enter => KeyCode::Enter,
        CrosstermKeyCode::Tab => KeyCode::Tab,
        _ => return,
    };
    app.handle_key_code(code);
}

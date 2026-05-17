use std::io;

use collection_ratatui_core::{App, I18n, KeyCode, init_i18n};
use crossterm::event::{self, Event, KeyCode as CrosstermKeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;

fn main() -> io::Result<()> {
    let i18n = init_i18n();
    let mut terminal = ratatui::init();
    let result = run_native(&mut terminal, i18n);
    ratatui::restore();
    result
}

fn run_native(terminal: &mut DefaultTerminal, i18n: I18n) -> io::Result<()> {
    let mut app = App::with_i18n(i18n);

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

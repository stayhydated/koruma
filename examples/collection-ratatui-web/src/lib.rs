use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use collection_ratatui_core::{App, KeyCode, init_i18n};
use ratatui::Terminal;
use ratzilla::event::KeyCode as RatzillaKeyCode;
use ratzilla::{DomBackend, WebRenderer};

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    run().unwrap();
}

fn run() -> io::Result<()> {
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    let app = Rc::new(RefCell::new(App::with_i18n(init_i18n())));

    terminal.on_key_event({
        let app = app.clone();
        move |key_event| {
            let code = match key_event.code {
                RatzillaKeyCode::Char(c) => KeyCode::Char(c),
                RatzillaKeyCode::Backspace => KeyCode::Backspace,
                RatzillaKeyCode::Delete => KeyCode::Delete,
                RatzillaKeyCode::Home => KeyCode::Home,
                RatzillaKeyCode::End => KeyCode::End,
                RatzillaKeyCode::PageUp => KeyCode::PageUp,
                RatzillaKeyCode::PageDown => KeyCode::PageDown,
                RatzillaKeyCode::Up => KeyCode::Up,
                RatzillaKeyCode::Down => KeyCode::Down,
                RatzillaKeyCode::Esc => KeyCode::Esc,
                RatzillaKeyCode::Enter => KeyCode::Enter,
                RatzillaKeyCode::Tab => KeyCode::Tab,
                _ => return,
            };
            app.borrow_mut().handle_key_code(code);
        }
    });

    terminal.draw_web(move |f| {
        app.borrow().render(f);
    });

    Ok(())
}

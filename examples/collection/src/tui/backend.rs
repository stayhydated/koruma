use std::io;

#[cfg(feature = "native")]
pub use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

#[cfg(feature = "web")]
pub use ratzilla::event::{KeyCode, KeyEvent};

#[cfg(feature = "native")]
pub type Terminal = ratatui::DefaultTerminal;

#[cfg(feature = "web")]
pub type Terminal = ratatui::Terminal<ratzilla::DomBackend>;

#[cfg(feature = "native")]
pub fn init() -> io::Result<Terminal> {
    ratatui::init();
    Ok(ratatui::init())
}

#[cfg(feature = "web")]
pub fn init() -> io::Result<Terminal> {
    let backend = ratzilla::DomBackend::new()?;
    ratatui::Terminal::new(backend)
}

#[cfg(feature = "native")]
pub fn restore() {
    ratatui::restore();
}

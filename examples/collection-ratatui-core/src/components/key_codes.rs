#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyCode {
    Char(char),
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Esc,
    Enter,
    Tab,
}

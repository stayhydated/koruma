use std::sync::Once;

use collection_ratatui_core::{App, KeyCode, init_i18n};
use ratatui::{Terminal, backend::TestBackend};

static INIT_I18N: Once = Once::new();

fn setup_i18n() {
    INIT_I18N.call_once(init_i18n);
}

fn render_output(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test backend should initialize");
    terminal
        .draw(|frame| app.render(frame))
        .expect("app render should succeed");
    format!("{}", terminal.backend())
}

fn line_after<'a>(output: &'a str, title: &str) -> &'a str {
    let lines: Vec<_> = output.lines().collect();
    let idx = lines
        .iter()
        .position(|line| line.contains(title))
        .unwrap_or_else(|| panic!("missing line containing {title:?}\n{output}"));

    lines
        .get(idx + 1)
        .copied()
        .unwrap_or_else(|| panic!("missing line after {title:?}\n{output}"))
}

fn line_has_visible_text(line: &str) -> bool {
    line.chars().any(|ch| ch.is_alphanumeric())
}

#[test]
fn compact_layout_keeps_module_and_display_visible() {
    setup_i18n();

    let app = App::new();
    let output = render_output(&app, 100, 20);

    assert!(line_after(&output, " Module ").contains("String"));
    assert!(line_has_visible_text(line_after(
        &output,
        " Display (to_string()) "
    )));
}

#[test]
fn compact_layout_keeps_typed_input_visible() {
    setup_i18n();

    let mut app = App::new();
    app.handle_key_code(KeyCode::Char('a'));
    let output = render_output(&app, 100, 20);

    assert!(line_after(&output, " Input ").contains('a'));
}

#[test]
fn short_layout_preserves_core_sections() {
    setup_i18n();

    let app = App::new();
    let output = render_output(&app, 100, 14);

    assert!(line_after(&output, " Module ").contains("String"));
    assert!(line_has_visible_text(line_after(
        &output,
        " Display (to_string()) "
    )));
    assert!(!output.contains(" Fluent (localized) "));
}

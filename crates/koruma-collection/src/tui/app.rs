//! TUI application for interactive validator testing.

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use koruma::showcase::{DynValidator, ValidatorShowcase, validators};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tui_input::{Input, InputRequest};

use crate::tui::i18n::Languages;

/// Application state for the TUI.
struct App {
    /// Current input text
    input: Input,
    /// List of all registered validators
    validators: Vec<&'static ValidatorShowcase>,
    /// Currently selected validator index
    selected: usize,
    /// Current validator instance (created from input)
    current_validator: Option<Box<dyn DynValidator>>,
    /// Whether the app should exit
    should_exit: bool,
}

impl App {
    fn new() -> Self {
        let validators = validators();
        Self {
            input: Input::default(),
            validators,
            selected: 0,
            current_validator: None,
            should_exit: false,
        }
    }

    fn current_showcase(&self) -> Option<&'static ValidatorShowcase> {
        self.validators.get(self.selected).copied()
    }

    fn validate_input(&mut self) {
        if let Some(showcase) = self.current_showcase() {
            let input = self.input.value();
            self.current_validator = Some((showcase.create_validator)(input));
        }
    }

    fn next_validator(&mut self) {
        if !self.validators.is_empty() {
            self.selected = (self.selected + 1) % self.validators.len();
            self.validate_input();
        }
    }

    fn prev_validator(&mut self) {
        if !self.validators.is_empty() {
            self.selected = if self.selected == 0 {
                self.validators.len() - 1
            } else {
                self.selected - 1
            };
            self.validate_input();
        }
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Esc => self.should_exit = true,
            KeyCode::Up => self.prev_validator(),
            KeyCode::Down => self.next_validator(),
            KeyCode::Left => self.prev_validator(),
            KeyCode::Right => self.next_validator(),
            KeyCode::Char(c) => {
                self.input.handle(InputRequest::InsertChar(c));
                self.validate_input();
            },
            KeyCode::Backspace => {
                self.input.handle(InputRequest::DeletePrevChar);
                self.validate_input();
            },
            KeyCode::Delete => {
                self.input.handle(InputRequest::DeleteNextChar);
                self.validate_input();
            },
            KeyCode::Home => {
                self.input.handle(InputRequest::GoToStart);
            },
            KeyCode::End => {
                self.input.handle(InputRequest::GoToEnd);
            },
            _ => {},
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // Initial validation
        self.validate_input();

        while !self.should_exit {
            terminal.draw(|frame| self.render(frame))?;

            if let Event::Key(key) = event::read()? {
                self.handle_key_event(key);
            }
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Layout constraints
        let constraints = vec![
            Constraint::Min(0),    // Top padding
            Constraint::Length(3), // Validator selector
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Input box
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Display output (to_string)
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Fluent output (to_fluent_string)
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Help text
            Constraint::Min(0),    // Bottom padding
        ];

        let vertical = Layout::vertical(constraints).split(area);

        let horizontal = Layout::horizontal([
            Constraint::Min(0),         // Left padding
            Constraint::Percentage(70), // Content
            Constraint::Min(0),         // Right padding
        ]);

        let validator_area = horizontal.split(vertical[1])[1];
        let input_area = horizontal.split(vertical[3])[1];
        let display_area = horizontal.split(vertical[5])[1];
        let fluent_area = horizontal.split(vertical[7])[1];
        let help_area = horizontal.split(vertical[9])[1];

        self.render_validator_selector(frame, validator_area);
        self.render_input(frame, input_area);
        self.render_display_output(frame, display_area);
        self.render_fluent_output(frame, fluent_area);
        self.render_help(frame, help_area);
    }

    fn render_validator_selector(&self, frame: &mut Frame, area: Rect) {
        let showcase = self.current_showcase();
        let (name, description) = showcase
            .map(|v| (v.name, v.description))
            .unwrap_or(("No validators", "No validators registered"));

        let text = vec![
            Line::from(vec![
                Span::styled("◀ ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    name,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ▶", Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(Span::styled(description, Style::default().fg(Color::Gray))),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(format!(
                " Validator ({}/{}) ",
                self.selected + 1,
                self.validators.len()
            ))
            .title_alignment(Alignment::Center);

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        // Determine validity emoji and style
        let (emoji, border_color) = match &self.current_validator {
            Some(v) if v.is_valid() => ("✅ ", Color::Green),
            Some(_) => ("❌ ", Color::Red),
            None => ("   ", Color::Yellow),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(" Input ")
            .title_alignment(Alignment::Center);

        let input_value = self.input.value();
        let text = Line::from(vec![
            Span::raw(emoji),
            Span::styled(input_value, Style::default().fg(Color::Yellow)),
        ]);
        let paragraph = Paragraph::new(text).block(block);

        frame.render_widget(paragraph, area);

        // Position cursor (offset by emoji width: 3 chars for emoji + space)
        let emoji_width = 3u16; // emoji takes ~2 chars + space
        let cursor_x = area.x + 1 + emoji_width + self.input.visual_cursor() as u16;
        let cursor_y = area.y + 1;
        frame.set_cursor_position((cursor_x.min(area.x + area.width - 2), cursor_y));
    }

    fn render_display_output(&self, frame: &mut Frame, area: Rect) {
        let (style, border_color, message) = match &self.current_validator {
            Some(v) => {
                #[cfg(feature = "fmt")]
                let msg = v.display_string();
                #[cfg(not(feature = "fmt"))]
                let msg = "(fmt feature required)".to_string();
                if v.is_valid() {
                    (Style::default().fg(Color::Green), Color::Green, msg)
                } else {
                    (Style::default().fg(Color::Magenta), Color::Magenta, msg)
                }
            },
            None => (
                Style::default().fg(Color::DarkGray),
                Color::DarkGray,
                "—".to_string(),
            ),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(" Display (to_string()) ")
            .title_alignment(Alignment::Center);

        let paragraph = Paragraph::new(message)
            .style(style)
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn render_fluent_output(&self, frame: &mut Frame, area: Rect) {
        let (style, border_color, message) = match &self.current_validator {
            Some(v) => {
                #[cfg(feature = "fluent")]
                let msg = v.fluent_string();
                #[cfg(not(feature = "fluent"))]
                let msg = "(fluent feature required)".to_string();
                if v.is_valid() {
                    (Style::default().fg(Color::Green), Color::Green, msg)
                } else {
                    (Style::default().fg(Color::LightBlue), Color::LightBlue, msg)
                }
            },
            None => (
                Style::default().fg(Color::DarkGray),
                Color::DarkGray,
                "—".to_string(),
            ),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(" Fluent (to_fluent_string()) ")
            .title_alignment(Alignment::Center);

        let paragraph = Paragraph::new(message)
            .style(style)
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let help_text = Line::from(vec![
            Span::styled("←/→ or ↑/↓", Style::default().fg(Color::Cyan)),
            Span::raw(" change validator  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" quit"),
        ]);

        let paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }
}

/// Run the TUI application.
pub fn run() -> io::Result<()> {
    super::i18n::init();
    super::i18n::change_locale(Languages::ZhCn).unwrap();
    let mut terminal = ratatui::init();
    let result = App::new().run(&mut terminal);
    ratatui::restore();
    result
}

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use koruma::showcase::{DynValidator, InputType, ValidatorShowcase, validators};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tui_input::{Input, InputRequest};

use koruma_shared_lib::Languages;

/// Module categories for validators.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValidatorModule {
    String,
    Format,
    Numeric,
    Collection,
    General,
}

impl ValidatorModule {
    const ALL: [Self; 5] = [
        Self::String,
        Self::Format,
        Self::Numeric,
        Self::Collection,
        Self::General,
    ];

    /// Get modules that have at least one validator in the showcase registry.
    fn available_modules(all_validators: &[&'static ValidatorShowcase]) -> Vec<Self> {
        Self::ALL
            .iter()
            .filter(|&&m| all_validators.iter().any(|&v| m.contains_validator(v)))
            .copied()
            .collect()
    }

    fn name(&self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Format => "Format",
            Self::Numeric => "Numeric",
            Self::Collection => "Collection",
            Self::General => "General",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::String => "String-based validators (alphanumeric, ascii, contains, etc.)",
            Self::Format => "Format validators (email, URL, phone, credit card, etc.)",
            Self::Numeric => "Numeric validators (positive, negative, range, etc.)",
            Self::Collection => "Collection validators (length, non-empty)",
            Self::General => "General-purpose validators (required)",
        }
    }

    /// Check if a validator belongs to this module based on its showcase module field.
    fn contains_validator(&self, showcase: &ValidatorShowcase) -> bool {
        match self {
            Self::String => showcase.module == "string",
            Self::Format => showcase.module == "format",
            Self::Numeric => showcase.module == "numeric",
            Self::Collection => showcase.module == "collection",
            Self::General => showcase.module == "general",
        }
    }
}

/// Application state for the TUI.
struct App {
    /// Current input text
    input: Input,
    /// List of all registered validators
    all_validators: Vec<&'static ValidatorShowcase>,
    /// Validators filtered by current module
    current_module_validators: Vec<&'static ValidatorShowcase>,
    /// Available modules (those with at least 1 validator)
    available_modules: Vec<ValidatorModule>,
    /// Currently selected module index (into available_modules)
    selected_module_idx: usize,
    /// Currently selected validator index within the module
    selected_validator: usize,
    /// Current validator instance (created from input)
    /// Ok(validator) when input parses successfully, Err(error) when it fails
    current_validator: Option<anyhow::Result<Box<dyn DynValidator>>>,
    /// Current language for fluent output
    current_language: Languages,
    /// Whether the app should exit
    should_exit: bool,
    /// Whether the module selection dialog is open
    show_module_dialog: bool,
}

impl App {
    fn new() -> Self {
        let all_validators = validators();
        let available_modules = ValidatorModule::available_modules(&all_validators);
        let selected_module_idx = 0;
        let current_module_validators = if available_modules.is_empty() {
            Vec::new()
        } else {
            Self::filter_validators_by_module(
                &all_validators,
                available_modules[selected_module_idx],
            )
        };

        let mut app = Self {
            input: Input::default(),
            all_validators,
            current_module_validators,
            available_modules,
            selected_module_idx,
            selected_validator: 0,
            current_validator: None,
            current_language: Languages::default(),
            should_exit: false,
            show_module_dialog: false,
        };
        app.validate_input();
        app
    }

    fn filter_validators_by_module(
        all: &[&'static ValidatorShowcase],
        module: ValidatorModule,
    ) -> Vec<&'static ValidatorShowcase> {
        all.iter()
            .filter(|&&v| module.contains_validator(v))
            .copied()
            .collect()
    }

    fn current_showcase(&self) -> Option<&'static ValidatorShowcase> {
        self.current_module_validators
            .get(self.selected_validator)
            .copied()
    }

    fn current_module(&self) -> Option<ValidatorModule> {
        self.available_modules
            .get(self.selected_module_idx)
            .copied()
    }

    fn validate_input(&mut self) {
        if let Some(showcase) = self.current_showcase() {
            let input = self.input.value();
            self.current_validator = Some((showcase.create_validator)(input));
        } else {
            self.current_validator = None;
        }
    }

    fn next_validator(&mut self) {
        if !self.current_module_validators.is_empty() {
            self.selected_validator =
                (self.selected_validator + 1) % self.current_module_validators.len();
            self.validate_input();
        }
    }

    fn prev_validator(&mut self) {
        if !self.current_module_validators.is_empty() {
            self.selected_validator = if self.selected_validator == 0 {
                self.current_module_validators.len() - 1
            } else {
                self.selected_validator - 1
            };
            self.validate_input();
        }
    }

    fn select_module(&mut self, index: usize) {
        if index < self.available_modules.len() {
            self.selected_module_idx = index;
            if let Some(module) = self.current_module() {
                self.current_module_validators =
                    Self::filter_validators_by_module(&self.all_validators, module);
            }
            self.selected_validator = 0;
            self.input = Input::default();
            self.validate_input();
        }
    }

    fn next_language(&mut self) {
        self.current_language = self.current_language.next();
        super::i18n::change_locale(self.current_language).unwrap();
    }

    fn toggle_module_dialog(&mut self) {
        self.show_module_dialog = !self.show_module_dialog;
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        // Handle module dialog keys
        if self.show_module_dialog {
            match key.code {
                KeyCode::Esc | KeyCode::Char('m') => self.toggle_module_dialog(),
                KeyCode::Up => {
                    if !self.available_modules.is_empty() {
                        self.selected_module_idx = if self.selected_module_idx == 0 {
                            self.available_modules.len() - 1
                        } else {
                            self.selected_module_idx - 1
                        };
                    }
                },
                KeyCode::Down => {
                    if !self.available_modules.is_empty() {
                        self.selected_module_idx =
                            (self.selected_module_idx + 1) % self.available_modules.len();
                    }
                },
                KeyCode::Enter => {
                    self.select_module(self.selected_module_idx);
                    self.toggle_module_dialog();
                },
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    let digit = c.to_digit(10).unwrap() as usize;
                    if digit > 0 && digit <= self.available_modules.len() {
                        self.select_module(digit - 1);
                        self.toggle_module_dialog();
                    }
                },
                _ => {},
            }
            return;
        }

        // Handle main app keys
        match key.code {
            KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('m') => self.toggle_module_dialog(),
            KeyCode::Up => self.prev_validator(),
            KeyCode::Down => self.next_validator(),
            KeyCode::Tab => self.next_language(),
            KeyCode::Char(c) => {
                let allow = if let Some(showcase) = self.current_showcase() {
                    match showcase.input_type {
                        InputType::Numeric => c.is_ascii_digit() || c == '-',
                        InputType::Text => true,
                    }
                } else {
                    true
                };

                if allow {
                    self.input.handle(InputRequest::InsertChar(c));
                    self.validate_input();
                }
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
            KeyCode::PageUp => {
                self.input.handle(InputRequest::GoToPrevChar);
            },
            KeyCode::PageDown => {
                self.input.handle(InputRequest::GoToNextChar);
            },
            _ => {},
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
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
            Constraint::Length(3), // Module selector
            Constraint::Length(1), // Spacer
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

        let module_area = horizontal.split(vertical[1])[1];
        let validator_area = horizontal.split(vertical[3])[1];
        let input_area = horizontal.split(vertical[5])[1];
        let display_area = horizontal.split(vertical[7])[1];
        let fluent_area = horizontal.split(vertical[9])[1];
        let help_area = horizontal.split(vertical[11])[1];

        self.render_module_selector(frame, module_area);
        self.render_validator_selector(frame, validator_area);
        self.render_input(frame, input_area);
        self.render_display_output(frame, display_area);
        self.render_fluent_output(frame, fluent_area);
        self.render_help(frame, help_area);

        // Render module dialog on top if open
        if self.show_module_dialog {
            self.render_module_dialog(frame, area);
        }
    }

    fn render_module_selector(&self, frame: &mut Frame, area: Rect) {
        let text = if let Some(module) = self.current_module() {
            vec![
                Line::from(vec![
                    Span::styled("◀ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        module.name(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" ▶", Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(Span::styled(
                    module.description(),
                    Style::default().fg(Color::Gray),
                )),
            ]
        } else {
            vec![Line::from(Span::styled(
                "No validators available",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ))]
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(" Module ")
            .title_alignment(Alignment::Center);

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn render_validator_selector(&self, frame: &mut Frame, area: Rect) {
        let showcase = self.current_showcase();
        let (name, description) = showcase
            .map(|v| (v.name, v.description))
            .unwrap_or(("No validators", "No validators registered for this module"));

        let text = vec![
            Line::from(vec![
                Span::styled("▲ ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    name,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ▼", Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(Span::styled(description, Style::default().fg(Color::Gray))),
        ];

        let title = if self.current_module_validators.is_empty() {
            " Validator (0) ".to_string()
        } else {
            format!(
                " Validator ({}/{}) ",
                self.selected_validator + 1,
                self.current_module_validators.len()
            )
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(title)
            .title_alignment(Alignment::Center);

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn render_module_dialog(&self, frame: &mut Frame, area: Rect) {
        // Calculate dialog size based on available modules
        let dialog_width = 60u16;
        let dialog_height = 6u16 + self.available_modules.len() as u16;
        let dialog_area = Rect::new(
            (area.width.saturating_sub(dialog_width)) / 2,
            (area.height.saturating_sub(dialog_height)) / 2,
            dialog_width,
            dialog_height,
        );

        // Clear the background
        frame.render_widget(Clear, dialog_area);

        // Build module list
        let mut lines: Vec<Line> = vec![
            Line::from(vec![Span::styled(
                "Select Module",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];

        if self.available_modules.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "No validators available",
                Style::default().fg(Color::DarkGray),
            )]));
        } else {
            for (i, module) in self.available_modules.iter().enumerate() {
                let is_selected = i == self.selected_module_idx;
                let number = format!("{}.", i + 1);
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };
                let prefix = if is_selected { "▶ " } else { "  " };

                lines.push(Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(number, style),
                    Span::raw(" "),
                    Span::styled(module.name(), style),
                ]));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
            Span::raw(" navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" select  "),
            Span::styled("Esc/m", Style::default().fg(Color::Cyan)),
            Span::raw(" close"),
        ]));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Module Selection ")
            .title_alignment(Alignment::Center);

        let paragraph = Paragraph::new(lines).block(block);

        frame.render_widget(paragraph, dialog_area);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        // Determine validity emoji and style
        let (emoji, border_color) = match &self.current_validator {
            Some(Ok(v)) if v.is_valid() => ("✅ ", Color::Green),
            Some(Ok(_)) => ("❌ ", Color::Red),
            Some(Err(_)) => ("⚠️ ", Color::Yellow), // Parse error
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
            Some(Ok(v)) => {
                let msg = v.display_string();
                if v.is_valid() {
                    (Style::default().fg(Color::Green), Color::Green, msg)
                } else {
                    (Style::default().fg(Color::Magenta), Color::Magenta, msg)
                }
            },
            Some(Err(e)) => (
                Style::default().fg(Color::Yellow),
                Color::Yellow,
                format!("Parse error: {}", e),
            ),
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
            Some(Ok(v)) => {
                let msg = v.fluent_string();
                if v.is_valid() {
                    (Style::default().fg(Color::Green), Color::Green, msg)
                } else {
                    (Style::default().fg(Color::LightBlue), Color::LightBlue, msg)
                }
            },
            Some(Err(_)) => (
                Style::default().fg(Color::Yellow),
                Color::Yellow,
                "—".to_string(),
            ),
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
            Span::styled("▲/▼", Style::default().fg(Color::Cyan)),
            Span::raw(" validator  "),
            Span::styled("m", Style::default().fg(Color::Cyan)),
            Span::raw(" modules  "),
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" language  "),
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
    let _ = super::i18n::change_locale(Languages::default());
    let mut terminal = ratatui::init();
    let result = App::new().run(&mut terminal);
    ratatui::restore();
    result
}

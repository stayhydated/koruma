use crate::components::key_codes::KeyCode;
use crate::components::searchable_select::SearchableSelect;
use crate::components::validator_module::ValidatorModule;
use crate::input::{Input, InputRequest};
use koruma::showcase::{DynValidator, InputType, ValidatorShowcase, validators};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::i18n::{I18n, change_locale, init as create_i18n, localize, localize_with_args};
use koruma_shared_lib::Languages;
use strum::IntoEnumIterator as _;

pub struct App {
    i18n: I18n,
    input: Input,
    all_validators: Vec<&'static ValidatorShowcase>,
    current_module_validators: Vec<&'static ValidatorShowcase>,
    available_modules: Vec<ValidatorModule>,
    selected_module_idx: usize,
    selected_validator: usize,
    current_validator: Option<anyhow::Result<Box<dyn DynValidator>>>,
    current_language: Languages,
    should_exit: bool,
    show_module_dialog: bool,
    show_language_dialog: bool,
    language_selector: SearchableSelect<Languages>,
    module_selector: SearchableSelect<ValidatorModule>,
}

impl App {
    pub fn new() -> Self {
        Self::with_i18n(create_i18n())
    }

    pub fn with_i18n(i18n: I18n) -> Self {
        koruma_collection::__link_showcase_validators();
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

        let current_language = Languages::default();
        change_locale(&i18n, current_language).expect("default language should be supported");
        let all_languages: Vec<Languages> = Languages::iter().collect();

        let initial_idx = all_languages
            .iter()
            .position(|&lang| lang == current_language)
            .unwrap_or(0);

        let mut language_selector = SearchableSelect::new(all_languages);
        language_selector.set_selected_index(initial_idx);
        language_selector.set_search_query("", |language| localize(&i18n, language));

        let mut module_selector = SearchableSelect::new(available_modules.clone());
        module_selector.set_selected_index(selected_module_idx);
        module_selector.set_search_query("", |module| module.name().to_string());

        let mut app = Self {
            i18n,
            input: Input::default(),
            all_validators,
            current_module_validators,
            available_modules,
            selected_module_idx,
            selected_validator: 0,
            current_validator: None,
            current_language,
            should_exit: false,
            show_module_dialog: false,
            show_language_dialog: false,
            language_selector,
            module_selector,
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

    fn language_label(&self, language: &Languages) -> String {
        localize(&self.i18n, language)
    }

    fn set_language_search_query(&mut self, query: &str) {
        let i18n = self.i18n.clone();
        self.language_selector
            .set_search_query(query, |language| localize(&i18n, language));
    }

    fn refresh_language_selector(&mut self) {
        let query = self.language_selector.get_search_query().to_string();
        self.set_language_search_query(&query);
        self.language_selector
            .set_selected_item(&self.current_language);
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

    fn toggle_language_dialog(&mut self) {
        self.show_language_dialog = !self.show_language_dialog;
        if self.show_language_dialog {
            self.refresh_language_selector();
        }
    }

    fn toggle_module_dialog(&mut self) {
        self.show_module_dialog = !self.show_module_dialog;
        if self.show_module_dialog {
            // When opening the dialog, sync the module selector with current module
            self.module_selector
                .set_selected_index(self.selected_module_idx);
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn handle_key_code(&mut self, code: KeyCode) {
        if self.show_module_dialog {
            match code {
                KeyCode::Esc => self.toggle_module_dialog(),
                KeyCode::Up => {
                    self.module_selector.move_up();
                },
                KeyCode::Down => {
                    self.module_selector.move_down();
                },
                KeyCode::Enter => {
                    if let Some(selected_module) = self.module_selector.get_selected_item()
                        && let Some(idx) = self
                            .available_modules
                            .iter()
                            .position(|&m| m == *selected_module)
                    {
                        self.select_module(idx);
                    }

                    self.toggle_module_dialog();
                },
                KeyCode::Char('/') => {
                    self.module_selector.toggle_search();
                },
                KeyCode::Char(c) => {
                    if self.module_selector.is_searching() {
                        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                            let current_query = self.module_selector.get_search_query().to_string();
                            let new_query = format!("{}{}", current_query, c);
                            self.module_selector
                                .set_search_query(&new_query, |module| module.name().to_string());
                        }
                    } else if c.is_ascii_digit() {
                        // Support digit selection when not searching
                        let digit = c.to_digit(10).unwrap() as usize;
                        if digit > 0 && digit <= self.available_modules.len() {
                            self.select_module(digit - 1);
                            self.toggle_module_dialog();
                        }
                    }
                },
                KeyCode::Backspace if self.module_selector.is_searching() => {
                    let current_query = self.module_selector.get_search_query().to_string();
                    if !current_query.is_empty() {
                        let new_query = &current_query[..current_query.len() - 1];
                        self.module_selector
                            .set_search_query(new_query, |module| module.name().to_string());
                    }
                },
                _ => {},
            }
            return;
        }

        if self.show_language_dialog {
            match code {
                KeyCode::Esc => self.toggle_language_dialog(),
                KeyCode::Up => self.language_selector.move_up(),
                KeyCode::Down => self.language_selector.move_down(),
                KeyCode::Enter => {
                    if let Some(selected_language) = self.language_selector.get_selected_item() {
                        self.current_language = *selected_language;
                        change_locale(&self.i18n, self.current_language).unwrap();
                        self.language_selector.clear_search();
                        self.refresh_language_selector();
                    }
                    self.toggle_language_dialog();
                },
                KeyCode::Char('/') => {
                    self.language_selector.toggle_search();
                },
                KeyCode::Char(c)
                    if self.language_selector.is_searching()
                        && (c.is_ascii_alphanumeric() || c == '-' || c == '_') =>
                {
                    let current_query = self.language_selector.get_search_query().to_string();
                    let new_query = format!("{}{}", current_query, c);
                    self.set_language_search_query(&new_query);
                },
                KeyCode::Backspace if self.language_selector.is_searching() => {
                    let current_query = self.language_selector.get_search_query().to_string();
                    if !current_query.is_empty() {
                        let new_query = &current_query[..current_query.len() - 1];
                        self.set_language_search_query(new_query);
                    }
                },
                _ => {},
            }
            return;
        }

        match code {
            KeyCode::Esc => self.should_exit = true,
            KeyCode::Enter => self.toggle_module_dialog(),
            KeyCode::Up => self.prev_validator(),
            KeyCode::Down => self.next_validator(),
            KeyCode::Tab => self.toggle_language_dialog(),
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
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(self, area);
        if let Some(cursor_position) = self.cursor_position(area) {
            frame.set_cursor_position(cursor_position);
        }
    }

    fn render_area(&self, area: Rect, buf: &mut Buffer) {
        let content_area = Self::content_area(area);

        if area.height >= 16 {
            let vertical = Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(content_area);

            self.render_module_selector(vertical[0], buf);
            self.render_validator_selector(vertical[1], buf);
            self.render_input(vertical[2], buf);
            self.render_display_output(vertical[3], buf);
            self.render_fluent_output(vertical[4], buf);
            self.render_help(vertical[5], buf);
        } else {
            let vertical = Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(content_area);

            self.render_module_selector(vertical[0], buf);
            self.render_validator_selector(vertical[1], buf);
            self.render_input(vertical[2], buf);
            self.render_display_output(vertical[3], buf);
            self.render_help(vertical[4], buf);
        }

        if self.show_module_dialog {
            self.render_module_dialog(area, buf);
        }

        if self.show_language_dialog {
            self.render_language_dialog(area, buf);
        }
    }

    fn content_area(area: Rect) -> Rect {
        let content_area = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Percentage(70),
            Constraint::Min(0),
        ])
        .split(area)[1];

        content_area
    }

    fn cursor_position(&self, area: Rect) -> Option<(u16, u16)> {
        let content_area = Self::content_area(area);
        let vertical = if area.height >= 16 {
            Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(content_area)
        } else {
            Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(content_area)
        };
        let input_area = vertical[2];
        if input_area.width < 2 || input_area.height < 2 {
            return None;
        }

        let emoji_width = 3u16;
        let cursor_x = input_area.x + 1 + emoji_width + self.input.visual_cursor() as u16;
        let cursor_y = input_area.y + 1;
        Some((cursor_x.min(input_area.x + input_area.width - 2), cursor_y))
    }

    fn render_module_selector(&self, area: Rect, buf: &mut Buffer) {
        let text = if let Some(module) = self.current_module() {
            vec![Line::from(vec![Span::styled(
                module.name(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])]
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

        paragraph.render(area, buf);
    }

    fn render_validator_selector(&self, area: Rect, buf: &mut Buffer) {
        let showcase = self.current_showcase();
        let (name, description) = showcase
            .map(|v| (v.name, v.description))
            .unwrap_or(("No validators", "No validators registered for this module"));

        let text = vec![
            Line::from(vec![Span::styled(
                name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
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

        paragraph.render(area, buf);
    }

    fn render_module_dialog(&self, area: Rect, buf: &mut Buffer) {
        self.module_selector.render_searchable_select(
            area,
            buf,
            "Module",
            |module, is_selected| {
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };
                vec![Span::styled(module.name(), style)]
            },
        );
    }

    fn render_language_dialog(&self, area: Rect, buf: &mut Buffer) {
        self.language_selector.render_searchable_select(
            area,
            buf,
            "Language",
            |language, is_selected| {
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };
                vec![Span::styled(self.language_label(language), style)]
            },
        );
    }

    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        let (emoji, border_color) = match &self.current_validator {
            Some(Ok(v)) if v.is_valid() => ("✅ ", Color::Green),
            Some(Ok(_)) => ("❌ ", Color::Red),
            Some(Err(_)) => ("⚠️ ", Color::Yellow),
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

        paragraph.render(area, buf);
    }

    fn render_display_output(&self, area: Rect, buf: &mut Buffer) {
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

        paragraph.render(area, buf);
    }

    fn render_fluent_output(&self, area: Rect, buf: &mut Buffer) {
        let (style, border_color, message) = match &self.current_validator {
            Some(Ok(v)) => {
                let i18n = &self.i18n;
                let msg = v.fluent_string_with(&mut |domain, id, args| {
                    localize_with_args(i18n, domain, id, args)
                });
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
            .title(" Fluent (localized) ")
            .title_alignment(Alignment::Center);

        let paragraph = Paragraph::new(message)
            .style(style)
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }

    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let help_text = Line::from(vec![
            Span::styled("▲/▼", Style::default().fg(Color::Cyan)),
            Span::raw(" validator  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" modules  "),
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" language  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" quit"),
        ]);

        let paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
        paragraph.render(area, buf);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_area(area, buf);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

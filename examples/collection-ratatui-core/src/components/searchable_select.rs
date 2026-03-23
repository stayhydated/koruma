use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// SearchableSelect component for handling searchable lists
#[derive(Clone, Debug)]
pub struct SearchableSelect<T> {
    pub items: Vec<T>,
    filtered_items: Vec<T>,
    selected_idx: usize,
    search_query: String,
    show_search: bool,
}

impl<T> SearchableSelect<T>
where
    T: Clone,
{
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items: items.clone(),
            filtered_items: items,
            selected_idx: 0,
            search_query: String::new(),
            show_search: false,
        }
    }

    pub fn toggle_search(&mut self) {
        self.show_search = !self.show_search;
        if !self.show_search {
            self.search_query.clear();
            self.filtered_items = self.items.clone();
            if self.selected_idx >= self.filtered_items.len() {
                self.selected_idx = 0;
            }
        }
    }

    pub fn set_search_query<F>(&mut self, query: &str, item_to_string: F)
    where
        F: Fn(&T) -> String,
    {
        self.search_query = query.to_lowercase();
        self.filter_items(item_to_string);
        if self.selected_idx >= self.filtered_items.len() {
            self.selected_idx = 0;
        }
    }

    fn filter_items<F>(&mut self, item_to_string: F)
    where
        F: Fn(&T) -> String,
    {
        if self.search_query.is_empty() {
            self.filtered_items = self.items.clone();
        } else {
            self.filtered_items = self
                .items
                .iter()
                .filter(|item| {
                    item_to_string(item)
                        .to_lowercase()
                        .contains(&self.search_query)
                })
                .cloned()
                .collect();
        }
    }

    pub fn move_up(&mut self) {
        if !self.filtered_items.is_empty() {
            if self.selected_idx > 0 {
                self.selected_idx -= 1;
            } else {
                self.selected_idx = self.filtered_items.len() - 1;
            }
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered_items.is_empty() {
            if self.selected_idx < self.filtered_items.len() - 1 {
                self.selected_idx += 1;
            } else {
                self.selected_idx = 0;
            }
        }
    }

    pub fn get_selected_item(&self) -> Option<&T> {
        self.filtered_items.get(self.selected_idx)
    }

    pub fn set_selected_index(&mut self, idx: usize) {
        if idx < self.filtered_items.len() {
            self.selected_idx = idx;
        }
    }

    pub fn is_searching(&self) -> bool {
        self.show_search
    }

    pub fn get_search_query(&self) -> &str {
        &self.search_query
    }

    pub fn render_searchable_select<F>(
        &self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        render_item: F,
    ) where
        F: Fn(&T, bool) -> Vec<Span>,
    {
        let dialog_width = 50u16;
        let dialog_height = 10u16 + self.filtered_items.len().min(10) as u16;
        let dialog_area = Rect::new(
            (area.width.saturating_sub(dialog_width)) / 2,
            (area.height.saturating_sub(dialog_height)) / 2,
            dialog_width,
            dialog_height,
        );

        frame.render_widget(Clear, dialog_area);

        let mut lines: Vec<Line> = vec![
            Line::from(vec![Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];

        // Search input
        if self.show_search {
            lines.push(Line::from(vec![
                Span::styled("Search: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    self.search_query.as_str(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("_"),
            ]));
            lines.push(Line::from(""));
        }

        if self.filtered_items.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "No items found",
                Style::default().fg(Color::DarkGray),
            )]));
        } else {
            let start_idx = if self.filtered_items.len() > 8 {
                self.selected_idx
                    .saturating_sub(3)
                    .min(self.filtered_items.len().saturating_sub(8))
            } else {
                0
            };
            let end_idx = (start_idx + 8).min(self.filtered_items.len());

            for (i, item) in self
                .filtered_items
                .iter()
                .enumerate()
                .take(end_idx)
                .skip(start_idx)
            {
                let is_selected = i == self.selected_idx;
                let _style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };
                let prefix = if is_selected { "▶ " } else { "  " };

                let mut item_spans = vec![Span::raw(prefix)];
                item_spans.extend(render_item(item, is_selected));

                lines.push(Line::from(item_spans));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
            Span::raw(" navigate  "),
            Span::styled("/", Style::default().fg(Color::Cyan)),
            Span::raw(" search  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" select  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" close"),
        ]));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(format!(" {} Selection ", title))
            .title_alignment(ratatui::layout::Alignment::Center);

        let paragraph = Paragraph::new(lines).block(block);

        frame.render_widget(paragraph, dialog_area);
    }
}

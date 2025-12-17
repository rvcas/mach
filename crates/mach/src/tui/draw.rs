use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};
use uuid::Uuid;

use crate::service::config::WeekStart;

use super::App;
use super::modes::{AddTodoState, DetailField, DetailState, SettingsState, UiMode};
use super::palette;
use super::state::{BACKLOG_COLUMNS, TodoView};

impl App {
    pub fn draw(&mut self, frame: &mut Frame<'_>) {
        match &self.ui_mode {
            UiMode::Board => self.draw_board(frame),
            UiMode::Backlog => self.draw_backlog_view(frame),
            UiMode::Settings(settings) => {
                self.draw_board(frame);

                let settings = settings.clone();

                self.draw_settings(frame, &settings);
            }
            UiMode::AddTodo(state) => {
                match state.target {
                    super::modes::AddTarget::Day(_) => self.draw_board(frame),
                    super::modes::AddTarget::BacklogColumn(_) => self.draw_backlog_view(frame),
                }
                let state = state.clone();
                self.draw_add_todo(frame, &state);
            }
            UiMode::Detail(state) => {
                if state.from_backlog {
                    self.draw_backlog_view(frame);
                } else {
                    self.draw_board(frame);
                }

                let state = state.clone();

                self.draw_detail(frame, &state);
            }
        }

        if self.show_help {
            self.draw_help(frame);
        }
    }

    pub fn draw_board(&self, frame: &mut Frame<'_>) {
        let day_count = self.state.columns.len();
        let mut constraints = Vec::with_capacity(day_count * 2 - 1);

        for i in 0..day_count {
            if i > 0 {
                constraints.push(Constraint::Length(1));
            }

            constraints.push(Constraint::Fill(1));
        }

        let areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(frame.area());

        let focused = self.cursor.focus;
        let mut col_idx = 0;

        for (i, &area) in areas.iter().enumerate() {
            if i % 2 == 0 {
                self.draw_day_column(frame, col_idx, area);

                col_idx += 1;
            } else {
                let sep_idx = i / 2;
                let adjacent_to_focus = sep_idx == focused || sep_idx + 1 == focused;

                let style = if adjacent_to_focus {
                    Style::default().fg(palette::FOCUS)
                } else {
                    Style::default().fg(palette::BORDER)
                };

                let lines: Vec<Line<'_>> = (0..area.height).map(|_| Line::from("│")).collect();
                let separator = Paragraph::new(lines).style(style);

                frame.render_widget(separator, area);
            }
        }
    }

    pub fn draw_backlog_view(&self, frame: &mut Frame<'_>) {
        let outer = Block::default()
            .title("Someday / Backlog")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette::FOCUS));

        let inner = outer.inner(frame.area());
        frame.render_widget(outer, frame.area());

        let mut constraints = Vec::with_capacity(BACKLOG_COLUMNS * 2 - 1);

        for i in 0..BACKLOG_COLUMNS {
            if i > 0 {
                constraints.push(Constraint::Length(1));
            }

            constraints.push(Constraint::Fill(1));
        }

        let areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(inner);

        let focused = self.backlog_cursor.column;
        let mut col_idx = 0;

        for (i, &area) in areas.iter().enumerate() {
            if i % 2 == 0 {
                self.draw_backlog_column(frame, col_idx, area);

                col_idx += 1;
            } else {
                let sep_idx = i / 2;
                let adjacent_to_focus = sep_idx == focused || sep_idx + 1 == focused;

                let style = if adjacent_to_focus {
                    Style::default().fg(palette::FOCUS)
                } else {
                    Style::default().fg(palette::BORDER)
                };

                let lines: Vec<Line<'_>> = (0..area.height).map(|_| Line::from("│")).collect();
                let separator = Paragraph::new(lines).style(style);

                frame.render_widget(separator, area);
            }
        }
    }

    fn draw_backlog_column(&self, frame: &mut Frame<'_>, col_idx: usize, area: Rect) {
        let focused = self.backlog_cursor.column == col_idx;
        let items = &self.board.backlog_columns[col_idx];

        let highlight_row = if focused {
            self.backlog_cursor.row_for(col_idx, &self.board)
        } else {
            None
        };

        let lines = self.build_todo_lines_with_separators(
            items,
            area.width,
            highlight_row,
            |row| self.backlog_cursor.line_style(col_idx, row, &self.board),
            |id| self.backlog_cursor.is_selected(id),
        );

        let para = Paragraph::new(lines);

        frame.render_widget(para, area);
    }

    fn draw_day_column(&self, frame: &mut Frame<'_>, idx: usize, area: Rect) {
        let column = &self.state.columns[idx];
        let focused = self.cursor.focus == idx;

        let title_style = if focused {
            Style::default()
                .fg(palette::FOCUS)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(palette::TEXT)
        };

        let title_line = Line::from(column.title.clone()).style(title_style);
        let underline = "─".repeat(area.width as usize);
        let underline_line = Line::from(underline).style(title_style);

        let content_area = Rect {
            x: area.x,
            y: area.y + 2,
            width: area.width,
            height: area.height.saturating_sub(2),
        };

        let items = self
            .board
            .days
            .get(idx)
            .map(|d| d.as_slice())
            .unwrap_or(&[]);

        let highlight_row = if focused {
            self.cursor.row_for(idx, &self.board)
        } else {
            None
        };

        let lines = self.build_todo_lines_with_separators(
            items,
            area.width,
            highlight_row,
            |row| self.cursor.line_style(idx, row, &self.board),
            |id| self.cursor.is_selected(id),
        );

        frame.render_widget(
            Paragraph::new(title_line).centered(),
            Rect { height: 1, ..area },
        );

        frame.render_widget(
            Paragraph::new(underline_line),
            Rect {
                y: area.y + 1,
                height: 1,
                ..area
            },
        );

        let body = Paragraph::new(lines);

        frame.render_widget(body, content_area);
    }

    fn build_todo_lines_with_separators<'a, F, S>(
        &self,
        items: &'a [TodoView],
        width: u16,
        highlight_row: Option<usize>,
        style_fn: F,
        is_selected_fn: S,
    ) -> Vec<Line<'a>>
    where
        F: Fn(usize) -> Style,
        S: Fn(Uuid) -> bool,
    {
        let separator = "-".repeat(width as usize);

        let mut lines = Vec::with_capacity(items.len() * 2);

        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                let adjacent_to_focus = highlight_row == Some(i - 1) || highlight_row == Some(i);

                let sep_style = if adjacent_to_focus {
                    Style::default().fg(palette::ACTIVE)
                } else {
                    Style::default().fg(palette::BORDER)
                };

                lines.push(Line::from(separator.clone()).style(sep_style));
            }

            let is_selected = is_selected_fn(item.id);

            let mut line = item.to_line_with_prefix(is_selected);

            if is_selected {
                line.style = line.style.patch(
                    Style::default()
                        .fg(palette::ACCENT)
                        .add_modifier(Modifier::BOLD),
                );
            } else if highlight_row == Some(i) {
                line.style = line.style.patch(style_fn(i));
            }

            lines.push(line);
        }

        lines
    }

    pub fn draw_settings(&self, frame: &mut Frame<'_>, settings: &SettingsState) {
        let area = centered_rect(30, 18, frame.area());

        let block = Block::default()
            .title("Settings")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette::FOCUS));

        let (monday_style, sunday_style) = match settings.week_start {
            WeekStart::Monday => (
                Style::default().fg(palette::ACTIVE),
                Style::default().fg(palette::TEXT_DIM),
            ),
            WeekStart::Sunday => (
                Style::default().fg(palette::TEXT_DIM),
                Style::default().fg(palette::ACTIVE),
            ),
        };

        let lines = vec![
            Line::from("Week Start"),
            Line::from(""),
            Line::from(vec![
                "[m] ".into(),
                ratatui::text::Span::styled("Monday", monday_style),
            ]),
            Line::from(vec![
                "[s] ".into(),
                ratatui::text::Span::styled("Sunday", sunday_style),
            ]),
            Line::from(""),
            Line::from("[Esc] close").style(Style::default().fg(palette::TEXT_DIM)),
        ];

        let paragraph = Paragraph::new(lines).block(block);

        frame.render_widget(Clear, area);

        frame.render_widget(paragraph, area);
    }

    pub fn draw_add_todo(&self, frame: &mut Frame<'_>, state: &AddTodoState) {
        let area = centered_rect(35, 15, frame.area());

        let block = Block::default()
            .title("Add Todo")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette::FOCUS));

        let inner = block.inner(area);

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let lines = vec![
            Line::from(format!("› {}_", state.input)).style(Style::default().fg(palette::ACTIVE)),
            Line::from(""),
            Line::from("[Enter] add  [Esc] cancel").style(Style::default().fg(palette::TEXT_DIM)),
        ];

        frame.render_widget(Paragraph::new(lines), inner);
    }

    pub fn draw_detail(&self, frame: &mut Frame<'_>, state: &DetailState) {
        let area = centered_rect(70, 50, frame.area());

        let block = Block::default()
            .title("Todo")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette::FOCUS));

        let inner = block.inner(area);

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let fields = [
            DetailField::Title,
            DetailField::Project,
            DetailField::Epic,
            DetailField::Date,
            DetailField::Status,
            DetailField::Notes,
        ];

        let mut lines: Vec<Line<'_>> = Vec::new();

        for field in fields {
            let is_focused = state.field == field;
            let is_editing = is_focused && state.editing.is_some();

            let label = field.label();

            let value = if is_editing {
                state.editing.as_ref().unwrap().clone()
            } else {
                state.field_value(field)
            };

            let style = if is_focused {
                Style::default().fg(palette::ACTIVE)
            } else {
                Style::default().fg(palette::TEXT)
            };

            if field == DetailField::Notes {
                lines.push(Line::from(""));

                let prefix = if is_editing { "› " } else { "  " };

                lines.push(Line::from(format!("{prefix}{label}:")).style(style));

                if is_editing {
                    let note_lines: Vec<&str> = value.split('\n').collect();

                    for (i, line) in note_lines.iter().enumerate() {
                        let cursor = if i == note_lines.len() - 1 { "_" } else { "" };
                        lines.push(Line::from(format!("    {line}{cursor}")).style(style));
                    }
                } else if value.is_empty() {
                    lines.push(
                        Line::from("    (empty)").style(Style::default().fg(palette::TEXT_DIM)),
                    );
                } else {
                    for line in value.lines() {
                        lines.push(Line::from(format!("    {line}")).style(style));
                    }
                }
            } else {
                let prefix = if is_focused { "› " } else { "  " };
                let suffix = if is_editing { "_" } else { "" };

                lines.push(Line::from(format!("{prefix}{label}: {value}{suffix}")).style(style));
            }
        }

        if let Some(ref err) = state.error {
            lines.push(Line::from(""));
            lines.push(Line::from(format!("Error: {err}")).style(Style::default().fg(palette::ERROR)));
        }

        lines.push(Line::from(""));

        lines.push(
            Line::from("[j/k] navigate  [Enter] edit/confirm  [x] toggle  [Esc] close")
                .style(Style::default().fg(palette::TEXT_DIM)),
        );

        lines.push(
            Line::from("[Ctrl+j] newline in notes").style(Style::default().fg(palette::TEXT_DIM)),
        );

        let paragraph = Paragraph::new(lines);

        frame.render_widget(paragraph, inner);
    }

    pub fn draw_help(&self, frame: &mut Frame<'_>) {
        let lines = match &self.ui_mode {
            UiMode::Board => vec![
                Line::from("Weekly View").style(Style::default().fg(palette::ACTIVE)),
                Line::from(""),
                Line::from("h/l      Move between days"),
                Line::from("j/k      Move within column"),
                Line::from("[/]      Previous/next week"),
                Line::from("Enter    Select (drag mode)"),
                Line::from("Space    Open todo details"),
                Line::from("a        Add new todo"),
                Line::from("x        Toggle completion"),
                Line::from("dd       Delete todo"),
                Line::from("s        Send to backlog"),
                Line::from("t        Move to today"),
                Line::from("T        Move to tomorrow"),
                Line::from("b        Open backlog"),
                Line::from("gs       Settings"),
                Line::from("?        Toggle help"),
                Line::from("q/Esc    Quit"),
            ],
            UiMode::Backlog => vec![
                Line::from("Backlog View").style(Style::default().fg(palette::ACTIVE)),
                Line::from(""),
                Line::from("h/l      Move between columns"),
                Line::from("j/k      Move within column"),
                Line::from("Enter    Select (drag mode)"),
                Line::from("Space    Open todo details"),
                Line::from("a        Add new todo"),
                Line::from("x        Toggle completion"),
                Line::from("dd       Delete todo"),
                Line::from("t        Move to today"),
                Line::from("T        Move to tomorrow"),
                Line::from("?        Toggle help"),
                Line::from("b/q/Esc  Return to weekly"),
            ],
            _ => vec![],
        };

        let height = lines.len() as u16 + 2;
        let width = 30;
        let area = frame.area();

        let popup_area = Rect {
            x: area.width.saturating_sub(width + 2),
            y: area.height.saturating_sub(height + 1),
            width,
            height,
        };

        let block = Block::default()
            .title("Help (?)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette::FOCUS));

        let paragraph = Paragraph::new(lines).block(block);

        frame.render_widget(Clear, popup_area);
        frame.render_widget(paragraph, popup_area);
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}

use std::{
    io,
    time::{Duration, Instant},
};

use chrono::{Datelike, Duration as ChronoDuration, NaiveDate};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use miette::{Context, IntoDiagnostic};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};
use tokio::runtime::Handle;
use uuid::Uuid;

use crate::{
    entity::todo,
    service::{
        Services,
        config::WeekStart,
        todo::{ListOptions, ListScope, MovePlacement, ReorderDirection},
    },
};

mod palette {
    #![allow(dead_code)]
    use ratatui::style::Color;

    // Text
    pub const TEXT: Color = Color::Reset;
    pub const TEXT_DIM: Color = Color::DarkGray;

    // States (hierarchy: ACCENT > ACTIVE > FOCUS)
    pub const FOCUS: Color = Color::LightBlue;
    pub const ACTIVE: Color = Color::Yellow;
    pub const ACCENT: Color = Color::Magenta;

    // Chrome
    pub const BORDER: Color = Color::DarkGray;
}

/// Launch the Ratatui application, blocking on the UI event loop.
pub async fn run(services: Services) -> miette::Result<()> {
    let handle = Handle::current();
    tokio::task::spawn_blocking(move || {
        let mut app = App::new(services, handle);
        app.run()
    })
    .await
    .into_diagnostic()??;
    Ok(())
}

const BACKLOG_COLUMNS: usize = 4;

struct App {
    services: Services,
    runtime: Handle,
    state: WeekState,
    board: BoardData,
    cursor: CursorState,
    backlog_cursor: BacklogCursor,
    week_pref: WeekStart,
    ui_mode: UiMode,
    pending_g: bool,
    pending_delete: bool,
    should_quit: bool,
}

impl App {
    fn new(services: Services, runtime: Handle) -> Self {
        let today = services.today();
        let week_pref = services.week_start();
        let state = WeekState::new(today, week_pref);
        let board = BoardData::new(state.columns.len());
        let mut cursor = CursorState::new(state.columns.len());
        if let Some(idx) = state.column_index(today) {
            cursor.set_focus_row(idx, 0);
        }
        Self {
            services,
            runtime,
            state,
            board,
            cursor,
            backlog_cursor: BacklogCursor::new(),
            week_pref,
            ui_mode: UiMode::Board,
            pending_g: false,
            pending_delete: false,
            should_quit: false,
        }
    }

    fn run(&mut self) -> miette::Result<()> {
        self.refresh_board().ok();

        let mut terminal = setup_terminal()?;
        let _guard = TerminalGuard;

        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(250);

        loop {
            terminal
                .draw(|frame| self.draw(frame))
                .into_diagnostic()
                .wrap_err("failed to draw frame")?;

            if self.should_quit {
                break;
            }

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).into_diagnostic()? {
                let evt = event::read().into_diagnostic()?;
                self.handle_event(evt);
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, evt: Event) {
        if let Event::Key(key) = evt {
            self.handle_key_event(key);
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match &self.ui_mode {
            UiMode::Settings(_) => {
                self.handle_settings_key(key);
                return;
            }
            UiMode::Backlog => {
                self.handle_backlog_key(key);
                return;
            }
            UiMode::AddTodo(_) => {
                self.handle_add_todo_key(key);
                return;
            }
            UiMode::Detail(_) => {
                self.handle_detail_key(key);
                return;
            }
            UiMode::Board => {}
        }

        if self.pending_g {
            self.pending_g = false;
            if key.modifiers.is_empty() && matches!(key.code, KeyCode::Char('s')) {
                self.open_settings();
                return;
            }
        }

        if !matches!(key.code, KeyCode::Char('d')) {
            self.pending_delete = false;
        }

        match key.code {
            KeyCode::Char('q') if key.modifiers.is_empty() => self.should_quit = true,
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('g') if key.modifiers.is_empty() => {
                self.pending_g = true;
            }
            KeyCode::Char('a') if key.modifiers.is_empty() => {
                self.open_add_todo_board();
            }
            KeyCode::Char('b') if key.modifiers.is_empty() => {
                self.open_backlog();
            }
            KeyCode::Char('h') => self.handle_horizontal(Horizontal::Left),
            KeyCode::Char('l') => self.handle_horizontal(Horizontal::Right),
            KeyCode::Char('j') => self.handle_vertical(Vertical::Down),
            KeyCode::Char('k') => self.handle_vertical(Vertical::Up),
            KeyCode::Char('[') => self.change_week(-1),
            KeyCode::Char(']') => self.change_week(1),
            KeyCode::Char('x') if key.modifiers.is_empty() => {
                self.mark_complete().ok();
            }
            KeyCode::Char('s') if key.modifiers.is_empty() => {
                self.move_to_backlog().ok();
            }
            KeyCode::Char('t') if key.modifiers.is_empty() => {
                self.move_to_today().ok();
            }
            KeyCode::Char('T') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.move_to_tomorrow().ok();
            }
            KeyCode::Char(' ') if key.modifiers.is_empty() => {
                self.open_detail_board();
            }
            KeyCode::Enter => self.toggle_selection(),
            KeyCode::Char('d') if key.modifiers.is_empty() => {
                if self.pending_delete {
                    self.delete_current().ok();
                    self.pending_delete = false;
                } else {
                    self.pending_delete = true;
                }
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        match &self.ui_mode {
            UiMode::Board => self.draw_board(frame),
            UiMode::Backlog => self.draw_backlog_view(frame),
            UiMode::Settings(settings) => {
                self.draw_board(frame);
                self.draw_settings(frame, settings);
            }
            UiMode::AddTodo(state) => {
                match state.target {
                    AddTarget::Day(_) => self.draw_board(frame),
                    AddTarget::BacklogColumn(_) => self.draw_backlog_view(frame),
                }
                self.draw_add_todo(frame, state);
            }
            UiMode::Detail(state) => {
                if state.from_backlog {
                    self.draw_backlog_view(frame);
                } else {
                    self.draw_board(frame);
                }
                self.draw_detail(frame, state);
            }
        }
    }

    fn draw_board(&self, frame: &mut Frame<'_>) {
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

    fn draw_backlog_view(&self, frame: &mut Frame<'_>) {
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

    fn change_week(&mut self, delta: i32) {
        if delta < 0 {
            self.state.prev_week();
        } else {
            self.state.next_week();
        }
        self.board.reset(self.state.columns.len());
        self.cursor
            .sync_after_refresh(self.state.columns.len(), &self.board);
        self.refresh_board().ok();
    }

    fn handle_horizontal(&mut self, dir: Horizontal) {
        let day_count = self.state.columns.len();
        if self.cursor.selection.is_some() {
            self.move_selected_horizontal(dir).ok();
        } else {
            match dir {
                Horizontal::Left => {
                    if self.cursor.focus == 0 {
                        self.state.prev_week();
                        self.cursor.focus = day_count - 1;
                        self.board.reset(day_count);
                        self.refresh_board().ok();
                    } else {
                        self.cursor.focus -= 1;
                    }
                }
                Horizontal::Right => {
                    if self.cursor.focus + 1 >= day_count {
                        self.state.next_week();
                        self.cursor.focus = 0;
                        self.board.reset(day_count);
                        self.refresh_board().ok();
                    } else {
                        self.cursor.focus += 1;
                    }
                }
            }
            self.cursor.selection = None;
        }
    }

    fn handle_vertical(&mut self, dir: Vertical) {
        if self.cursor.selection.is_some() {
            let reorder_dir = match dir {
                Vertical::Up => ReorderDirection::Up,
                Vertical::Down => ReorderDirection::Down,
            };
            self.reorder_selected(reorder_dir).ok();
        } else {
            self.cursor.move_vertical(dir, &self.board);
        }
    }

    fn toggle_selection(&mut self) {
        if self.cursor.selection.is_some() {
            self.cursor.selection = None;
            return;
        }

        if let Some(id) = self.current_target_id() {
            let row = self.cursor.row_for(self.cursor.focus, &self.board);
            self.cursor.selection = Some(Selection {
                id,
                column: self.cursor.focus,
                row,
            });
        }
    }

    fn move_selected_horizontal(&mut self, dir: Horizontal) -> miette::Result<()> {
        let Some(selection) = self.cursor.selection else {
            return Ok(());
        };

        let day_count = self.state.columns.len();
        let (target_col, week_changed) = match dir {
            Horizontal::Left => {
                if selection.column == 0 {
                    self.state.prev_week();
                    (day_count - 1, true)
                } else {
                    (selection.column - 1, false)
                }
            }
            Horizontal::Right => {
                if selection.column + 1 >= day_count {
                    self.state.next_week();
                    (0, true)
                } else {
                    (selection.column + 1, false)
                }
            }
        };

        let target_date = self.state.columns[target_col].date;
        self.runtime.block_on(self.services.todos.move_to_scope(
            selection.id,
            ListScope::Day(target_date),
            MovePlacement::Top,
        ))?;

        if week_changed {
            self.board.reset(day_count);
        }
        self.refresh_board()?;

        self.cursor.selection = Some(Selection {
            column: target_col,
            row: None,
            ..selection
        });
        self.cursor.focus = target_col;

        Ok(())
    }

    fn reorder_selected(&mut self, dir: ReorderDirection) -> miette::Result<()> {
        if let Some(selection) = self.cursor.selection {
            self.runtime
                .block_on(self.services.todos.reorder(selection.id, dir))?;
            if let Some(sel) = &mut self.cursor.selection {
                sel.row = None;
            }
            self.refresh_board()?;
        }
        Ok(())
    }

    fn refresh_board(&mut self) -> miette::Result<()> {
        for (idx, column) in self.state.columns.iter().enumerate() {
            let opts = ListOptions {
                scope: ListScope::Day(column.date),
                include_done: true,
            };
            let todos = self.runtime.block_on(self.services.todos.list(opts))?;
            self.board
                .set_day(idx, todos.into_iter().map(TodoView::from).collect());
        }

        self.refresh_backlog()?;

        self.cursor
            .sync_after_refresh(self.state.columns.len(), &self.board);

        Ok(())
    }

    fn refresh_backlog(&mut self) -> miette::Result<()> {
        let all_backlog = self
            .runtime
            .block_on(self.services.todos.list(ListOptions {
                scope: ListScope::Backlog,
                include_done: true,
            }))?;

        let mut columns: [Vec<TodoView>; BACKLOG_COLUMNS] = Default::default();
        for todo in all_backlog {
            let col = (todo.backlog_column as usize).min(BACKLOG_COLUMNS - 1);
            columns[col].push(TodoView::from(todo));
        }

        for (col, items) in columns.into_iter().enumerate() {
            self.board.set_backlog_column(col, items);
        }

        self.backlog_cursor.sync_after_refresh(&self.board);

        Ok(())
    }

    fn current_target_id(&self) -> Option<Uuid> {
        self.cursor
            .selection
            .map(|sel| sel.id)
            .or_else(|| self.cursor.current_todo_id(&self.board))
    }

    fn delete_current(&mut self) -> miette::Result<()> {
        if let Some(id) = self.current_target_id() {
            let deleted = self.runtime.block_on(self.services.todos.delete(id))?;
            if deleted {
                self.cursor.selection = None;
                self.refresh_board()?;
            }
        }
        Ok(())
    }

    fn mark_complete(&mut self) -> miette::Result<()> {
        if let Some(id) = self.current_target_id() {
            let current_status = self
                .board
                .day_status_of(id)
                .unwrap_or("pending")
                .to_string();

            let focus = self.cursor.focus;
            let prev_row = self.cursor.row_for(focus, &self.board);
            self.cursor.selection = None;

            if current_status == "done" {
                self.runtime
                    .block_on(self.services.todos.mark_pending(id))?;
            } else {
                let today = self.services.today();
                self.runtime
                    .block_on(self.services.todos.mark_done(id, today))?;
            }

            self.refresh_board()?;

            if let Some((new_col, row)) = self.board.find_day_position(id) {
                self.cursor.set_focus_row(new_col, row);
            } else if let Some(row) = prev_row {
                let len = self.board.day_len(focus);
                if len > 0 {
                    let new_row = row.min(len.saturating_sub(1));
                    self.cursor.set_focus_row(focus, new_row);
                }
            }
        }
        Ok(())
    }

    fn move_to_backlog(&mut self) -> miette::Result<()> {
        if let Some(id) = self.current_target_id() {
            if matches!(self.board.day_status_of(id), Some("done")) {
                return Ok(());
            }
            self.cursor.selection = None;
            self.runtime.block_on(self.services.todos.move_to_scope(
                id,
                ListScope::Backlog,
                MovePlacement::Bottom,
            ))?;
            self.refresh_board()?;
        }
        Ok(())
    }

    fn open_backlog(&mut self) {
        self.ui_mode = UiMode::Backlog;
    }

    fn handle_backlog_key(&mut self, key: KeyEvent) {
        if !matches!(key.code, KeyCode::Char('d')) {
            self.pending_delete = false;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('b') => {
                self.ui_mode = UiMode::Board;
            }
            KeyCode::Char('h') => self.handle_backlog_horizontal(Horizontal::Left),
            KeyCode::Char('l') => self.handle_backlog_horizontal(Horizontal::Right),
            KeyCode::Char('j') => self.handle_backlog_vertical(Vertical::Down),
            KeyCode::Char('k') => self.handle_backlog_vertical(Vertical::Up),
            KeyCode::Enter => self.toggle_backlog_selection(),
            KeyCode::Char('x') if key.modifiers.is_empty() => {
                self.mark_backlog_complete().ok();
            }
            KeyCode::Char('a') if key.modifiers.is_empty() => {
                self.open_add_todo_backlog();
            }
            KeyCode::Char('t') if key.modifiers.is_empty() => {
                self.move_backlog_to_day(0).ok();
            }
            KeyCode::Char('T') | KeyCode::Char('t')
                if key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                self.move_backlog_to_day(1).ok();
            }
            KeyCode::Char('d') if key.modifiers.is_empty() => {
                if self.pending_delete {
                    self.delete_backlog_current().ok();
                    self.pending_delete = false;
                } else {
                    self.pending_delete = true;
                }
            }
            KeyCode::Char(' ') if key.modifiers.is_empty() => {
                self.open_detail_backlog();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    fn move_backlog_to_day(&mut self, days_from_today: i64) -> miette::Result<()> {
        let Some(id) = self.backlog_current_target_id() else {
            return Ok(());
        };

        if matches!(self.board.backlog_status_of(id), Some("done")) {
            return Ok(());
        }

        let target_date = self.services.today() + ChronoDuration::days(days_from_today);

        self.backlog_cursor.selection = None;
        self.runtime.block_on(self.services.todos.move_to_scope(
            id,
            ListScope::Day(target_date),
            MovePlacement::Top,
        ))?;

        self.refresh_board()?;
        Ok(())
    }

    fn handle_backlog_horizontal(&mut self, dir: Horizontal) {
        if self.backlog_cursor.selection.is_some() {
            self.move_backlog_selected_horizontal(dir).ok();
        } else {
            self.backlog_cursor.move_horizontal(dir);
        }
    }

    fn handle_backlog_vertical(&mut self, dir: Vertical) {
        if self.backlog_cursor.selection.is_some() {
            let reorder_dir = match dir {
                Vertical::Up => ReorderDirection::Up,
                Vertical::Down => ReorderDirection::Down,
            };
            self.reorder_backlog_selected(reorder_dir).ok();
        } else {
            self.backlog_cursor.move_vertical(dir, &self.board);
        }
    }

    fn toggle_backlog_selection(&mut self) {
        if self.backlog_cursor.selection.is_some() {
            self.backlog_cursor.selection = None;
            return;
        }

        if let Some(id) = self.backlog_cursor.current_todo_id(&self.board) {
            let row = self
                .backlog_cursor
                .row_for(self.backlog_cursor.column, &self.board);
            self.backlog_cursor.selection = Some(BacklogSelection {
                id,
                column: self.backlog_cursor.column,
                row,
            });
        }
    }

    fn move_backlog_selected_horizontal(&mut self, dir: Horizontal) -> miette::Result<()> {
        let Some(selection) = self.backlog_cursor.selection else {
            return Ok(());
        };

        let target_col = match dir {
            Horizontal::Left => {
                if selection.column == 0 {
                    return Ok(());
                }
                selection.column - 1
            }
            Horizontal::Right => {
                if selection.column + 1 >= BACKLOG_COLUMNS {
                    return Ok(());
                }
                selection.column + 1
            }
        };

        self.runtime.block_on(
            self.services
                .todos
                .set_backlog_column(selection.id, target_col as i64),
        )?;

        self.refresh_backlog()?;

        self.backlog_cursor.selection = Some(BacklogSelection {
            column: target_col,
            row: None,
            ..selection
        });
        self.backlog_cursor.column = target_col;

        Ok(())
    }

    fn reorder_backlog_selected(&mut self, dir: ReorderDirection) -> miette::Result<()> {
        if let Some(selection) = self.backlog_cursor.selection {
            self.runtime
                .block_on(self.services.todos.reorder(selection.id, dir))?;
            if let Some(sel) = &mut self.backlog_cursor.selection {
                sel.row = None;
            }
            self.refresh_backlog()?;
        }
        Ok(())
    }

    fn backlog_current_target_id(&self) -> Option<Uuid> {
        self.backlog_cursor
            .selection
            .map(|sel| sel.id)
            .or_else(|| self.backlog_cursor.current_todo_id(&self.board))
    }

    fn delete_backlog_current(&mut self) -> miette::Result<()> {
        if let Some(id) = self.backlog_current_target_id() {
            let deleted = self.runtime.block_on(self.services.todos.delete(id))?;
            if deleted {
                self.backlog_cursor.selection = None;
                self.refresh_backlog()?;
            }
        }
        Ok(())
    }

    fn mark_backlog_complete(&mut self) -> miette::Result<()> {
        if let Some(id) = self.backlog_current_target_id() {
            let current_status = self
                .board
                .backlog_status_of(id)
                .unwrap_or("pending")
                .to_string();

            let col = self.backlog_cursor.column;
            let prev_row = self.backlog_cursor.row_for(col, &self.board);
            self.backlog_cursor.selection = None;

            if current_status == "done" {
                self.runtime
                    .block_on(self.services.todos.mark_pending(id))?;
            } else {
                let today = self.services.today();
                self.runtime
                    .block_on(self.services.todos.mark_done(id, today))?;
            }

            self.refresh_backlog()?;

            if let Some((new_col, row)) = self.board.find_backlog_position(id) {
                self.backlog_cursor.column = new_col;
                self.backlog_cursor.rows[new_col] = row;
            } else if let Some(row) = prev_row {
                let len = self.board.backlog_col_len(col);
                if len > 0 {
                    self.backlog_cursor.rows[col] = row.min(len.saturating_sub(1));
                }
            }
        }
        Ok(())
    }

    fn open_settings(&mut self) {
        let settings = SettingsState {
            week_start: self.week_pref,
        };
        self.ui_mode = UiMode::Settings(settings);
    }

    fn handle_settings_key(&mut self, key: KeyEvent) {
        if let UiMode::Settings(settings) = &mut self.ui_mode {
            let mut apply: Option<WeekStart> = None;
            let mut close = false;
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => close = true,
                KeyCode::Char('m') => {
                    let target = WeekStart::Monday;
                    if settings.week_start != target {
                        settings.week_start = target;
                        apply = Some(target);
                    }
                }
                KeyCode::Char('s') => {
                    let target = WeekStart::Sunday;
                    if settings.week_start != target {
                        settings.week_start = target;
                        apply = Some(target);
                    }
                }
                _ => {}
            }
            let _ = settings;
            if close {
                self.ui_mode = UiMode::Board;
            }
            if let Some(new_pref) = apply {
                self.apply_week_start(new_pref);
            }
        }
    }

    fn apply_week_start(&mut self, week_start: WeekStart) {
        if week_start == self.week_pref {
            return;
        }

        self.week_pref = week_start;

        if let Err(err) = self
            .runtime
            .block_on(self.services.config.save_week_start(week_start))
        {
            eprintln!("failed to save week start preference: {err}");
        }

        self.state = WeekState::new(self.services.today(), week_start);
        self.board = BoardData::new(self.state.columns.len());
        self.cursor = CursorState::new(self.state.columns.len());
        if let Some(idx) = self.state.column_index(self.services.today()) {
            self.cursor.set_focus_row(idx, 0);
        }

        self.refresh_board().ok();
    }

    fn draw_settings(&self, frame: &mut Frame<'_>, settings: &SettingsState) {
        let area = centered_rect(50, 25, frame.area());
        let block = Block::default()
            .title("Settings")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette::FOCUS));

        let week_text = match settings.week_start {
            WeekStart::Sunday => "Week Start: Sunday",
            WeekStart::Monday => "Week Start: Monday",
        };
        let instructions = vec![
            Line::from(week_text),
            Line::from("m: Monday, s: Sunday"),
            Line::from("Esc/Enter: close"),
        ];
        let paragraph = Paragraph::new(instructions).block(block);
        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);
    }

    fn open_add_todo_board(&mut self) {
        let target_date = self.state.columns[self.cursor.focus].date;
        self.ui_mode = UiMode::AddTodo(AddTodoState {
            input: String::new(),
            target: AddTarget::Day(target_date),
        });
    }

    fn open_add_todo_backlog(&mut self) {
        self.ui_mode = UiMode::AddTodo(AddTodoState {
            input: String::new(),
            target: AddTarget::BacklogColumn(self.backlog_cursor.column),
        });
    }

    fn handle_add_todo_key(&mut self, key: KeyEvent) {
        let UiMode::AddTodo(ref mut state) = self.ui_mode else {
            return;
        };

        match key.code {
            KeyCode::Esc => {
                self.ui_mode = match state.target {
                    AddTarget::Day(_) => UiMode::Board,
                    AddTarget::BacklogColumn(_) => UiMode::Backlog,
                };
            }
            KeyCode::Enter => {
                let input = std::mem::take(&mut state.input);
                let target = state.target.clone();
                if !input.trim().is_empty() {
                    self.submit_add_todo(input.trim().to_string(), target.clone())
                        .ok();
                }
                self.ui_mode = match target {
                    AddTarget::Day(_) => UiMode::Board,
                    AddTarget::BacklogColumn(_) => UiMode::Backlog,
                };
            }
            KeyCode::Char(c) => {
                state.input.push(c);
            }
            KeyCode::Backspace => {
                state.input.pop();
            }
            _ => {}
        }
    }

    fn submit_add_todo(&mut self, title: String, target: AddTarget) -> miette::Result<()> {
        match target {
            AddTarget::Day(date) => {
                self.runtime
                    .block_on(self.services.todos.add(&title, Some(date), None))?;
                self.refresh_board()?;
            }
            AddTarget::BacklogColumn(col) => {
                let model = self
                    .runtime
                    .block_on(self.services.todos.add(&title, None, None))?;
                self.runtime
                    .block_on(self.services.todos.set_backlog_column(model.id, col as i64))?;
                self.refresh_backlog()?;
            }
        }
        Ok(())
    }

    fn draw_add_todo(&self, frame: &mut Frame<'_>, state: &AddTodoState) {
        let area = centered_rect(60, 15, frame.area());
        let block = Block::default()
            .title("Add Todo")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette::FOCUS));

        let inner = block.inner(area);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let input_line =
            Line::from(format!("› {}_", state.input)).style(Style::default().fg(palette::ACTIVE));
        frame.render_widget(Paragraph::new(input_line), inner);
    }

    fn open_detail_board(&mut self) {
        let Some(id) = self.cursor.current_todo_id(&self.board) else {
            return;
        };
        self.open_detail(id, false);
    }

    fn open_detail_backlog(&mut self) {
        let Some(id) = self.backlog_cursor.current_todo_id(&self.board) else {
            return;
        };
        self.open_detail(id, true);
    }

    fn open_detail(&mut self, id: Uuid, from_backlog: bool) {
        let Ok(model) = self.runtime.block_on(self.services.todos.get(id)) else {
            return;
        };

        self.ui_mode = UiMode::Detail(DetailState {
            todo_id: model.id,
            title: model.title,
            date: model.scheduled_for,
            status: model.status,
            notes: model.notes.unwrap_or_default(),
            field: DetailField::Title,
            editing: None,
            from_backlog,
        });
    }

    fn handle_detail_key(&mut self, key: KeyEvent) {
        let UiMode::Detail(ref mut state) = self.ui_mode else {
            return;
        };

        if state.editing.is_some() {
            self.handle_detail_edit_key(key);
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                let from_backlog = state.from_backlog;
                self.ui_mode = if from_backlog {
                    UiMode::Backlog
                } else {
                    UiMode::Board
                };
                self.refresh_board().ok();
                self.refresh_backlog().ok();
            }
            KeyCode::Char('j') => {
                let UiMode::Detail(ref mut state) = self.ui_mode else {
                    return;
                };
                state.field = state.field.next();
            }
            KeyCode::Char('k') => {
                let UiMode::Detail(ref mut state) = self.ui_mode else {
                    return;
                };
                state.field = state.field.prev();
            }
            KeyCode::Enter => {
                let UiMode::Detail(ref mut state) = self.ui_mode else {
                    return;
                };
                if state.field.is_editable() {
                    state.editing = Some(state.field_value(state.field));
                }
            }
            KeyCode::Char('x') => {
                self.toggle_detail_status();
            }
            _ => {}
        }
    }

    fn handle_detail_edit_key(&mut self, key: KeyEvent) {
        let UiMode::Detail(ref mut state) = self.ui_mode else {
            return;
        };

        let Some(ref mut input) = state.editing else {
            return;
        };

        let is_notes = state.field == DetailField::Notes;

        match key.code {
            KeyCode::Esc => {
                self.finish_detail_edit(false);
            }
            KeyCode::Char('j') if is_notes && key.modifiers.contains(KeyModifiers::CONTROL) => {
                input.push('\n');
            }
            KeyCode::Enter => {
                self.finish_detail_edit(true);
            }
            KeyCode::Char(c) => {
                input.push(c);
            }
            KeyCode::Backspace => {
                input.pop();
            }
            _ => {}
        }
    }

    fn finish_detail_edit(&mut self, save: bool) {
        let UiMode::Detail(ref mut state) = self.ui_mode else {
            return;
        };

        let Some(input) = state.editing.take() else {
            return;
        };

        if !save {
            return;
        }

        let id = state.todo_id;
        let field = state.field;

        match field {
            DetailField::Title => {
                if !input.trim().is_empty()
                    && self
                        .runtime
                        .block_on(self.services.todos.update_title(id, input.trim().to_string()))
                        .is_ok()
                {
                    let UiMode::Detail(ref mut state) = self.ui_mode else {
                        return;
                    };
                    state.title = input.trim().to_string();
                }
            }
            DetailField::Date => {
                let new_date = if input.trim().eq_ignore_ascii_case("none")
                    || input.trim().eq_ignore_ascii_case("someday")
                    || input.trim().is_empty()
                {
                    Some(None)
                } else {
                    NaiveDate::parse_from_str(input.trim(), "%Y-%m-%d")
                        .ok()
                        .map(Some)
                };

                if let Some(date) = new_date
                    && self
                        .runtime
                        .block_on(self.services.todos.update_scheduled_for(id, date))
                        .is_ok()
                {
                    let UiMode::Detail(ref mut state) = self.ui_mode else {
                        return;
                    };
                    state.date = date;
                }
            }
            DetailField::Notes => {
                let notes = if input.trim().is_empty() {
                    None
                } else {
                    Some(input.clone())
                };
                if self
                    .runtime
                    .block_on(self.services.todos.update_notes(id, notes))
                    .is_ok()
                {
                    let UiMode::Detail(ref mut state) = self.ui_mode else {
                        return;
                    };
                    state.notes = input;
                }
            }
            DetailField::Status => {}
        }
    }

    fn toggle_detail_status(&mut self) {
        let UiMode::Detail(ref mut state) = self.ui_mode else {
            return;
        };

        let id = state.todo_id;
        let today = self.services.today();

        if state.status == "done" {
            if let Ok(model) = self.runtime.block_on(self.services.todos.mark_pending(id)) {
                state.status = model.status;
            }
        } else if let Ok(model) = self.runtime.block_on(self.services.todos.mark_done(id, today)) {
            state.status = model.status;
        }
    }

    fn draw_detail(&self, frame: &mut Frame<'_>, state: &DetailState) {
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
                    lines.push(Line::from("    (empty)").style(Style::default().fg(palette::TEXT_DIM)));
                } else {
                    for line in value.lines() {
                        lines.push(Line::from(format!("    {line}")).style(style));
                    }
                }
            } else {
                let prefix = if is_focused { "› " } else { "  " };
                let suffix = if is_editing { "_" } else { "" };
                lines.push(
                    Line::from(format!("{prefix}{label}: {value}{suffix}")).style(style),
                );
            }
        }

        lines.push(Line::from(""));
        lines.push(
            Line::from("[j/k] navigate  [Enter] edit/confirm  [x] toggle  [Esc] close")
                .style(Style::default().fg(palette::TEXT_DIM)),
        );
        lines.push(
            Line::from("[Ctrl+j] newline in notes")
                .style(Style::default().fg(palette::TEXT_DIM)),
        );

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    fn move_to_today(&mut self) -> miette::Result<()> {
        let Some(id) = self.cursor.current_todo_id(&self.board) else {
            return Ok(());
        };

        let today = self.services.today();
        self.runtime.block_on(
            self.services
                .todos
                .move_to_scope(id, crate::service::todo::ListScope::Day(today), MovePlacement::Top),
        )?;

        self.refresh_board()?;
        Ok(())
    }

    fn move_to_tomorrow(&mut self) -> miette::Result<()> {
        let Some(id) = self.cursor.current_todo_id(&self.board) else {
            return Ok(());
        };

        let tomorrow = self.services.today() + ChronoDuration::days(1);
        self.runtime.block_on(
            self.services.todos.move_to_scope(
                id,
                crate::service::todo::ListScope::Day(tomorrow),
                MovePlacement::Top,
            ),
        )?;

        self.refresh_board()?;
        Ok(())
    }
}

struct WeekState {
    week_start: NaiveDate,
    columns: Vec<ColumnMeta>,
}

impl WeekState {
    fn new(today: NaiveDate, preference: WeekStart) -> Self {
        let week_start = start_of_week(today, preference);
        Self {
            week_start,
            columns: build_columns(week_start),
        }
    }

    fn prev_week(&mut self) {
        self.week_start -= ChronoDuration::days(7);
        self.columns = build_columns(self.week_start);
    }

    fn next_week(&mut self) {
        self.week_start += ChronoDuration::days(7);
        self.columns = build_columns(self.week_start);
    }

    fn column_index(&self, date: NaiveDate) -> Option<usize> {
        self.columns.iter().position(|col| col.date == date)
    }
}

#[derive(Clone)]
struct ColumnMeta {
    title: String,
    date: NaiveDate,
}

#[derive(Clone)]
struct SettingsState {
    week_start: WeekStart,
}

#[derive(Clone)]
struct AddTodoState {
    input: String,
    target: AddTarget,
}

#[derive(Clone)]
enum AddTarget {
    Day(NaiveDate),
    BacklogColumn(usize),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DetailField {
    Title,
    Date,
    Status,
    Notes,
}

impl DetailField {
    fn next(self) -> Self {
        match self {
            Self::Title => Self::Date,
            Self::Date => Self::Status,
            Self::Status => Self::Notes,
            Self::Notes => Self::Notes,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Title => Self::Title,
            Self::Date => Self::Title,
            Self::Status => Self::Date,
            Self::Notes => Self::Status,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Date => "Date",
            Self::Status => "Status",
            Self::Notes => "Notes",
        }
    }

    fn is_editable(self) -> bool {
        !matches!(self, Self::Status)
    }
}

#[derive(Clone)]
struct DetailState {
    todo_id: Uuid,
    title: String,
    date: Option<NaiveDate>,
    status: String,
    notes: String,
    field: DetailField,
    editing: Option<String>,
    from_backlog: bool,
}

impl DetailState {
    fn field_value(&self, field: DetailField) -> String {
        match field {
            DetailField::Title => self.title.clone(),
            DetailField::Date => self
                .date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "none".to_string()),
            DetailField::Status => self.status.clone(),
            DetailField::Notes => self.notes.clone(),
        }
    }
}

enum UiMode {
    Board,
    Backlog,
    Settings(SettingsState),
    AddTodo(AddTodoState),
    Detail(DetailState),
}

fn build_columns(week_start: NaiveDate) -> Vec<ColumnMeta> {
    let mut cols = Vec::with_capacity(7);
    for offset in 0..7 {
        let date = week_start + ChronoDuration::days(offset);
        let title = format!(
            "{} {:02}/{:02}",
            weekday_label(date.weekday()),
            date.month(),
            date.day()
        );
        cols.push(ColumnMeta { title, date });
    }
    cols
}

fn weekday_label(day: chrono::Weekday) -> &'static str {
    match day {
        chrono::Weekday::Mon => "Mon",
        chrono::Weekday::Tue => "Tue",
        chrono::Weekday::Wed => "Wed",
        chrono::Weekday::Thu => "Thu",
        chrono::Weekday::Fri => "Fri",
        chrono::Weekday::Sat => "Sat",
        chrono::Weekday::Sun => "Sun",
    }
}

fn start_of_week(date: NaiveDate, preference: WeekStart) -> NaiveDate {
    let weekday = date.weekday();
    let offset = match preference {
        WeekStart::Sunday => weekday.num_days_from_sunday() as i64,
        WeekStart::Monday => weekday.num_days_from_monday() as i64,
    };
    date - ChronoDuration::days(offset)
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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

struct BoardData {
    days: Vec<Vec<TodoView>>,
    backlog_columns: [Vec<TodoView>; BACKLOG_COLUMNS],
}

impl BoardData {
    fn new(num_days: usize) -> Self {
        Self {
            days: vec![Vec::new(); num_days],
            backlog_columns: Default::default(),
        }
    }

    fn reset(&mut self, num_days: usize) {
        self.days = vec![Vec::new(); num_days];
        for col in &mut self.backlog_columns {
            col.clear();
        }
    }

    fn set_day(&mut self, idx: usize, todos: Vec<TodoView>) {
        if idx >= self.days.len() {
            self.days.resize(idx + 1, Vec::new());
        }
        self.days[idx] = todos;
    }

    fn day_len(&self, idx: usize) -> usize {
        self.days.get(idx).map(|d| d.len()).unwrap_or(0)
    }

    fn day_todo_id_at(&self, col: usize, row: usize) -> Option<Uuid> {
        self.days.get(col)?.get(row).map(|todo| todo.id)
    }

    fn set_backlog_column(&mut self, col: usize, todos: Vec<TodoView>) {
        if col < BACKLOG_COLUMNS {
            self.backlog_columns[col] = todos;
        }
    }

    fn backlog_col_len(&self, col: usize) -> usize {
        if col < BACKLOG_COLUMNS {
            self.backlog_columns[col].len()
        } else {
            0
        }
    }

    fn backlog_todo_id_at(&self, col: usize, row: usize) -> Option<Uuid> {
        self.backlog_columns.get(col)?.get(row).map(|todo| todo.id)
    }

    fn find_day_position(&self, id: Uuid) -> Option<(usize, usize)> {
        for (idx, day) in self.days.iter().enumerate() {
            if let Some(pos) = day.iter().position(|todo| todo.id == id) {
                return Some((idx, pos));
            }
        }
        None
    }

    fn find_backlog_position(&self, id: Uuid) -> Option<(usize, usize)> {
        for (col, items) in self.backlog_columns.iter().enumerate() {
            if let Some(pos) = items.iter().position(|todo| todo.id == id) {
                return Some((col, pos));
            }
        }
        None
    }

    fn day_status_of(&self, id: Uuid) -> Option<&str> {
        for day in &self.days {
            if let Some(todo) = day.iter().find(|todo| todo.id == id) {
                return Some(todo.status.as_str());
            }
        }
        None
    }

    fn backlog_status_of(&self, id: Uuid) -> Option<&str> {
        for col in &self.backlog_columns {
            if let Some(todo) = col.iter().find(|todo| todo.id == id) {
                return Some(todo.status.as_str());
            }
        }
        None
    }
}

#[derive(Clone)]
struct TodoView {
    id: Uuid,
    title: String,
    status: String,
}

impl TodoView {
    fn to_line_with_prefix(&self, selected: bool) -> Line<'_> {
        let text = if selected {
            format!("› {}", self.title)
        } else {
            self.title.clone()
        };
        let mut line = Line::from(text);
        if self.status == "done" {
            line.style = Style::default()
                .fg(palette::TEXT_DIM)
                .add_modifier(Modifier::CROSSED_OUT | Modifier::DIM);
        } else {
            line.style = Style::default().fg(palette::TEXT);
        }
        line
    }
}

impl From<todo::Model> for TodoView {
    fn from(model: todo::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            status: model.status,
        }
    }
}

struct CursorState {
    focus: usize,
    day_rows: Vec<usize>,
    selection: Option<Selection>,
}

impl CursorState {
    fn new(num_days: usize) -> Self {
        Self {
            focus: 0,
            day_rows: vec![0; num_days],
            selection: None,
        }
    }

    fn move_vertical(&mut self, dir: Vertical, board: &BoardData) {
        let len = board.day_len(self.focus);
        if len == 0 {
            return;
        }
        let row = &mut self.day_rows[self.focus];
        match dir {
            Vertical::Up => {
                if *row > 0 {
                    *row -= 1;
                }
            }
            Vertical::Down => {
                if *row + 1 < len {
                    *row += 1;
                }
            }
        }
        self.selection = None;
    }

    fn row_for(&self, col: usize, board: &BoardData) -> Option<usize> {
        let len = board.day_len(col);
        if len == 0 {
            return None;
        }
        self.day_rows.get(col).copied().filter(|r| *r < len)
    }

    fn line_style(&self, col: usize, row: usize, board: &BoardData) -> Style {
        if let Some(selection) = self.selection
            && selection.column == col
            && selection.row == Some(row)
        {
            return Style::default()
                .fg(palette::ACCENT)
                .add_modifier(Modifier::BOLD);
        }

        if self.focus == col
            && let Some(current_row) = self.row_for(col, board)
            && current_row == row
        {
            return Style::default().fg(palette::ACTIVE);
        }

        Style::default().fg(palette::TEXT)
    }

    fn is_selected(&self, id: Uuid) -> bool {
        self.selection.map(|s| s.id == id).unwrap_or(false)
    }

    fn current_todo_id(&self, board: &BoardData) -> Option<Uuid> {
        let row = self.row_for(self.focus, board)?;
        board.day_todo_id_at(self.focus, row)
    }

    fn sync_after_refresh(&mut self, day_count: usize, board: &BoardData) {
        self.day_rows.resize(day_count, 0);

        if self.focus >= day_count {
            self.focus = day_count.saturating_sub(1);
        }

        for (idx, row) in self.day_rows.iter_mut().enumerate() {
            let len = board.day_len(idx);
            if len == 0 {
                *row = 0;
            } else if *row >= len {
                *row = len - 1;
            }
        }

        if let Some(selection) = self.selection {
            if let Some((col, row)) = board.find_day_position(selection.id) {
                self.selection = Some(Selection {
                    column: col,
                    row: Some(row),
                    ..selection
                });
                self.day_rows[col] = row;
            } else {
                self.selection = None;
            }
        }
    }

    fn set_focus_row(&mut self, col: usize, row: usize) {
        self.focus = col;
        if col < self.day_rows.len() {
            self.day_rows[col] = row;
        }
        self.selection = None;
    }
}

struct BacklogCursor {
    column: usize,
    rows: [usize; BACKLOG_COLUMNS],
    selection: Option<BacklogSelection>,
}

impl BacklogCursor {
    fn new() -> Self {
        Self {
            column: 0,
            rows: [0; BACKLOG_COLUMNS],
            selection: None,
        }
    }

    fn move_horizontal(&mut self, dir: Horizontal) {
        match dir {
            Horizontal::Left => {
                if self.column > 0 {
                    self.column -= 1;
                }
            }
            Horizontal::Right => {
                if self.column + 1 < BACKLOG_COLUMNS {
                    self.column += 1;
                }
            }
        }
        self.selection = None;
    }

    fn move_vertical(&mut self, dir: Vertical, board: &BoardData) {
        let len = board.backlog_col_len(self.column);
        if len == 0 {
            return;
        }
        let row = &mut self.rows[self.column];
        match dir {
            Vertical::Up => {
                if *row > 0 {
                    *row -= 1;
                }
            }
            Vertical::Down => {
                if *row + 1 < len {
                    *row += 1;
                }
            }
        }
        self.selection = None;
    }

    fn row_for(&self, col: usize, board: &BoardData) -> Option<usize> {
        let len = board.backlog_col_len(col);
        if len == 0 {
            return None;
        }
        let row = self.rows[col];
        if row < len { Some(row) } else { None }
    }

    fn line_style(&self, col: usize, row: usize, board: &BoardData) -> Style {
        if let Some(selection) = self.selection
            && selection.column == col
            && selection.row == Some(row)
        {
            return Style::default()
                .fg(palette::ACCENT)
                .add_modifier(Modifier::BOLD);
        }

        if self.column == col
            && let Some(current_row) = self.row_for(col, board)
            && current_row == row
        {
            return Style::default().fg(palette::ACTIVE);
        }

        Style::default().fg(palette::TEXT)
    }

    fn is_selected(&self, id: Uuid) -> bool {
        self.selection.map(|s| s.id == id).unwrap_or(false)
    }

    fn current_todo_id(&self, board: &BoardData) -> Option<Uuid> {
        let row = self.row_for(self.column, board)?;
        board.backlog_todo_id_at(self.column, row)
    }

    fn sync_after_refresh(&mut self, board: &BoardData) {
        for col in 0..BACKLOG_COLUMNS {
            let len = board.backlog_col_len(col);
            if len == 0 {
                self.rows[col] = 0;
            } else if self.rows[col] >= len {
                self.rows[col] = len - 1;
            }
        }

        if let Some(selection) = self.selection {
            if let Some((col, row)) = board.find_backlog_position(selection.id) {
                self.selection = Some(BacklogSelection {
                    column: col,
                    row: Some(row),
                    ..selection
                });
                self.rows[col] = row;
            } else {
                self.selection = None;
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Horizontal {
    Left,
    Right,
}

#[derive(Clone, Copy)]
enum Vertical {
    Up,
    Down,
}

#[derive(Clone, Copy)]
struct Selection {
    id: Uuid,
    column: usize,
    row: Option<usize>,
}

#[derive(Clone, Copy)]
struct BacklogSelection {
    id: Uuid,
    column: usize,
    row: Option<usize>,
}

fn setup_terminal() -> miette::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()
        .into_diagnostic()
        .wrap_err("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .into_diagnostic()
        .wrap_err("failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
        .into_diagnostic()
        .wrap_err("failed to initialize terminal")
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
}

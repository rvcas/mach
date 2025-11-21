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
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use tokio::runtime::Handle;
use uuid::Uuid;

use crate::{
    entity::todo,
    service::{
        Services,
        todo::{ListOptions, ListScope, MovePlacement, ReorderDirection},
    },
};

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

struct App {
    services: Services,
    runtime: Handle,
    state: WeekState,
    board: BoardData,
    cursor: CursorState,
    pending_delete: bool,
    should_quit: bool,
}

impl App {
    fn new(services: Services, runtime: Handle) -> Self {
        let today = services.today();
        let state = WeekState::new(today);
        let board = BoardData::new(state.columns.len());
        let mut cursor = CursorState::new(state.columns.len());
        if let Some(idx) = state.column_index(today) {
            cursor.set_focus_row(FocusTarget::Day(idx), 0);
        }
        Self {
            services,
            runtime,
            state,
            board,
            cursor,
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
        if !matches!(key.code, KeyCode::Char('d')) {
            self.pending_delete = false;
        }

        match key.code {
            KeyCode::Char('q') if key.modifiers.is_empty() => self.should_quit = true,
            KeyCode::Esc => self.should_quit = true,
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
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(10)])
            .split(frame.area());

        let week_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(self.state.day_constraints())
            .split(chunks[0]);

        for (idx, area) in week_columns.iter().enumerate() {
            self.draw_column(frame, idx, *area);
        }

        self.draw_backlog(frame, chunks[1]);
    }

    fn draw_column(&self, frame: &mut Frame<'_>, idx: usize, area: Rect) {
        let column = &self.state.columns[idx];
        let focus = FocusTarget::Day(idx);
        let focused = self.cursor.focus == focus;
        let title = Line::from(column.title.clone());
        let border_style = if focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let block = Block::default()
            .title(title)
            .title_style(Style::default())
            .borders(Borders::ALL)
            .border_style(border_style);

        let mut lines = self.board.day_lines(idx);
        if let Some(row) = self.cursor.row_for(focus, &self.board) {
            if let Some(line) = lines.get_mut(row) {
                let highlight = self.cursor.line_style(focus, row, &self.board);
                line.style = line.style.patch(highlight);
            }
        }

        let body = if lines.is_empty() {
            Paragraph::new("No tasks yet").block(block)
        } else {
            Paragraph::new(lines).block(block)
        };
        frame.render_widget(body, area);
    }

    fn draw_backlog(&self, frame: &mut Frame<'_>, area: Rect) {
        let focus = FocusTarget::Backlog;
        let focused = self.cursor.focus == focus;
        let border_style = if focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let block = Block::default()
            .title("Someday / Backlog")
            .title_style(Style::default())
            .borders(Borders::ALL)
            .border_style(border_style);

        let mut lines = self.board.backlog_lines();
        if let Some(row) = self.cursor.row_for(focus, &self.board) {
            if let Some(line) = lines.get_mut(row) {
                let highlight = self.cursor.line_style(focus, row, &self.board);
                line.style = line.style.patch(highlight);
            }
        }

        let body = if lines.is_empty() {
            Paragraph::new("Backlog is empty").block(block)
        } else {
            Paragraph::new(lines).block(block)
        };
        frame.render_widget(body, area);
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
        if self.cursor.selection.is_some() {
            let target = self.cursor.horizontal_target(dir, self.state.columns.len());
            self.move_selected_to(target).ok();
        } else {
            self.cursor.move_horizontal(dir, self.state.columns.len());
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
            self.cursor
                .move_vertical(dir, &self.board, self.state.columns.len());
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
                focus: self.cursor.focus,
                row,
            });
        }
    }

    fn move_selected_to(&mut self, target: FocusTarget) -> miette::Result<()> {
        if let Some(selection) = self.cursor.selection {
            if selection.focus == target {
                return Ok(());
            }

            let scope = target.to_scope(&self.state);
            self.runtime.block_on(self.services.todos.move_to_scope(
                selection.id,
                scope,
                MovePlacement::Bottom,
            ))?;

            self.cursor.selection = Some(Selection {
                focus: target,
                row: None,
                ..selection
            });
            self.cursor.focus = target;
            self.refresh_board()?;
        }

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

        let backlog = self
            .runtime
            .block_on(self.services.todos.list(ListOptions {
                scope: ListScope::Backlog,
                include_done: true,
            }))?;
        self.board
            .set_backlog(backlog.into_iter().map(TodoView::from).collect());

        self.cursor
            .sync_after_refresh(self.state.columns.len(), &self.board);

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
            let current_status = self.board.status_of(id).unwrap_or("pending").to_string();

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

            if let Some((new_focus, row)) = self.board.find_position(id) {
                self.cursor.set_focus_row(new_focus, row);
            } else if let Some(row) = prev_row {
                let len = self.board.len_for(focus);
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
            if matches!(self.board.status_of(id), Some("done")) {
                return Ok(());
            }
            self.cursor.selection = None;
            self.runtime.block_on(self.services.todos.move_to_scope(
                id,
                ListScope::Backlog,
                MovePlacement::Bottom,
            ))?;
            self.refresh_board()?;
            if let Some((focus, row)) = self.board.find_position(id) {
                self.cursor.set_focus_row(focus, row);
            } else {
                self.cursor.set_focus_row(FocusTarget::Backlog, 0);
            }
        }
        Ok(())
    }
}

struct WeekState {
    week_start: NaiveDate,
    columns: Vec<ColumnMeta>,
}

impl WeekState {
    fn new(today: NaiveDate) -> Self {
        let week_start = start_of_week(today);
        Self {
            week_start,
            columns: build_columns(week_start),
        }
    }

    fn day_constraints(&self) -> Vec<Constraint> {
        let count = self.columns.len() as u32;
        self.columns
            .iter()
            .map(|_| Constraint::Ratio(1, count))
            .collect()
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

fn start_of_week(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday().number_from_monday() as i64 - 1;
    date - ChronoDuration::days(weekday)
}

struct BoardData {
    days: Vec<Vec<TodoView>>,
    backlog: Vec<TodoView>,
}

impl BoardData {
    fn new(num_days: usize) -> Self {
        Self {
            days: vec![Vec::new(); num_days],
            backlog: Vec::new(),
        }
    }

    fn reset(&mut self, num_days: usize) {
        self.days = vec![Vec::new(); num_days];
        self.backlog.clear();
    }

    fn set_day(&mut self, idx: usize, todos: Vec<TodoView>) {
        if idx >= self.days.len() {
            self.days.resize(idx + 1, Vec::new());
        }
        self.days[idx] = todos;
    }

    fn day_lines(&self, idx: usize) -> Vec<Line<'_>> {
        self.days
            .get(idx)
            .map(|todos| todos.iter().map(TodoView::to_line).collect())
            .unwrap_or_default()
    }

    fn len_for(&self, focus: FocusTarget) -> usize {
        match focus {
            FocusTarget::Day(idx) => self.days.get(idx).map(|d| d.len()).unwrap_or(0),
            FocusTarget::Backlog => self.backlog.len(),
        }
    }

    fn todo_id_at(&self, focus: FocusTarget, row: usize) -> Option<Uuid> {
        match focus {
            FocusTarget::Day(idx) => self.days.get(idx)?.get(row).map(|todo| todo.id),
            FocusTarget::Backlog => self.backlog.get(row).map(|todo| todo.id),
        }
    }

    fn set_backlog(&mut self, todos: Vec<TodoView>) {
        self.backlog = todos;
    }

    fn backlog_lines(&self) -> Vec<Line<'_>> {
        self.backlog.iter().map(TodoView::to_line).collect()
    }

    fn find_position(&self, id: Uuid) -> Option<(FocusTarget, usize)> {
        for (idx, day) in self.days.iter().enumerate() {
            if let Some(pos) = day.iter().position(|todo| todo.id == id) {
                return Some((FocusTarget::Day(idx), pos));
            }
        }

        if let Some(pos) = self.backlog.iter().position(|todo| todo.id == id) {
            return Some((FocusTarget::Backlog, pos));
        }

        None
    }

    fn status_of(&self, id: Uuid) -> Option<&str> {
        for day in &self.days {
            if let Some(todo) = day.iter().find(|todo| todo.id == id) {
                return Some(todo.status.as_str());
            }
        }
        self.backlog
            .iter()
            .find(|todo| todo.id == id)
            .map(|todo| todo.status.as_str())
    }
}

#[derive(Clone)]
struct TodoView {
    id: Uuid,
    title: String,
    status: String,
}

impl TodoView {
    fn to_line(&self) -> Line<'_> {
        let mut line = Line::from(self.title.clone());
        if self.status == "done" {
            line.style = Style::default().add_modifier(Modifier::CROSSED_OUT);
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusTarget {
    Day(usize),
    Backlog,
}

impl FocusTarget {
    fn to_scope(self, state: &WeekState) -> ListScope {
        match self {
            FocusTarget::Day(idx) => {
                let date = state.columns[idx].date;
                ListScope::Day(date)
            }
            FocusTarget::Backlog => ListScope::Backlog,
        }
    }
}

struct CursorState {
    focus: FocusTarget,
    day_rows: Vec<usize>,
    backlog_row: usize,
    selection: Option<Selection>,
}

impl CursorState {
    fn new(num_days: usize) -> Self {
        Self {
            focus: FocusTarget::Day(0),
            day_rows: vec![0; num_days],
            backlog_row: 0,
            selection: None,
        }
    }

    fn move_horizontal(&mut self, dir: Horizontal, day_count: usize) {
        self.focus = match (self.focus, dir) {
            (FocusTarget::Day(0), Horizontal::Left) => FocusTarget::Backlog,
            (FocusTarget::Day(idx), Horizontal::Left) => FocusTarget::Day(idx - 1),
            (FocusTarget::Day(idx), Horizontal::Right) if idx + 1 < day_count => {
                FocusTarget::Day(idx + 1)
            }
            (FocusTarget::Day(_), Horizontal::Right) => FocusTarget::Backlog,
            (FocusTarget::Backlog, Horizontal::Right) => FocusTarget::Day(0),
            (FocusTarget::Backlog, Horizontal::Left) => {
                FocusTarget::Day(day_count.saturating_sub(1))
            }
        };
        self.selection = None;
    }

    fn move_vertical(&mut self, dir: Vertical, board: &BoardData, day_count: usize) {
        match self.focus {
            FocusTarget::Day(idx) => {
                if idx >= day_count {
                    return;
                }
                let len = board.len_for(self.focus);
                if len == 0 {
                    self.day_rows[idx] = 0;
                    return;
                }
                let row = &mut self.day_rows[idx];
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
            }
            FocusTarget::Backlog => {
                let len = board.len_for(FocusTarget::Backlog);
                if len == 0 {
                    self.backlog_row = 0;
                    return;
                }
                match dir {
                    Vertical::Up => {
                        if self.backlog_row > 0 {
                            self.backlog_row -= 1;
                        }
                    }
                    Vertical::Down => {
                        if self.backlog_row + 1 < len {
                            self.backlog_row += 1;
                        }
                    }
                }
            }
        }
        self.selection = None;
    }

    fn horizontal_target(&self, dir: Horizontal, day_count: usize) -> FocusTarget {
        match (self.focus, dir) {
            (FocusTarget::Day(0), Horizontal::Left) => FocusTarget::Backlog,
            (FocusTarget::Day(idx), Horizontal::Left) => FocusTarget::Day(idx - 1),
            (FocusTarget::Day(idx), Horizontal::Right) if idx + 1 < day_count => {
                FocusTarget::Day(idx + 1)
            }
            (FocusTarget::Day(_), Horizontal::Right) => FocusTarget::Backlog,
            (FocusTarget::Backlog, Horizontal::Right) => FocusTarget::Day(0),
            (FocusTarget::Backlog, Horizontal::Left) => {
                FocusTarget::Day(day_count.saturating_sub(1))
            }
        }
    }

    fn row_for(&self, focus: FocusTarget, board: &BoardData) -> Option<usize> {
        let len = board.len_for(focus);
        if len == 0 {
            return None;
        }
        match focus {
            FocusTarget::Day(idx) => self.day_rows.get(idx).copied().filter(|r| *r < len),
            FocusTarget::Backlog => {
                if self.backlog_row < len {
                    Some(self.backlog_row)
                } else {
                    None
                }
            }
        }
    }

    fn line_style(&self, focus: FocusTarget, row: usize, board: &BoardData) -> Style {
        if let Some(selection) = self.selection {
            if selection.focus == focus && selection.row == Some(row) {
                return Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD);
            }
        }

        if self.focus == focus {
            if let Some(current_row) = self.row_for(focus, board) {
                if current_row == row {
                    return Style::default().fg(Color::Yellow);
                }
            }
        }

        Style::default()
    }

    fn current_todo_id(&self, board: &BoardData) -> Option<Uuid> {
        let row = self.row_for(self.focus, board)?;
        board.todo_id_at(self.focus, row)
    }

    fn sync_after_refresh(&mut self, day_count: usize, board: &BoardData) {
        self.day_rows.resize(day_count, 0);

        match self.focus {
            FocusTarget::Day(idx) if idx >= day_count => {
                self.focus = FocusTarget::Day(day_count.saturating_sub(1));
            }
            _ => {}
        }

        for (idx, row) in self.day_rows.iter_mut().enumerate() {
            let len = board.len_for(FocusTarget::Day(idx));
            if len == 0 {
                *row = 0;
            } else if *row >= len {
                *row = len - 1;
            }
        }

        let backlog_len = board.len_for(FocusTarget::Backlog);
        if backlog_len == 0 {
            self.backlog_row = 0;
        } else if self.backlog_row >= backlog_len {
            self.backlog_row = backlog_len - 1;
        }

        if let Some(selection) = self.selection {
            if let Some((focus, row)) = board.find_position(selection.id) {
                self.selection = Some(Selection {
                    focus,
                    row: Some(row),
                    ..selection
                });
                self.set_row_for(focus, row);
            } else {
                self.selection = None;
            }
        }
    }

    fn set_row_for(&mut self, focus: FocusTarget, row: usize) {
        match focus {
            FocusTarget::Day(idx) => {
                if idx < self.day_rows.len() {
                    self.day_rows[idx] = row;
                }
            }
            FocusTarget::Backlog => {
                self.backlog_row = row;
            }
        }
    }

    fn set_focus_row(&mut self, focus: FocusTarget, row: usize) {
        self.focus = focus;
        self.set_row_for(focus, row);
        self.selection = None;
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
    focus: FocusTarget,
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

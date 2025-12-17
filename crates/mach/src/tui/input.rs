use chrono::NaiveDate;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::service::config::WeekStart;
use crate::service::todo::ReorderDirection;

use super::App;
use super::cursor::{BacklogSelection, Horizontal, Selection, Vertical};
use super::modes::{AddTarget, DetailField, UiMode};
use super::state::BACKLOG_COLUMNS;

impl App {
    pub fn handle_event(&mut self, evt: Event) {
        if let Event::Key(key) = evt
            && key.kind == KeyEventKind::Press
        {
            self.handle_key_event(key);
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('?') {
            if matches!(self.ui_mode, UiMode::Board | UiMode::Backlog) {
                self.show_help = !self.show_help;
            }

            return;
        }

        if self.show_help {
            self.show_help = false;

            return;
        }

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

    pub fn handle_backlog_key(&mut self, key: KeyEvent) {
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

    pub fn handle_settings_key(&mut self, key: KeyEvent) {
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

    pub fn handle_add_todo_key(&mut self, key: KeyEvent) {
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

    pub fn handle_detail_key(&mut self, key: KeyEvent) {
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

    pub fn handle_detail_edit_key(&mut self, key: KeyEvent) {
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

    pub fn finish_detail_edit(&mut self, save: bool) {
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
                if input.trim().is_empty() {
                    let UiMode::Detail(ref mut state) = self.ui_mode else {
                        return;
                    };
                    state.error = Some("title cannot be empty".to_string());
                    return;
                }

                match self
                    .runtime
                    .block_on(self.services.todos.update_title(id, input.trim().to_string()))
                {
                    Ok(_) => {
                        let UiMode::Detail(ref mut state) = self.ui_mode else {
                            return;
                        };
                        state.title = input.trim().to_string();
                        state.error = None;
                    }
                    Err(e) => {
                        let UiMode::Detail(ref mut state) = self.ui_mode else {
                            return;
                        };
                        state.error = Some(e.to_string());
                    }
                }
            }
            DetailField::Project => {
                let project = if input.trim().is_empty() {
                    None
                } else {
                    Some(input.trim().to_string())
                };

                match self
                    .runtime
                    .block_on(self.services.todos.update_project(id, project.clone()))
                {
                    Ok(_) => {
                        let UiMode::Detail(ref mut state) = self.ui_mode else {
                            return;
                        };
                        state.project = project;
                        state.error = None;
                    }
                    Err(e) => {
                        let UiMode::Detail(ref mut state) = self.ui_mode else {
                            return;
                        };
                        state.error = Some(e.to_string());
                    }
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

                let Some(date) = new_date else {
                    let UiMode::Detail(ref mut state) = self.ui_mode else {
                        return;
                    };
                    state.error = Some("invalid date format (use YYYY-MM-DD or 'none')".to_string());
                    return;
                };

                match self
                    .runtime
                    .block_on(self.services.todos.update_scheduled_for(id, date))
                {
                    Ok(_) => {
                        let UiMode::Detail(ref mut state) = self.ui_mode else {
                            return;
                        };
                        state.date = date;
                        state.error = None;
                    }
                    Err(e) => {
                        let UiMode::Detail(ref mut state) = self.ui_mode else {
                            return;
                        };
                        state.error = Some(e.to_string());
                    }
                }
            }
            DetailField::Notes => {
                let notes = if input.trim().is_empty() {
                    None
                } else {
                    Some(input.clone())
                };

                match self
                    .runtime
                    .block_on(self.services.todos.update_notes(id, notes))
                {
                    Ok(_) => {
                        let UiMode::Detail(ref mut state) = self.ui_mode else {
                            return;
                        };
                        state.notes = input;
                        state.error = None;
                    }
                    Err(e) => {
                        let UiMode::Detail(ref mut state) = self.ui_mode else {
                            return;
                        };
                        state.error = Some(e.to_string());
                    }
                }
            }
            DetailField::Epic | DetailField::Status => {}
        }
    }

    pub fn handle_horizontal(&mut self, dir: Horizontal) {
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

    pub fn handle_vertical(&mut self, dir: Vertical) {
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

    pub fn toggle_selection(&mut self) {
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

    pub fn handle_backlog_horizontal(&mut self, dir: Horizontal) {
        if self.backlog_cursor.selection.is_some() {
            self.move_backlog_selected_horizontal(dir).ok();
        } else {
            self.backlog_cursor.move_horizontal(dir);
        }
    }

    pub fn handle_backlog_vertical(&mut self, dir: Vertical) {
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

    pub fn toggle_backlog_selection(&mut self) {
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

    pub fn change_week(&mut self, delta: i32) {
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

    pub fn move_backlog_selected_horizontal(&mut self, dir: Horizontal) -> miette::Result<()> {
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
}

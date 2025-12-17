use chrono::Duration as ChronoDuration;
use uuid::Uuid;

use crate::service::config::WeekStart;
use crate::service::todo::{ListOptions, ListScope, MovePlacement, ProjectFilter, ReorderDirection};

use super::App;
use super::cursor::{CursorState, Horizontal, Selection};
use super::modes::{AddTarget, AddTodoState, DetailField, DetailState, SettingsState, UiMode};
use super::state::{BACKLOG_COLUMNS, BoardData, TodoView, WeekState};

impl App {
    pub fn refresh_board(&mut self) -> miette::Result<()> {
        for (idx, column) in self.state.columns.iter().enumerate() {
            let opts = ListOptions {
                scope: ListScope::Day(column.date),
                include_done: true,
                project: ProjectFilter::Any,
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

    pub fn refresh_backlog(&mut self) -> miette::Result<()> {
        let all_backlog = self
            .runtime
            .block_on(self.services.todos.list(ListOptions {
                scope: ListScope::Backlog,
                include_done: true,
                project: ProjectFilter::Any,
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

    pub fn current_target_id(&self) -> Option<Uuid> {
        self.cursor
            .selection
            .map(|sel| sel.id)
            .or_else(|| self.cursor.current_todo_id(&self.board))
    }

    pub fn backlog_current_target_id(&self) -> Option<Uuid> {
        self.backlog_cursor
            .selection
            .map(|sel| sel.id)
            .or_else(|| self.backlog_cursor.current_todo_id(&self.board))
    }

    pub fn delete_current(&mut self) -> miette::Result<()> {
        if let Some(id) = self.current_target_id() {
            let deleted = self.runtime.block_on(self.services.todos.delete(id))?;

            if deleted {
                self.cursor.selection = None;
                self.refresh_board()?;
            }
        }
        Ok(())
    }

    pub fn delete_backlog_current(&mut self) -> miette::Result<()> {
        if let Some(id) = self.backlog_current_target_id() {
            let deleted = self.runtime.block_on(self.services.todos.delete(id))?;

            if deleted {
                self.backlog_cursor.selection = None;
                self.refresh_backlog()?;
            }
        }
        Ok(())
    }

    pub fn mark_complete(&mut self) -> miette::Result<()> {
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

    pub fn mark_backlog_complete(&mut self) -> miette::Result<()> {
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

            self.refresh_board()?;

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

    pub fn move_to_backlog(&mut self) -> miette::Result<()> {
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

    pub fn move_to_today(&mut self) -> miette::Result<()> {
        let Some(id) = self.cursor.current_todo_id(&self.board) else {
            return Ok(());
        };

        let today = self.services.today();

        self.runtime.block_on(self.services.todos.move_to_scope(
            id,
            ListScope::Day(today),
            MovePlacement::Top,
        ))?;

        self.refresh_board()?;

        Ok(())
    }

    pub fn move_to_tomorrow(&mut self) -> miette::Result<()> {
        let Some(id) = self.cursor.current_todo_id(&self.board) else {
            return Ok(());
        };

        let tomorrow = self.services.today() + ChronoDuration::days(1);

        self.runtime.block_on(self.services.todos.move_to_scope(
            id,
            ListScope::Day(tomorrow),
            MovePlacement::Top,
        ))?;

        self.refresh_board()?;

        Ok(())
    }

    pub fn move_backlog_to_day(&mut self, days_from_today: i64) -> miette::Result<()> {
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

    pub fn move_selected_horizontal(&mut self, dir: Horizontal) -> miette::Result<()> {
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

    pub fn reorder_selected(&mut self, dir: ReorderDirection) -> miette::Result<()> {
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

    pub fn reorder_backlog_selected(&mut self, dir: ReorderDirection) -> miette::Result<()> {
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

    pub fn open_backlog(&mut self) {
        self.ui_mode = UiMode::Backlog;
    }

    pub fn open_settings(&mut self) {
        let settings = SettingsState {
            week_start: self.week_pref,
        };

        self.ui_mode = UiMode::Settings(settings);
    }

    pub fn apply_week_start(&mut self, week_start: WeekStart) {
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

    pub fn open_add_todo_board(&mut self) {
        let target_date = self.state.columns[self.cursor.focus].date;
        self.ui_mode = UiMode::AddTodo(AddTodoState {
            input: String::new(),
            target: AddTarget::Day(target_date),
        });
    }

    pub fn open_add_todo_backlog(&mut self) {
        self.ui_mode = UiMode::AddTodo(AddTodoState {
            input: String::new(),
            target: AddTarget::BacklogColumn(self.backlog_cursor.column),
        });
    }

    pub fn submit_add_todo(&mut self, title: String, target: AddTarget) -> miette::Result<()> {
        match target {
            AddTarget::Day(date) => {
                self.runtime
                    .block_on(self.services.todos.add(&title, Some(date), None, None, None))?;
                self.refresh_board()?;
            }
            AddTarget::BacklogColumn(col) => {
                let model = self
                    .runtime
                    .block_on(self.services.todos.add(&title, None, None, None, None))?;
                self.runtime
                    .block_on(self.services.todos.set_backlog_column(model.id, col as i64))?;
                self.refresh_backlog()?;
            }
        }
        Ok(())
    }

    pub fn open_detail_board(&mut self) {
        let Some(id) = self.cursor.current_todo_id(&self.board) else {
            return;
        };
        self.open_detail(id, false);
    }

    pub fn open_detail_backlog(&mut self) {
        let Some(id) = self.backlog_cursor.current_todo_id(&self.board) else {
            return;
        };
        self.open_detail(id, true);
    }

    pub fn open_detail(&mut self, id: Uuid, from_backlog: bool) {
        let Ok(model) = self.runtime.block_on(self.services.todos.get(id)) else {
            return;
        };

        let epic_title = model.epic_id.and_then(|eid| {
            self.runtime
                .block_on(self.services.todos.get_epic_title(eid))
                .ok()
        });

        self.ui_mode = UiMode::Detail(DetailState {
            todo_id: model.id,
            title: model.title,
            project: model.project,
            epic_title,
            date: model.scheduled_for,
            status: model.status,
            notes: model.notes.unwrap_or_default(),
            field: DetailField::Title,
            editing: None,
            from_backlog,
            error: None,
        });
    }

    pub fn toggle_detail_status(&mut self) {
        let UiMode::Detail(ref mut state) = self.ui_mode else {
            return;
        };

        let id = state.todo_id;
        let today = self.services.today();

        if state.status == "done" {
            if let Ok(model) = self.runtime.block_on(self.services.todos.mark_pending(id)) {
                state.status = model.status;
            }
        } else if let Ok(model) = self
            .runtime
            .block_on(self.services.todos.mark_done(id, today))
        {
            state.status = model.status;
        }
    }
}

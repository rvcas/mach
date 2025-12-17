use ratatui::style::{Modifier, Style};
use uuid::Uuid;

use super::palette;
use super::state::{BACKLOG_COLUMNS, BoardData};

#[derive(Clone, Copy)]
pub enum Horizontal {
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub enum Vertical {
    Up,
    Down,
}

#[derive(Clone, Copy)]
pub struct Selection {
    pub id: Uuid,
    pub column: usize,
    pub row: Option<usize>,
}

#[derive(Clone, Copy)]
pub struct BacklogSelection {
    pub id: Uuid,
    pub column: usize,
    pub row: Option<usize>,
}

pub struct CursorState {
    pub focus: usize,
    pub day_rows: Vec<usize>,
    pub scroll_offsets: Vec<usize>,
    pub selection: Option<Selection>,
}

impl CursorState {
    pub fn new(num_days: usize) -> Self {
        Self {
            focus: 0,
            day_rows: vec![0; num_days],
            scroll_offsets: vec![0; num_days],
            selection: None,
        }
    }

    pub fn move_vertical(&mut self, dir: Vertical, board: &BoardData) {
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

    pub fn row_for(&self, col: usize, board: &BoardData) -> Option<usize> {
        let len = board.day_len(col);

        if len == 0 {
            return None;
        }

        self.day_rows.get(col).copied().filter(|r| *r < len)
    }

    pub fn line_style(&self, col: usize, row: usize, board: &BoardData) -> Style {
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

    pub fn is_selected(&self, id: Uuid) -> bool {
        self.selection.map(|s| s.id == id).unwrap_or(false)
    }

    pub fn current_todo_id(&self, board: &BoardData) -> Option<Uuid> {
        let row = self.row_for(self.focus, board)?;

        board.day_todo_id_at(self.focus, row)
    }

    pub fn sync_after_refresh(&mut self, day_count: usize, board: &BoardData) {
        self.day_rows.resize(day_count, 0);
        self.scroll_offsets.resize(day_count, 0);

        if self.focus >= day_count {
            self.focus = day_count.saturating_sub(1);
        }

        for (idx, row) in self.day_rows.iter_mut().enumerate() {
            let len = board.day_len(idx);

            if len == 0 {
                *row = 0;
                self.scroll_offsets[idx] = 0;
            } else {
                if *row >= len {
                    *row = len - 1;
                }
                if self.scroll_offsets[idx] >= len {
                    self.scroll_offsets[idx] = len.saturating_sub(1);
                }
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

    pub fn ensure_visible(&mut self, col: usize, visible_rows: usize) {
        if col >= self.scroll_offsets.len() {
            return;
        }

        let row = self.day_rows.get(col).copied().unwrap_or(0);
        let scroll = self.scroll_offsets[col];

        if row < scroll {
            self.scroll_offsets[col] = row;
        } else if row >= scroll + visible_rows {
            self.scroll_offsets[col] = row.saturating_sub(visible_rows.saturating_sub(1));
        }
    }

    pub fn scroll_offset(&self, col: usize) -> usize {
        self.scroll_offsets.get(col).copied().unwrap_or(0)
    }

    pub fn set_focus_row(&mut self, col: usize, row: usize) {
        self.focus = col;

        if col < self.day_rows.len() {
            self.day_rows[col] = row;
        }

        self.selection = None;
    }
}

pub struct BacklogCursor {
    pub column: usize,
    pub rows: [usize; BACKLOG_COLUMNS],
    pub scroll_offsets: [usize; BACKLOG_COLUMNS],
    pub selection: Option<BacklogSelection>,
}

impl BacklogCursor {
    pub fn new() -> Self {
        Self {
            column: 0,
            rows: [0; BACKLOG_COLUMNS],
            scroll_offsets: [0; BACKLOG_COLUMNS],
            selection: None,
        }
    }

    pub fn move_horizontal(&mut self, dir: Horizontal) {
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

    pub fn move_vertical(&mut self, dir: Vertical, board: &BoardData) {
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

    pub fn row_for(&self, col: usize, board: &BoardData) -> Option<usize> {
        let len = board.backlog_col_len(col);

        if len == 0 {
            return None;
        }

        let row = self.rows[col];

        if row < len { Some(row) } else { None }
    }

    pub fn line_style(&self, col: usize, row: usize, board: &BoardData) -> Style {
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

    pub fn is_selected(&self, id: Uuid) -> bool {
        self.selection.map(|s| s.id == id).unwrap_or(false)
    }

    pub fn current_todo_id(&self, board: &BoardData) -> Option<Uuid> {
        let row = self.row_for(self.column, board)?;

        board.backlog_todo_id_at(self.column, row)
    }

    pub fn sync_after_refresh(&mut self, board: &BoardData) {
        for col in 0..BACKLOG_COLUMNS {
            let len = board.backlog_col_len(col);
            if len == 0 {
                self.rows[col] = 0;
                self.scroll_offsets[col] = 0;
            } else {
                if self.rows[col] >= len {
                    self.rows[col] = len - 1;
                }
                if self.scroll_offsets[col] >= len {
                    self.scroll_offsets[col] = len.saturating_sub(1);
                }
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

    pub fn ensure_visible(&mut self, col: usize, visible_rows: usize) {
        if col >= BACKLOG_COLUMNS {
            return;
        }

        let row = self.rows[col];
        let scroll = self.scroll_offsets[col];

        if row < scroll {
            self.scroll_offsets[col] = row;
        } else if row >= scroll + visible_rows {
            self.scroll_offsets[col] = row.saturating_sub(visible_rows.saturating_sub(1));
        }
    }

    pub fn scroll_offset(&self, col: usize) -> usize {
        if col < BACKLOG_COLUMNS {
            self.scroll_offsets[col]
        } else {
            0
        }
    }
}

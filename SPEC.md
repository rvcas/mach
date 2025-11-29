# Mach Todo TUI — Product & Implementation Plan

## Vision

- Replace Tweek with a local-first Ratatui experience that keeps weekly planning
  frictionless.
- Provide both CLI subcommands (quick add/list) and an immersive TUI that mirrors
  Tweek's columns (weekdays + Someday backlog).
- Persist data via SeaORM 2 so schema-sync keeps SQLite in lockstep with
  the entity definitions without manual migrations.

## Core Concepts

- **Todo Entity**
  - Fields: `id` (UUID), `title` (String), `status` (String, defaults to `"pending"`),
    `scheduled_for` (`Option<Date>`), `order_index` (i64 for deterministic intra-column sorting),
    `backlog_column` (i64 for backlog column assignment),
    `created_at`, `updated_at`, `notes` (optional text), `metadata` (JSON for future tags/links).
  - Backlog/Someday is derived by `scheduled_for.is_none()`.
  - Only string statuses for now (`pending`, `done`), but stored as free-form
    string to allow future states (`in_progress`, `blocked`, etc.).
- **Week View**
  - Current week is calculated relative to `week_start` preference (Sunday or Monday).
  - Each column renders tasks sorted by `order_index`.
- **Backlog View**
  - Fullscreen view with 4 columns for organizing someday items.
  - Items assigned to columns via `backlog_column` field.
- Uncompleted todos that were scheduled in the past and are not backlog
  automatically roll into "today" during daily refresh
  (run on app start / midnight tick).
- Backlog items (`scheduled_for = None`) that get marked as done receive
  today's date so they appear in the current week's columns.

## Behavioral Rules

1. Adding without flags schedules the todo for "today" (respecting week start).
2. `--some-day` flag inserts into backlog (`scheduled_for = None`).
3. The list command shows today's tasks by default; `--some-day` flips to backlog.
4. Daily rollover job:
   - Find todos where `status != "done"`, `scheduled_for < today`,
     `scheduled_for.is_some()` → set `scheduled_for = today`.
   - Maintain `order_index` by appending to bottom of today's list.
5. Completion sets `status = "done"` and locks the todo to its current
   `scheduled_for` date. Completed items remain visible in the TUI but always
   sink below unfinished todos within the same column; CLI views require `--done`.

## CLI Behaviors

- `mach add [--some-day] "Buy milk"`: validates input, writes todo through
  service layer (SeaORM).
- `mach list [--some-day] [--done]`: prints a table
  (title, status, scheduled_for, order) for the filtered set.
- CLI shares service layer with TUI; never bypasses domain logic
  (e.g., auto-rollover runs before listing).

## TUI Interaction Model

### Weekly View (Board)

- **Navigation**: `h/j/k/l` move cursor left/down/up/right across columns/rows.
  - `h` at week start wraps to previous week's last day.
  - `l` at week end wraps to next week's first day.
- **Weekly navigation**:
  - `[` / `]` page the board back or forward by one week.
- **Selection**: `Enter` toggles selection (indicated by `›` prefix + magenta highlight). When selected:
  - `h/l` moves todo across days (wraps across weeks, updates `scheduled_for`).
  - `j/k` adjusts `order_index` inside the current column.
  - Second `Enter` drops selection.
- **Editing**:
  - `a`: open add todo popup for the focused column (new todo appears at top).
  - `Space`: open todo details modal (edit title, date, notes).
  - `dd`: delete highlighted/selected todo.
  - `x`: toggle completion status on the focused/selected todo.
  - `s`: move the focused/selected todo to Someday/backlog (pending items only).
  - `t`: move focused todo to today.
  - `T` (shift): move focused todo to tomorrow.
- **Views**:
  - `b`: open fullscreen backlog view.
  - `gs`: open settings modal.
- **Quit**: `q` or `Esc` exits the application.

### Backlog View

The backlog is a fullscreen view with 4 columns for organizing someday items.

- **Navigation**: `h/j/k/l` move cursor across 4 columns and rows.
- **Selection**: `Enter` toggles selection. When selected:
  - `h/l` moves todo between backlog columns (updates `backlog_column`).
  - `j/k` adjusts `order_index` inside the current column.
- **Editing**:
  - `a`: open add todo popup for the focused backlog column.
  - `Space`: open todo details modal (edit title, date, notes).
  - `dd`: delete highlighted/selected todo.
  - `x`: toggle completion status.
  - `t`: move focused/selected todo to today.
  - `T` (shift): move focused/selected todo to tomorrow.
- **Return**: `b`, `q`, or `Esc` returns to weekly view.

### Add Todo Popup

- Type todo title.
- `Enter`: submit (adds to top of target column).
- `Esc`: cancel.
- `Backspace`: delete character.

### Todo Details Modal

- `j/k`: navigate between fields (Title, Date, Status, Notes).
- `Enter`: edit focused field / confirm edit.
- `Ctrl+j`: insert newline (in notes field).
- `x`: toggle completion status.
- `Esc`: close modal (or cancel current edit).
- Date format: `YYYY-MM-DD`, or `none`/`someday` to clear.
- Changes auto-save on confirm.

### Settings Modal

- `m`: set week start to Monday.
- `s`: set week start to Sunday.
- `Esc` or `Enter`: close modal.

### Ordering / Visibility

- Completed todos always render at the bottom of their column, under all
  unfinished items; they remain visible rather than hidden.
- Newly added todos appear at the top of the column, above completed entries.
- Moved todos (via `h/l`) appear at the top of the target column.

### Visual Design

- **Terminal-adaptive palette** (uses ANSI colors that respect user's theme):
  - LightBlue: column focus (separators, title underline).
  - Yellow: row focus (focused todo, adjacent row separators).
  - Magenta + Bold: selected todo (with `›` prefix).
  - DarkGray: unfocused separators, completed todos.
- Vertical line separators (`│`) between columns.
- Dashed line separators (`---`) between todos within a column.
- Centered column titles with full-width underlines.
- Works on both light and dark terminal themes.

### Help Overlay

- `?`: toggle help popup (bottom-right, context-aware).

### Future Hotkeys

- `/`: search.

## Persistence & Sync

- SQLite database backed by `mach.db` in user data directory.
- SeaORM 2 entity modules under `crates/mach/src/entity/`; `entity/mod.rs`
  re-exports for registry scanning.
- Features enabled in `Cargo.toml`: `entity-registry`, `schema-sync`,
  `sqlx-sqlite`, `runtime-tokio-rustls`, `with-chrono`.
- On startup:
  1. Connect to SQLite database.
  2. Run `db.get_schema_registry("mach::entity::*").sync(db)` to reconcile schema.
  3. Execute rollover task before launching CLI output or TUI.

## Configuration

- `MachConfig` rows live inside the database (no external config files):
  - `week_start`: `"monday"` or `"sunday"` (default).
  - `keybindings`: optional overrides (future).
  - `auto_rollover`: bool (default true).
  - TUI exposes a settings modal (`gs`) where `Week Start` can be toggled
    and saved immediately via SeaORM upsert.

## Implementation Checklist

- [x] Define SeaORM entity + migration-less schema
      (Todo table, metadata JSON column, config table if needed).
- [x] Implement persistence services: add, list (with filters), delete, reorder,
      move, complete, rollover.
- [x] Wire CLI subcommands to services (`add`, `list`) respecting flags.
- [x] Build rollover runner invoked on startup + scheduled timer.
- [x] Scaffold Ratatui app shell: layout, input handling loop, state store.
- [x] Implement navigation + selection + reorder semantics (including keeping
      completed todos pinned under active ones).
- [x] Support deletion (`dd`) and completion toggle (`x`).
- [x] Render backlog as separate fullscreen view with 4 columns.
- [x] Add inline todo creation (`a` key).
- [x] Add configuration loading/saving (week start toggle via TUI modal).
- [x] Implement terminal-adaptive visual design.
- [x] Add todo details modal (`Space` key) for editing title, date, notes.
- [x] Add `t`/`T` shortcuts in weekly view to move todos to today/tomorrow.
- [x] Add help overlay (`?` key) with context-aware shortcuts.
- [ ] Tests: unit tests for services (rollover, ordering) + integration tests
      for CLI.

## Known Gaps / Open Questions

- Do we allow multiple workspaces/boards? (Assume single board for MVP.)
- Should deletion be permanent or soft-delete (status `archived`)? For now,
  permanent with possible undo later.
- Need accessibility plan for non-Vim users (perhaps optional Emacs/Arrow mode).
- Completion keybindings should remain customizable; default is `x` but expose
  it via settings later in case platforms reserve it.

This SPEC should evolve; update checkpoints as tasks complete or requirements shift.

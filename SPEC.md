# Mach Todo TUI — Product & Implementation Plan

## Vision

- Replace Tweek with a local-first Ratatui experience that keeps weekly planning
  frictionless.
- Provide both CLI subcommands (quick add/list) and an immersive TUI that mirrors
  Tweek’s columns (weekdays + Someday backlog).
- Persist data via SeaORM 2 + Turso so schema-sync keeps SQLite in lockstep with
  the entity definitions without manual migrations.

## Core Concepts

- **Todo Entity**
  - Fields: `id` (UUID), `title` (String), `status` (String, defaults to `"pending"`),
    `scheduled_for` (`Option<Date>`), `order_index` (i64 for deterministic intra-column sorting),
    `created_at`, `updated_at`, `notes` (optional text), `metadata` (JSON for future tags/links).
  - Backlog/Someday is derived by `scheduled_for.is_none()`.
  - Only string statuses for now (`pending`, `done`), but stored as free-form
    string to allow future states (`in_progress`, `blocked`, etc.).
- **Week View**
  - Current week is calculated relative to `week_start` preference (Sunday or Monday).
  - Each column renders tasks sorted by `order_index`; backlog gets its own column/panel.
- Uncompleted todos that were scheduled in the past and are not backlog
  automatically roll into “today” during daily refresh
  (run on app start / midnight tick).
- Backlog items (`scheduled_for = None`) that get marked as done receive
  today’s date so they appear in the current week’s columns.

## Behavioral Rules

1. Adding without flags schedules the todo for “today” (respecting week start).
2. `--some-day` flag inserts into backlog (`scheduled_for = None`).
3. The list command shows today’s tasks by default; `--some-day` flips to backlog.
4. Daily rollover job:
   - Find todos where `status != "done"`, `scheduled_for < today`,
    `scheduled_for.is_some()` → set `scheduled_for = today`.
   - Maintain `order_index` by appending to bottom of today’s list.
5. Completion sets `status = "done"` and locks the todo to its current
  `scheduled_for` date. Completed items remain visible in the TUI but always
  sink below unfinished todos within the same column; CLI views require `--done`.

## CLI Behaviors

- `mach add [--some-day] "Buy milk"`: validates input, writes todo through
  service layer (Turso/SeaORM).
- `mach list [--some-day] [--done]`: prints a table
  (title, status, scheduled_for, order) for the filtered set.
- CLI shares service layer with TUI; never bypasses domain logic
  (e.g., auto-rollover runs before listing).

## TUI Interaction Model

- **Navigation**: `h/j/k/l` move cursor left/down/up/right across columns/rows.
- Weekly navigation:
  - `[` / `]` page the board back or forward by one week (vim-style history jump).
  - Optional `Shift+[`, `Shift+]` could jump multiple weeks later.
- **Selection**: `Enter` toggles selection. When selected:
  - `h/l` moves todo across days/backlog (updates `scheduled_for` or None).
  - `j/k` adjusts `order_index` inside the current column.
  - Second `Enter` drops selection.
- **Ordering / Visibility**
  - Completed todos always render at the bottom of their column, under all
    unfinished items; they remain visible rather than hidden.
  - Newly added todos for “today” appear at the top of the active stack, above
    any completed entries.
- **Editing**
  - `dd`: delete highlighted todo (prompt for confirmation if desired).
  - `x`: toggle completion status on the focused/selected todo.
  - `s`: move the focused/selected todo to Someday/backlog (pending items only).
- **Settings**
  - `gs`: open the settings modal; `m` sets week start to Monday, `s` sets it to Sunday, persisting immediately in the database. `Esc`/`Enter` closes the modal.
- **Other hotkeys** (future): `/` to search, `a` to add inline, `?` help.

## Persistence & Sync

- Turso local builder backed by `mach.db`.
- SeaORM 2 entity modules under `crates/mach/src/entity/`; `entity/mod.rs`
  re-exports for registry scanning.
- Features enabled in `Cargo.toml`: `entity-registry`, `schema-sync`,
  `sqlx-sqlite`, `runtime-tokio-rustls`, `with-chrono`.
- On startup:
  1. Connect to Turso.
  2. Run `db.get_schema_registry("mach::entity::*").sync(db)` to reconcile schema.
  3. Execute rollover task before launching CLI output or TUI.

## Configuration

- `MachConfig` rows live inside the Turso database (no external config files):
  - `week_start`: `"monday"` or `"sunday"` (default).
  - `keybindings`: optional overrides (future).
  - `auto_rollover`: bool (default true).
  - TUI exposes a settings modal (e.g., `gs`) where `Week Start` can be toggled
    with `h/l` and saved immediately via SeaORM upsert.

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
- [x] Render backlog column distinct from dated columns.
- [ ] Add configuration loading/saving (week start toggle via TUI modal, future
      keybindings).
- [ ] Tests: unit tests for services (rollover, ordering) + integration tests
      for CLI.
- [ ] Update SPEC.md + AGENTS.md as features land.

## Known Gaps / Open Questions

- Do we allow multiple workspaces/boards? (Assume single board for MVP.)
- Should deletion be permanent or soft-delete (status `archived`)? For now,
  permanent with possible undo later.
- Need accessibility plan for non-Vim users (perhaps optional Emacs/Arrow mode).
- Completion keybindings should remain customizable; default is `x` but expose it via settings later in case platforms reserve it.

This SPEC should evolve; update checkpoints as tasks complete or requirements shift.

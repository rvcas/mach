# Version 0.3.0 (2025-12-20)

- Add workspaces entity for top-level organization
- Add projects entity within workspaces for grouping related todos
- Add `mach workspaces create` command to create a workspace
- Add `mach workspaces list` command to list workspaces with stats
- Add `mach workspaces update` command to rename a workspace
- Add `mach projects create` command to create a project in a workspace
- Add `mach projects list` command to list projects with stats
- Add `mach projects update` command to update project name/status
- Add `mach projects done` command to mark a project as done
- Add `mach projects reopen` command to reopen a project
- Add `mach done` command to mark a todo as done by title or id
- Add `mach reopen` command to reopen a completed todo
- Add `mach update` command to update todo title, date, notes, workspace, project
- Add `mach delete` command to delete a todo by title or id
- Add `-w/--workspace` and `-p/--project` flags to `mach add`
- Add `-i/--id` flag to `mach list`, `mach workspaces list`, `mach projects list`
- Add workspace and project columns to `mach list` output
- Add visible aliases for all commands (shown in `--help`)

# Version 0.2.4 (2025-12-04)

- Fix double key press on Windows by filtering to only handle key press events
- Fix auto-rollover to preserve relative sort order of rolled-over todos

# Version 0.2.3 (2025-11-29)

- Add sponsor link to CLI help message (`mach --help`)

# Version 0.2.2 (2025-11-28)

- Readme updates

# Version 0.2.1 (2025-11-28)

- Fix schema registry path causing "no such table" error on first run

# Version 0.2.0 (2025-11-28)

- Add fullscreen Backlog view (`b` key) with 4-column layout for organizing someday items
- Add inline todo creation (`a` key) from both Weekly and Backlog views
- Add todo details modal (`Space` key) for editing title, date, and notes
- Add Settings modal (`gs`) for configuring week start day (Monday/Sunday)
- Add context-aware help popup (`?` key) for Weekly and Backlog views
- Add `t`/`T` shortcuts to move todos to today/tomorrow from both views
- Add `h`/`l` navigation wrapping across weeks
- Add dashed line separators between todos with focus-aware highlighting
- Add terminal-adaptive color palette (LightBlue, Yellow, Magenta, DarkGray)
- Add backlog column assignment (`backlog_column` field) for organizing someday items
- Refactor TUI from 2000+ line monolith into focused modules
- Fix bug where marking backlog items complete didn't refresh the weekly view

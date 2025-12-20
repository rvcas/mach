---
title: How It Works
description: Understanding mach's behavior and data model.
---

## Scheduling

- **Scheduled todos** have a date and appear in that day's column
- **Backlog todos** have no date (`scheduled_for = None`) and appear in the backlog view
- Use `s` to send a todo to the backlog, `t`/`T` to schedule it for today/tomorrow

## Automatic Rollover

Overdue incomplete todos automatically roll forward to today when you launch mach.

If you had a task scheduled for yesterday that you didn't complete, it will appear in today's column the next time you open the app. This keeps your focus on what's actionable now.

## Ordering

- **New todos** appear at the top of their column
- **Completed todos** sink to the bottom, below all incomplete items
- **Moved todos** (via `h`/`l`) appear at the top of the target column
- Use `j`/`k` while selected to manually reorder within a column

## Completion Behavior

When you mark a backlog item complete (`x`), it receives today's date so it appears in your weekly view as a completed task. This gives you a record of when things got done.

## Workspaces & Projects

Organize your todos with a two-level hierarchy:

- **Workspaces** — Top-level containers for related work (e.g., "Personal", "Work")
- **Projects** — Groups within a workspace (e.g., "Q1 Goals", "Home Renovation")

Todos can optionally belong to a workspace and/or project. If a todo is assigned to a project, it automatically belongs to that project's workspace.

Projects have a status: `pending`, `done`, or `permanent`. Permanent projects are for ongoing work that's never "complete" (like "Daily Standup").

## Data Storage

Mach stores everything in a local SQLite database:

- **macOS**: `~/Library/Application Support/mach/mach.db`
- **Linux**: `~/.local/share/mach/mach.db`
- **Windows**: `%APPDATA%\mach\mach.db`

No cloud sync, no account required. Your data stays on your machine.

## Week Start Preference

By default, weeks start on Sunday. Press `gs` in the weekly view to open settings and switch to Monday if you prefer.

This preference is stored in the database and persists across sessions.

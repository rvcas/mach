---
title: CLI Reference
description: Complete reference for all mach command-line commands.
---

## Overview

Mach can be used entirely from the command line. Run `mach` with no arguments to launch the TUI, or use subcommands to manage todos, workspaces, and projects directly.

## Todos

### mach add

Add a new todo. Alias: `a`

```sh
mach add "Buy groceries"
mach add --some-day "Learn piano"
mach add -w myworkspace "Team meeting"
mach add -p myproject "Fix bug"
```

| Flag                  | Description                              |
| --------------------- | ---------------------------------------- |
| `-s`, `--some-day`    | Add to backlog instead of today          |
| `-w`, `--workspace`   | Assign to workspace (name or UUID)       |
| `-p`, `--project`     | Assign to project (name or UUID)         |

When using `-p/--project`, the todo automatically inherits the project's workspace.

### mach list

List todos. Alias: `l`

```sh
mach list              # today's tasks
mach list --some-day   # backlog items
mach list --done       # completed items
mach list -i           # include id column
```

| Flag               | Description               |
| ------------------ | ------------------------- |
| `-s`, `--some-day` | List backlog items        |
| `-d`, `--done`     | Include completed todos   |
| `-i`, `--id`       | Show UUID column          |

### mach done

Mark a todo as done. Alias: `d`

```sh
mach done "Buy groceries"
mach done 550e8400-e29b-41d4-a716-446655440000
```

The reference can be a todo title or UUID. If multiple todos match the title, you'll be prompted to use the UUID instead (run `mach list -i` to see UUIDs).

### mach reopen

Reopen a completed todo (set status back to pending). Alias: `r`

```sh
mach reopen "Buy groceries"
```

### mach update

Update a todo's properties. Alias: `u`

```sh
mach update "Buy groceries" --title "Buy organic groceries"
mach update "Fix bug" --day 2025-01-15
mach update "Fix bug" --day someday
mach update "Meeting" --notes "Discuss Q1 roadmap"
mach update "Task" -w myworkspace -p myproject
```

| Flag                | Description                                    |
| ------------------- | ---------------------------------------------- |
| `-t`, `--title`     | New title                                      |
| `-d`, `--day`       | New date (YYYY-MM-DD) or "none"/"someday"      |
| `-n`, `--notes`     | New notes                                      |
| `-w`, `--workspace` | Assign to workspace (name or UUID)             |
| `-p`, `--project`   | Assign to project (name or UUID)               |

### mach delete

Delete a todo. Alias: `rm`

```sh
mach delete "Buy groceries"
mach delete 550e8400-e29b-41d4-a716-446655440000
```

## Workspaces

Workspaces provide top-level organization for grouping related projects and todos.

### mach workspaces create

Create a new workspace. Alias: `w c`

```sh
mach workspaces create "Personal"
mach w c "Work"
```

### mach workspaces list

List all workspaces with statistics. Alias: `w l`

```sh
mach workspaces list
mach w l -i    # include id column
```

| Flag         | Description       |
| ------------ | ----------------- |
| `-i`, `--id` | Show UUID column  |

Output shows: name, project count, todo count, completed, remaining, created date, updated date.

### mach workspaces update

Update a workspace. Alias: `w u`

```sh
mach workspaces update "Personal" --name "Personal Life"
```

| Flag           | Description |
| -------------- | ----------- |
| `-n`, `--name` | New name    |

## Projects

Projects belong to a workspace and group related todos together.

### mach projects create

Create a new project. Alias: `p c`

```sh
mach projects create -w "Work" "Q1 Goals"
mach p c -w Work --permanent "Daily Standup"
```

| Flag                | Description                                    |
| ------------------- | ---------------------------------------------- |
| `-w`, `--workspace` | Workspace name or UUID (required)              |
| `-p`, `--permanent` | Set status to permanent instead of pending     |

### mach projects list

List projects. Alias: `p l`

```sh
mach projects list
mach p l -w Work       # filter by workspace
mach p l -i            # include id column
```

| Flag                | Description                  |
| ------------------- | ---------------------------- |
| `-w`, `--workspace` | Filter by workspace          |
| `-i`, `--id`        | Show UUID column             |

Output shows: name, status, todo count, completed, remaining, created date, updated date.

### mach projects update

Update a project. Alias: `p u`

```sh
mach projects update "Q1 Goals" --name "Q1 OKRs"
mach projects update "Q1 Goals" --status done
```

| Flag             | Description                               |
| ---------------- | ----------------------------------------- |
| `-n`, `--name`   | New name                                  |
| `-s`, `--status` | New status: pending, done, or permanent   |

### mach projects done

Mark a project as done. Alias: `p d`

```sh
mach projects done "Q1 Goals"
```

### mach projects reopen

Reopen a project (set status to pending). Alias: `p r`

```sh
mach projects reopen "Q1 Goals"
```

## Reference Resolution

Commands that accept a reference (like `done`, `update`, `delete`) can use either:

- **Title/Name**: Matches by the todo/workspace/project title or name
- **UUID**: Matches by the unique identifier

If multiple items match a title, you'll get an error asking you to use the UUID. Run the corresponding list command with `-i` to see UUIDs.

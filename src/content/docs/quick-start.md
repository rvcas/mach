---
title: Quick Start
description: Get up and running with mach in minutes.
---

## Launch the TUI

After [installing](/installation/), simply run:

```sh
mach
```

This opens the weekly planner view. Your cursor starts on today's column.

## Basic Navigation

- **`h`/`l`** — Move left/right between days
- **`j`/`k`** — Move up/down within a column
- **`[`/`]`** — Jump to previous/next week

## Add Your First Todo

1. Press **`a`** to open the add popup
2. Type your task
3. Press **`Enter`** to add it

The todo appears at the top of the focused column.

## Move and Organize

1. Navigate to a todo with `j`/`k`
2. Press **`Enter`** to select it (you'll see the `›` indicator)
3. Use **`h`/`l`** to move it between days
4. Use **`j`/`k`** to reorder within the column
5. Press **`Enter`** again to drop it

## Complete a Todo

Press **`x`** on any todo to toggle its completion status. Completed todos sink to the bottom of the column.

## Use the Backlog

Press **`b`** to open the backlog — a fullscreen 4-column view for "someday" items. Great for ideas you want to capture but not schedule yet.

- **`t`** — Move a backlog item to today
- **`T`** — Move to tomorrow
- **`b`** or **`Esc`** — Return to weekly view

## CLI Commands

You can also manage todos from the command line:

```sh
# Add a todo for today
mach add "Buy groceries"

# Add to backlog
mach add --some-day "Learn piano"

# List today's tasks
mach list

# List backlog items
mach list --some-day

# List completed items
mach list --done
```

## Get Help

Press **`?`** anytime in the TUI to see available keyboard shortcuts for the current view.

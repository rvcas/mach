# Mach

A terminal-based weekly planner inspired by [Tweek](https://tweek.so). Plan your week with vim-style navigation, organize someday items in a backlog, and stay focused without leaving your terminal.

## Features

- **Weekly view** — 7-day columns showing your week at a glance
- **Backlog view** — 4-column organizer for "someday" items
- **Vim navigation** — `h/j/k/l` to move, `Enter` to select and drag
- **Local-first** — SQLite storage, no cloud, no account needed
- **Adaptive colors** — Uses terminal theme colors for universal compatibility

## Installation

### From source

```sh
git clone https://github.com/orbistry/mach
cd mach
cargo install --path crates/mach
```

### With Cargo

```sh
cargo install mach
```

## Quick Start

Launch the TUI:

```sh
mach
```

Add a todo from the command line:

```sh
mach add "Buy groceries"
mach add --some-day "Learn piano"
```

List todos:

```sh
mach list              # today's tasks
mach list --some-day   # backlog items
mach list --done       # completed items
```

## Keyboard Shortcuts

### Weekly View

| Key | Action |
|-----|--------|
| `h/l` | Move left/right between days |
| `j/k` | Move down/up within a column |
| `[/]` | Previous/next week |
| `Enter` | Select item (then `h/l` moves it, `j/k` reorders) |
| `Space` | Open todo details (edit title, date, notes) |
| `a` | Add new todo to focused column |
| `x` | Toggle completion |
| `dd` | Delete todo |
| `s` | Send to backlog |
| `t` | Move to today |
| `T` | Move to tomorrow |
| `b` | Open backlog view |
| `gs` | Settings (week start day) |
| `q/Esc` | Quit |

### Backlog View

| Key | Action |
|-----|--------|
| `h/j/k/l` | Navigate across 4 columns |
| `Enter` | Select item (then `h/l` moves between columns) |
| `Space` | Open todo details |
| `a` | Add new todo |
| `x` | Toggle completion |
| `dd` | Delete |
| `t` | Move to today |
| `T` | Move to tomorrow |
| `b/q/Esc` | Return to weekly view |

### Todo Details

| Key | Action |
|-----|--------|
| `j/k` | Navigate between fields |
| `Enter` | Edit / confirm |
| `Ctrl+j` | New line (in notes) |
| `x` | Toggle completion |
| `Esc` | Close (or cancel edit) |

### Add Todo Popup

| Key | Action |
|-----|--------|
| `Enter` | Submit |
| `Esc` | Cancel |
| `Backspace` | Delete character |

## How It Works

- Todos scheduled for a day appear in that day's column
- Overdue incomplete todos automatically roll forward to today
- Completed todos sink to the bottom of their column
- New todos appear at the top of the column

## License

[MIT](LICENSE)

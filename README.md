# Mach

A terminal-based weekly planner inspired by [Tweek](https://tweek.so). Plan your week with vim-style navigation, organize someday items in a backlog, and stay focused without leaving your terminal.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)

## Features

- **Weekly view** — 7-day columns showing your week at a glance
- **Backlog view** — 4-column organizer for "someday" items
- **Vim navigation** — `h/j/k/l` to move, `Enter` to select and drag
- **Local-first** — SQLite storage, no cloud, no account needed
- **Colorblind-friendly** — Cyan/Yellow/Magenta palette for clear focus states

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
| `a` | Add new todo to focused column |
| `x` | Toggle completion |
| `dd` | Delete todo |
| `s` | Send to backlog |
| `b` | Open backlog view |
| `gs` | Settings (week start day) |
| `q/Esc` | Quit |

### Backlog View

| Key | Action |
|-----|--------|
| `h/j/k/l` | Navigate across 4 columns |
| `Enter` | Select item (then `h/l` moves between columns) |
| `a` | Add new todo |
| `x` | Toggle completion |
| `dd` | Delete |
| `t` | Move to today |
| `T` | Move to tomorrow |
| `b/q/Esc` | Return to weekly view |

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

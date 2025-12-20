# <img src="src/assets/logo.svg" width="28" height="28" alt="mach logo" align="center"> mach

A terminal-based weekly planner inspired by [Tweek](https://tweek.so). Plan your week with vim-style navigation, organize someday items in a backlog, and stay focused without leaving your terminal.

## Features

- **Weekly view** — 7-day columns showing your week at a glance
- **Backlog view** — 4-column organizer for "someday" items
- **Workspaces & Projects** — Organize todos with a two-level hierarchy
- **Vim navigation** — `h/j/k/l` to move, `Enter` to select and drag
- **Local-first** — SQLite storage, no cloud, no account needed
- **Adaptive colors** — Uses terminal theme colors for universal compatibility

## Installation

### Homebrew

```sh
brew install rvcas/tap/mach
```

### Shell (Linux/macOS)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/rvcas/mach/releases/latest/download/machich-installer.sh | sh
```

### PowerShell (Windows)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/rvcas/mach/releases/latest/download/machich-installer.ps1 | iex"
```

### npm/pnpm/bun

```sh
npm install -g @rvcas/mach
```

### Cargo

```sh
cargo install machich
```

### From source

```sh
git clone https://github.com/rvcas/mach
cd mach
cargo install --path crates/mach
```

## Quick Start

Launch the TUI:

```sh
mach
```

Add a todo from the command line:

```sh
mach add Buy groceries
mach add --some-day Learn piano
```

List todos:

```sh
mach list              # today's tasks
mach list --some-day   # backlog items
mach list --done       # completed items
```

## Keyboard Shortcuts

### Weekly View

| Key     | Action                                            |
| ------- | ------------------------------------------------- |
| `h/l`   | Move left/right between days                      |
| `j/k`   | Move down/up within a column                      |
| `[/]`   | Previous/next week                                |
| `Enter` | Select item (then `h/l` moves it, `j/k` reorders) |
| `Space` | Open todo details (edit title, date, notes)       |
| `a`     | Add new todo to focused column                    |
| `x`     | Toggle completion                                 |
| `dd`    | Delete todo                                       |
| `s`     | Send to backlog                                   |
| `t`     | Move to today                                     |
| `T`     | Move to tomorrow                                  |
| `b`     | Open backlog view                                 |
| `gs`    | Settings (week start day)                         |
| `?`     | Toggle help                                       |
| `q/Esc` | Quit                                              |

### Backlog View

| Key       | Action                                         |
| --------- | ---------------------------------------------- |
| `h/j/k/l` | Navigate across 4 columns                      |
| `Enter`   | Select item (then `h/l` moves between columns) |
| `Space`   | Open todo details                              |
| `a`       | Add new todo                                   |
| `x`       | Toggle completion                              |
| `dd`      | Delete                                         |
| `t`       | Move to today                                  |
| `T`       | Move to tomorrow                               |
| `?`       | Toggle help                                    |
| `b/q/Esc` | Return to weekly view                          |

### Todo Details

| Key      | Action                  |
| -------- | ----------------------- |
| `j/k`    | Navigate between fields |
| `Enter`  | Edit / confirm          |
| `Ctrl+j` | New line (in notes)     |
| `x`      | Toggle completion       |
| `Esc`    | Close (or cancel edit)  |

### Add Todo Popup

| Key         | Action           |
| ----------- | ---------------- |
| `Enter`     | Submit           |
| `Esc`       | Cancel           |
| `Backspace` | Delete character |

## How It Works

- Todos scheduled for a day appear in that day's column
- Overdue incomplete todos automatically roll forward to today
- Completed todos sink to the bottom of their column
- New todos appear at the top of the column

## Sponsor

If you find mach useful, consider [sponsoring](https://github.com/sponsors/rvcas) its development.

## License

[Apache-2.0](LICENSE)

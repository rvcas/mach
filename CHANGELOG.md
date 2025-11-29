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

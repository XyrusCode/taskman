# Changelog

## [0.2.0]
- CLI subcommands: `taskman add <title>` and `taskman list` (TUI remains default)
- Unit tests for AppData add / toggle / delete
- Version bumped to 0.2.0

## [0.1.0]
- Initial release: task list + process monitor TUI
- JSON persistence (~/.local/share/taskman/data.json)
- Timer-driven process refresh (1s tick, only when Processes view active)
- Sort processes by CPU / memory / name
- Kill selected process
- Rolling snapshot history (last 60 samples)
- Linux first

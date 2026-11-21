# taskman

Lightweight terminal task manager — task list + process monitor — built in Rust.

Aim: a task manager that doesn't increase your load just by being opened. Small
footprint, instant startup, timer-driven refresh (no busy polling), and JSON
persistence with no SQL engine or migrations.

Inspired by Dave Plummer's original Windows Task Manager (~80 KB, every
allocation accounted for, skip work the user can't see).

## Features

- **Task list**: add, complete, delete to-do items (persisted to JSON)
- **Process monitor**: list processes with PID, name, CPU%, memory, status
- **Process grouping**: Applications vs Background sections (like TMOG) — user
  sessions such as `ai.opencode` stand out from kernel threads and daemons
- **Command-line visibility**: each process shows its cmdline, so you can tell
  which process is your opencode session vs a system daemon
- **● / ○ markers**: ● = Application, ○ = Background, on every row
- **Sort**: cycle between CPU / memory / name sorting
- **Kill**: terminate a selected process
- **History capture**: rolling snapshots of process stats (last 60 samples)
- **Keyboard-driven**: `j`/`k` navigate, `1`/`2`/`Tab` switch views, `q` quits

## Build & run

```sh
cargo build --release
./target/release/taskman
```

## Keys

| Key | Action |
|-----|--------|
| `1` / `2` | Switch to Tasks / Processes view |
| `Tab` | Toggle between views |
| `j` / `Down` | Move selection down |
| `k` / `Up` | Move selection up |
| `a` | Start adding a task (Enter to confirm, Esc to cancel) |
| `c` / `Enter` | Complete / reopen selected task |
| `d` / `x` | Delete selected task (Tasks) / kill selected process (Processes) |
| `s` | Cycle process sort (CPU → MEM → NAME) |
| `r` | Refresh process list |
| `q` | Quit |

## Release

Bump `version` in `Cargo.toml`, land on `main`. GitHub Actions builds the Linux
binary, creates the release, and mirrors it to Cloudflare R2.

## License

MIT

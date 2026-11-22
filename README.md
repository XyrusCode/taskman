# taskman

Lightweight terminal task manager — task list + process monitor — built in Rust.

Aim: a task manager that doesn't increase your load just by being opened. Small
footprint, instant startup, timer-driven refresh (no busy polling), and JSON
persistence with no SQL engine or migrations.

Inspired by Dave Plummer's original Windows Task Manager (~80 KB, every
allocation accounted for, skip work the user can't see).

## Features

- **Summary** (default): CPU, MEM, SWAP gauges, load avg, uptime, task counts — like TMOG's Performance view
- **CPU**: overall CPU sparkline + per-core utilization sparklines (CSP per-core)
- **Processes**: grouped Applications/Background list with cmdlines, sorting
- **Task list**: add, complete, delete (on Summary view)
- **Process grouping**: Applications vs Background sections — user sessions like `ai.opencode` are identifiable
- **● / ○ markers**: ● = Application, ○ = Background, on every process row
- **Keyboard-driven**: `j`/`k` navigate, `1`/`2`/`3` switch views, `q` quits

## Keys

| Key | Action |
|-----|--------|
| `1` / `2` / `3` | Switch Summary / CPU / Processes |
| `Tab` | Cycle views |
| `j` / `Up` | Move selection up |
| `k` / `Down` | Move selection down |
| `a` | Start adding a task (Enter to confirm, Esc to cancel) |
| `c` / `Enter` | Complete / reopen selected task (Summary) |
| `d` / `x` | Delete task (Summary) / kill process (Processes) |
| `s` | Cycle process sort (CPU → MEM → NAME) |
| `r` | Refresh system stats |
| `q` | Quit |

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

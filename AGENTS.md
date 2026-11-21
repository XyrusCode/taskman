# taskman — Repository Instructions

## Build & Verify via CI, not locally

- **We do not run builds or tests locally.** No `cargo build`, `cargo check`, `cargo test` for verification. CI is the source of truth.
- After making changes: commit, push to the feature branch, open the PR, then **watch CI on GitHub** (`gh pr checks --watch`).
- **When CI fails:** fix the root cause, push, and poll CI until green.
- CI runs `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo check`, and `cargo test` on `ubuntu-latest`.

## Conventions

- Version is maintained in **Cargo.toml** only.
- Feature commits carry the version bump, e.g. `feat: add process view (v0.1.0)`.
- Merge manually with a regular merge commit via `gh pr merge` (no squash/auto-merge).

## Release & R2 Mirroring

- **Trigger:** Bump `version` in `Cargo.toml` and land on `main`. The `release.yml` workflow auto-detects whether a release for that version already exists (idempotent; no duplicate releases).
- **Build:** Linux (`x86_64-unknown-linux-gnu`) release binary uploaded as a GitHub Release asset.
- **R2 mirror:** After GitHub Release creation, the `mirror` job downloads the asset and uploads it to Cloudflare R2 under `taskman-releases/v<VERSION>/`.
- **Mirror URLs:** `https://releases.xyruscode.com/taskman/v<VERSION>/taskman`
- **Auth:** `${{ secrets.CLOUDFLARE_API_TOKEN }}` passed to `wrangler r2 object put` with `--remote` flag.

## Design principles (task manager philosophy)

- Minimal footprint: small binary, instant startup, no heavy runtime or web view.
- Timer-driven refresh (1s tick), no busy polling; skip work when nothing changed.
- Two-pass CPU accounting: diff kernel-reported cumulative times between intervals.
- Skip invisible work: process refresh only runs when the Processes view is active.
- JSON persistence (`serde_json`) — no SQL engine, no migrations, human-readable.
- Linux first; cross-platform later via `sysinfo` + `crossterm`.

# Handover: cockpit fork surfaces (claude pane, 2026-09-09)

Job: research only, no product code. Output:
`docs/research/cockpit-fork-surfaces-claude-2026-09-09.md`. A codex pane ran the same job
independently; the two were not coordinated.

## What was surveyed

- Read in full: `AGENTS.md`, `CONTRIBUTING.md`, `README.md`, `justfile`, `clippy.toml`,
  `deny.toml`, `rust-toolchain.toml`, `Cargo.toml`, `docs/herdr-api-notes.md`,
  `docs/qa-install.md`, `docs/specs/README.md` and both spec folders, `herdr-plugin.toml`,
  `herdr/pane.sh`, `herdr/install.sh`, `scripts/swap-binary.sh`, `.github/workflows/*.yml`,
  and the two prior maps under `~/Documents/personal/briain/docs/research/`.
- Five read-only subagents surveyed `src/config.rs`, `src/main.rs`, `src/lib.rs` startup,
  `src/app.rs` and `src/ui.rs` pickers, `src/herdr.rs`, `src/export.rs`, `src/model.rs`,
  `src/git.rs`, `src/world.rs`, and every test file. Every line reference in the research
  document was then re-read by hand against this checkout at `4c09022`; drifted numbers
  from the prior maps were corrected.
- Live, read-only: `herdr --version` (0.7.5), the `plugin`, `plugin link`,
  `plugin uninstall`, `plugin pane open`, `pane split`, `agent rename` help pages,
  `herdr agent list`, the operator's herdr config and popup wrapper, the installed plugin
  record, and briain's data layout: 29 project notes (8 active with a checkout), nine
  `worktrees/` entries of which only two are live git worktrees, `runs/<id>/status.json`
  and `job.yaml` shapes, and the cockpit directory.
- Not done, by instruction: no rustup, no cargo, no build, no `herdr plugin link`, no pane
  opened, no config, plugin, or keybinding changed. `just ci` timing is therefore an
  estimate (upstream CI: about seven minutes cold); measure it in the first sitting.

## The three riskiest points

1. **The parser's unrecognised-flag fallthrough** (`src/config.rs:47-50`). `--project x`
   on a binary without the new match arm makes `x` the repo path. The manifest change
   that expands `REVIEWR_PROJECT` into `--project` must never run against an older
   binary. Ship parser arms first, and add the negative parser test.
2. **The not-a-git-repo startup ordering.** `repo_root` (`src/lib.rs:340-342`) falls back
   to the raw cwd, `App::new` takes it once (`:369`), and `TurnHost::open(app.repo)`
   (`:894`) keys the private refs off it. Project and run resolution, and the in-TUI
   picker, must complete before `app_for` at `src/lib.rs:81`, rewriting `cfg.repo` and
   `cfg.base`. A `Mode`-based picker that retargets a running App is the expensive
   alternative and the largest rebase surface.
3. **The bracketed paste and upstream's frozen-shape tests.** `pasted`
   (`src/herdr.rs:414-433`) frames the whole batch; the header must be one line with no
   escape byte. `format_all` (`src/export.rs:32-36`), `candidates` (`src/herdr.rs:391-402`)
   and `normalized_json_contains_every_key` (`src/config.rs:1090-1109`) are asserted
   exactly by upstream tests; keep every change additive and gated on a set identity or
   a configured `send_to`, or every upstream rebase conflicts.

## Next

Start with sitting 1 of the plan in section 13 of the research document: toolchain, the
first `just ci` (record the time here), `herdr plugin uninstall persiyanov.reviewr`, then
`just install` and `herdr plugin link .` in `~/Documents/personal/herdr-reviewr`, and the
spec folder `docs/specs/2026-09-09-briain-cockpit-review/`.

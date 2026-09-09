---
id: cockpit-fork-surfaces-claude
date: 2026-09-09
project: herdr-reviewr
kind: implementation-map
reviewr_version: 0.36.2
reviewr_commit: 4c09022 (main, origin BrianIsaac/herdr-reviewr, upstream persiyanov/herdr-reviewr)
herdr_version: 0.7.5
---

# The briain cockpit flow: every surface of this fork an implementer must change

A research job, no product code. One of two independent maps (a codex pane wrote the
other); they were not coordinated. Every claim below carries a path and line from this
checkout at `4c09022`. The two earlier maps under
`~/Documents/personal/briain/docs/research/reviewr-cockpit-integration-{claude,codex}-2026-09-09.md`
were read first and their line references re-verified here; where they drifted, this
document's numbers win.

The flow, in the operator's words: ctrl+a then d, read the registered projects, select a
project, reviewr opens the pane for that path, and anything commented goes back to the
cockpit, the main orchestrator agent. The operator chose a fork over a wrapper, so the
deliverable is a small, separable commit set that survives `git rebase upstream/main`.

## 0. Summary for the implementer

- **Five features, five commits, plus a spec commit first.** Sequence and estimate are in
  section 12. Six to eight sittings, the first of which is toolchain setup and the spec.
- **The one startup fact everything hangs on.** `Config::parse` takes the first non-flag
  token as the repo and otherwise falls back to the process cwd (`src/config.rs:47-52`);
  `repo_root` resolves it with `git rev-parse --show-toplevel` and returns the raw path when
  that fails (`src/lib.rs:340-342`); `App::new` receives that path once (`src/lib.rs:369`)
  and the world worker's `TurnHost::open(app.repo.clone())` is keyed off it when the event
  loop starts (`src/lib.rs:894`). Project and run resolution must therefore complete before
  `app_for` runs at `src/lib.rs:81`, so the App, the base, and the turn tracker all see the
  chosen worktree. A non-repo cwd never exits; it opens an empty pane
  (`src/app.rs:1238-1252`), which is the state the picker replaces.
- **The one parser fact that bites.** An unrecognised flag falls into `_ => {}` and its
  value is then read as the positional repo path (`src/config.rs:47-52`). `--project briain`
  on today's binary silently sets `repo = "briain"`. Every new flag needs an explicit match
  arm before it is ever passed.
- **Send has no notion of a named target.** Candidates are every entry with a non-null
  `agent`, in reviewr's own `HERDR_WORKSPACE_ID`, other than reviewr's pane
  (`src/herdr.rs:391-402`, `:311-313`). The cockpit's cwd is never consulted. `send_to`
  adds a resolver beside `send_target`, not inside `candidates`.
- **The paste has no header.** `format_all` is location, snippet, text, blocks joined by one
  blank line (`src/export.rs:16-36`); `App::export` builds it and consumes the store only on
  success (`src/app.rs:4724-4751`). The header is a wrapper around `format_all`, gated on a
  review identity the picker sets, so upstream's exact-text tests keep passing.
- **The plugin cannot pass argv.** The pane command is `sh -c 'exec "$HERDR_PLUGIN_ROOT/bin/herdr-reviewr"'`
  (`herdr-plugin.toml:23-24`) and `herdr plugin pane open` carries only `--cwd` and
  `--env KEY=VALUE` (verified from `herdr plugin pane open --help`, 0.7.5). The `pick`
  action passes env, and the manifest command expands it into flags.
- **`pane.sh` refuses a non-repo cwd before opening** (`herdr/pane.sh:236-249`). The `pick`
  mode must branch around that check; the cockpit's cwd
  `~/.local/share/briain/cockpit` is not a git repo (verified live, `herdr agent list`).

## 1. Sources and method

| Source | State | Read on |
| --- | --- | --- |
| This worktree, branch `agent/herdr-reviewr-research-job-no-product-20260909T050353-5fda68` at `4c09022` | clean, equals `main` and `upstream/main` v0.36.2 | 2026-09-09 |
| `AGENTS.md`, `CONTRIBUTING.md`, `README.md`, `justfile`, `clippy.toml`, `deny.toml`, `rust-toolchain.toml`, `Cargo.toml`, `docs/`, `herdr-plugin.toml`, `herdr/pane.sh`, `herdr/install.sh`, `scripts/*.sh`, `.github/workflows/*.yml` | read in full | 2026-09-09 |
| The two prior maps (claude and codex, 2026-09-09) | read in full, line refs re-verified | 2026-09-09 |
| `herdr --version`, `herdr plugin --help`, `plugin link --help`, `plugin uninstall --help`, `plugin pane open --help`, `pane split --help`, `agent rename --help`, `herdr agent list` | live, read-only, 0.7.5 | 2026-09-09 |
| `~/.config/herdr/config.toml` (lines 1-24), `~/.local/bin/briain-reviewr`, `~/.config/herdr/plugins.json`, `~/.config/herdr/plugins/config/persiyanov.reviewr/config.toml`, `~/.local/state/herdr/plugins/persiyanov.reviewr/bin/` | read only | 2026-09-09 |
| `~/.local/share/briain/notes/projects/*.md`, `worktrees/`, `runs/<id>/{status.json,job.yaml}` | read only; `git rev-parse` per worktree | 2026-09-09 |
| briain source for the contract side: `src/briain/schema.py:108-130`, `src/briain/dispatcher/spawn.py:288-299`, `src/briain/cockpit/home.py:66,210-217`, `src/briain/config.py:9,88-90,1580-1588` | read only | 2026-09-09 |
| Upstream CI run times, `gh run list -R persiyanov/herdr-reviewr -w ci` | one green run: 06:13:08 to 06:20:18 on 2026-09-05 | 2026-09-09 |

Five read-only subagents (three analysers over config/startup, app/ui, herdr/export/model
and git; one locator over tests, scripts, docs and CI; one analyser over the send path)
surveyed the source. Every line reference in this document was then re-read by hand with
`sed -n`. Nothing was installed, built, linked, opened, or changed: no rustup, no cargo, no
`herdr plugin link`, no pane, no keybinding, no config.

## 2. The repository as found

- **Layout.** `src/` is 26 files, `lib.rs` 3569 lines, `app.rs` 5522, `ui.rs` 4667,
  `config.rs` 1110, `herdr.rs` 670, `git.rs` 1998, `export.rs` 233, `model.rs` 275
  (`wc -l`). Tests: 318 unit tests beside the code and 503 integration tests in `tests/`
  (`grep -c '#[test]'`), the integration files being `app_flow.rs` (264 tests),
  `render.rs` (130), `git_repo.rs` (54), `pane_actions.rs` (29), `pr_candidates.rs` (24),
  `send_flow.rs` (1, its own process), `pr_live.rs` (1, ignored).
- **Toolchain.** `rust-toolchain.toml` pins channel `1.97.0` with `rustfmt` and `clippy`,
  profile `minimal`. `clippy.toml` sets `msrv = "1.97.0"`. `Cargo.toml` is edition 2024,
  `rust-version = "1.97"`, package and binary name `herdr-reviewr`, dependencies
  `anyhow`, `dirs 6`, `fff-search`, `pulldown-cmark`, `ratatui 0.30`, `serde`,
  `serde_json`, `similar`, `syntect`, `toml 1.1`, `two-face`, `unicode-width`; dev
  `tempfile`. There is no YAML parser (`Cargo.toml:15-31`). `deny.toml` denies wildcards
  and unknown registries and allows a fixed licence list; adding a crate means passing
  `cargo deny check` in `.github/workflows/audit.yml`.
- **Gates.** `just ci` is `fmt-check`, `lint` (`cargo clippy --all-targets --all-features -- -D warnings`),
  `test` (`cargo test --all-features`), then `cargo build --release` (`justfile:12-21, 56-57`).
  CI runs the same four steps (`.github/workflows/ci.yml:26-36`). The one green upstream
  run took about seven minutes on a cold GitHub runner. Locally, expect the first
  `cargo build` to take several minutes (syntect, fff-search, ratatui) and an incremental
  `just test` one to three minutes; measure once and write the number into the handover.
- **Conventions that bind this work.** Spec-first under `docs/specs/YYYY-MM-DD-<slug>/`
  (`AGENTS.md`, "Spec-first"; `docs/specs/README.md`). Changelog bullet under
  `## [Unreleased]` (`CHANGELOG.md:7`, format at `:11-15`). Test names read as sentences
  (`CONTRIBUTING.md:56-58`). The three named invariants: No writes, Comments survive,
  Continuity (`AGENTS.md`, "Spec-first").
- **Plugin identity.** Manifest id `persiyanov.reviewr` (`herdr-plugin.toml:1`), hard-coded
  again in `src/herdr.rs:209` (`plugin config-dir persiyanov.reviewr`), `herdr/pane.sh:56,277`,
  `herdr/install.sh:13,88`, `scripts/qa-install.sh:11`, `tests/pane_actions.rs:465`. Keep
  the id: the config directory, the `persiyanov.reviewr.<action>` keybinding names, the
  state symlink, and every test key off it. Renaming it is a separate decision with its own
  rebase cost.
- **Local install state.** The github install lives at
  `~/.config/herdr/plugins/github/persiyanov.reviewr-e87b654a74f8` (v0.36.2 per
  `~/.config/herdr/plugins.json`), the stable symlink
  `~/.local/state/herdr/plugins/persiyanov.reviewr/bin/herdr-reviewr` points into it, and
  the operator's config is `theme = "tokyo-night"`, `default_scope = "branch"`,
  `auto_open = false`. No `~/.cargo`, no `~/.rustup`, no `just` on this machine; `cc`,
  `gcc`, `ld`, `pkg-config`, `build-essential`, `libssl-dev` are present, and
  `~/.local/bin` is on `PATH`.

## 3. Startup, the CLI parser, and the plugin config (shared by every feature)

### 3.1 `src/main.rs` (16 lines)

`main` scans the whole argv for `--resolve-plugin-config` and, if found, prints the
normalised config and exits without touching the terminal (`src/main.rs:8-14`); otherwise it
calls `herdr_reviewr::run()` (`:15`). The comment at `:2-7` states the contract: a non-UI
flag must be excluded both here and in `pane.sh`'s `is_reviewr_pane`
(`herdr/pane.sh:120-140`, the `index("--resolve-plugin-config")` filter at `:135`). The new
flags (`--project`, `--run`, `--pick`, `--send-to`) are UI flags: a pane running them is a
reviewr pane and must count, so nothing changes in either half.

### 3.2 `Config::parse` (`src/config.rs:30-61`)

- `Config` fields: `repo`, `poll`, `base`, `theme`, `wrap`, `plugin_config_dir`
  (`src/config.rs:13-23`).
- The loop matches `--poll`, `--base`, `--theme`, `--wrap`, then
  `other if !other.starts_with('-') => repo = Some(...)`, then `_ => {}` (`:38-50`). The cwd
  fallback is `:52`. `Config::from_env` is `:64-66`.
- Tests: `defaults_when_no_args` (`:667-672`), `flags_and_positional_repo` (`:674-680`),
  `poll_has_a_floor` (`:682-686`), using the `parse(&[...])` helper at `:663-665`. Copy
  `flags_and_positional_repo` for each new flag and add one negative case proving that a
  new flag's value is not read as the repo.

### 3.3 The plugin config file (`src/config.rs:69-81, 331-480`)

- `PLUGIN_CONFIG_KEYS: [&str; 11]` at `:69-81`. Unknown top-level key rejects the whole file
  at `:342-344` through `unknown_key_error` (`:586-588`). Values are validated key by key
  from `:346` to `:478`; any failure returns early, so the file is all or nothing.
- `PluginConfig` struct `:159-171`, `Default` `:173-189`, accessors around `:198-242`,
  `to_json()` `:250-273`. `to_json` is what `--resolve-plugin-config` prints and what
  `pane.sh` reads through `cfg_field` (`herdr/pane.sh:40-49`).
- The `editor` key at `:439-454` is the template for a string key: `table.get`, `as_str`,
  a non-empty filter, `value_error`, store on the struct, expose an accessor, add to
  `to_json`.
- Tests to copy: `the_editor_key_carries_its_whole_command_and_reaches_the_resolved_json`
  (`:759-783`), `unknown_key_and_syntax_error_fail_the_whole_file` (`:785-804`),
  `every_invalid_value_fails_instead_of_falling_back` (`:806-845`, table-driven), and
  `normalized_json_contains_every_key` (`:1090-1109`), which asserts the JSON object has
  exactly `PLUGIN_CONFIG_KEYS.len()` keys and therefore fails the moment a key is added
  without a `to_json` entry.
- Fixture pattern: `tempfile::tempdir()`, write `config.toml`, call `plugin_config_in(dir)`.

### 3.4 `run()` and the order of startup (`src/lib.rs:72-144`)

1. `Config::from_env()` at `:73`, `log::init()` at `:74`.
2. `plugin_config_dir` from the env only at `:79`; `plugin_config` at `:80`.
3. `app_for(&cfg, &initial_config)` at `:81`: `ready_app` (`:359-376`) calls
   `repo_root(cfg)` at `:360`, reads `default_scope` from the plugin config at `:361`, and
   constructs `App::new(repo, scope, cfg.base.clone())` at `:369`; on a config error,
   `App::blocked(repo_root(cfg), Scope::Uncommitted, cfg.base.clone())` at `:351`.
4. `ratatui::init()` and the first paint at `:83-99`, before any git call.
5. `herdr::label_pane()` at `:102` (fire and forget, `src/herdr.rs:129-140`).
6. The `herdr plugin config-dir` fallback at `:110-129`, rebuilding the app through
   `app_for` if a directory comes back.
7. `app.reload()` at `:135-140` (synchronous first world build, `src/app.rs:1231-1256`).
8. `event_loop` at `:141`; inside it the world worker is spawned with
   `TurnHost::open(app.repo.clone())` at `:894`, the only call site.
9. `herdr::clear_pane_label()` at `:142` after the loop.

The design consequence for feature 1 is in section 4.4.

### 3.5 Environment reads

`HERDR_PLUGIN_CONFIG_DIR` (`src/config.rs:302`), `HERDR_BIN_PATH` (`src/herdr.rs:76`),
`HERDR_WORKSPACE_ID` and `HERDR_PANE_ID` (`src/herdr.rs:130, 147, 229`), `VISUAL` and
`EDITOR` (`src/lib.rs:236-237`), `HERDR_REVIEW_LOG` (`src/log.rs:18`), `PATH`
(`src/proc.rs:12, 71`). Nothing reads `HOME` or `XDG_*` directly; `dirs::cache_dir()` is
used once at `src/search.rs:30`. The picker adds `BRIAIN_DATA_DIR` with the default
`~/.local/share/briain` (briain's own precedence is flag, then `BRIAIN_DATA_DIR`, then
config, `src/briain/config.py:1580-1588`, defaults at `:88-90`); `dirs::data_local_dir()`
gives `~/.local/share` on Linux and macOS's `~/Library/Application Support`, so on macOS
resolve `$HOME/.local/share/briain` explicitly rather than trusting `dirs`.

## 4. Feature 1: project picker and run picker, `--project` and `--run`

### 4.1 The data on disk (verified 2026-09-09)

- **Projects.** `~/.local/share/briain/notes/projects/<id>.md`, 29 notes. Frontmatter is
  YAML between `---` lines with plain scalars: `id`, `name`, `status` (one of `active`,
  `paused`, `archived`), `working_dir` (a path or `null`), `github`, and more
  (`ProjectFrontmatter`, `~/Documents/personal/briain/src/briain/schema.py:108-130`).
  Eight are `active` with a `working_dir` that is a git checkout; `apio`, `gamebot` and
  `dis-geospatial` have `working_dir: null`. Only `id`, `status` and `working_dir` are
  needed. The `description` value can be a long single-quoted string containing colons
  (`herdr-reviewr.md` line 4), so the scanner must key on the line prefix, tolerate
  single or double quotes around the value, and stop at the closing `---`.
- **Worktrees.** `~/.local/share/briain/worktrees/<job-id>/` holds nine directories and
  three stray files (two `.zip`, one `.log`). Of the nine, only the two live
  `herdr-reviewr-research-*` worktrees pass `git rev-parse --show-toplevel`; the three
  `day0-*` entries answer `fatal: not a git repository: .../day0/.git/worktrees/<id>` (the
  main checkout pruned them), and `gemini-hackathon-*`, `protean-*` and `scratch-probe` are
  plain directories. A run row must therefore show an unavailable state rather than open
  an empty pane.
- **Runs.** `~/.local/share/briain/runs/<job-id>/status.json` is one JSON object with
  `state`, `phase`, `exit_reason`, `ts`, `backend`, `window_target`, `harness`, and after
  teardown `wip_pending`, `worktree_reclaim_safe`, `push_status`, `unpushed_commit_count`
  (sample read from `day0-give-day0-a-no-auth-deve-20260812T061134-0ecd16`). It carries no
  project or worktree field. `runs/<job-id>/job.yaml` carries `project: <id>` and `mode:`.
  `runs/` has 2177 entries; walk `worktrees/` and join by basename, never the reverse.
- **Branch and base.** Run branches are `agent/<job-id>` (verified on the two live
  worktrees). briain's merge target is local `main`, then `master`, then the current branch
  (`default_merge_branch`, `src/briain/dispatcher/spawn.py:288-299`). The operator's chain
  for the fork is `main`, `master`, then upstream: implement as local `refs/heads/main`,
  else `refs/heads/master`, else the branch's `@{upstream}` if one resolves, else leave
  `base` as `None` so reviewr's own chain (pick, then `origin/HEAD`) applies. This fork's
  own origin does have `origin/HEAD -> origin/main` (`git branch -a`), unlike the eight
  project checkouts the earlier map tabled.

### 4.2 New module: `src/briain.rs` (pure, no subprocess except git)

- `pub struct Project { id, status, working_dir: Option<PathBuf> }`,
  `pub fn projects(data_dir: &Path) -> Vec<Project>` reading `notes/projects/*.md`,
  `pub fn active_projects(...)` filtering `status == "active"` and an existing dir.
- `pub struct Run { id, project: Option<String>, state: Option<String>, phase, worktree: PathBuf, branch: Option<String>, available: RunAvailability }` and
  `pub fn runs(data_dir: &Path) -> Vec<Run>` walking `worktrees/*/` (directories only),
  joining `runs/<id>/status.json` through `serde_json` and `runs/<id>/job.yaml` through
  the same line scanner, and classifying availability with `git::worktree_of`
  (`src/git.rs:79-101`, returns `Root`, `Outside`, or `Unknown`).
- `pub fn resolve_base(worktree: &Path) -> Option<String>` implementing the chain in 4.1,
  using the existing `git_tristate(repo, ["rev-parse", "--verify", "--quiet", ...])`
  shape (`src/git.rs:451-460`) or a new `git::branch_exists` beside `resolve_base_entry`
  (`src/git.rs:922-933`).
- `pub struct ReviewIdentity { project: Option<String>, run: Option<String> }`, set on
  `App` (new field beside `repo` at `src/app.rs:599`) for feature 3.
- `pub fn data_dir() -> PathBuf`: `BRIAIN_DATA_DIR`, else `$HOME/.local/share/briain`.
- Frontmatter scanner: a 30-line function over the first `---` block, matching
  `^(id|status|working_dir):\s*(.*)$`, stripping one pair of matching quotes and
  reading `null` as `None`. Do not add `serde_yaml`; it is unmaintained and would need a
  `deny.toml` advisory ignore like the `bincode` one at `deny.toml:6-10`.
- Tests beside the code with `tempfile::tempdir()` fixtures: a note with a quoted
  `working_dir`, a note with `working_dir: null`, an archived note, a description line
  containing `id:` text, a worktree whose git dir is pruned, a stray `.zip` in
  `worktrees/`, a run with no `job.yaml`. Use `tests/common/mod.rs` `Repo::init()`
  (`:21`) and `add_worktree` (`:102`) for a real linked worktree.

### 4.3 CLI: `--project <id>`, `--run <id>`, `--pick`

- Add to `Config` (`src/config.rs:13-23`): `project: Option<String>`, `run: Option<String>`,
  `pick: bool`. Add match arms before the positional arm (`:38-50`). Env equivalents
  `REVIEWR_PROJECT`, `REVIEWR_RUN`, `REVIEWR_PICK` are read in `Config::from_env` only
  (`:64-66`) so `parse` stays pure and testable; they exist because the plugin pane can
  carry env and not argv (section 7).
- README flag table `README.md:199-204` gains three rows.

### 4.4 Startup resolution and the in-TUI picker

Two shapes were weighed:

- **(A) Resolve before the App exists.** In `run()` between `:80` and `:81`: if `--project`
  or `--run` is given, resolve it to `(repo, base, identity)` and overwrite `cfg.repo` and
  `cfg.base`; if `--pick` is given, or neither flag is given and `cfg.repo` is not a repo
  (`git::is_repo`, `src/git.rs:62-64`), run a small picker loop that draws with ratatui
  and returns the same triple; then fall through to `app_for` unchanged. The picker loop
  needs the terminal, so `ratatui::init()` and `claim_input_modes` (`:83-99`) move ahead of
  `app_for`, or the picker draws its own frames through the same `terminal` value; the
  first-paint guarantee at `:83-99` is kept because the picker is itself the first paint.
  `App::new` at `:369` and `TurnHost::open` at `:894` then see the chosen worktree with no
  change.
- **(B) A `Mode::ProjectPick` inside App** with a retarget that replaces `app.repo`. This
  touches every place that assumes `repo` is fixed: the stashes per tab
  (`swap_active_with_stash`, `AGENTS.md`), the `DiffCache`, the search worker, the frozen
  diff under a compose, the world worker's `TurnHost` already opened on the old path at
  `src/lib.rs:894`, and the base pick ref of the old worktree. It is a large diff across
  `app.rs` and `lib.rs` and the worst rebase surface in the repo.

Recommend (A). It is about 150 lines in a new `src/pick.rs` plus 30 in `run()`, it leaves
`App` untouched, and a cancelled picker exits the pane with status 0, which is what a
popup-style flow wants. Because the picker runs before `App`, it does not use `Mode`; the
`Mode::Picker` render at `src/ui.rs:3046-3088` and the shared helpers `menu_popup`
(`src/ui.rs:2963-2969`), `body_popup` (`:2943-2953`), `menu_scroll` (`:2972-2977`),
`menu_hit` (`:2982-3001`), `selectable_row` (`:4016`) and `framed_title` (`:4633-4635`)
are the code to copy for its look, and the key handling to copy is the `Mode::Picker`
block at `src/lib.rs:1745-1766` (Esc closes, bare Enter picks, digits jump, keymap Up and
Down move). A two-level menu: level one `projects` and `runs`, level two the rows; a run
row reads `<job-id>  <state> · <project> · <branch>` and an unavailable row is dim and
inert, in the `picker_trail` style (`src/ui.rs:3029-3034`).

Startup scope for a run: pass `Scope::Branch` to `App::new` (`src/app.rs:850`) regardless
of `default_scope`, since a run is reviewed against its base; a project keeps
`default_scope`. Passing the resolved base through `cfg.base` means `base_pick_available`
(`src/app.rs:4423-4426`, `self.base.is_none()`) is false and `B` is disabled, exactly as
`--base` behaves today (`README.md:278`). Note `classify_flag` (`src/git.rs:905-919`)
resolves the flag verbatim first, and git's own `rev-parse main` prefers `refs/heads/main`
over `refs/remotes/origin/main`, so a bare `main` from the chain resolves to the local
branch; pass the fully spelt `refs/heads/main` anyway so the header reads unambiguously
(`strip_base_prefix`, `src/git.rs:665-670`, turns it back into the bare name for display).

### 4.5 Tests

- `src/config.rs`: three new flag tests plus the negative case (section 3.2).
- `src/briain.rs`: unit tests listed in 4.2.
- `src/pick.rs`: pure state tests (rows, cursor, digit past end is inert) in the shape of
  `the_picker_moves_by_key_and_a_digit_past_the_last_row_is_inert`
  (`tests/app_flow.rs:5295-5315`).
- `tests/render.rs`: one painted-frame test for the picker, in the shape of
  `the_picker_titles_the_count_and_aligns_the_dim_trail_in_one_column`
  (`tests/render.rs:2999-3022`), using the `TestBackend` harness at `tests/render.rs:34-52`.
- `tests/git_repo.rs`: the base chain (`main`, then `master`, then upstream, then none)
  in the shape of `the_chain_is_flag_then_pick_then_default` (`tests/git_repo.rs:98-121`),
  with `Repo::set_origin_default` (`tests/common/mod.rs:64`) for the upstream case.

## 5. Feature 2: `send_to` config key and `--send-to` flag

### 5.1 What exists

- `send_to_agent` (`src/app.rs:4351-4368`) refuses an empty store, then matches
  `herdr::send_target()`: `One` exports at once, `Many` opens the picker, `Err` puts the
  sentence in `app.status`.
- `send_target` (`src/herdr.rs:242-266`) reads `agent_env()` (`:228-230`), calls
  `agent_list()` (`:234-236`), filters with `candidates` (`:391-402`) on
  `is_agent_other_than(me)` (`:311-313`) and `workspace_id == ws`, then matches the count.
- `AgentPane` (`:32-54`) carries `agent`, `agent_status`, `pane_id`, `tab_id`,
  `workspace_id`, `cwd`, `name`, `display_agent`, `state_labels`. `AgentChoice` (`:56-63`)
  is the row: `pane_id`, `name`, `state`, `tab`. `row_name` (`:282-289`) prefers `name`,
  then `display_agent`, then `agent`, then the pane id.
- `export_to_agent` (`src/app.rs:4714-4719`) builds `export::Agent { pane, name }` and,
  on success, sets `last_sent_pane`. `Agent::export` (`src/export.rs:141-147`) is
  `send_text` then a best-effort `focus`.
- `status` fades after `STATUS_TTL` of four seconds (`src/lib.rs:379`, `:982-991`) and
  paints in the footer (`STATUS_MIN` at `src/ui.rs:2669`). "Refuses visibly" therefore means a
  footer sentence for four seconds, the same channel as today's
  `no agent here - copy to the clipboard instead`.
- Live on 2026-09-09: the cockpit is `w7:p1`, `agent: claude`, `name: null`,
  `cwd: /home/brian-isaac/.local/share/briain/cockpit`, in the same workspace as the two
  job panes `w7:pA` and `w7:pB` (`herdr agent list`). So today's Send opens a three-row
  picker reading `claude`, `claude`, `codex`.

### 5.2 Changes

- `src/config.rs`: `"send_to"` in `PLUGIN_CONFIG_KEYS` (`:69-81`, becomes `[&str; 12]`);
  `send_to: Option<String>` on `PluginConfig` (`:159-171`) and `Default` (`:173-189`);
  a `table.get("send_to")` block after `editor` (`:454`) accepting a non-empty string;
  accessor; `to_json` entry (`:250-273`); README key list (`README.md:218-231`). CLI
  `--send-to <target>` in `Config::parse` (`:38-50`) and on `Config`; the flag wins over
  the key, mirroring `--theme` over `theme` (`src/app.rs:986-989`).
- `src/herdr.rs`: a new `pub enum SendTo { Cockpit, Name(String), Pane(String) }` with
  `parse(&str)` (a pane id matches `^w[0-9A-Za-z]+:p[0-9A-Za-z]+$`, `cockpit` is the
  keyword, anything else is a name), and
  `pub fn send_target_for(target: &SendTo, cockpit_dir: &Path) -> Result<AgentChoice>`
  that calls `agent_list()`, keeps `is_agent_other_than(me)`, and matches: `Pane` on
  `pane_id`, `Name` on `name`, `Cockpit` on `name == "cockpit"` or `cwd` equal to the
  cockpit dir after `Path::new(cwd)` comparison. Zero matches bail with
  `send target <x> not found - copy to the clipboard instead`; two or more bail naming
  both pane ids. Do not filter by workspace here: the target is explicit, and the split
  places reviewr in the cockpit's workspace anyway. Do not touch `candidates` (`:391-402`)
  or the picker path, so upstream's tests at `:550-593` stay green.
- `src/app.rs`: `send_to_agent` (`:4351-4368`) checks a new `self.send_to: Option<SendTo>`
  (set from `set_plugin_config` at `:992` and from the flag in `ready_app`) before
  `send_target()`, and on `Ok(choice)` calls `export_to_agent(&choice)`, on `Err` sets
  `status`. The `AgentChoice.name` for the cockpit case should be `cockpit` so the
  success message reads `added 3 comments to cockpit` (`src/export.rs:130-132`).
- Cockpit dir: `briain::data_dir().join("cockpit")` (`DEFAULT_COCKPIT_DIRNAME`,
  `src/briain/cockpit/home.py:66,210-217`).

### 5.3 Tests

- `src/herdr.rs`: pure tests over hand-built `AgentPane` values, in the shape of
  `picker_rows_exclude_our_own_pane_and_every_non_agent_pane` (`:581-593`): cockpit by
  cwd, cockpit by name, name match, pane match, absent target, two cockpits.
- `src/config.rs`: `send_to` accepted, empty rejected, JSON round trip, and the
  `normalized_json_contains_every_key` count (`:1090-1109`).
- `tests/send_flow.rs`: extend the one test (it re-execs itself with `HERDR_BIN_PATH`
  pointing at the fake `herdr` script, `:87-120`, fixture JSON at `:24-30`) with a
  `send_to = "cockpit"` case: an agent whose `cwd` is the fake cockpit dir receives the
  paste with no picker, and removing it yields the refusal sentence with the comment
  retained. Add `cwd` to the fixture rows as the existing ones already do (`:22-30`).

## 6. Feature 3: the one-line header on paste and clipboard

### 6.1 What exists

- `format_comment` (`src/export.rs:16-18`), `normalize_text` (`:22-29`), `format_all`
  (`:32-36`). `ExportTarget` (`:39-47`) takes `&str`. `Clipboard` (`:65-102`), `Agent`
  (`:117-148`). `App::export` (`src/app.rs:4724-4751`) is the one caller of `format_all`
  and the one consumer of the store; both `Send` and `Copy` go through it
  (`src/lib.rs:1843-1846`, `:1907-1910`).
- `Comment::location` (`src/model.rs:136-147`). Nothing in the payload names the repo, the
  branch, or the base (prior maps, confirmed).
- Bracketed paste: `pasted` (`src/herdr.rs:414-433`) wraps the whole text in
  `ESC[200~ ... ESC[201~` and strips any embedded terminator. Tests at `:608-623`.
- Branch name: `git symbolic-ref --quiet --short HEAD` is used inline at
  `src/git.rs:505` and `:679`; there is no public `head_branch`. `head_oid` is
  `src/git.rs:1250-1252`. The resolved base is `app.branch_base.winner`
  (`src/app.rs:603`; `BaseStatus`, `src/git.rs:564-568`; `ResolvedBase::name()` and
  `oid()`, `:533-562`).

### 6.2 Changes

- `src/git.rs`: `pub fn head_branch(repo) -> Option<String>` beside `head_oid`
  (`:1250`), the same `symbolic-ref` call, `None` when detached.
- `src/export.rs`: `pub struct ReviewHeader { project: Option<String>, run: Option<String>, branch: Option<String>, base: Option<(String, String)>, count: usize }`
  with `fn line(&self) -> String` producing exactly
  `review: <project> | <run> | <branch> | <base>@<short oid> | <n> comments`, using `-`
  for an absent part, and `pub fn format_batch(header: Option<&ReviewHeader>, comments: &[&Comment]) -> String`
  that returns `format_all` unchanged when `header` is `None` and otherwise
  `header.line() + "\n\n" + format_all(...)`. One line, no control characters, no newline
  inside the header: the cockpit keys on the first line and `pasted` must not find an
  `ESC` in it.
- `src/app.rs:4724-4751`: build the header from `self.identity` (feature 1),
  `git::head_branch(&self.repo)`, `self.branch_base.winner`, and `refs.len()`; call
  `format_batch`. Only a set identity produces a header, so a bare `cargo run` in a repo
  still pastes upstream's exact text.
- `--base` supplied by the picker means `branch_base.winner` is a `Branch` with `name`
  and `oid` after the first branch-scope build (`world.rs:136-146` calls `resolve_base`
  at `:141`); in `Uncommitted` scope the winner may be `None` until a branch build has
  run, so the header prints `-@-` there rather than forcing a build in `export`.

### 6.3 Tests

- `src/export.rs`: two tests beside `all_sorts_by_file_then_start_with_blank_separator`
  (`:225-232`): `a_batch_with_an_identity_leads_with_one_review_line` and
  `a_batch_without_an_identity_is_the_bare_blocks`; and a test that the line never
  contains `\n` or `\x1b`.
- `tests/app_flow.rs`: with `FakeTarget` (`:25-60`, captures the text), set an identity
  and assert `target.last()` starts with `review: `; leave every existing exact-text
  assertion alone by not setting an identity in `app_on` (`tests/common/mod.rs:149-153`).
- `tests/send_flow.rs`: the assertion `pane send-text w8:p2 \x1b[200~` (`:176-180`) holds
  with a header because the frame still opens right after the pane id; add
  `\x1b[200~review: ` when the identity is set.

## 7. Feature 4: the plugin `pick` action

### 7.1 What exists

- `herdr-plugin.toml`: one `[[panes]]` entrypoint `pane` whose command is
  `sh -c 'exec "$HERDR_PLUGIN_ROOT/bin/herdr-reviewr"'` (`:20-24`); actions `toggle`,
  `open`, `close` (`:26-42`); events (`:45-51`).
- `herdr/pane.sh`: validates config through `--resolve-plugin-config` (`:31-49`),
  refreshes the stable symlinks (`:55-65`), reads `ws`, `pane`, `cwd` from the env and
  `HERDR_PLUGIN_CONTEXT_JSON` (`:88-92`), sweeps the workspace's reviewr panes
  (`:104-181`), dispatches on `mode` (`:210-231`), picks the cwd from the focused pane's
  live `foreground_cwd` and refuses when neither candidate is a repo (`:236-249`), sets
  focus (`:252-253`), builds placement args (`:257-275`), opens with
  `herdr plugin pane open --plugin ... --entrypoint pane "$@" --cwd "$cwd" "$focus"`
  (`:277-278`), and renames a tab-placement tab (`:285-288`).
- `herdr plugin pane open` accepts `--env KEY=VALUE` (help output, 0.7.5; also
  `docs/herdr-api-notes.md:72-75`). Actions run headless with stdout to the plugin log
  (`docs/herdr-api-notes.md:110-112`), so the picker itself lives in the pane (feature 1).
- The operator's ctrl+a d today is a `type = "popup"` key command running
  `~/.local/bin/briain-reviewr` (`~/.config/herdr/config.toml:18-24`); the replacement is
  a `type = "plugin_action"` binding to `persiyanov.reviewr.pick`
  (`docs/herdr-api-notes.md:159-166`). That keybinding change is the operator's, outside
  this repo.

### 7.2 Changes

- `herdr-plugin.toml`: change the pane command to expand env into flags:
  `exec "$HERDR_PLUGIN_ROOT/bin/herdr-reviewr" ${REVIEWR_PICK:+--pick} ${REVIEWR_PROJECT:+--project "$REVIEWR_PROJECT"} ${REVIEWR_RUN:+--run "$REVIEWR_RUN"} ${REVIEWR_SEND_TO:+--send-to "$REVIEWR_SEND_TO"}`
  (ids are slugs without spaces, quoted anyway). Add
  `[[actions]] id = "pick"`, title `reviewr: pick a briain project or run`, contexts
  `["pane", "workspace"]`, command `["bash", "herdr/pane.sh", "pick"]`. Bump nothing
  else; `min_herdr_version` stays `0.7.5` because `--env` is verified there.
- `herdr/pane.sh`: a `pick` arm in the `case "$mode"` at `:210-231` that does not
  no-op on an existing reviewr pane (a second review beside the first is legitimate), a
  branch around the `is_git_repo` refusal at `:243-249` for `pick` (use the live cwd if
  it is a repo, else the context cwd, else `$HOME`, since `--cwd` must be a directory and
  reviewr ignores it under `--pick`), placement forced to `split` beside the focused
  pane (the `pane` variable at `:89`, falling back to the first pane at `:259-261`; the
  operator asked for a split beside the focused agent pane, so `pick` should also read
  `focused_pane_id` from the context JSON at `:239` before that fallback), and
  `--env REVIEWR_PICK=1` plus `--env REVIEWR_PROJECT="$2"` when `pane.sh pick <id>` is
  invoked with an argument. A `pick` pane is a reviewr pane by `argv0`
  (`is_reviewr_pane`, `:120-140`), so `toggle` and `close` keep sweeping it.
- The `send_to` default for the flow is config, not the action: the operator sets
  `send_to = "cockpit"` once in `config.toml`.

### 7.3 Tests

`tests/pane_actions.rs` drives the real script with the fake `herdr` at `:19-55` and
reads its log. Copy `open_prefers_the_focused_panes_live_foreground_cwd` (`:652-683`)
and `a_toggle_open_falls_back_when_the_live_cwd_is_not_a_repo` (`:732-754`) into
`pick_opens_beside_the_focused_pane_when_its_cwd_is_not_a_repo` (assert the log holds
`--placement split --target-pane <focused> --env REVIEWR_PICK=1`) and
`pick_with_an_id_passes_it_through_the_env`. Copy
`manifest_auto_open_hooks_created_and_opened` (`:265-289`) for a manifest test that the
`pick` action exists and the pane command names the four env expansions.

## 8. Feature 5: README section and the rebase recipe

- README: a `## briain cockpit` section after `## Building from source` (`README.md:457`)
  covering `--project`, `--run`, `--pick`, `--send-to` and the `send_to` key, the `pick`
  action and its keybinding snippet, the header line, and the data it reads. Update the
  flag table (`:199-204`) and the key list (`:218-231`).
- Rebase recipe, in the same section or in `docs/fork.md`:

```
git fetch upstream
git rebase upstream/main        # on main, one commit per feature on top
just ci                         # fmt-check, lint, test, release build
just install && herdr plugin link .   # only if the linked checkout moved
```

  Expected conflict points, in order of likelihood: `src/config.rs:69-81` (the keys
  array) and `:38-50` (the parser), `README.md` flag and key tables, `herdr-plugin.toml`
  pane command, `herdr/pane.sh` dispatch, `CHANGELOG.md:7`. Keep the fork's commits
  touching those files minimal and additive.
- `CHANGELOG.md`: one bullet per feature under `## [Unreleased]` (`:7`), in the bold
  summary form at `:11-15`.
- `herdr/install.sh:13` hard-codes `REPO="persiyanov/herdr-reviewr"`, so
  `herdr plugin install BrianIsaac/herdr-reviewr` would download upstream's binary and
  none of this work. The fork is used through `just install` and `herdr plugin link`
  only; say so in the README section and do not change `install.sh`.

## 9. The spec folder

`docs/specs/2026-09-09-briain-cockpit-review/spec.md` with the headings of
`docs/specs/2026-08-28-auto-open-worktree-opened/spec.md` (`Status`, `Date`, then
`## Problem`, `## Proposal`, `## Invariants`, `## Alternatives`, `## Out of scope`,
`## Open questions`; lines 1-70 of that file), and `plan.md` in the shape of its sibling
(`## Problem`, `## Goal`, `## Ticket Map`, one `## Ticket N` per feature with
`**What to build**`, `**Blocked by**`, `**Status**`, `### Acceptance criteria`,
`## Out of Scope`, `## Replan`). The README under `docs/specs/README.md` mentions
`tickets/`, but neither existing folder has one; `plan.md` alone matches practice.

Invariants to cite in the spec: No writes (the registry read is read-only; the picker
passes a base and never writes a pick ref), Comments survive (the header is added at
export time, the store is untouched), Continuity (the picker runs before any place state
exists).

## 10. Toolchain, build and link loop

### 10.1 A machine with no Rust, installing in `$HOME` only

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain none
export PATH="$HOME/.cargo/bin:$PATH"      # add the same line to ~/.bashrc or ~/.zshrc
cd ~/Documents/personal/herdr-reviewr
rustup show                               # installs 1.97.0 with rustfmt and clippy from rust-toolchain.toml
cargo install just --locked               # lands in ~/.cargo/bin; or a release tarball into ~/.local/bin
just ci
```

`rustup show` in the checkout is what triggers the pinned toolchain install
(`CONTRIBUTING.md:7-8`, "installs itself on first build"). `cc`, `ld` and `pkg-config` are
already present, and the clipboard path shells out to `wl-copy`, `xclip` or `xsel`
(`src/export.rs:57-62`), so no X or Wayland dev packages are needed for the build.
Optional: `cargo install cargo-deny --locked` to run `cargo deny check` locally
(`deny.toml:2`).

### 10.2 Replacing the github install with the linked fork

```
herdr plugin uninstall persiyanov.reviewr     # config is keyed by id and survives (README.md:63, :474-477)
cd ~/Documents/personal/herdr-reviewr
just install                                  # cargo build --release, then scripts/swap-binary.sh into bin/ (justfile:32-35)
herdr plugin link .                           # skips [[build]] (docs/herdr-api-notes.md:12-13)
herdr plugin list
```

Then close and reopen every reviewr pane with the toggle keybinding; the three rules in
`AGENTS.md` "QA install" and `docs/qa-install.md:27-49` apply unchanged: never overwrite
in place (`swap-binary.sh` does the fresh-inode swap), a swap never restarts a running
pane, and only the user reopens panes. The rebuild loop is `just install` then reopen.
`just qa-install` targets the github install directory
(`scripts/qa-install.sh:11`) and is not the fork's loop.

The registered `herdr-reviewr` project note already points at
`/home/brian-isaac/Documents/personal/herdr-reviewr` with origin
`BrianIsaac/herdr-reviewr` (`notes/projects/herdr-reviewr.md`), which is the checkout to
link. Verify with `herdr plugin list --json` that exactly one `persiyanov.reviewr` remains
and that its `plugin_root` is the checkout; a github install and a link under one id was
not tested here.

## 11. The herdr API calls involved

| Call | Used by | Contract |
| --- | --- | --- |
| `herdr agent list` | `send_target`, the new `send_target_for`, turn sampling | no flags; `pane_id`, `tab_id`, `workspace_id`, `agent`, `agent_status`, `cwd`, optional `name`; JSON error envelope on stderr (`docs/herdr-api-notes.md:169-186`, `:200-206`) |
| `herdr pane send-text <pane> <text>` | `send_text` (`src/herdr.rs:409-412`) | literal bytes, no Enter, unchanged since 0.7.0 (`:210-212`) |
| `herdr agent focus <pane>` | `focus` (`src/herdr.rs:436-439`) | best effort after a send |
| `herdr tab list --workspace <ws>` | picker trail labels (`src/herdr.rs:318-324`) | `label` joined on `tab_id` (`:188-192`) |
| `herdr plugin pane open --plugin --entrypoint pane --placement split --target-pane --direction --cwd --env --focus` | `pane.sh` (`:277-278`) and the new `pick` | split needs `--target-pane`; new pane id at `.result.plugin_pane.pane.pane_id` (`:77-86`); `--env` verified in `--help` |
| `herdr pane list --workspace <ws>`, `pane process-info --pane <id>` | `pane.sh` sweep (`:104-181`) | identity by `argv0` basename, `pane_not_found` envelope (`:41-61`) |
| `herdr plugin config-dir persiyanov.reviewr` | `plugin_config_dir_with` (`src/herdr.rs:207-223`) | bounded by 2 s (`:103`) |
| `herdr pane rename <id> [label] [--clear]` | `label_pane`, `clear_pane_label` | display only (`:66-68`) |
| `herdr agent rename <pane> cockpit` | optional operator step so the cockpit row and `SendTo::Name` match | names `[a-z0-9_-]{1,32}` (`:184-186`); not needed when `send_to = "cockpit"` resolves by cwd |
| `herdr plugin link <path>`, `plugin uninstall <id>`, `plugin list --json` | the build loop | help output, 0.7.5 |

Every failing call writes a JSON envelope to stderr and reviewr logs it, showing its own
sentence (`docs/herdr-api-notes.md:200-206`; `src/herdr.rs:79-97`). The new resolver keeps
that shape.

## 12. Risks

1. **The unrecognised-flag fallthrough.** `--project x` on a binary that lacks the arm makes
   `x` the repo (`src/config.rs:47-50`). During the rebase window, a manifest that expands
   `REVIEWR_PROJECT` against an older binary would open reviewr on a directory named after
   the project id. Mitigation: land the parser arms and the manifest change in the same
   commit series, and have `pane.sh pick` require `--resolve-plugin-config` to succeed as
   today (`:31-37`), which any binary passes; better, add a `--version`-style capability
   probe only if the window is real. Test: the negative parser case in 3.2.
2. **The not-a-git-repo ordering.** The picker must run before `app_for`
   (`src/lib.rs:81`) and before `TurnHost::open` (`:894`). If it is placed after the
   first paint at `:83-99` but resolution is written back only to `app.repo`, the world
   worker, the base ref namespace (`refs/worktree/reviewr/`, per worktree by design,
   `src/git.rs:1006-1012`), and the `repo_root` doc contract (`src/lib.rs:333-339`,
   "every App goes through this") diverge. Design (A) in 4.4 avoids it by rewriting
   `cfg.repo` and `cfg.base` before construction. Test: an integration test that
   constructs through `run`-equivalent code with `--run` and asserts `app.repo` is the
   worktree top level and `app.base` the resolved branch.
3. **Bracketed paste and the header.** `pasted` (`src/herdr.rs:414-433`) frames the whole
   text; a header containing a newline or an escape byte breaks the cockpit's first-line
   rule or the frame. Keep the header a single sanitised line and assert it in
   `src/export.rs`. `tests/send_flow.rs:176-187` assertions on the frame's first and last
   bytes must be updated only where an identity is set.
4. **Upstream tests that count or freeze shapes.** `normalized_json_contains_every_key`
   (`src/config.rs:1090-1109`), the exact-text export tests (`src/export.rs:197-232`,
   `tests/app_flow.rs` through `FakeTarget`), and `picker_rows_*` (`src/herdr.rs:550-593`)
   all pass if the new behaviour is additive and gated on an identity or a configured
   target. Any change to `format_all` or `candidates` themselves would fail them and
   collide on every rebase.
5. **Broken worktrees.** Seven of nine `worktrees/` entries fail `git rev-parse`
   (4.1). Opening one gives the empty state (`src/app.rs:1238-1252`) with no message.
   The run picker must mark them and refuse, and the `--run <id>` flag must print a
   sentence and fall back to the picker rather than open empty.
6. **`dirs` on macOS.** `dirs::data_local_dir()` is not `~/.local/share` on macOS; use
   `$HOME/.local/share/briain` unless `BRIAIN_DATA_DIR` is set (3.5). The plugin
   declares both platforms (`herdr-plugin.toml:8`).
7. **Focus target for the split.** `pane.sh` reads `HERDR_PANE_ID` at `:89` and falls
   back to the first pane at `:259-261`; whether an action's env carries `HERDR_PANE_ID`
   for the focused pane was not verified live. `focused_pane_id` in the context JSON
   (`:239`) is documented (`docs/herdr-api-notes.md:102-109`) and should be preferred in
   `pick`.
8. **The `herdr agent list` order and two cockpits.** If the operator ever runs two
   cockpits, `SendTo::Cockpit` is ambiguous; refuse naming both pane ids rather than pick
   the first (`docs/herdr-api-notes.md:172-176` on ordering).
9. **`install.sh` points at upstream.** Never `herdr plugin install` the fork from GitHub
   (section 8).
10. **Status fade.** Four seconds (`src/lib.rs:379`) is the whole visibility of a refusal;
    the sentence must name the target and the clipboard fallback in under the footer's
    width, as the existing refusals do (`src/herdr.rs:250, 258`).

## 13. Suggested commit sequence and estimate

One feature per commit, spec first, all on `main` of the fork, rebased onto
`upstream/main` before each push:

1. `docs(spec): briain cockpit review flow` - `docs/specs/2026-09-09-briain-cockpit-review/{spec,plan}.md`.
2. `feat(pick): project and run picker with --project, --run and --pick` -
   `src/briain.rs` (new), `src/pick.rs` (new), `src/config.rs` (`Config`, `parse`,
   `from_env`), `src/lib.rs` (`run`, before `app_for`), `src/git.rs` (`head_branch`,
   `branch_exists`), `src/app.rs` (`identity` field), tests in `src/briain.rs`,
   `src/config.rs`, `tests/render.rs`, `tests/git_repo.rs`, `CHANGELOG.md`.
3. `feat(send): send_to config key and --send-to flag` - `src/config.rs` (key, flag,
   `to_json`), `src/herdr.rs` (`SendTo`, `send_target_for`), `src/app.rs`
   (`send_to_agent`, `set_plugin_config`), tests in `src/herdr.rs`, `src/config.rs`,
   `tests/send_flow.rs`, `CHANGELOG.md`.
4. `feat(export): one-line review header on send and copy` - `src/export.rs`
   (`ReviewHeader`, `format_batch`), `src/app.rs` (`export`), tests in `src/export.rs`,
   `tests/app_flow.rs`, `tests/send_flow.rs`, `CHANGELOG.md`.
5. `feat(plugin): pick action opening a split beside the focused pane` -
   `herdr-plugin.toml`, `herdr/pane.sh`, `tests/pane_actions.rs`, `CHANGELOG.md`.
6. `docs(readme): briain cockpit section and the upstream rebase recipe` - `README.md`,
   `docs/fork.md` if split out.

Estimate, in sittings of two to four hours:

| Sitting | Work |
| --- | --- |
| 1 | rustup, just, first `just ci` (measure it), uninstall the github plugin, link the checkout, the spec folder |
| 2 | `src/briain.rs` with its tests against the real data layout; base chain in `git.rs` |
| 3 | `src/pick.rs`, the `run()` change, flags, render test; live check in a pane |
| 4 | `send_to` end to end including `send_flow.rs` |
| 5 | header line, `pick` action and `pane_actions.rs` tests |
| 6 | README, changelog, a full rebase rehearsal against `upstream/main`, live walk of ctrl+a d |

Six sittings if the picker design (A) holds; eight if the operator wants the picker as a
`Mode` inside a running App (design (B)), which is the one choice here that changes the
size of the work.

# Cockpit fork: sitting 3 handover

Date: 2026-09-09 (Asia/Singapore)
Status: Ticket 1 complete; tickets 2–4 untouched.

## Delivered

- `06940d0 feat(config): parse cockpit selectors and reserve --send-to`
- `4878dc3 feat(pick): project and run picker with --project, --run and --pick`
- A separate docs commit contains this handover, spec acceptance progress and the Unreleased entry.

Read the [spec](../../specs/2026-09-09-briain-cockpit-review/spec.md) and
[Ticket 2](../../specs/2026-09-09-briain-cockpit-review/plan.md) before sitting 4.
The existing plan and sitting 1/2 handovers were not edited.

The parser has explicit arms for `--project`, `--run`, `--pick` and reserved
`--send-to`. New valued flags cannot assign their values to `repo`; missing values
leave the following flag available to its own parser arm. Missing/empty values,
selector conflicts and selector-plus-path conflicts set `Config::launch_error`,
which startup paints and refuses. `Config::parse` retains its stock return type
and unrelated legacy behavior, including unknown flags and last-positional wins.
All original fixtures/assertions remain intact. `send_to` is only a CLI value at
this boundary: it has no config-file schema entry, routing or display consumer yet.

`src/pick.rs` owns the pre-App `Picker`, `Selection` and `ReviewIdentity`. The first
level lists projects plus All retained runs; a project's second level offers its
checkout and associated runs. All retained runs also exposes unassociated runs
without inventing a project. Digits and mouse clicks highlight, Enter opens,
Esc goes back/cancels, movement/refresh use the configured keymap. Release events
are inert. Unlike Send, modified Enter is safe and accepted. Unavailable rows
are dim and carry reasons; explicit duplicate/invalid/unavailable lookups refuse.
Rescan preserves project ids (including note renames) and run ids, then the nearest
surviving neighbour, then clamps. Rendering and hit testing share scrolling geometry.

`Selection::revalidate` canonicalizes and calls the existing
`checkout_availability`; Enter and launch application both validate. Explicit ids
pass `validate_id` and are looked up in discovered rows, never joined into paths.
`apply_selection` computes everything before mutating launch config: explicit base
wins, otherwise existing `git::launch_base`, including None. Runs override scope to
Branch; projects retain the config default. Canonical root, base, scope override
and review identity live in Config, so fallback/recovery cannot re-resolve an edited
project note or return to the launch cwd. `App::review_identity` defaults to None
in unchanged constructors and is set only by resolved startup. Export is unchanged.

Design **A** is implemented in `run`/`resolve_launch`: initialize and paint startup,
resolve config directory, validate the whole file, resolve selection, then construct
App, seed baseline, reload and enter the worker loop. Initial config errors stay in
a pre-App repair/cancel screen. Config edits during the input wait gate the action
as well as the next paint. Later recovery uses the same resolved Config. Valid
ordinary repository paths bypass registry/menu and retain stock base/scope/export
identity behavior. There is no new App mode or in-session repository switch.

**No writes:** discovery/selection/base choice remain read-only, using the sitting 2
helpers unchanged. **Continuity:** no review place state or worker exists until
selection ends; picker rescans reconcile identities. **Comments survive:** comment
storage/export/consume logic is unchanged, and existing authored-state recovery is
retained. No persistence, Send routing, export header or plugin action was added.

## Verification

The first negative parser test failed on stock source with `--project astraweave`
assigning `astraweave` to repo. It passes after the parser arms. Targeted commands:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo test --lib config::tests
cargo test --test pick_config
cargo test --test pick
cargo test --lib startup_tests
cargo test --test git_repo
cargo test --test render
cargo clippy --all-targets --all-features -- -D warnings
```

All pass: 25 existing config tests, 5 parser regressions, 9 picker tests,
7 startup tests, 59 Git integration tests, and 130 existing render tests.
Startup tests use temporary real repositories/linked worktrees, injected registry
roots and events; they cover bypass/cancellation, invalid visible lookup, config
repair before discovery, canonical session recovery, explicit-base priority,
run/project scope and None fallback. TestBackend covers narrow/tall/empty frames,
unavailable reasons, mouse hit regions, keymap input and identity reconciliation.

Iteration fixed GitFail-to-anyhow adaptation, Clippy style findings, and a test
fixture that dropped its TempDir too early. The repaired test has a bounded retry
assertion. No upstream assertion was weakened or removed.

Full gate:

```bash
RUSTFLAGS='-D warnings' just ci
```

**Exit 0; 555.597 seconds = 9m 15.597s.** Python `time.monotonic()` measured
from `2026-09-09T09:43:12.019430+00:00`. The first full gate invocation passed.
**857 passed, 0 failed, 1 ignored**: 325 unit, 264 app_flow, 11 briain_registry,
59 git_repo, 29 pane_actions, 9 pick, 5 pick_config, 24 pr_candidates, 130 render,
1 send_flow. The existing pr_live test remains ignored; binary/doc harnesses have
zero tests. This adds 21 tests to sitting 2's 836. Logs/timing are temporary at
`/tmp/reviewr-s3-ci.{log,json}`; durable measured outcomes are recorded here.

The unchanged shared terminal mode helpers and editor code did not require
`just smoke-edit`. A supervised PTY harness additionally checks picker cancellation,
absent-run refusal and conflicting-selector refusal. It uses a temporary registry,
removes HERDR_* and excludes the installed herdr from PATH/common-bin lookup.
Git's own GIT_TRACE records actual subprocesses (the host-path resolver bypasses
PATH wrappers). Each case exits 0, restores exact termios attributes and emits
alternate-screen, bracketed-paste and mouse-release sequences. Picker and absent-run
cases each perform one discovery Git query; conflicts perform zero. None read
reviewr baseline refs or run snapshot/write commands. No review worker starts.
Passed against both debug and the final release binary. Temporary harness:
`/tmp/reviewr-s3-smoke-pick.py`; release results: `/tmp/reviewr-s3-smoke-pick.json`. This is automated PTY coverage,
not operator acceptance in a real pane; live acceptance remains sitting 6.

## PTY latency

Rebuilt the starting main source `a3879fa27677ab1f362829de138c7b2a0c2241a9`
in an immutable `git archive` checkout, with a separate target directory:

```bash
CARGO_TARGET_DIR=/tmp/reviewr-sitting-3-before-target RUSTFLAGS='-D warnings' \
  cargo build --release --manifest-path /tmp/reviewr-s3-before-source/Cargo.toml
```

The build exited 0 and reported 7m 42s, reusing dependencies from the initial
interrupted build. Old SHA-256:
`078839b8b526a4a7db2e157326eab976774b140f518c226a86b2c00a6383be44`
(matches sitting 2's final binary). New source: `4878dc3`; new SHA-256:
`51ce374d2389a1af9a5a2791dc18e7fc824b45e09937d0a959ec2f3fdf17ba66`.
The full gate's release build reported 7m 58s; total gate time is above.

Ran the unchanged `scripts/bench_tui.py --binary <old-or-new> --fixture
--fixture-dir /tmp/reviewr-s3-fixture --iterations 10 --json <temporary-output>`.
Same deterministic fixture, 160×45 PTY, 300ms quiet gap. No concurrent build/test
work; HERDR_* removed and the installed herdr excluded from PATH/common-bin lookup.
No pane action or export was injected. All ten runs exited 0.

Order: **A1, B1, A2, B2, A3, B3, B4, A4, B5, A5**. The two reversed pairs
were added because the first three pairs showed one-frame timing variation on
both binaries. One-minute system load at starts, in that order: 4.13, 2.61, 3.43,
2.37, 5.38, 3.49, 3.28, 2.17, 1.49, 1.17. Interleaving and reversed order constrain
run-order effects, but do not remove the background-load/paint-band uncertainty.

Painted-frame medians in milliseconds, each run using 10 presses per scenario:

| Scenario | Old A1–A5 | New B1–B5 | Median of five run medians, old → new |
|---|---|---|---|
| tab_enter_all_files | 111.3, 95.4, 110.0, 110.8, 111.1 | 95.2, 95.5, 109.8, 111.1, 95.2 | 110.8 → 95.5 |
| tab_enter_changes | 99.4, 98.5, 99.8, 99.6, 99.1 | 99.4, 98.8, 98.9, 99.6, 99.5 | 99.4 → 99.4 |
| tab_enter_all_files_then_f | 111.8, 95.4, 111.4, 96.1, 111.5 | 95.5, 111.3, 96.0, 95.7, 95.9 | 111.4 → 95.9 |
| file_next_changes | 19.3, 14.8, 19.1, 19.3, 15.9 | 18.6, 18.1, 20.3, 17.8, 18.9 | 19.1 → 18.6 |
| file_next_all_files | 1.1, 1.0, 0.9, 1.1, 1.1 | 0.9, 0.9, 1.0, 0.9, 1.0 | 1.1 → 0.9 |

These are medians of run medians, not pooled sample medians. All-files scenarios
show two overlapping timing bands roughly one frame apart on both binaries.
Aggregate medians favor the new binary by 15.3/15.5ms, but individual pairs can
match or reverse; this is **not a proven throughput improvement**. Changes-tab
median is unchanged, and file-step medians differ by 0.5ms/0.2ms. No consistent
latency regression was observed. Absolute timings differ from sitting 2 and the
older macOS baseline, so those are not used as this sitting's A/B control.

Because the measured medians moved, **the single committed
`scripts/bench-results/baseline.json` is refreshed from B5**, an actual final-binary
run, not a fabricated aggregate. Its binary path/source revision are normalized
to the committed source and its SHA-256 is recorded. No per-round result files
were added to the repository. Raw logs/JSON are `/tmp/reviewr-s3-{A1..A5,B1..B5}`
with `.log`/`.json` suffixes; `/tmp/reviewr-s3-bench-meta.json` records order/load.
The raw script source revision predates the feature commit (source was uncommitted
while tested); the binary hashes above identify the actual tested artifacts.


## Departures and exact next step: sitting 4

No ticket-scope departure. The menu makes its two levels explicit through a
Project checkout row and an All retained runs entry. The parser remains API-additive
by carrying new-flag errors rather than changing every stock parse caller to Result.
Initial config repair is pre-App, while later recovery keeps the existing worker
mechanism and receives the selected Config. Environment-to-argv plugin wiring stays
with ticket 4. Neither registry discovery nor the Git helper implementations were
rewritten. The old benchmark build was moved to an immutable `git archive` checkout
so ongoing source edits could not contaminate it.

Sitting 4 is **Ticket 2 only**, commit boundary
`feat(send): send_to config key and --send-to flag`:

1. Add optional non-empty, single-line `send_to` to the whole-file config schema,
   default/getter/allowed-key list/normalized JSON. Keep exact schema/key-count tests.
2. Consume the already parsed `Config::send_to`: CLI beats config on startup,
   reread and recovery; absence preserves stock Send and candidates. Use the same
   effective value for dispatch and help/footer display.
3. Add `SendTo::{Cockpit, Name, Pane}` and `send_target_for` beside unchanged
   `send_target`/`candidates`. Resolve each attempt across workspaces, exclude self
   and non-agents, match panes/names exactly, resolve cockpit by name OR data-root
   cockpit cwd, deduplicate pane ids and require exactly one result.
4. Zero/multiple/failed lookup must visibly name the refusal/ambiguity and clipboard
   fallback while retaining all comments; never silently pick another target.
   Empty exports call no herdr. Resolve once per attempt; bracketed paste without
   Enter consumes only after successful delivery, including focus-failure success.
5. Add pure resolver, recovery/display and fake-herdr `tests/send_flow.rs` cases;
   preserve turn sampling and all upstream fixtures, then run the full gate.

Do not implement review headers or plugin actions in sitting 4. `ReviewIdentity`
is already available for ticket 3, but no export consumer should be added yet.

## Operator boundary and final status

No subagent or detached child was launched. Builds, tests and temporary PTY child
processes were supervised in this session. No plugin link/install, real herdr
pane/tab command, operator config edit or agent message was performed. No pane was
opened or closed. After merge the operator rebuilds the primary checkout at
`/home/brian-isaac/Documents/personal/herdr-reviewr`; this worktree was never linked.

All authored work is committed. The sole final porcelain exception is the expected
` M AGENTS.md`, which was neither edited, staged nor committed.

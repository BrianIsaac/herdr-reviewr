# Cockpit fork: sitting 2 handover

Date: 2026-09-09 (Asia/Singapore)
Status: registry/base foundation half of Ticket 1 implemented; sitting 3 not started.

## Delivered and commits

- `f77d144 feat(briain): project and run registry with data-root fixtures`
- `84a7781 feat(git): launch base chain and HEAD branch helpers`
- A separate `docs(...)` commit contains this handover, spec progress and the Unreleased entry.

Read the [spec](../../specs/2026-09-09-briain-cockpit-review/spec.md) and
[Ticket 1](../../specs/2026-09-09-briain-cockpit-review/plan.md) before continuing.
Neither the existing plan nor the sitting 1 handover was edited.

`src/briain.rs` exposes `Project`, `Run`, `Availability`, `projects(root)`,
`runs(root)`, `validate_id`, `checkout_availability`, `resolve_data_root` and
`data_root`. Environment precedence is `BRIAIN_DATA_DIR`, otherwise
`$HOME/.local/share/briain`, identically on Linux/macOS. Empty override or missing
HOME returns an error. Tests inject paths without mutating process environment.

Project discovery reads regular `.md` notes, opening frontmatter only; body keys
never participate. The scalar subset supports plain strings, YAML single quotes
with doubled apostrophes, JSON-compatible double quotes, and null spellings.
Unsupported relevant values, duplicate fields, invalid ids, missing/non-absolute
paths and inactive projects produce unavailable rows with reasons. Metadata is
bounded to 256 KiB; project reading stops at the closing delimiter, so a large
Markdown body is harmless. An unambiguous id survives another malformed field,
so an invalid duplicate cannot leave a valid sibling arbitrarily selectable.
Malformed identities remain unavailable and are not guessed from filenames.

Run discovery enumerates only directories under `worktrees/`, ignores stray
files, and joins metadata by exact basename. `job.yaml` supplies project;
`status.json` supplies state/phase. Missing or malformed metadata remains unknown
with diagnostic strings; a valid checkout remains available. There is no scan
of historical `runs/`, nor an inference from a job id. Git-file linked worktrees
are checked through existing `git::toplevel`; broken/pruned links stay visible
and unavailable. `checkout_availability` is reusable for Enter-time revalidation.
It requires an actual worktree root, rather than any subdirectory of a checkout.

`git::head_branch` reads actual symbolic HEAD and returns None for detached or
unborn HEAD. `git::launch_base` returns the first commit-resolvable full ref:
`refs/heads/main`, `refs/heads/master`, then the selected branch's configured
tracking ref, else None. It uses `for-each-ref %(upstream)` for the same tracking
relationship as `@{upstream}`, because a missing/dangling tracking target makes
`rev-parse @{upstream}` fatal rather than an ordinary absence. Tracking a remote
named `team` and tracking another local branch both work. Explicit base priority
belongs to sitting 3 orchestration; the helper is not called by startup yet.

**No writes:** discovery/base helpers contain no mutations; tests preserve refs,
index bytes and stock private-pick/fallback results. **Comments survive** and
**Continuity:** App, world reconciliation, comment/export state and terminal
ownership are untouched. Stock `resolve_base` and all upstream assertions remain
unchanged. No dependency was added, including no `serde_yaml`.

## Live inspection and fixtures

Read-only inspection covered the astraweave and herdr-reviewr project notes,
retained worktree directories and matched run metadata, including this run.
The current run had status keys `backend`, `briefing_bytes`, `briefing_path`,
`exit_reason`, `harness`, `phase`, `state`, `ts`, `window_target`; lifecycle values
were `running` / `interactive`. Its job had `project: herdr-reviewr` and
`mode: interactive`. Other retained runs showed green/red interactive lifecycle
states; a stray zip existed beside directories. The `.git` entries were files.

Sanitized fixtures live in `tests/fixtures/briain/`. They retain only fields
needed for parsing and metadata joins, with synthetic description edge cases
and temporary-path substitution. Tests generate real repositories/linked
worktrees using `tests/common/mod.rs`, then invalidate a temporary gitdir to
exercise a pruned link. No test depends on the live mutable data root.

## Verification

Final targeted command:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
RUSTFLAGS='-D warnings' cargo test --test briain_registry --test git_repo
```

**11 registry tests and all 59 Git integration tests passed.** Coverage includes
quoted/plain/null paths, colons, embedded `id:`, body keys, inactive/malformed/
duplicate projects, missing registry, traversal ids, stray files, historical-only
runs, absent/bad metadata, pruned links, root revalidation, bounded frontmatter,
and a large ignored body. Git coverage adds main-before-master/tracking,
divergent origin/main, master-only, remote/local tracking-only, missing/dangling
tracking, detached/unborn HEAD and no rung.

Full gate:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
RUSTFLAGS='-D warnings' just ci
```

**Exit 0; 494.110 seconds = 8m 14.110s.** Python `time.monotonic()` measured
wall time from `2026-09-09T08:54:17.554848+00:00`; the release build reported
7m 55s. The full gate passed on its first invocation. Logs/timing are temporary
at `/tmp/reviewr-s2-ci-a4c7ed.{log,json}`; the durable outcomes are recorded here.

**836 passed, 0 failed, 1 ignored**: 318 unit, 264 app_flow, 11 briain_registry,
59 git_repo, 29 pane_actions, 24 pr_candidates, 130 render, 1 send_flow. The
pre-existing pr_live test remains ignored; binary/doc harnesses have zero tests.
This adds 16 tests to sitting 1's 820 without weakening or removing any upstream
assertion. Formatting and Clippy pass with warnings as errors. Targeted iteration
fixed a fixture-count mismatch and two Clippy nits before the full gate.

No smoke-edit run was needed: no editor path or terminal mode changed. Real
operator/picker acceptance remains deferred; nothing here claims that flow works.

## PTY latency

Rebuilt the unchanged source at `962b8e6` before editing source, using a second
target directory:

```bash
CARGO_TARGET_DIR=/tmp/reviewr-sitting-2-before-target RUSTFLAGS='-D warnings' cargo build --release
```

That build exited 0 and reported 2m 42s (including a build-directory lock wait).
The new binary is the full gate's `target/release/herdr-reviewr`, with feature
source at `84a7781`. Old SHA-256:
`2e42a167ef9eb7200c625ebff5c9764a2df62dc076adeb4ae4ab25f868849338`
(matches sitting 1's upstream binary). New SHA-256:
`078839b8b526a4a7db2e157326eab976774b140f518c226a86b2c00a6383be44`.

Ran `scripts/bench_tui.py --binary <old-or-new> --fixture --fixture-dir
/tmp/reviewr-s2-fixture-a4c7ed --iterations 10 --json <temporary-output>` in
**A1, B1, A2, B2, A3, B3** order, sequentially after compilation stopped. Both
variants used the same deterministic fixture, 160×45 PTY and 300ms quiet gap.
The benchmark environment removed `HERDR_*` and exposed only Git on PATH, so no
installed herdr command/config lookup could run. The normal benchmark script
and scenarios were unchanged. No pane actions or exports were injected.

Painted-frame medians, milliseconds (10 presses per scenario in each run):

| Scenario | A1 old | B1 new | A2 old | B2 new | A3 old | B3 new | Median of run medians, old → new |
|---|---:|---:|---:|---:|---:|---:|---:|
| tab_enter_all_files | 48.9 | 33.3 | 49.3 | 49.2 | 49.2 | 49.3 | 49.2 → 49.2 |
| tab_enter_changes | 36.8 | 36.4 | 36.7 | 36.8 | 36.6 | 37.0 | 36.7 → 36.8 |
| tab_enter_all_files_then_f | 49.6 | 49.8 | 34.1 | 49.6 | 49.3 | 49.8 | 49.3 → 49.8 |
| file_next_changes | 19.0 | 19.2 | 20.1 | 19.5 | 17.6 | 20.3 | 19.0 → 19.5 |
| file_next_all_files | 1.0 | 1.0 | 1.0 | 1.0 | 0.9 | 1.0 | 1.0 → 1.0 |

The final column is the median of three run medians, not a pooled sample median.
Initial All-files/chased-key runs showed approximately one-frame variation on
both binaries; the third pair checked that uncertainty. Stable scenario medians
changed by at most 0.5ms. First-byte medians across runs were also within 0.1ms
except next-file Changes (19.0 → 19.5ms, which paints synchronously).
System one-minute load averages at run starts were 1.39, 1.59, 1.33, 1.26, 0.75,
0.46; A/B were interleaved on the same machine with no concurrent build/test work.

**No material repeatable latency shift: the committed
`scripts/bench-results/baseline.json` is unchanged.** Its older macOS absolute
numbers are not the comparison instrument for this Linux sitting. The additive
helpers have no runtime caller yet; this PTY check covers unchanged review paths,
not future registry/picker startup latency. Temporary raw JSON/log pairs are
`/tmp/reviewr-s2-{A1,B1,A2,B2,A3,B3}-a4c7ed.{json,log}`. Their script-reported
source revision is the current checkout; binary provenance is recorded above.
No per-round benchmark artifact was committed.


## Departures and exact next step: sitting 3

No scope departure: this sitting delivers only Ticket 1 steps 1–2. A dedicated
registry integration harness exercises real temporary repositories, while Git
cases extend the existing harness. The bounded scalar subset is explicit and
does not attempt general YAML. The upstream-query implementation detail above
preserves the specified base semantics. No existing plan document was revised.

1. Start with explicit parser arms and negative fallthrough tests in
   `src/config.rs` for `--project`, `--run`, `--pick`, and reserve/parse `--send-to`
   before any caller or manifest emits them. Prove `--project astraweave` never
   assigns `astraweave` to `repo`; cover each valued flag, required nonempty ids,
   missing values without consuming a following flag, selector conflicts and
   selector-plus-positional-path conflicts. Preserve unrelated legacy parsing.
2. Then implement `src/pick.rs`: pure two-level projects/runs menu state and
   rendering, key/digit/mouse conventions, inactive/unavailable reasons,
   cancellation and identity-based reconciliation. Validate explicit ids before
   path joins; refuse invalid/duplicate/unavailable selections visibly. Recheck
   the chosen root on Enter and carry its canonical identity forward.
3. Integrate pre-App startup design **A**: paint startup before discovery/config
   fallback; resolve validated root/base/scope/review identity before `app_for`,
   `App::new`, initial reload, baseline seeding or `TurnHost::open`. An explicit
   base wins; otherwise use `launch_base`, and leave None to stock fallback. Runs
   start in Branch, projects retain configured default scope. Preserve selection
   through config-directory fallback and config recovery; never construct App or
   workers on a placeholder cwd or retarget a running App. Restore terminal state
   on cancellation. Add the planned startup/recovery/render tests and full gate.

The `feat(pick): ...` boundary belongs to sitting 3. Routing, export headers and
plugin actions remain later sittings.

## Operator boundary and final status

No subagent or detached child was launched. Builds, tests and PTY sessions were
supervised in this session. No plugin link/install, real herdr pane/tab command,
pane open/close, operator config edit or agent message was performed. The linked
plugin still points to the primary checkout; after merge, the operator rebuilds
there. This worktree was never linked.

All authored work is committed. The only final porcelain exception is the
pre-existing ` M AGENTS.md`; it was neither edited, staged nor committed.

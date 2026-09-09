# Cockpit fork sitting 6 handover

Date: 2026-09-09. Scope: Ticket 5 documentation, isolated upstream rebase rehearsal,
quiet-load benchmark assessment and automated gate. Live operator acceptance is
outside this pane and happens after merge/rebuild.

## Delivered commits

- `a431e4b docs(readme): cockpit review flow and pick binding`: registry/retained-run
  discovery, selectors, base/scope/B policy, Send precedence/refusals, contextual
  header/placeholders, literal selection copying, manual submission, independent
  pick splits and multi-review toggle/close behavior. Documents the primary-checkout
  build/link/reopen loop and downloaded-upstream limitation.
- `72ade69 docs(rebase): upstream rebase recipe and rehearsal record`: `docs/fork.md`
  records dated backup, clean prerequisite, pinned upstream/main replay from
  `4c09022`, abort/recovery, range-diff, conflict checks, gate, install and acceptance.
- No `chore(bench)` commit: another heavy job remained active at the load checks;
  the user-authorized quiet-machine exception applies. No benchmark result was
  manufactured and the single baseline file is untouched.
- A separate `docs(cockpit)` commit records only completed Ticket 5 criteria in the
  spec and finalizes the Unreleased changelog. This handover has its own commit.

Binding evidence: installed `herdr 0.7.5`, read-only `herdr --default-config` lines
54–63 document `[keys]`, `prefix`, and explicit `prefix+n` action syntax. The current
operator config already uses `prefix = "ctrl+a"`, `key = "prefix+d"`, and the
`briain-reviewr` popup. README replaces that example with `type = "plugin_action"`
and `command = "persiyanov.reviewr.pick"`; no operator config was edited.

## Rebase rehearsal and conflict surface

Fetched `upstream/main` = `4c090225af706bf3aaa24b39fea890a72994f40f`, still the adopted
v0.36.2 boundary. There were no newer upstream commits. Rehearsal therefore proves
replay mechanics and tree/patch equivalence, not future upstream compatibility.

Source `a431e4bfab84b1d5a9dda06c72370e5909e9c51c` was replayed in detached worktree
`/tmp/reviewr-s6-rebase-20260909` using `git rebase --onto upstream/main 4c09022`.
Result `c01a692`: 19 non-merge patches, zero conflicts, zero manual resolutions.
Range-diff matched all 19 with `=`; the independent initial research series reordered
when merges flattened. Final source and rebased trees were identical. The worktree
was clean and removed after its targeted tests; no rebased history was pushed,
installed or applied to main. No origin remote branch was touched.

Observed resolution was automatic clean replay for every changed file. The complete
file-by-file record is in [docs/fork.md](../../fork.md#rehearsal-2026-09-09). The
coupled surfaces inspected were:

- `src/briain.rs`, `src/git.rs`, `src/pick.rs`: registry/base/startup picker boundaries.
- `src/config.rs`: parser plus schema/default/getter/12-key normalized JSON coherence.
- `src/lib.rs`, `src/startup_tests.rs`, `src/app.rs`: pre-App selection, recovery,
  identity and dispatch ownership.
- `src/herdr.rs`, `src/export.rs`, `src/ui.rs`: explicit exact-one routing vs stock
  candidates, shared batch shape, consume-on-success and effective target display.
- `herdr/pane.sh`, `herdr-plugin.toml`: dedicated pick before legacy dispatch, live
  focused target, quoted argv, preserved identity/version/hooks.
- `tests/app_flow.rs`, `tests/briain_registry.rs`, `tests/git_repo.rs`,
  `tests/pane_actions.rs`, `tests/pick.rs`, `tests/pick_config.rs`, `tests/render.rs`,
  `tests/send_flow.rs`, plus all six briain fixture files: exact stock assertions and
  fork regressions retained.
- `README.md`, `CHANGELOG.md`, `scripts/bench-results/baseline.json`, spec/plan,
  research and prior handovers: clean replay without manual edits.

The separable feature series remains: registry `f77d144`, base helpers `84a7781`,
parser `06940d0`, startup picker `4878dc3`, Send `202d0f5`, export `60021a4`, plugin
pick `a28f3ac`, then this sitting's documentation. These implement the five planned
tickets through focused commits, not literally one commit per ticket.

## Verification

- Throwaway `bash -n herdr/pane.sh`: passed. No shell file changed this sitting.
- Throwaway `CARGO_TARGET_DIR=/tmp/reviewr-s6-target RUSTFLAGS='-D warnings' cargo test
  --all-features --lib --test briain_registry --test git_repo --test pick
  --test pick_config --test app_flow --test render --test send_flow --test pane_actions`:
  **850 passed**, zero failed/ignored. Counts: lib 333, app_flow 266, registry 11,
  git_repo 59, pane_actions 34, pick 9, pick_config 5, render 131, send_flow 2.
- Full `CARGO_TARGET_DIR=/tmp/reviewr-s6-target RUSTFLAGS='-D warnings' just ci`:
  **passed in 505.300 seconds (8m 25.300s)** on retry, including formatting, Clippy,
  **874 passed, zero failed, 1 ignored**, and the optimized release build.
- `git diff --check`: passed. No runtime source change or terminal/editor conflict
  resolution required `just smoke-edit` this sitting.

The single full gate runs at the end on the source branch, as explicitly requested,
using `/tmp/reviewr-s6-target` with the package artifacts rebuilt for this checkout. The isolated replay's tree was
identical. The earlier plan's wording about a separate full rehearsal gate is
superseded by this sitting's targeted-rehearsal/one-final-gate instruction.

The initial final-gate attempt failed after **19.617 seconds**: 12 `pane_actions`
tests referenced the already-deleted throwaway's compile-time `CARGO_MANIFEST_DIR`.
Sharing the target directory across identical source checkouts reused stale test
executables. `CARGO_TARGET_DIR=/tmp/reviewr-s6-target cargo clean -p herdr-reviewr`
removed only this package's cached artifacts, then the entire gate was rerun.
No product/test assertion was changed or weakened. A future rehearsal should use
separate package build caches or clean the package before changing checkout paths.

## Latency: deferred under load

All load averages below are `uptime`'s 1/5/15-minute values, local Asia/Singapore time
on 2026-09-09. Each check also ran `pgrep -af 'codex|pnpm|uv run|cargo'`:

- **19:01:53: 2.49 / 2.74 / 2.90**, before this session's test build.
- **19:03:56: 6.48 / 4.15 / 3.39**, after targeted tests and before the full gate;
  this includes this session's recent build load as well as the other job.

- **19:15:18: 3.00 / 3.54 / 3.50**, after the successful gate; the same separate
  job was still active. No quiet measurement window was established.

All three checks found the separate Astraweave workstream: `uv run … invent glyphs
regenerate` (PID 1535357), `pnpm exec tsx scripts/roundtrip-drawings.ts … --maintenance`
(PID 1535378), and its Codex session (PID 1524413). This was an active workload,
not just this pane's own Codex wrapper. It was not interrupted or messaged.

No new benchmark ran, so **there are no sitting 6 measured medians**. The unchanged,
load-contaminated sitting 3 baseline (`source_rev = 4878dc3`) still reports these
first-byte / painted medians in ms, for reference only:

- Enter All files: 0.9 / 95.2.
- Enter Changes: 1.1 / 99.5.
- Enter All files then `f`: 0.9 / 95.9.
- Next file in Changes: 18.9 / 18.9.
- Next file in All files: 1.0 / 1.0.

These are not revalidated results or a claimed latency pass. A later quiet sitting
must rebuild the current main binary and the comparison tip in a second target
directory, run repeated interleaved fixture A/B captures under the same load, record
load/per-scenario medians and replace only the single baseline from an actual final
run. The adopted upstream `4c09022` is available as the old comparison tip.

## Exact remaining operator checklist, after merge

None of the live checks below is claimed passed by this pane.

1. In `/home/brian-isaac/Documents/personal/herdr-reviewr`, merge the accepted work,
   export `PATH="$HOME/.cargo/bin:$PATH"`, run `just install`, then refresh
   `herdr plugin link .`. Confirm `herdr plugin list` and executable links point to
   this permanent checkout. Do not use the downloaded upstream plugin.
2. In the operator's herdr config, rebind **Ctrl+A then d** from the stopgap wrapper
   `~/.local/bin/briain-reviewr` to `persiyanov.reviewr.pick` using the README's
   `prefix+d` / `plugin_action` snippet. Replace the existing binding. Configure
   reviewr `send_to = "cockpit"` if absent (or intentionally use `--send-to cockpit`);
   split placement alone does not select the Send destination.
3. Export pending comments and close/reopen old reviewr panes manually. Rebuild or
   refresh alone does not restart them; generic toggle/close may close multiple reviews.
4. Focus the cockpit **agent**, press **Ctrl+A then d**, pick **astraweave** and verify
   its actual checkout opens in the adjacent split. Add a comment and press **s**.
   Verify header and body land in the cockpit input with **no Enter**; submission is
   manual. Confirm the header names the actual project/run/branch/base context.
5. Verify comment **Copy parity** with an equivalent batch (successful export consumes
   comments), and verify selected-text Copy remains literal.
6. Select a **retained run**; verify its checkout and Branch scope, actual branch and
   correct run identity, including placeholders for unavailable metadata.
7. Exercise **absent and ambiguous explicit targets**: visible refusal, no delivery to
   another agent, clipboard alternative offered and all comments retained.
8. Verify **comment survival** across refresh, agent edits and failed export; only
   successful explicit export consumes the batch. Record actual live outcomes.

## Boundaries and final status

**No writes**, **Comments survive** and **Continuity** remain unchanged by this
documentation-only sitting. The repository's maintenance worktree/rebase is separate
from reviewr's runtime No writes contract. No subagent or detached child process was
launched; build/test children were supervised in this session. No real herdr pane,
tab, send, install, link, reopen or agent/cockpit message was performed. No operator
keybinding/reviewr config was edited. Sitting 1–5 handovers and existing plans were
not edited. AGENTS.md was not rewritten, staged or committed.

Final authored work is committed; the expected sole porcelain exception is
` M AGENTS.md`. Outstanding work is the quiet-machine latency retake and the operator
checklist above; live acceptance is not a completion claim of this pane.

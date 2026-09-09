# Maintaining the cockpit fork

Adopted upstream boundary: **`4c090225af706bf3aaa24b39fea890a72994f40f`**
(`4c09022`, upstream v0.36.2). Recorded 2026-09-09. Upstream updates are deliberate
rebases; the downloaded upstream plugin cannot run these fork features.

## Rebase recipe

Start with a clean fork branch: inspect `git status --short` and finish or explicitly
stash your own work first. Never stage, rewrite or erase a briain-managed AGENTS.md
brief to clean a workspace; use a clean detached worktree for rehearsal instead.
Do not rewrite shared main as part of a rehearsal.

From the permanent clone, record the current tip and a unique dated backup before
replaying. The example creates a separate local integration branch; it does not
update main or any origin branch:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cd /home/brian-isaac/Documents/personal/herdr-reviewr
# Confirm clean status and that HEAD is the intended fork tip before continuing.
git status --short
git remote -v
git fetch upstream
fork_boundary=4c090225af706bf3aaa24b39fea890a72994f40f
fork_backup="backup/cockpit-before-rebase-$(date +%Y%m%dT%H%M%S)"
git branch "$fork_backup" HEAD
fork_upstream=$(git rev-parse upstream/main)
git switch -c "integrate/cockpit-$(date +%Y%m%dT%H%M%S)"
git rebase --onto "$fork_upstream" "$fork_boundary"
```

`fork_upstream` pins the fetched `upstream/main` for this attempt. This intentionally
flattens merge commits while preserving the feature patches. Inspect all conflicts,
resolve the coupled surfaces below, stage only resolved paths, then use
`git rebase --continue`. Do not use blanket “ours”, “theirs”, or `--skip` to silence
conflicts. To abandon an in-progress attempt:

```bash
git rebase --abort
```

Abort returns the integration branch to its pre-rebase tip. The dated backup also
preserves that tip after a completed rebase; switch to the backup to inspect or
recover it without resetting shared main. After completion:

```bash
git range-diff "$fork_boundary..$fork_backup" "$fork_upstream..HEAD"
git diff --check
bash -n herdr/pane.sh
RUSTFLAGS='-D warnings' just ci
```

Review every changed or unmatched patch in `range-diff`; a clean textual replay is
not behavioral verification. Preserve **No writes**, **Comments survive** and
**Continuity**. Keep discovery/base/parser/startup, explicit Send, contextual export,
plugin pick, and documentation changes separable. The five planned ticket boundaries
currently span several focused commits (foundations and parser landed separately).
Do not squash them into an opaque upstream-resolution patch.

If terminal ownership, editor dialects or `run_editor` changed, run `just smoke-edit`.
For git/reload/render/highlight changes, rebuild the old tip into a second target
directory and interleave repeated old/new `scripts/bench_tui.py --fixture` runs under
the same quiet load. Check `uptime` and `pgrep -af 'codex|pnpm|uv run|cargo'` first;
defer if another heavy job is active. Record per-scenario medians and replace only
`scripts/bench-results/baseline.json` with an actual final run, never per-round files.

After review and gate success, update the adopted boundary in this guide to the
pinned `fork_upstream` OID. Arrange shared-history integration explicitly with the
operator; this recipe does **not** push or force-push any rewritten history.

Once the accepted history is integrated into the permanent primary checkout, the
operator rebuilds and refreshes the local plugin there:

```bash
cd /home/brian-isaac/Documents/personal/herdr-reviewr
export PATH="$HOME/.cargo/bin:$PATH"
just install
herdr plugin link .
herdr plugin list
readlink -f ~/.local/bin/herdr-reviewr
readlink -f ~/.local/state/herdr/plugins/persiyanov.reviewr/bin/herdr-reviewr
```

If switching from a downloaded plugin, first run
`herdr plugin uninstall persiyanov.reviewr`. Confirm the plugin source and both
executable links resolve to the permanent checkout. Never link a throwaway worktree.
`herdr/install.sh` downloads upstream binaries even when installing from the fork's
GitHub repository; that installation cannot distribute the cockpit changes.

Export comments, then manually close/reopen panes to use the new binary. Refresh
cannot replace a running process. Repeat the operator checklist in the
[sitting 6 handover](plans/progress/cockpit-fork-sitting-6-handover.md), including the
pick binding, checkout/split, contextual Send without Enter, Copy parity, retained
run, target refusals and comment survival. Generic toggle/close may close multiple
reviews. Never automate pane opens.

## Rehearsal: 2026-09-09

Fetched `upstream` without changing origin refs. Fetched `upstream/main` was still
`4c090225af706bf3aaa24b39fea890a72994f40f`, exactly the adopted boundary: there were
**no new upstream commits**. This rehearses replay and verification mechanics only;
it provides no evidence about conflicts with a future upstream release.

Source tip: `a431e4bfab84b1d5a9dda06c72370e5909e9c51c` (sitting 5 merge plus README).
A detached worktree at `/tmp/reviewr-s6-rebase-20260909` replayed it with:

```bash
git worktree add --detach /tmp/reviewr-s6-rebase-20260909 a431e4b
git -C /tmp/reviewr-s6-rebase-20260909 rebase --onto upstream/main 4c09022
```

Result: `c01a692`, 19 non-merge patches, **zero conflicts**, no manual resolutions.
`git range-diff 4c09022..a431e4b upstream/main..c01a692` matched all 19 patches with
`=`. The first two independent research series reordered when merges flattened;
no patch was dropped or changed. `git diff a431e4b c01a692` was empty.
No backup branch was needed for this disposable detached replay: the source branch
remained at its original tip. The permanent-update recipe above includes one.

The file-by-file conflict surface below distinguishes observed automatic resolution
from the checks required if these files overlap a future upstream change:

| File | Observed resolution; future overlap check |
| --- | --- |
| `src/briain.rs` | Clean addition; keep bounded registry parsing, exact joins, unknown metadata and duplicate refusal. |
| `src/git.rs` | Clean replay; keep launch-only base chain/actual HEAD helpers separate from stock resolution and private refs. |
| `src/config.rs` | Clean replay; merge parser arms, 12-key allowlist, defaults/getter/normalized JSON and CLI-over-config recovery together. |
| `src/pick.rs` | Clean addition; keep startup selection identity and unavailable-row behavior. |
| `src/lib.rs` | Clean replay; resolve before App/workers, preserve terminal cleanup and selected launch through recovery. |
| `src/startup_tests.rs` | Clean addition; preserve invalid selector, cancellation and recovery checks. |
| `src/app.rs` | Clean replay; preserve identity/effective Send target through recovery and one shared export path. |
| `src/herdr.rs` | Clean replay; keep explicit target resolver separate from unchanged stock candidates; exact-one refusal. |
| `src/export.rs` | Clean replay; preserve stock body bytes, identity-gated header, missing values and literal selection copying. |
| `src/ui.rs` | Clean replay; display the same effective Send target used by dispatch. |
| `herdr/pane.sh` | Clean replay; validation before pick, focused live agent check, dedicated split branch before legacy sweep. |
| `herdr-plugin.toml` | Clean replay; preserve identity/version/hooks and quoted env-to-argv expansion. |
| `tests/app_flow.rs` | Clean replay; preserve comment survival, recovery, Copy/Send parity and literal-selection checks. |
| `tests/briain_registry.rs` | Clean addition; retain real-layout and malformed registry cases. |
| `tests/git_repo.rs` | Clean replay; keep base-chain and no-private-write tests with stock assertions. |
| `tests/pane_actions.rs` | Clean replay; keep exact manifest assertions and fake-herdr argv/refusal/legacy tests. |
| `tests/pick.rs` | Clean addition; retain picker navigation and unavailable selection checks. |
| `tests/pick_config.rs` | Clean addition; retain selector/parser/env precedence assertions. |
| `tests/render.rs` | Clean replay; preserve stock snapshots and picker sizing/target-label checks. |
| `tests/send_flow.rs` | Clean replay; retain one framed paste, no Enter, refusal retention and consume-once tests. |
| `tests/fixtures/briain/README.md` | Clean addition; retain sanitized fixture layout explanation. |
| `tests/fixtures/briain/notes/projects/null.md` | Clean addition; retain null scalar case. |
| `tests/fixtures/briain/notes/projects/plain.md` | Clean addition; retain plain scalar case. |
| `tests/fixtures/briain/notes/projects/quoted.md` | Clean addition; retain quoted scalar case. |
| `tests/fixtures/briain/runs/opaque-run/job.yaml` | Clean addition; keep project association here, not in status JSON. |
| `tests/fixtures/briain/runs/opaque-run/status.json` | Clean addition; keep lifecycle metadata here. |
| `README.md` | Clean replay; reconcile upstream tables with cockpit selectors, Send and local-build instructions. |
| `CHANGELOG.md` | Clean replay; retain implemented fork entries under Unreleased without duplicating upstream release entries. |
| `scripts/bench-results/baseline.json` | Clean replay; old measurement remains load-contaminated, not a quiet-machine acceptance result. |
| `docs/specs/2026-09-09-briain-cockpit-review/spec.md` | Clean replay; preserve invariants and truthful acceptance evidence. |
| `docs/specs/2026-09-09-briain-cockpit-review/plan.md` | Clean replay; historical plan unchanged. |

The two research files (`docs/research/cockpit-fork-surfaces-{claude,codex}-2026-09-09.md`),
their two `docs/plans/progress/cockpit-fork-surfaces-{claude,codex}-handover.md` files,
and each sitting 1–5 handover also replayed cleanly without edits. No conflict was
fabricated to populate this record. AGENTS.md is outside this feature series and
was not rewritten, staged or committed.

Validation in the throwaway passed: `bash -n herdr/pane.sh` and
`CARGO_TARGET_DIR=/tmp/reviewr-s6-target RUSTFLAGS='-D warnings' cargo test --all-features
--lib --test briain_registry --test git_repo --test pick --test pick_config
--test app_flow --test render --test send_flow --test pane_actions` (see the handover
for counts). The worktree was clean and removed with `git worktree remove` after
testing; its rebased history was never pushed or installed. The session's single
full `RUSTFLAGS='-D warnings' just ci` gate runs on the source branch at the end,
whose replayed tree was identical, as required by the sitting 6 scope. No terminal
ownership/editor code changed during replay, so `just smoke-edit` was not required.
The quiet-load latency retake remains separately gated; see the handover for its
actual outcome.

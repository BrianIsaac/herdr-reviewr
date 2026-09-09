# Cockpit fork sitting 5 handover

Date: 2026-09-09. Scope: tickets 3 and 4 only, in that order.

## Delivered and commit boundaries

1. `60021a4 feat(export): one-line review header on send and copy`
   adds pure `ReviewHeader` / `format_batch` around unchanged `format_all`, with one
   identity-gated call from `App::export`. Current HEAD supplies the actual branch;
   existing `branch_base.winner` supplies the resolved base and seven-character OID.
   Missing context uses the spec placeholders. No diff is built to discover a missing
   base. Metadata delimiters and controls are sanitized; comment formatting is unchanged.
   Both comment export destinations receive the same batch. Selected-text copying stays
   literal, ordinary sessions retain exact body-only bytes, failed export retains every
   comment, and successful delivery consumes once even if focus fails.
2. `a28f3ac feat(plugin): pick action opening a split beside the focused pane`
   adds the manifest action and dedicated dispatch before the legacy workspace sweep.
   Whole-file validation comes first. Pick prefers context `focused_pane_id`, requires an
   exact agent row and a nonempty live process group, and forces a focused split in the
   configured direction. It uses an existing live/context directory, falling back to HOME,
   and requires a successful open with a returned pane id. Existing reviews are untouched.
   Quoted shell argv carries `REVIEWR_PICK`, `REVIEWR_PROJECT`, `REVIEWR_RUN`,
   `REVIEWR_SEND_TO` and `REVIEWR_BASE`; the existing absolute pane entrypoint expands
   these into supported flags. Explicit project/run values suppress the picker hint.
   Legacy action dispatch, identity, version and lifecycle hooks are preserved.

Spec acceptance progress and the Unreleased changelog are updated separately from the
feature commits. No existing plan or sitting 1–4 handover was edited.

## Verification and measured outcomes

- Ticket 3 targeted: 7 export unit tests, 1 contextual App export test, 1 selected-session
  literal-copy test and 2 fake-herdr Send tests passed (11 total).
- Ticket 4 targeted: `bash -n herdr/pane.sh` passed; `cargo test --test pane_actions`
  passed all 34 tests in 2.01 seconds. These include all existing shell action tests and
  5 new tests covering the pick action, refusals, directory fallback, manifest argv and
  picker UI identity. The fixture logs NUL-delimited arguments to verify boundaries.
- Full gate: `RUSTFLAGS='-D warnings' just ci` exited 0 in **558.416 seconds**
  (**9m 18.416s**): **874 passed, 0 failed, 1 ignored**, with formatting, Clippy and
  release build green. The gate ran once after both feature commits.
- `git diff --check` passed. Existing body-only/bare-session exact assertions were retained.
  A byte comparison confirmed that `pane.sh` outside the inserted pick block is unchanged;
  a parsed manifest comparison confirmed only the pick action and pane command changed.

The first shell test run had 33 passes and one fixture failure: it requested direction
`left`, but the existing validated schema permits `right` or `down`. The fixture was
corrected to `down`; the schema and product direction behavior were not expanded.

## Departures and invariants

No ticket-scope departure. The permitted single full gate runs after both feature
commits because ticket 4 changes no Rust runtime paths from ticket 3. It covers all
features, Clippy, formatting, the full suite and a release build under `-D warnings`.

**No writes:** no git/reload/render algorithm changed; export only calls the existing
read-only actual-branch helper. No PTY latency capture was required and the committed
baseline was left alone. No shared terminal modes changed, so `just smoke-edit` was not
required. **Comments survive:** all failure and consume-on-success semantics remain;
comments stay in memory. **Continuity:** no startup/session ownership or reconciliation
behavior changed. **Additive-only:** headers require identity and pick has its own branch;
stock formatting and legacy action code are preserved.

## Exact next step: sitting 6

1. Write the README cockpit-flow section and `pick` binding documentation: registry and
   retained runs, selectors and base/scope policy, explicit Send precedence/refusals,
   header context/placeholders, literal selection copying, no automatic Enter and the
   fact that generic toggle/close can close multiple reviews. Verify the installed
   keybinding prefix syntax before publishing the snippet. The operator must rebind
   ctrl+a d from the stopgap wrapper to `persiyanov.reviewr.pick`; this session made no
   keybinding or reviewr config edit.
2. Write the upstream-rebase recipe and rehearse against `upstream/main` in a throwaway
   worktree. Record the conflict surface, adopted boundary (initially `4c09022`), dated
   backup, abort/range-diff procedure and verification results. Inspect the separable
   feature series; do not rewrite shared history or auto-push it. Include the primary
   checkout rebuild/link/reopen workflow and upstream-download limitation.
3. Re-take the latency baseline on a quiet machine. Sitting 3's capture occurred while
   another heavy job loaded the machine. Use the repository's same-load/interleaved
   comparison discipline and retain only the one committed baseline.
4. Operator-only live check after merge and rebuild in
   `/home/brian-isaac/Documents/personal/herdr-reviewr`: focus the agent, **ctrl+a d**,
   pick **astraweave**, verify its checkout in the adjacent split, comment, **s**.
   Confirm the header and body land in the cockpit input with **no Enter**. Verify Copy
   parity, a retained run, absent/ambiguous target refusal and comment survival. Record
   actual results; none of these live checks is claimed passed here.

## Operator boundary and final status

All tests used fake herdr; no real pane/tab/send command, plugin link, install, pane
reopen, cockpit/agent message or machine configuration edit was performed. No subagent
or detached child process was launched. Builds and test subprocesses were supervised
within this session. The linked plugin still points at the primary checkout; the
operator rebuilds it there after merge and closes/reopens panes manually.

All authored work is committed. The sole final porcelain exception is the expected
` M AGENTS.md`, which was neither edited, staged nor committed. Sitting 6 remains
entirely deferred, including README work, rebase rehearsal, latency re-take and live checks.

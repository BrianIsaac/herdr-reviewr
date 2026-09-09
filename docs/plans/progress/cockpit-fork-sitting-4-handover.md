# Cockpit fork: sitting 4 handover

Date: 2026-09-09 (Asia/Singapore)
Status: Ticket 2 implemented; review header and plugin pick action remain pending.

## Delivered

- `202d0f5` `feat(send): send_to config key and --send-to flag`
- A separate docs commit contains this handover, ticket 2 acceptance progress in the
  spec, and the Unreleased changelog update. Existing plans and sitting 1–3 handovers
  were not edited.

`PluginConfig` now accepts optional `send_to`, includes it in the 12-key allowlist,
provides its getter, and exports it as a string or null in normalized JSON. Empty,
whitespace-only, wrong-type and control-character values invalidate the whole file;
unknown keys still invalidate the whole file. Strings retain their exact spelling.
The existing CLI parser is consumed, with an additional control-character rejection
that retains its no-positional-fallthrough behavior.

`App::send_destination` is the common effective value for dispatch and the Send
footer/help label. A private launch override is set by `ready_app` and the blocked
constructor; CLI wins over the validated file through startup, rereads and recovery.
Config-only destinations update on reread and disappear when unset. Bare constructors
keep None, preserving the existing workspace Send/picker behavior and exact labels.

`herdr::SendTo::{Cockpit, Name, Pane}` parses the documented alphanumeric
`^w[0-9A-Za-z]+:p[0-9A-Za-z]+$` pane spelling; `cockpit` is reserved and all other
strings are exact agent names. `send_target_for` enumerates once per attempt across
all workspaces, excludes self/non-agents, deduplicates matching pane ids, and requires
exactly one pane. Cockpit matches its name OR the data root's `cockpit` cwd, using
`briain::data_root` and lexical Path comparison. No hardcoded operator path or cwd
canonicalization was added. Name/pane requests do not depend on the briain data root.
Unavailable data-root resolution refuses cockpit visibly.

Resolution failures and ambiguity name the requested target and clipboard fallback;
ambiguity includes each distinct matching pane id. Explicit delivery failure gets
the same target/fallback treatment. Nothing falls back to workspace candidates.
Empty Send returns before any herdr call. Successful resolution reuses the existing
`export_to_agent` / `Agent` export: one bracketed paste, embedded terminator stripping,
no Enter, best-effort focus, and consumption only after successful paste. Cockpit
success names cockpit even when its agent is unnamed. No export formatter or
ReviewIdentity consumer was added.

**Comments survive:** tests compare both retained comments after lookup and delivery
failures; a successful paste consumes once even when focus fails. **Continuity:** no
review place-state reconciliation changes. **No writes:** no git path changed and no
operator config was edited. `send_target`, `candidates`, turn sampling, the stock
picker rows, `format_all`, and upstream exact assertions remain intact.

## Verification

Targeted checks passed before the full gate:

- `cargo test --lib config::tests`: 27 passed.
- `cargo test --lib herdr::tests`: 18 passed.
- `cargo test --lib app::tests`: 19 passed.
- `cargo test --lib send_destination_cli_wins`: 1 passed.
- `cargo test --test send_flow`: 2 passed.
- `cargo test --test pick_config`: 5 passed.
- `cargo test --test render explicit_send_destination`: 1 passed.
- `cargo test --test app_flow`: 264 passed.

The targeted batch took 13.811 seconds. Its log is temporary at
`/tmp/reviewr-s4-targeted.log`. Pure resolver fixtures cover cwd/name cockpit matches,
deduplication, exact names/panes, other workspaces, self/non-agent exclusion, absence
and ambiguity. Recovery tests cover both CLI and config-only destinations; TestBackend
checks the same getter in collapsed and expanded footer/help. The fake-herdr test
checks exact invocation bytes, one lookup, paste framing/embedded terminator removal,
no Enter/retry, malformed/failed enumeration, absent/ambiguous lookup, failed delivery,
empty-store no-op, focus-failure consume-once, and config-only reread dispatch.

Full gate: `RUSTFLAGS='-D warnings' just ci`.

**Exit 0; 547.712 seconds = 9m 7.712s.** Python `time.monotonic()` measured
from `2026-09-09T10:20:12.407724+00:00`. Release compilation reported **8m 04s**.
**866 passed, 0 failed, 1 ignored**: 332 unit, 264 app_flow, 11 briain_registry,
59 git_repo, 29 pane_actions, 9 pick, 5 pick_config, 24 pr_candidates, 131 render,
2 send_flow. The existing pr_live test remains ignored; binary/doc harnesses contain
zero tests. This adds 9 tests to sitting 3's 857. Temporary logs/timing:
`/tmp/reviewr-s4-ci.{log,json}`; first attempt:
`/tmp/reviewr-s4-ci-attempt1.{log,json}`.

The first gate attempt failed at Clippy after 16.801 seconds with three findings in
new code (`single_match_else`, `assigning_clones`, `format_push_string`). All were
fixed without suppression and the entire gate was restarted. Earlier development
iterations corrected an accidental validation-call argument, a test import, and a
render fixture that needed a comment because stock Send is hidden with an empty store.
No assertion was weakened or removed.

No PTY latency capture: the UI change only extends the Send action label; git,
reload, diff/highlight and render/layout algorithms are untouched. The committed
latency baseline is unchanged. Shared terminal modes/editor paths are unchanged, so
`just smoke-edit` was not required.

## Departures and exact next step: sitting 5

No ticket-scope departure. CLI control validation was added alongside the file-schema
rule to keep both target displays single-line. The effective destination is a getter
over the launch override and current validated config, rather than a second cached
resolved target; resolution stays exclusively on each non-empty Send attempt.
One feature commit was sufficient; no preceding schema-only commit was needed.

Sitting 5 starts by reading the spec and tickets 3–4. Implement them as two separate
feature commits, in this order:

1. Ticket 3: pure `ReviewHeader` and `format_batch` around unchanged `format_all`,
   consumed by the single `App::export` path for comment Send and Copy. Use the
   existing session `ReviewIdentity`; ordinary sessions retain exact body-only bytes
   and selected-text copying remains literal. Emit the specified one-line header with
   actual branch, resolved `branch_base.winner`, seven-character OID, missing-value
   placeholders and actual exported count; sanitize delimiters/controls. Do not build
   a diff at export time to obtain missing base context. Test copy/send parity,
   framing, no-identity byte parity and empty/failure behavior.
   Commit: `feat(export): one-line review header on send and copy`.
2. Ticket 4: add the plugin `pick` action and `herdr/pane.sh` dispatch, including safe
   environment-to-argv mapping for already-supported flags. Validate config first;
   pick alone bypasses the launch-cwd repo guard and opens a new split beside the
   focused live agent with explicit target/focus/env args. Preserve plugin identity,
   lifecycle hooks and legacy open/toggle/close behavior. Fake-herdr shell tests only.
   Commit: `feat(plugin): pick action opening a split beside the focused pane`.

Run each ticket's targeted tests and the required full gate; update spec/changelog
progress with measured results. README/rebase work and live operator acceptance remain
sitting 6. Do not add either sitting 5 feature to the Send commit.

## Operator boundary and final status

No subagent or detached child was launched. Builds/tests and fake-herdr subprocesses
were supervised in this session. No real herdr pane/tab/send command, plugin link,
install, pane reopen or agent message was performed. No reviewr/herdr machine config
was edited. The operator rebuilds the linked primary checkout after merge:
`/home/brian-isaac/Documents/personal/herdr-reviewr`.

All authored work is committed. The sole final porcelain exception is the expected
` M AGENTS.md`, which was neither edited, staged nor committed.

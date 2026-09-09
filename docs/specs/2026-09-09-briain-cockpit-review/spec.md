# Briain cockpit review flow

Status: Tickets 1–4 implemented; Ticket 5 documentation/rehearsal complete; quiet-machine latency retake and post-merge operator acceptance pending
Date: 2026-09-09

## Implementation progress

- Sitting 1: specification and setup completed; no runtime features.
- Sitting 2: `src/briain.rs` now discovers projects and retained worktrees from injectable
  data roots, with bounded scalar parsing, availability reasons, duplicate refusal and
  sanitized real-layout fixtures. Missing run metadata stays unknown. Git-only helpers
  report actual HEAD branch and choose local main, local master, configured tracking ref,
  or None without changing stock base resolution or writing private picks.
- Sitting 3: explicit parser arms reserve all four new flags without positional fallthrough.
  The startup picker resolves canonical checkout/root, launch base, scope and review identity
  before App construction, baseline seeding or workers. Config repair and directory fallback
  preserve that boundary; later recovery carries the selected session. `--send-to` is parsed
  and retained only; routing remains ticket 2.
- Sitting 4: optional whole-file `send_to` and the parsed `--send-to` now drive explicit
  cross-workspace routing. CLI precedence survives rereads/recovery; the effective destination
  also labels Send. Each attempt requires one exact target and refuses visibly with clipboard
  fallback on failure or ambiguity. Existing paste/consume behavior and stock routing remain.
- Sitting 5: selected sessions now export one contextual header through the shared comment
  Send/Copy path; ordinary sessions retain body-only bytes and text selections stay literal.
  The additive plugin `pick` action opens an independent focused split beside the focused
  live agent, with quoted environment-to-argv mapping and fake-herdr coverage only.
- Verification and measured latency: [sitting 2 handover](../../plans/progress/cockpit-fork-sitting-2-handover.md)
  and [sitting 3 handover](../../plans/progress/cockpit-fork-sitting-3-handover.md).

### Ticket 1 acceptance progress

These mirror Ticket 1's acceptance criteria without revising the existing plan.

- [x] Registry fixtures cover quoted/plain/null paths, description colons and embedded keys,
  body keys, inactive/malformed/duplicate projects, missing registry and traversal ids.
- [x] Worktrees are enumerated first; stray files are ignored; metadata is joined from the
  specified files; unknown metadata stays unknown and pruned worktrees remain unavailable.
- [x] Base tests cover the complete local-main/master/tracking chain, divergent refs,
  detached/unborn HEAD and missing rungs; None preserves stock fallback and private refs.
- [x] New flags cannot fall through to positional paths; explicit valid ids bypass input;
  invalid lookups refuse visibly; ordinary repository paths retain stock base/scope/identity.
- [x] Non-git startup chooses the root before App/workers; cancellation restores terminal
  modes without starting review workers; config recovery retains selection and CLI overrides.
- [x] TestBackend covers narrow, tall, empty and unavailable picker frames; existing App
  non-repo behavior and exact upstream fixtures remain unchanged.
- [ ] Operator-only live pane acceptance: deferred until after merge and primary-checkout rebuild. No installed plugin was rebuilt
  or pane opened during sitting 3.

### Ticket 2 acceptance progress

These mirror Ticket 2's acceptance criteria without revising the existing plan.

- [x] `send_to` validates as one non-empty string; unknown/wrong-type/empty/control values
  block the whole file. The 12-key allowlist, default/getter and normalized JSON stay in sync.
- [x] CLI override survives startup, config reread and recovery; absence preserves
  `send_target`, `candidates`, existing picker rows and turn sampling.
- [x] Pure fixtures prove cockpit by cwd or name, deduplication, exact name/pane matches,
  other workspaces, self/non-agent exclusion, absent and ambiguous targets.
- [x] Explicit failures name the target and clipboard fallback, never choose another agent,
  and retain every comment. Empty-store Send calls no herdr.
- [x] Fake-herdr `send_flow` proves one paste to cockpit, no Enter, correct framing and
  embedded-terminator stripping, failed enumeration/send retention, and focus-failure
  consume-once behavior.
- [x] App/recovery/display tests use the same effective destination as dispatch. Full
  `RUSTFLAGS='-D warnings' just ci` passed: 866 passed, 0 failed, 1 ignored in 547.712 seconds;
  upstream exact assertions remain intact. See the [sitting 4 handover](../../plans/progress/cockpit-fork-sitting-4-handover.md).
- [ ] Operator-only live cockpit acceptance remains deferred until after merge and primary-checkout rebuild. No real send,
  plugin install/link, pane action or machine-config edit occurred in sitting 4.

### Ticket 3 acceptance progress

These mirror Ticket 3's acceptance criteria without revising the existing plan.

- [x] Pure tests pin the exact one-line shape, seven-character base OID, missing-value
  placeholders, actual exported count, delimiter/control sanitization and no-identity parity.
- [x] FakeTarget captures identical comment Copy/Send payloads; failed export retains all
  comments, successful export consumes once, empty export calls no target, and selected-text
  copying stays literal in an identity-bearing session.
- [x] `tests/send_flow.rs` frames the header and body together once, strips embedded paste
  terminators, sends no Enter and never retries after successful delivery, including focus failure.
- [x] Header context comes from the selected session, actual HEAD branch and existing resolved
  `branch_base.winner`; a missing base stays missing without an export-time diff build.
- [x] Existing body-only and bare-session exact assertions remain unchanged; all 11 targeted
  export/App/Send tests passed. Full `RUSTFLAGS='-D warnings' just ci` passed: 874 passed,
  0 failed, 1 ignored in 558.416 seconds, including formatting, Clippy and release build.

### Ticket 4 acceptance progress

These mirror Ticket 4's acceptance criteria without revising the existing plan.

- [x] Whole-file config validation precedes actions. Pick alone bypasses the repository cwd
  guard; legacy action dispatch and its gates remain unchanged.
- [x] Pick prefers the context's focused agent, verifies its live process group, forces the
  configured split direction, focuses the new pane and passes explicit selector environment.
  Missing/non-agent/dead/unreadable targets refuse without opening elsewhere.
- [x] Existing reviews stay intact while repeated picks open independent sessions. Fake-herdr
  tests cover multiple reviews, quoted cwd/id arguments, directory fallback and legacy actions.
- [x] Parsed manifest tests retain identity/version/entrypoint/hooks and add `pick`; executing
  its pane command against a fake binary proves safe expansion to supported flags. Explicit
  project/run selectors suppress the picker hint; picker launches remain recognizable review UI.
- [x] `bash -n herdr/pane.sh`, all 34 `pane_actions` tests and the full `-D warnings` gate
  passed. No real herdr pane action was invoked. Measured outcomes and the exact sitting 6
  scope are in the [sitting 5 handover](../../plans/progress/cockpit-fork-sitting-5-handover.md).

### Ticket 5 acceptance progress

These mirror Ticket 5 without editing the historical plan. The sitting 6 instruction
limits rehearsal to targeted tests and runs one final full gate on this branch;
live acceptance belongs to the operator after merge/rebuild.

- [x] README covers registry and retained runs, selectors, base/scope/B policy, explicit
  Send precedence/refusals, contextual header/placeholders, literal selection copying,
  no automatic Enter and multi-review toggle/close behavior.
- [x] Operator `persiyanov.reviewr.pick` binding documents `[keys] prefix = "ctrl+a"`
  and `key = "prefix+d"`, verified against installed herdr 0.7.5's `--default-config`.
  No operator keybinding/config was edited; the manifest does not install the binding.
- [x] README and `docs/fork.md` document permanent-primary-checkout install/link/reopen,
  executable-link verification and the upstream-download limitation.
- [x] Rebase recipe records adopted boundary `4c09022`, clean prerequisite, fetch,
  dated backup, pinned upstream replay, abort/recovery, range-diff, coupled file
  surfaces, gate and operator rebuild. No rewritten-history push is implicit.
- [x] Detached throwaway replay against fetched upstream/main completed without conflicts;
  all 19 patches matched and final trees were identical. Upstream still equals the
  adopted boundary, so this does not test future upstream overlaps. All 850 targeted
  tests and shell syntax passed there; the clean throwaway was removed. Final source
  `RUSTFLAGS='-D warnings' just ci` passed: 874 passed, 1 ignored, 505.300 seconds.
  An initial stale cross-checkout test-cache failure was fixed by cleaning this
  package's artifacts and rerunning the full gate; no test/source edits were needed.
- [ ] Quiet-machine latency retake: another active regeneration/drawing job was present
  at all three checks. The baseline remains untouched; no sitting 6 medians or
  `chore(bench)` commit are claimed. See the handover for loads and old reference values.
- [ ] Operator acceptance after merge/rebuild: Ctrl+A then d → astraweave → comment → s,
  correct checkout/adjacent split, header/body in cockpit input without Enter, Copy
  parity, retained run, absent/ambiguous target refusals and comment survival.
  **None of these live checks is claimed passed by sitting 6.**

See [rebase recipe/rehearsal](../../fork.md) and the
[sitting 6 handover](../../plans/progress/cockpit-fork-sitting-6-handover.md).

## Problem

The operator's flow, verbatim: "ctrl a then d -> let it read the registered projects -> select a project -> reviewr opens the pane for that path and then anything we comment etc goes back to cockpit which is our main orchestrator agent."

Upstream v0.36.2 starts from a positional path or cwd. The cockpit cwd is not a repository. Send chooses agents in the review pane's workspace, and exported comments do not name the project or run. This fork adds five separable features, distributed through a linked local build.

Sources: [Claude implementer map](../../research/cockpit-fork-surfaces-claude-2026-09-09.md), especially sections 4 and 13, and [Codex implementer map](../../research/cockpit-fork-surfaces-codex-2026-09-09.md). Where they differ, this contract follows the Claude map and the operator's six-sitting sequence.

## Proposal

### 1. Project and run selection before startup

Use design **(A)**: resolve launch intent to repository root, base, scope and review identity before `app_for`, `App::new`, baseline seeding, reload or `TurnHost::open`. Implement discovery in `src/briain.rs` and a small terminal picker in `src/pick.rs`. Never add project selection to a running App's `Mode` or change its repository after construction.

`--project <id>` selects a registered project; `--run <id>` selects a retained run; `--pick` opens a two-level projects/runs menu. Selectors take required non-empty ids: bare `--project` and bare `--run` are errors, not alternate picker spellings. Reject conflicting selectors and selector-plus-positional-path combinations visibly. An explicit invalid id reports the reason and returns to the picker, never silently selects a different checkout. With no selector, an ordinary repository launch preserves upstream behavior; a non-repository launch enters the picker. Esc cancels startup with exit 0 and restores terminal modes. Arrow/keymap movement, Enter, digits and mouse selection follow existing menu conventions; invalid digits do nothing. Unavailable rows are dim, inert and explain why.

Paint startup before slow discovery or the bounded herdr config-directory fallback. Whole-file config validation gates review work. Carry the resolved selection in launch configuration through fallback and config recovery, so neither can revert to the cockpit cwd or re-resolve an edited project note. Reuse terminal ownership and restoration; no worker starts on the placeholder cwd.

Read `BRIAIN_DATA_DIR`, otherwise `$HOME/.local/share/briain` on both Linux and macOS. Read opening frontmatter from `notes/projects/*.md`: `id`, `status`, `working_dir`; active projects need a usable checkout. Tolerate unrelated fields, quoted scalars, null paths and descriptions containing colons. A bounded scalar scanner reads only the opening block; no new YAML dependency. Reject unsupported/malformed relevant values visibly rather than guessing. Ignore Markdown body keys. Duplicate ids cannot resolve arbitrarily.

Walk directories under `worktrees/`, not thousands of historical `runs/` entries. Join by exact basename to `runs/<id>/status.json` for lifecycle state/phase and `job.yaml` for project association. Status JSON does not contain the project. Never infer the actual branch or project from a job id. Ignore stray zip/log files; retain unavailable rows for broken/pruned git links. Missing metadata stays unknown and cannot select another project; a valid checkout may still be reviewed with absent project/state. Explicit ids must be basenames, not traversal paths. Validate the root again on Enter. Fixtures use the real layout, including git-file linked worktrees.

For briain launches, an explicit `--base` retains priority. Otherwise choose the first resolvable commit ref: local `refs/heads/main`, local `refs/heads/master`, then the selected branch's configured `@{upstream}`. Here upstream means the tracking branch, not a remote literally named upstream. If none resolves, leave `base = None`, preserving upstream's private pick then `origin/HEAD` fallback. Keep all git subprocesses in `src/git.rs`; no global rewrite of the stock base resolver. Pass the full ref spelling through `cfg.base`, using existing dynamic flag semantics rather than introducing session OID pinning. A supplied/resolved base disables `B` as `--base` already does. Runs start in Branch; projects retain configured `default_scope`. Branch includes committed work and WIP. Last turn continues sampling agents belonging to the selected worktree, independently of Send routing.

**Parser fallthrough trap:** `Config::parse` ignores unknown flags and assigns their next non-flag token as the repository path (the last positional assignment wins). Today `--project astraweave` can mean repo `astraweave`. Land explicit parser arms for every new flag before any caller/manifest emits it, with a negative regression proving its value never becomes `repo`. Missing values must not consume a following flag. Keep unrelated legacy parsing behavior intact.

### 2. Explicit Send destination

Add optional non-empty, single-line `send_to` to the whole-file config schema, defaults, accessor, allowed-key list and normalized JSON. Add `--send-to <target>`; CLI wins over config, including rereads and recovery. Absence preserves the existing workspace one/many/no-agent behavior. Resolve on each Send, not in rendering; show the effective destination in help/footer.

Add `SendTo::{Cockpit, Name, Pane}` and `send_target_for` beside the unchanged `send_target`/`candidates`. Explicit lookup searches all workspaces and excludes self and non-agents. A pane id matches exactly; other names match the exact agent name. Reserved `cockpit` matches name `cockpit` OR cwd equal to the data root's `cockpit` directory, deduplicated by pane id. Require exactly one match. Zero, multiple or failed lookup produces a visible footer refusal naming the target (and ambiguous pane ids) and clipboard fallback, retaining every comment. Never fall back to the focused agent, first result, last-sent pane or generic picker.

Use existing `pane send-text` bracketed paste and best-effort focus; do not press Enter. Delivery failure keeps comments. Successful paste consumes once even if focus fails. Resolve once per attempt; do not retry against a replacement target. Cockpit success names `cockpit`. The operator will eventually set `send_to = "cockpit"`; sitting 1 does not edit their config.

### 3. One-line review header on Send and Copy

Add `ReviewIdentity { project, run }` to the selected session, preserved through recovery. Add a pure `ReviewHeader` and `format_batch` wrapper around unchanged `format_all`. Both comment Send and comment Copy use the one `App::export` path. Only sessions with a briain identity get the header; ordinary path sessions keep upstream's exact bytes. Selected-text copying remains literal.

Shape:

```text
review: <project> | <run> | <branch> | <base>@<short oid> | <n> comments

<existing comment blocks>
```

Use `-` for absent project, run or branch (including detached/unborn), and `-@-` for unavailable base. Use a consistent seven-character base OID abbreviation. Count actual exported comments. Read actual branch and resolved `branch_base.winner`, never invent branch from the run id; when no branch-scope result exists, print missing base rather than forcing a diff build. The header is export/session context, not historical provenance for every surviving comment. Sanitize metadata delimiters and controls so the header has exactly one line and no ESC; retain comment formatting and bracketed-paste terminator stripping. Frame the entire header and body together. Empty export remains a no-op.

### 4. Plugin `pick` action

Keep plugin id `persiyanov.reviewr`, version/minimum version, existing actions and lifecycle hooks. Add action `pick` invoking `bash herdr/pane.sh pick`. The existing pane entrypoint uses its absolute binary path and safely translates `REVIEWR_PICK`, `REVIEWR_PROJECT`, `REVIEWR_RUN`, `REVIEWR_SEND_TO` into supported flags (or corresponding `from_env` values); no argv passthrough is assumed in `plugin pane open`. Keep argument boundaries through quoted shell argv construction. `parse` stays pure; CLI takes precedence over environment values. An explicit project/run wins over the action's picker hint.

Validate config first. `pick` bypasses only the launch-cwd repository guard, permitting the cockpit cwd; use an existing live/context directory or HOME. Force a split in configured direction beside the focused agent pane, preferring `focused_pane_id` from action context. Refuse missing/non-agent/dead targets, rather than falling back to an arbitrary first pane. Pass `--target-pane`, `--focus`, and `--env REVIEWR_PICK=1`; optional `pane.sh pick <id>` carries `REVIEWR_PROJECT`. Require the returned pane id before success.

Following the Claude design, `pick` opens a new independent review even if another reviewr exists. It never closes or retargets an existing pane. Generic open/toggle/close behavior stays unchanged, including their workspace sweep; document that toggle/close can close multiple review panes. Picker argv still identifies a review UI process. Split placement does not determine Send routing.

### 5. README and rebase recipe

Document registry layout, selectors, base/scope policy, explicit target precedence/refusals, header semantics, retained-run limitations, manual submission and the linked build loop. Document `Ctrl+A`, then `d` as an operator binding to `persiyanov.reviewr.pick`, not a manifest default. Verify installed prefix syntax before publishing the snippet. Keybinding/config edits and real pane opens are operator actions during later acceptance.

Link only `/home/brian-isaac/Documents/personal/herdr-reviewr`, never a disposable worktree: uninstall the GitHub plugin, `just install`, `herdr plugin link .`, verify list and stable executable links. Rebuild with `just install`, then the operator closes/reopens panes. `herdr/install.sh` still downloads upstream assets: GitHub installation of this fork does not distribute the fork features.

Keep five feature commits separable after this spec. The README recipe must require a clean fork branch, fetch upstream, create a dated backup branch, rebase onto upstream/main from the recorded adopted upstream boundary (initially `4c09022`), inspect `git range-diff`, run `just ci`, rebuild in the primary checkout and repeat acceptance. Explain `git rebase --abort`, update the adopted boundary after success, and do not auto-push rewritten shared history. Review config keys/parser, startup/recovery, shell dispatch/manifest, export shape, README tables and changelog conflicts explicitly.

## Invariants

- **No writes:** registry discovery and launch base selection are read-only. No checkout, staging, branch mutation, worktree creation or base-pick write. Existing git writes remain private refs under `refs/worktree/reviewr/`.
- **Comments survive:** comments remain in memory and survive refresh/recovery and failed resolution/delivery; consume only after explicit successful export. No persistence added.
- **Continuity:** selection finishes before review place state exists. World events reconcile by identity; a running App never switches repositories.
- **Additive-only:** preserve stock `format_all`, `candidates`, App constructors/fixtures and direct-path behavior. New behavior is gated by launch identity or explicit target. Keep upstream exact-text and agent-row assertions; extend normalized-JSON schema and key count together rather than weakening exact-shape tests.

Enforcement: registry/base fixtures and private-ref tests; parser negative and startup recovery tests; explicit Send failure/consume tests; header/no-identity byte parity and framing tests; shell target/config/legacy action tests. The plan maps these to implementation files.

## Alternatives

Design (B), a project picker inside a running App, would require retargeting tab stashes, caches, frozen drafts, search/world workers and private-ref ownership. It expands rebase risk and violates the simple one-App/one-worktree ownership boundary. Rejected. Also rejected: bare optional-value selectors, a wrapper popup, global Send candidate changes, global base-chain changes, always-on headers, a dedicated second pane entrypoint, and GitHub fork distribution in this series.

## Out of scope

Product code in sitting 1; comment persistence; an in-session repository switch; JSON export protocol; automatic submission; registry writes; release publishing/version bumps; changing plugin identity; automatic pane reopen; edits to either operator config file; unrelated upstream fixes.

## Open questions

None blocking the six-sitting implementation. Live keybinding syntax and end-to-end acceptance remain verification work, not completed claims.

## Acceptance

In sitting 6, with the operator's binding and `send_to = "cockpit"` configured: focus the agent, press **ctrl+a d**, pick **astraweave**, verify its checkout appears in an adjacent split, add a line comment, press **s**, and verify the header and comment land in the cockpit input without Enter. Copy produces the same batch. Repeat with a retained run; remove the target and verify visible refusal and retained comments. Sitting 1 does not run this acceptance or open panes.

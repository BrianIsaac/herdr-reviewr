# Briain cockpit review flow

Status: Specified for implementation; no features shipped in sitting 1
Date: 2026-09-09

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

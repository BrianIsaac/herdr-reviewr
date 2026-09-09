# Briain cockpit review flow: Plan

Delivers the sibling [spec.md](spec.md). Date: 2026-09-09.

## Problem

The cockpit launches from a non-repository directory. Reviewr currently accepts a path, sends to workspace candidates and exports no project/run context. Five additive features must join that flow without changing upstream review ownership.

## Goal

Ctrl+a d → pick astraweave → review its checkout → comment → s → a contextual batch in the cockpit input. Design (A) resolves selection before App construction. Six sittings follow Claude map section 13; the five feature boundaries remain separate commits, even though the picker spans sittings 2–3.

## Ticket Map

1. Project/run discovery, base chain, parser and startup picker. Blocked by: sitting 1 setup/spec.
2. `send_to` and `--send-to` routing. Blocked by: ticket 1 data root/launch configuration.
3. Shared review header. Blocked by: ticket 1 identity and ticket 2 delivery verification.
4. Plugin pick split. Blocked by: supported parser/startup/routing flags in tickets 1–2.
5. README, rebase recipe and operator acceptance. Blocked by: tickets 1–4.

As in the existing spec folders, tickets are sections of this plan rather than duplicate files in `tickets/`.

## Sitting 1: setup and specification

**Status:** documentation/setup sitting; measured outcomes live in [the handover](../../plans/progress/cockpit-fork-sitting-1-handover.md).

- [x] Install home-only rustup, pinned 1.97.0/rustfmt/clippy and just; record versions.
- [x] Time the first `just ci` on clean primary main before implementation; retain exact failure without fixing upstream.
- [x] Uninstall downloaded plugin; run `just install` and link primary checkout; verify registry and executable links without pane opens/config edits.
- [x] Write spec/plan covering all five features and add an explicitly planned Unreleased entry.
- [x] Commit `docs(spec): briain cockpit review flow`, then commit the versions/gate/link/next-step handover.

## Ticket 1: select projects and retained runs before review startup

**What to build:** `src/briain.rs` registry module, additive helpers in `src/git.rs`, `src/pick.rs` startup menu, CLI/environment intent in `src/config.rs`, pre-App resolution in `src/lib.rs`, session identity in `src/app.rs`.

**Blocked by:** sitting 1. **Status:** pending; sittings 2 and 3.

### Implementation plan

1. Sitting 2 begins by rereading spec and handover, then inspecting real `notes/projects/*.md`, retained `worktrees/` entries and matching `runs/<id>/{status.json,job.yaml}` read-only. Build fixture copies of the relevant shapes, not tests depending on live mutable state. Implement `Project`, `Run`, availability, data-root resolution and scalar parsing in `src/briain.rs`; register the module in `src/lib.rs`.
2. Add git-only helpers in `src/git.rs` for actual HEAD branch and the launch base chain: `refs/heads/main`, `refs/heads/master`, configured tracking ref, None. Do not change stock `resolve_base`. Use `tests/common/mod.rs` real repositories/linked worktrees and divergent local/remote refs. Keep explicit base precedence in launch orchestration.
3. Sitting 3 starts with explicit parser arms and tests for `--project`, `--run`, `--pick` before wiring any caller. Reserve/parse `--send-to` before its later consumer. Prove with a negative test that `--project astraweave` cannot assign `astraweave` to repo; repeat for other valued flags, missing values and next-flag boundaries. Reject conflicts without changing unrelated stock parsing.
4. Implement pure menu state and ratatui rendering in `src/pick.rs`: projects/runs levels, movement, digits, Enter, Esc, mouse, unavailable reason, identity reconciliation on rescan. Copy visual conventions from current menu helpers, not Send's destructive Enter behavior.
5. In `run`, paint startup and resolve config/selection before `app_for`. Canonicalize and revalidate selection. Carry root/base/scope/identity through CLI config-directory fallback and recovery. Start the world worker only with the chosen root. Run scope is Branch; project scope is configured default. No mode in a running App and no late repo mutation.

### Acceptance criteria

- [ ] Fixtures cover quoted/plain/null paths, colons and embedded `id:` in descriptions, body keys, inactive/malformed/duplicate projects, missing registry and traversal ids.
- [ ] Enumerate worktrees first, ignore stray files, join project from job.yaml and lifecycle from status.json; missing metadata stays unknown, pruned worktree is visibly unavailable.
- [ ] Base tests cover main before master/tracking, divergent origin/main, master-only, tracking-only, missing/dangling upstream, detached/unborn and no rung. None preserves stock fallback and private refs remain untouched.
- [ ] Flags cannot fall through to positional path; explicit ids bypass menus; invalid lookup refuses visibly; ordinary paths retain upstream behavior.
- [ ] Non-git startup selects the correct root before App/worker creation. Cancellation restores terminal and starts no review worker. Recovery retains resolved selection and CLI overrides.
- [ ] `TestBackend` covers narrow/tall picker frames and unavailable rows. Existing App non-repo unit behavior and exact upstream fixtures remain unchanged.

### Verification

Run targeted registry/config/picker tests, `cargo test --test git_repo`, new startup-flow tests and render tests, then `just ci`. Since git/render paths change, capture before/after PTY latency with the old binary in a second target directory and interleave same-load runs. Keep one committed baseline, updating it only if numbers move. Run `just smoke-edit` if shared terminal modes change. Later live checks are operator-controlled; never script pane opens.

Commit boundary: `feat(pick): project and run picker with --project, --run and --pick`. If sitting 2 needs an intermediate commit for handoff, keep it confined to registry/base foundations so ticket 1 remains a separable feature series. Update the spec progress and changelog with actual delivered behavior.

## Ticket 2: explicit cockpit Send

**What to build:** schema/default/getter/JSON and CLI precedence in `src/config.rs`; `SendTo` and `send_target_for` in `src/herdr.rs`; effective target, dispatch and recovery in `src/app.rs`/`src/lib.rs`; destination display in `src/ui.rs`.

**Blocked by:** ticket 1. **Status:** pending; sitting 4.

### Acceptance criteria

- [ ] `send_to` validates as one non-empty string; unknown/wrong-type/empty/control values block the whole file. Allowed key count and JSON remain in sync.
- [ ] CLI override survives config reread/recovery; absence preserves `send_target`, `candidates`, existing picker rows and turn sampling.
- [ ] Pure fixtures prove cockpit by cwd or name, deduplication, name/pane exact matches, other workspaces, self/non-agent exclusion, absent and ambiguous target.
- [ ] Explicit failures display the target and clipboard fallback, never choose another agent, and retain comments. Empty-store Send calls no herdr.
- [ ] `tests/send_flow.rs` fake herdr proves one paste to cockpit, no Enter, correct framing, failed enumeration/send retention, and focus-failure consume-once behavior.
- [ ] App/recovery/display tests use the same effective destination as dispatch. `just ci` passes with upstream exact-shape assertions retained.

Commit: `feat(send): send_to config key and --send-to flag` plus applicable changelog/spec progress.

## Ticket 3: contextual batch export

**What to build:** `ReviewHeader`/`format_batch` in `src/export.rs`; one identity-gated wrapper at `App::export`; branch lookup helper from ticket 1. Preserve `format_all`, comment anchors, sorting and `ExportTarget`.

**Blocked by:** tickets 1–2. **Status:** pending; sitting 5, first commit.

### Acceptance criteria

- [ ] Exact one-line shape, seven-character base OID, missing values, actual count, delimiter/control sanitization and no header without identity have pure tests.
- [ ] Send and comment Copy receive identical payloads through `FakeTarget`; selection copy stays literal. Empty export does nothing; failed export retains all comments.
- [ ] `tests/send_flow.rs` frames the whole header/body once and preserves embedded-terminator stripping; no Enter and no retry after successful delivery.
- [ ] Header describes session/export context, not each comment's historic snapshot. No export-time diff build to obtain an absent base.
- [ ] Existing body-only and bare-session exact assertions remain unchanged; targeted export/App/send tests and `just ci` pass.

Commit: `feat(export): one-line review header on send and copy` plus changelog/spec progress.

## Ticket 4: plugin pick split

**What to build:** additive `pick` action and safe environment-to-argv expansion in `herdr-plugin.toml`; dedicated pick dispatch in `herdr/pane.sh`; fake-herdr coverage in `tests/pane_actions.rs`.

**Blocked by:** tickets 1–2 parser and consumers. **Status:** pending; sitting 5, second commit.

### Acceptance criteria

- [ ] Config validates before actions; pick alone bypasses non-repository cwd rejection. Other modes retain all existing gates.
- [ ] Pick forces split beside the focused agent from context, respects direction, focuses the new pane and passes supported env selectors. Missing/non-agent/dead target refuses without opening elsewhere.
- [ ] An existing review stays intact while pick opens an independent session; no toggle-style close branch. Shell tests cover multiple sessions, quoted cwd/id arguments and normal legacy actions.
- [ ] Parsed manifest retains id/version/hooks and entrypoint, adds pick, and emits only already-supported flags. UI process identity still recognizes picker launches.
- [ ] `bash -n herdr/pane.sh`, `cargo test --test pane_actions`, `just ci` pass. No real pane action is scripted against the operator's workspace.

Commit: `feat(plugin): pick action opening a split beside the focused pane` plus changelog/spec progress.

## Ticket 5: README, rebase and acceptance

**What to build:** README cockpit section/flag/config tables; `docs/fork.md` if a separate rebase guide improves readability. Consolidate changelog only for implemented features.

**Blocked by:** tickets 1–4. **Status:** pending; sitting 6.

### Acceptance criteria

- [ ] README describes registry and retained runs, selectors, base/scope/B policy, Send precedence/refusals, header missing values/context, no automatic Enter and multi-pane close behavior.
- [ ] Document operator binding `persiyanov.reviewr.pick` after verifying actual prefix syntax. Do not claim it is installed by the manifest.
- [ ] Document primary-checkout `just install`/link/reopen loop and upstream download limitation; do not link a disposable worktree.
- [ ] Rebase recipe records initial upstream boundary `4c09022`, clean-status prerequisite, fetch, dated backup, replay, abort, range-diff, coupled conflict surfaces, gate and install. No implicit rewritten-history push.
- [ ] Rehearse rebase against upstream/main in an isolated checkout, inspect five feature boundaries and rerun `just ci`; rerun terminal/latency checks where changed paths require them.
- [ ] Operator acceptance: ctrl+a d → astraweave → comment → s → header/body in cockpit input. Verify correct checkout/split, Copy parity, retained-run selection, absent/ambiguous-target refusal and comment survival. Record actual evidence, not a predicted pass.

Commit: `docs(readme): briain cockpit section and the upstream rebase recipe`.

## Out of Scope

See spec. Sitting 1 adds no runtime behavior, changes no operator config and opens no pane. It records upstream gate failures without fixing them. No subagents and no changes to unrelated processes or AGENTS.md.

## Replan

- 2026-09-09: initial plan, follows Claude map section 13 and design (A).
- If live registry scalar shapes exceed the scanner contract, add a fixture and explicitly revise parsing scope before implementation; do not silently scrape arbitrary YAML.
- If a startup/config refactor would construct App on a placeholder or retarget workers, revise it to preserve design (A).
- If the untouched gate fails, record exact command/error/timing and carry it forward separately from fork acceptance; do not label the baseline green.

# Cockpit fork: implementation surfaces

Date: 9 September 2026. Research only; proposed additions below are not implemented. The chosen direction is a small fork, with five feature commits and explicit upstream rebases. The source baseline is `4c090225af706bf3aaa24b39fea890a72994f40f` (`v0.36.2`); package and manifest both declare `0.36.2` (`Cargo.toml:3`, `herdr-plugin.toml:3`). The operator's flow and fork decision are recorded in this job's `job.yaml:7–19`, under the run directory defined below.

## Sources and reading order

Repository references are relative to this checkout and use **current, pre-implementation line numbers**. Proposed files and symbols are explicitly marked **new**: their citations identify the existing seam, not imaginary implementation lines. `B/` means `/home/brian-isaac/Documents/personal/briain/`; `D/` means `/home/brian-isaac/.local/share/briain/`; `J/` means `D/runs/herdr-reviewr-research-job-no-product-20260909T050445-1786ff/`. External paths are local research evidence, not dependencies to embed in the fork (`J/job.yaml:1–21`, `D/notes/projects/herdr-reviewr.md:2–8`).

Read this map alongside `AGENTS.md:16–42`, `CONTRIBUTING.md:30–58`, and `docs/specs/README.md:1–8`. **No writes**, **Comments survive**, and **Continuity** are the named invariants: preserve private-ref-only git writes, in-memory comments, and identity-based reconciliation (`AGENTS.md:22–24`). The map surveys configuration, CLI/startup, App/UI, git/world, herdr/export/model, integration fixtures, packaging, all current `docs/` guides and both existing spec folders; the entry-point inventory is `AGENTS.md:28–42` and the relevant implementation is cited per feature below.

Prior maps read: `B/docs/research/reviewr-cockpit-integration-claude-2026-09-09.md` and `B/docs/research/reviewr-cockpit-integration-codex-2026-09-09.md`. Their earlier wrapper recommendations are superseded by this job's fork decision, not reopened here (`J/job.yaml:7`; prior Claude map:48–53; prior Codex map:7–9). Their central references were checked against this checkout: Send is still `src/app.rs:4356`, agent enumeration `src/herdr.rs:234`, candidacy `src/herdr.rs:391`, configuration keys `src/config.rs:69`, base resolution `src/git.rs:596`, and formatting `src/export.rs:32`.

Corrections to carry forward:

- The CLI comment says the first positional token wins; the loop actually assigns every non-flag token, so the last wins. Unknown flags are ignored, allowing an unrecognised flag's following value to become the repository path (`src/config.rs:28–52`). New selectors require deliberate parsing, not extra unchecked `it.next()` arms.
- The binary's non-repository path is an empty state, not a fatal rejection. The shell action is the rejecting boundary (`src/app.rs:1238–1251`, `tests/app_flow.rs:2541`, `herdr/pane.sh:236–249`). Preserve the existing direct-path test while adding picker behaviour at startup.
- Branch scope compares its merge-base with the live worktree and includes untracked work. The committed-only sketch in API notes is stale (`src/git.rs:1196–1209`, `README.md:180–187`, versus `docs/herdr-api-notes.md:214–217`).
- Send uses `pane send-text`, not the retired `agent send` named in some architecture prose. It wraps the entire batch in bracketed paste, then focuses without Enter (`src/herdr.rs:404–437`, `src/export.rs:141–147`, `docs/herdr-api-notes.md:210–212`).

## 1. Project and run selection before repository startup

### Existing ownership and required ordering

`Config::parse` currently yields a concrete `repo`, defaulting to cwd. `run` reads environment-resolved plugin configuration, calls `app_for`, initialises the terminal and paints; only then does the bounded herdr config-directory fallback run, potentially rebuilding the App. Next comes `reload`, followed by `event_loop` (`src/config.rs:13–65`, `src/lib.rs:72–141`). Even the first App construction touches git: `repo_root` resolves the top level and `App::new` seeds the persisted baseline (`src/lib.rs:340–375`, `src/app.rs:850–863`).

**Proposed order:** parse launch intent → validate plugin configuration and paint a startup/config view → discover and choose a project/run → validate the selected worktree and base → finalise the resolved launch configuration → construct the review App → reload → start repository workers. Keep config-directory fallback after a first paint, and prevent its rebuild from discarding a selection. Neither `ready_app`, baseline seeding, review reload nor worker ownership should run against the cockpit placeholder path (`src/lib.rs:79–136`, `src/lib.rs:359–375`, `src/lib.rs:889–907`).

`event_loop` starts `TurnHost::open(app.repo.clone())` once; search ownership is lazy, and config recovery reconstructs from a cloned `cfg`. Therefore changing only `app.repo` after entering the loop leaves worker ownership and recovery pointing at the previous path. Store the selected path, launch identity and base in the resolved configuration **before** `event_loop`; config recovery must reuse that selection without rereading a possibly edited registry entry (`src/lib.rs:893–907`, `src/lib.rs:920–927`, `src/lib.rs:1573–1583`).

### Exact change surfaces

| File / current seam | Proposed change |
| --- | --- |
| `src/config.rs:13`, `src/config.rs:30`, `src/config.rs:64` | Add **new** `LaunchSelection` with path/project/run/picker intent, preserving whether a path was explicit. Parse `--project <id>` and `--run <id>` before cwd fallback. Reject conflicting selectors, empty ids and missing required values visibly. Keep legacy flag behaviour outside the new selector combinations unless separately specified. |
| **New** `src/projects.rs`; module declarations at `src/lib.rs:11–36`; existing dependencies `Cargo.toml:14–31` | Isolate **new** `BriainPaths`, `ProjectEntry`, `RunEntry`, `SelectedReview`, `read_projects`, `read_runs`, `resolve_project`, `resolve_run`. Accept paths as function arguments so fixtures never touch the real registry. Keep discovery read-only. |
| **New** `src/startup.rs`; `src/lib.rs:72–141`, `src/lib.rs:347–375` | Add **new** `StartupState` / `StartupMode::{ProjectPick, RunPick}` and a small startup event loop returning a validated `SelectedReview` or cancellation. Own configuration errors and selector errors without constructing a working review App. Initialise/restore terminal modes once through the existing helpers. |
| `src/app.rs:598`, `src/app.rs:850`, `src/app.rs:1056` | Add session identity storage and initialise it through a setter or constructor wrapper; preserve the existing `App::new` interface for generic fixtures. Carry identity through config recovery alongside the comment store. Repository switching after comments exist is outside this small change. |
| `src/ui.rs:34`, `src/ui.rs:3002–3094` | Add **new** `render_startup`, project/run row renderers and matching hit tests. Reuse menu geometry/scroll conventions, but render from startup state rather than an App whose repository is not selected. |
| `src/git.rs:68`, `src/git.rs:838`, `src/git.rs:943` | Add **new** strict selected-worktree validation and `resolve_review_base`; keep git subprocesses here. Use `worktree_of` when distinguishing outside-git from an unavailable git command matters. Add branch-name lookup near `head_oid` (`src/git.rs:1250`). |
| `src/model.rs:120`, `src/model.rs:152`; `src/herdr.rs:372`; `src/export.rs:32` | Do not add registry access to comments, Send or turn sampling. A **new** session-level `ReviewContext` can live in `model.rs`; feature 3 consumes it. `agent_samples` continues using the selected worktree through `TurnHost`, independently of cockpit routing. |

**How the new picker fits the existing modes:** `Mode::Picker` already means the agent Send chooser, with `AgentChoice` rows, a last-sent cursor and a saved underlying mode. Its Enter sends and consumes comments; it must not become a repository selector (`src/app.rs:310–326`, `src/app.rs:4372–4416`, `src/herdr.rs:58–72`). A separate startup mode enum is the recommended boundary. If an implementer instead adds `Mode::ProjectPick` / `Mode::RunPick` to App, they must audit `Mode` modal classification (`src/app.rs:335–345`), recovery (`src/app.rs:1080–1084`), text editing (`src/app.rs:3296`), action availability (`src/app.rs:3602`), footer hints (`src/app.rs:4107`), renderer exhaustiveness (`src/ui.rs:82–88`), keyboard modal dispatch (`src/lib.rs:1740–1783`) and mouse dispatch (`src/lib.rs:2444`). That alternative still needs a pre-worker startup loop; an enum arm alone does not fix ordering.

### Registry contract

Read only the opening YAML frontmatter of `D/notes/projects/*.md`; active rows need a non-empty `id`, `status = active`, and usable `working_dir`. Briain's schema permits `working_dir = null`, and its record has other fields and a Markdown body: tolerate unrelated fields without scraping the body for keys (`B/src/briain/schema.py:108–124`, `D/notes/projects/herdr-reviewr.md:1–24`). Proposed cases: quoted scalars, spaces, Unicode, missing or malformed frontmatter, duplicate ids, null paths, paused/archived projects and a missing registry. Surface unavailable records without letting one bad note make every valid project disappear. Do not silently select the first row after an explicit id lookup fails.

Walk retained `D/worktrees/<job-id>/` entries and join each basename to `D/runs/<job-id>/status.json`. Read lifecycle metadata defensively: the current job's status has state, phase, backend, window target and harness, **no project id** (`J/status.json:1`). Join `runs/<job-id>/job.yaml` for project association; this job has `project: herdr-reviewr` at `J/job.yaml:20`. The dispatcher creates `worktrees/<job-id>`, conventionally branches `agent/<job-id>`, and copies `job.yaml` into the run directory (`B/src/briain/dispatcher/spawn.py:555–576`). Do not infer project or actual branch from the job-id string. A missing/malformed status, orphan checkout, missing project association or removed worktree should remain a visible unavailable row or explicit lookup failure, not turn into another project review.

Use git to validate the selected checkout: a linked worktree has a git-file representation, so requiring `.git` to be a directory is wrong. Reuse `Repo`'s linked-worktree fixtures and the private-ref isolation tests (`tests/git_repo.rs:805–842`, `src/git.rs:68–78`). Canonicalise the chosen root, retain full job id, and validate again on Enter to cover deletion between enumeration and selection. Restrict explicit ids to registry/basename lookup rather than joining arbitrary `../` input to the data root. These are proposed validation requirements derived from the directory join above (`J/job.yaml:1`, `B/src/briain/dispatcher/spawn.py:555–576`).

There is no YAML parser in the direct dependency list; `serde`, `serde_json` and TOML are present. Add a maintained YAML parser only after checking its licence/advisories, and commit its lockfile change with this feature; TOML parsing is not a substitute for YAML frontmatter (`Cargo.toml:14–31`, `deny.toml:4–39`). Keep the default data root at the operator's requested location behind `BriainPaths`; custom briain roots are a spec decision, not another guessed config key (`J/job.yaml:10`, `B/src/briain/cockpit/home.py:210–219`).

### Base and initial scope

The stock chain is flag → this worktree's private pick → `origin/HEAD`, skipping unresolved sources. Picker branch names resolve **origin before local**, whereas flags first resolve verbatim (`src/git.rs:592–636`, `src/git.rs:903–932`). Thus a picker row named `main` can select remote history instead of the local merge target. Do not globally replace this chain: it is an existing contract with regression coverage (`docs/specs/2026-08-28-worktree-private-refs/spec.md:40–48`, `tests/git_repo.rs:578–595`).

Add a launch-only resolver for briain selections: explicit `--base` if supplied and valid, otherwise **`refs/heads/main`, then `refs/heads/master`, then a defined upstream**. Proposed interpretation of the final rung: the selected branch's configured tracking ref, obtained via `for-each-ref --format=%(upstream)` and resolved as a full commit ref. This is neither the remote literally named `upstream` nor the existing `origin/HEAD` fallback. The spec must ratify that meaning; if the operator means the upstream remote's default branch, name that alternative explicitly. Do not reuse `recorded_upstream` unchanged: it strips the remote prefix and suppresses base candidates for PR-name discovery (`src/git.rs:943–967`). With no resolvable rung, show a useful no-base error and allow an explicit override; never manufacture a base from HEAD.

Recommended small implementation: return the source label/full ref and full OID, then pass the validated OID as the existing base flag for a briain session. Store the human label separately for display and export. This pins the reviewed base for the session and avoids silently following another ref when local main changes; record this pinning policy in the spec. Validate that pinned base remains resolvable before a new review build rather than allowing the stock skip chain to substitute a persisted pick after pruning (`src/git.rs:610–636`, `src/git.rs:838–843`). Direct-path sessions retain stock base semantics. Display the actual pinned name/OID instead of pretending the regular `B` picker remains available: a non-empty `App.base` disables it (`src/app.rs:4422–4423`, `src/ui.rs:1365–1373`).

Make briain project/run launches start in **Branch** as a proposed workflow default; ordinary positional launches retain `PluginConfig::default_scope`. Setting `--base` alone does not do this (`src/lib.rs:359–369`, `src/config.rs:173–181`, `src/lib.rs:3424–3448`). Branch includes committed work and WIP; the header's base OID names the base ref tip, not necessarily the merge-base actually used by git (`src/git.rs:1196–1209`). Preserve Last turn's worktree membership logic: the cockpit can receive comments without driving that worktree's turns (`src/herdr.rs:364–402`, `tests/app_flow.rs:2797–2866`).

### Tests to copy and add

- CLI unit pattern: `src/config.rs:663–685`. Add id selectors, bare picker invocation if adopted below, flag ordering, conflicts, missing values, and preservation of positional paths. Keep no-args defaults tested.
- Discovery: **new** `src/projects.rs` unit tests using `tempfile` (`Cargo.toml:30–31`) and fixture frontmatter/JSON/YAML; inject `BriainPaths`, never global environment mutations. Include an inactive project, a retained finished run, malformed joins and exact id lookup.
- Git: `tests/common/mod.rs:15–71` builds real git repositories and remote refs without a network. Add divergent local/origin main, master-only, tracking-only, missing/dangling upstream, detached/unborn HEAD and no base. Keep private-ref isolation and no-base tests (`tests/git_repo.rs:438–471`, `tests/git_repo.rs:578–595`, `tests/git_repo.rs:788–842`).
- Startup: **new** `tests/startup_flow.rs`, following the child-process environment pattern in `tests/send_flow.rs:84–118`. Assert a non-git cwd can paint/select both kinds, invalid config blocks selection work, cancellation launches no review worker, explicit ids skip menus, and config recovery retains the selected root rather than cwd. Keep `a_non_repo_path_yields_an_empty_state_not_an_error` (`tests/app_flow.rs:2541`) for the lower-level App API.
- Keyboard/render: copy narrow/tall `TestBackend` coverage (`tests/render.rs:20–51`, `tests/render.rs:3044–3067`) and modal cancellation/rebinding patterns (`tests/app_flow.rs:5295–5509`). Assert stable selection by id on rescan and disabled missing paths. Startup Escape may quit safely because no review comments exist; do not copy Send's destructive Enter action (`src/app.rs:4413–4416`).

## 2. Explicit `send_to` configuration and CLI destination

### Complete configuration thread

Add `Config.send_to: Option<String>` and `--send-to <target>` to `Config::parse`, a private `PluginConfig.send_to`, default `None`, a getter, parsing and normalised JSON. Update the allowed-key array length from eleven to twelve **with** the key; adding a field alone still rejects the entire file (`src/config.rs:13–23`, `src/config.rs:30–60`, `src/config.rs:69–81`, `src/config.rs:159–186`, `src/config.rs:250–272`, `src/config.rs:331–344`). The getter follows `editor()` and parsing should use `string_value`; require a non-empty single-line token, with pane/name existence deferred until Send (`src/config.rs:239–242`, `src/config.rs:572`). An absent cockpit is a Send refusal, not a whole-file config failure that makes browsing impossible.

Follow CLI theme precedence: add **new** `App::set_cli_send_to` and `App::effective_send_to`, set it in `ready_app`, and resolve CLI override before the validated file value. A file reread can change the destination for the next Send, but cannot override the CLI; config recovery must rebuild the same override (`src/lib.rs:369–375`, `src/app.rs:986–1008`, `src/lib.rs:1577–1592`). Do not create a separately parsed shell setting: `pane.sh` already invokes `--resolve-plugin-config` before all actions and consumes `PluginConfig::to_json` (`herdr/pane.sh:22–49`, `src/config.rs:649–653`). With no `send_to`, preserve the existing workspace one/many/no-agent behaviour (`src/herdr.rs:242–266`).

### Resolution and delivery thread

Extend `herdr::send_target` with an optional requested destination, or add a compatibility wrapper plus **new** `send_target_for`. Resolve explicit destinations from the **full** `agent_list` before the legacy workspace `candidates` filter. `AgentPane` already has optional cwd/name and required pane id; no new herdr API is needed (`src/herdr.rs:42–53`, `src/herdr.rs:234–266`, `src/herdr.rs:391–402`). Proposed policy:

1. Exclude non-agent rows and reviewr itself with `is_agent_other_than` (`src/herdr.rs:311–313`). Match a literal pane id exactly, or an exact agent `name`; do not match the display fallback `claude`/`codex` as a unique name (`src/herdr.rs:279–289`).
2. For reserved `cockpit`, match canonical cwd equal to `D/cockpit` **or** exact `name == cockpit`, deduplicate by pane id, and require exactly one match. The default cockpit root comes from briain's data root; it must never be inferred from the selected project cwd (`B/src/briain/cockpit/home.py:210–219`). Missing cwd may still match the name; a failed cwd canonicalisation is not a match.
3. Search all workspaces for an explicit target; zero matches or more than one unique cockpit/name match refuses visibly and retains comments. Do not fall back to the focused agent, last-sent pane, enumeration order or the legacy picker (`src/herdr.rs:242–266`, `src/app.rs:4356–4366`). This global explicit-target policy is proposed; the unset path remains workspace-scoped.

Change `App::send_to_agent` to pass the effective target while keeping its empty-store early return. A successful explicit resolution produces `SendTarget::One` and uses `export_to_agent`; no comment-store or export-trait routing changes are needed (`src/app.rs:4356–4367`, `src/app.rs:4714–4744`, `src/export.rs:39–47`). Resolve afresh on each Send, then address the chosen pane once; a disappearing pane fails through `send_text`. Never re-resolve to a replacement after a failed write (`src/export.rs:112–147`).

Add destination text to the footer/help or context header through `ui.rs`, sourced from the same effective config used by dispatch, with short refusal messages in `App.status`. Preserve painted-config gating so a changed file cannot reinterpret an already painted action (`src/ui.rs:76–77`, `src/app.rs:4107`, `src/lib.rs:459–486`). Avoid herdr lookups inside rendering; successful confirmation already names the actual resolved agent (`src/export.rs:128–138`). `agent_samples` and `candidates` retain their existing roles for Last turn and the unset default (`src/herdr.rs:372–402`).

Tests: copy the optional string/JSON round-trip at `src/config.rs:760–782`, whole-file rejection at `src/config.rs:786–807`, and key-count coverage at `src/config.rs:1091–1100`. Add empty/wrong-type/control-character values and CLI-over-file precedence. Copy pure `AgentPane` fixtures at `src/herdr.rs:446`, name/optional-field tests at `src/herdr.rs:523–609`, and sampling independence at `src/herdr.rs:481–508`. Extend `tests/send_flow.rs:88–239` using its existing fake executable and child environment: explicit cockpit among several agents, remote workspace, named target, pane-id target, duplicate names, name/cwd ambiguity, absent target, failed enumeration, disappearing pane, empty store and unchanged default picker. Add config recovery/display tests alongside `tests/app_flow.rs:5509` and `src/lib.rs:2871`.

## 3. Shared review header for Send and comment-copy

Add a **new** `ReviewContext`/`ReviewHeader` value in `src/model.rs` beside the session model, containing project id, optional run id, actual branch representation and base label/full OID. Store it on App and pass it into a **new** pure `export::format_review(context, comments)` from the one `App::export` call site. `format_review` prepends the header and delegates comment bodies to `format_all` (`src/model.rs:120–153`, `src/app.rs:598–603`, `src/app.rs:4724–4735`, `src/export.rs:15–35`). This reaches both `Agent` and `Clipboard` through `ExportTarget`; selected-text clipboard copies are a different action and should remain literal selections (`src/export.rs:39–47`, `src/export.rs:81–100`, `src/export.rs:141–147`, `src/app.rs:3091–3095`).

Required shape, with concrete values substituted:

```text
review: project | run | branch | base@oid | n comments

path:start-end
snippet
comment
```

The header requirement comes from `J/job.yaml:10`; the existing location/snippet/body shape comes from `src/export.rs:15–35` and `src/model.rs:135–146`. Proposed missing-value policy: `-` for no run or unregistered project, `detached@<HEAD OID>` for detached HEAD, `unborn` for no commit, and `none@-` for no base in a generic direct-path session. Use the full base OID in exported text; abbreviated OIDs are currently a screen convenience (`src/git.rs:848`, `src/git.rs:1250–1251`). Define escaping of `|`, newline, carriage return and terminal controls in metadata so the header stays one line. Do not alter comment prose to make it machine-parseable: this is a human-readable envelope, not the deferred JSON protocol (`README.md:479–484`).

**Identity must describe what was reviewed.** Project/run identity comes from the validated startup selection, not the outgoing pane cwd. Branch must come from git, not `agent/<job-id>` convention. Capture branch/base display context with the review snapshot and carry it through config recovery; avoid an independent fresh git query in the formatter or renderer that labels an old displayed diff with newer state (`src/world.rs:46–52`, `src/world.rs:118–145`, `src/app.rs:601–603`, `src/app.rs:1056–1063`, `AGENTS.md:24`). Existing `WorldInput` and `WorldSnapshot` are the seams if branch identity must travel with results (`src/world.rs:28–52`).

Do not blindly read `app.branch_base` for every export: world builds for Uncommitted/Last turn construct an empty `BaseStatus`. Keep the launch-pinned base context available across scopes, and define direct-path sessions' header context separately (`src/world.rs:121–145`). A single header also cannot assert that all comments were authored at its one revision: comments survive refreshes and can span worktree/commit views, while `Comment.rev` records only `Worktree` or a picked commit range (`src/model.rs:68–75`, `src/model.rs:120–132`, `AGENTS.md:23`). Document the header as session/export context; do not silently re-anchor old comments or promise an immutable snapshot of every comment. If the spec demands per-comment historical provenance, that is extra scope beyond this one-line header.

Keep `format_comment`, sorting, removed-side notation and multiline normalisation unchanged. Keep empty export a no-op and consume only after successful destination export (`src/export.rs:15–35`, `src/model.rs:137–145`, `src/app.rs:4724–4744`). Format the header **before** `herdr::send_text` frames the whole batch: one `PASTE_START`, sanitised body including metadata, one `PASTE_END`. Preserve the linear embedded-terminator removal, no Enter and focus-failure-is-success semantics (`src/herdr.rs:409–432`, `src/export.rs:141–147`).

Tests: retain body-only `block_is_location_snippet_text`, removed-side, multiline and sorting tests (`src/export.rs:198–231`). Add pure header tests for project/run, direct path, detached/unborn/no-base, one/many comments and escaped metadata; count the actual exported references (`src/app.rs:4729–4731`). Use `FakeTarget` in `tests/app_flow.rs:25–55` and `a_failed_export_keeps_comments_and_success_consumes_them` at `tests/app_flow.rs:1410` to prove identical header text for both destinations and retention on failure. Keep `src/model.rs:242–248` location tests unchanged. Extend bracketed-paste tests (`src/herdr.rs:609–624`) with a terminator in header metadata and preserve splice cases; the integration log checks framing and focus (`tests/send_flow.rs:175–188`). Add a focus-failure case: delivered text consumes exactly once even if the focus call fails (`src/export.rs:143–145`).

## 4. `pick` plugin action and focused-agent split

Add `[[actions]] id = "pick"` in `herdr-plugin.toml` with the existing pane/workspace contexts and `command = ["bash", "herdr/pane.sh", "pick"]`. Add a dedicated **new** pane entrypoint, for example `project-picker`, whose absolute binary command enters project selection (`herdr-plugin.toml:20–42`, `docs/herdr-api-notes.md:20–23`, `docs/herdr-api-notes.md:92`). Preserve the ordinary `pane` entrypoint so open/toggle and auto-open keep their present cwd-based behaviour (`herdr-plugin.toml:24`, `herdr-plugin.toml:44–51`).

**Resolve the no-id bootstrap explicitly in the spec.** An action has no selected project id yet. Recommended small CLI extension: bare `--project` opens the project picker, while `--project <id>` skips it; bare `--run` opens the run picker. The project menu can also expose a Runs view. The dedicated pane command can therefore be `exec "$HERDR_PLUGIN_ROOT/bin/herdr-reviewr" --project` without guessing an id. Selection resolves in that same pane before review startup, so no second pane or shell menu is required. This extends the requested id-taking flags; document and test optional-value parsing so a following `--send-to` is not swallowed as the project id (`src/config.rs:37–48`, `J/job.yaml:10`). An alternative explicit `--pick` flag is viable, but then the action does not literally launch with `--project`; choose one spelling in the spec, not two accidental implementations.

The action launches the terminal picker; it must not try to read a project interactively in its own shell process. The earlier source survey found no supplied pane PTY for action processes (`B/docs/research/reviewr-cockpit-integration-codex-2026-09-09.md:52`, referencing herdr v0.7.5 `src/app/api/plugins/runtime.rs:121–151`). The locally documented boundary supplies a pane command and an action command as separate concepts (`docs/herdr-api-notes.md:15–23`).

In `herdr/pane.sh`, recognise `pick` before the generic unknown-mode refusal and route it around **only** the cwd git-repository guard. Keep whole-file config validation first (`herdr/pane.sh:22–49`, `herdr/pane.sh:210–249`). Do not remove that guard from `open`, `toggle` or `auto-open`. The picker is explicitly allowed to start in the non-git cockpit cwd; only its later selected checkout needs to be git (`J/job.yaml:10`, `src/lib.rs:340–375`).

For `pick`, force split placement and validate the focused pane as an agent using `agent list`, selecting `.focused_pane_id` from the action context rather than silently falling back to the workspace's first pane. Keep a single captured target id for the open; if it vanishes, report the open failure without opening elsewhere. The current general placement branch may choose the first pane when no pane id exists, which does not establish the requested focused-agent relation (`herdr/pane.sh:88–108`, `herdr/pane.sh:255–280`, `docs/herdr-api-notes.md:101–112`, `docs/herdr-api-notes.md:171–183`). Split adjacency is placement, not the Send destination: the new config/flag in feature 2 owns routing (`src/herdr.rs:391–402`).

Use `plugin pane open --plugin <id> --entrypoint project-picker --placement split --direction <configured direction> --target-pane <focused-agent> --cwd <launch cwd> --focus`. Require `.result.plugin_pane.pane.pane_id` before reporting success; split must use `--target-pane`, which determines workspace (`herdr/pane.sh:277–280`, `docs/herdr-api-notes.md:79–92`). No binary-argv passthrough is documented for this call; a fixed entrypoint avoids relying on one (`herdr-plugin.toml:24`, prior Codex map:54).

The existing workspace-wide process sweep recognises reviewr by executable basename and closes every matching reviewr pane on toggle. Specify `pick` as an open-only/no-op-if-already-open action initially, or explicitly permit independent sessions; never inherit toggle's close-all branch accidentally, because that can discard unsent comments (`herdr/pane.sh:120–140`, `herdr/pane.sh:190–229`, `README.md:447–450`). Recommended first contract: refuse visibly if reviewr is already open in that workspace, leaving it intact. That keeps one session per workspace for this action without silently ignoring the user's request to pick another project. Generic open/close/toggle semantics remain unchanged.

Tests: extend `tests/pane_actions.rs` using `fake_herdr` and `run_with_context`, which inject `HERDR_REVIEWR_BIN`, config dir, fake herdr, workspace/pane and JSON context (`tests/pane_actions.rs:19–54`, `tests/pane_actions.rs:122–132`). Add non-git cockpit success, non-agent/no-focused-pane refusal, dead target, config refusal before calls, forced split despite overlay/tab config, quoted cwd, correct entrypoint, and existing-reviewr refusal. Parse the manifest like `manifest_auto_open_hooks_created_and_opened` (`tests/pane_actions.rs:265`). Preserve ordinary live-cwd tests (`tests/pane_actions.rs:652–789`), invalid-config tests (`tests/pane_actions.rs:152–181`), process identity (`tests/pane_actions.rs:291–327`) and flag-dispatch parity (`tests/pane_actions.rs:570–590`). If a new non-UI helper flag is introduced, update both `src/main.rs:1–13` and the process exclusion at `herdr/pane.sh:130–135`; picker flags themselves run UI and must still count as reviewr.

## 5. README, developer loop and explicit rebase recipe

Add a README cockpit section beside Quick start / Configuration, with project versus retained-run selection, CLI selectors, `send_to` precedence, missing/ambiguous cockpit refusal, the exact header and manual submission. Update the flag table and config example, base documentation and Send limitation instead of leaving contradictory stock-only wording (`README.md:76–98`, `README.md:195–234`, `README.md:265–278`, `README.md:429–450`). State that Last turn observes worktree agents and a finished run is better reviewed in Branch; selection does not create or restore a deleted worktree (`src/herdr.rs:372–402`, `src/git.rs:1196–1209`, `B/src/briain/dispatcher/spawn.py:555–575`).

Document the operator's `Ctrl+A`, then `d` gesture as a **user keybinding**, not a manifest default. The eventual action id is `persiyanov.reviewr.pick`; the API notes put keybindings in user config and explain fully qualified action names. Have the implementer verify the installed prefix-key syntax before offering the binding snippet; this research changes no binding (`J/job.yaml:7`, `docs/herdr-api-notes.md:159–166`). The old popup-wrapper binding is historical evidence, not proof the fork action is already wired (`B/docs/research/reviewr-cockpit-integration-codex-2026-09-09.md:22`).

### Spec folder and changelog

Create **new** `docs/specs/2026-09-09-briain-cockpit-flow/spec.md`, then `plan.md` and five tickets under `tickets/`. The spec should settle bare-selector syntax, upstream meaning, startup Branch default, base pinning, ambiguous cockpit policy, existing-pane handling and header missing/history semantics described above. Use named invariants and a test mapping like the current auto-open spec (`docs/specs/README.md:3–6`, `docs/specs/2026-08-28-auto-open-worktree-opened/spec.md:45–55`). Each feature commit should carry its applicable spec/ticket changes and an Unreleased changelog bullet; do not copy the managed worktree brief into a feature commit (`CONTRIBUTING.md:32–38`, `AGENTS.md:62–65`). This map is implementation research, not an approved behaviour spec.

### Gate and timing

`just ci` runs, in recipe order, `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`, then `cargo build --release` (`justfile:12–21`, `justfile:55–57`). CI also exports `RUSTFLAGS=-D warnings`; use `RUSTFLAGS='-D warnings' just ci` for that environment parity, since the recipe alone does not set it (`.github/workflows/ci.yml:8–10`, `.github/workflows/ci.yml:26–36`). Rust forbids unsafe code and enables clippy's pedantic group; environment-dependent tests must set environment on child commands, as Send's test does (`Cargo.toml:33–49`, `tests/send_flow.rs:84–118`).

**No timings were measured in this research because builds/tests were prohibited.** Planning allowance, not a benchmark: 10–30 minutes for a cold gate after toolchain setup, 1–5 minutes for a warm gate; dependency downloads, host load and release LTO can extend either. Record actual wall time on the implementer's first run. The source establishes the commands and thin LTO, not those estimates (`justfile:55–57`, `Cargo.toml:51–53`). Do not quote a test count as current evidence from an older spec completion (`docs/specs/2026-08-28-auto-open-worktree-opened/plan.md:64–68`).

`cargo deny check` is a separate dependency check, not a `just ci` dependency. Run it if adding YAML dependencies, with the repository's yanked-source, licence and registry policy (`deny.toml:1–39`, `justfile:55–57`). `just smoke-edit` takes about a minute and is required if startup work alters the shared terminal-mode stack; it is outside CI (`CONTRIBUTING.md:21–24`, `justfile:50–53`). Changes to reload/render/git require before/after PTY latency measurements, same-load interleaving, and the single baseline file updated only if numbers move (`AGENTS.md:13–14`, `CONTRIBUTING.md:40–50`). Record acceptance results with the feature, not in this unexecuted survey.

### Machine with no Rust: later commands, not executed

The pinned toolchain is **1.97.0**, minimal profile with rustfmt and clippy; clippy's MSRV agrees (`rust-toolchain.toml:1–4`, `clippy.toml:1`, `Cargo.toml:5`). The official Rustup installation guide documents home-directory installation and a no-toolchain bootstrap; the Just manual documents installation through Cargo. These external installation details were checked on 9 September 2026: [Rustup installation, lines 17 and 24–36](https://rust-lang.github.io/rustup/installation/index.html), [Just packages, line 21](https://just.systems/man/en/packages.html).

Run later, without sudo or changing `$HOME`; all tool installations below stay under the actual home directory. The flags below adapt the documented Rustup bootstrap to the repository pin (`rust-toolchain.toml:2–4`; official Rustup source above).

```bash
export CARGO_HOME="$HOME/.cargo"
export RUSTUP_HOME="$HOME/.rustup"
export PATH="$CARGO_HOME/bin:$PATH"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
  sh -s -- -y --no-modify-path --default-toolchain none
rustup toolchain install 1.97.0 --profile minimal --component rustfmt --component clippy
cargo +1.97.0 install just --locked --root "$CARGO_HOME"
rustc +1.97.0 --version
just --version
```

If the pinned toolchain/components cannot be downloaded, record that failure rather than editing the pin to make the gate pass (`rust-toolchain.toml:2–4`, `.github/workflows/ci.yml:19–22`). Before building, check that a C linker/toolchain and normal project prerequisites are available; home-only Rustup does not provision OS development packages. This is a prerequisite check for the implementer, not authorisation to install system packages. The existing prerequisites include git and the shell action relies on bash/jq (`README.md:34–40`, `herdr/pane.sh:1–17`, `herdr/pane.sh:40–41`).

### Build and link the fork, then iterate

The local route is `just install`, which builds release into `target/release` and installs a fresh binary at **this checkout's** `bin/herdr-reviewr` via `swap-binary.sh`. It is not `cargo install --path .`, despite the old manifest comment suggesting that command (`justfile:31–35`, `herdr-plugin.toml:11–24`). From the chosen development checkout, after verification:

```bash
RUSTFLAGS='-D warnings' just ci
just install
herdr plugin uninstall persiyanov.reviewr
herdr plugin link .
herdr plugin list --plugin persiyanov.reviewr
```

Uninstall the GitHub source **before linking**, otherwise local rebuilds do not affect the active plugin. The plugin id remains `persiyanov.reviewr`, and its id-keyed config survives the source swap (`README.md:460–476`, `herdr-plugin.toml:1`). Confirm the list reports the local checkout. On subsequent edits, run the relevant tests/gate and `just install`; the operator then closes and reopens reviewr with their keybinding. An existing pane retains the old process, and scripted opens target whichever workspace is focused (`README.md:470–476`, `docs/qa-install.md:35–49`). Do not replace the installed binary in place or improvise QA swaps (`docs/qa-install.md:27–33`).

Keep this fork's distribution explicitly local-linked for these five features. Merely installing `BrianIsaac/herdr-reviewr` through the GitHub plugin route would still execute an installer whose download repository is hard-coded to `persiyanov/herdr-reviewr`; that can silently supply the upstream binary (`herdr/install.sh:12–20`). Publishing fork release assets is separate work, with matched package/manifest versions and tag contracts (`docs/RELEASING.md:7–16`). Do not claim the release installer distributes the fork until that work exists.

### Rebase recipe to include in the README

`origin` is the operator's fork and `upstream` is the source project by this job's decision (`J/job.yaml:7`). Keep the five functional commits separable, review upstream changes deliberately and preserve a backup before replay. The following is a proposed manual recipe, never run by this research; `4c09022` is the initial replay boundary and must be updated to the last adopted upstream commit after each successful upgrade (`Cargo.toml:3`, `J/job.yaml:7`).

```bash
git status --short
# Continue only with a clean developer checkout and the fork branch checked out.
git fetch upstream --tags
git branch backup/cockpit-before-rebase-YYYYMMDD
git rebase --onto upstream/main 4c09022
# Resolve each conflict by its feature/spec; use git rebase --abort if abandoning it.
git range-diff 4c09022..backup/cockpit-before-rebase-YYYYMMDD upstream/main..HEAD
RUSTFLAGS='-D warnings' just ci
just install
```

The recipe deliberately does not push rewritten history. Review the replay and publish only through the fork owner's agreed branch policy; a new review branch avoids rewriting a shared branch. Update the recorded upstream base, inspect config allowed keys and recovery, the startup/worker boundary, Send framing, and plugin action contracts on each replay; these are the coupled surfaces documented above (`src/config.rs:69–81`, `src/lib.rs:889–907`, `src/lib.rs:1577–1583`, `src/herdr.rs:409–432`, `herdr/pane.sh:210–280`). Re-run the picker/non-git, absent-target, header/clipboard, paste and shell-action acceptance cases, plus terminal smoke/performance checks when their trigger paths changed (`CONTRIBUTING.md:21–24`, `CONTRIBUTING.md:47–58`).

## herdr API contract checklist for the implementer

These are the repository's documented **0.7.5** contracts, not new live probes from this research (`docs/herdr-api-notes.md:1–3`).

| Call / environment | Contract and use |
| --- | --- |
| `agent list` | No flags; `.result.agents` with pane/tab/workspace/status/cwd. Filter locally. Optional name/display/state fields can be absent or null. Used by explicit Send and focused-agent validation (`docs/herdr-api-notes.md:171–186`). |
| `pane send-text <pane> <text>` | Literal text, no Enter. Rust adds paste framing. A failed call returns a JSON stderr envelope; do not expose raw argv/review text as a status message (`docs/herdr-api-notes.md:196–212`, `src/herdr.rs:79–95`). |
| `agent focus <pane>` | Focus for human submission after delivery; failure must not retry the paste (`docs/herdr-api-notes.md:196–198`, `src/export.rs:141–147`). |
| `tab list --workspace <ws>` | `.result.tabs`, join by `tab_id` to **label**, not number. Best-effort legacy Send picker enrichment (`docs/herdr-api-notes.md:188–190`, `src/herdr.rs:318–334`). |
| `plugin pane open` | Fixed entrypoint command; split requires `--target-pane`; response id at `.result.plugin_pane.pane.pane_id`. Pane command resolves against its cwd, so executable path must be absolute (`docs/herdr-api-notes.md:79–92`). |
| `pane list --workspace <ws>` / `pane process-info --pane <id>` | List carries live `foreground_cwd`, not process identity. Process-info carries foreground argv; use basename, not mutable title/name. Missing pane is distinct from unreadable listing (`docs/herdr-api-notes.md:44–65`, `docs/herdr-api-notes.md:104–109`). |
| `pane close <id>` | Works independently of the in-memory plugin-pane registry; `plugin pane close` cannot find forgotten/layout panes (`docs/herdr-api-notes.md:87–90`). |
| `plugin config-dir <id>` | Plain config directory path; plain panes can read the same config. `HERDR_PLUGIN_CONFIG_DIR` wins when set (`docs/herdr-api-notes.md:69–71`, `src/config.rs:298–314`). |
| Action context and pane environment | Action invoke uses the **focused workspace** and ignores caller context overrides. Pane identity variables belong to the new pane, not the send recipient; never spoof them to force routing (`docs/herdr-api-notes.md:41–43`, `docs/herdr-api-notes.md:94–112`). |
| `plugin link`, `plugin action invoke`, keybinding | Link skips build; invoke is `<action> --plugin <id>`; keybinding is `<plugin_id>.<action_id>` in user config. Document these without running them in research (`docs/herdr-api-notes.md:12`, `docs/herdr-api-notes.md:34–35`, `docs/herdr-api-notes.md:159–166`). |

## Suggested commit sequence and estimate

These are suggested conventional messages and engineering estimates, not measured delivery promises. One sitting means roughly two to four focused hours including targeted tests; allow **six to nine sittings** for the five features and an operator acceptance pass. The largest work is the startup/config/worker boundary and new data parsing, not the existing diff engine (`src/lib.rs:72–141`, `src/lib.rs:889–907`, `src/config.rs:331–344`).

1. **`feat(cockpit): select registered projects and retained runs before review startup`** — new registry/startup modules, selectors, local-main/master/upstream resolver, initial Branch policy, session identity and tests; start the spec/plan/tickets and changelog. **Three to four sittings**. Existing seams: `src/config.rs:30`, `src/lib.rs:347`, `src/app.rs:850`, `src/git.rs:596`.
2. **`feat(send): route reviews to an explicit cockpit or agent target`** — complete `send_to` schema/CLI/recovery thread, pure resolution, visible refusal/display and fake-herdr tests. **One to two sittings**. Existing seams: `src/config.rs:69`, `src/herdr.rs:242`, `src/app.rs:4356`, `tests/send_flow.rs:88`.
3. **`feat(export): prefix sent and copied comments with review context`** — shared pure formatter, metadata policy and parity/paste/consume tests. **One sitting**. Existing seams: `src/model.rs:150`, `src/export.rs:32`, `src/app.rs:4724`, `src/herdr.rs:424`.
4. **`feat(plugin): open the project picker beside the focused agent`** — dedicated pane entrypoint, `pick` action, non-git ordering and focused-target shell tests. **Half to one sitting**. Existing seams: `herdr-plugin.toml:20–42`, `herdr/pane.sh:210–280`, `tests/pane_actions.rs:122`.
5. **`docs(cockpit): document the fork workflow and upstream rebase procedure`** — final README consolidation, build/link/uninstall loop, setup/pin instructions, manual keybinding and acceptance evidence. **Half to one sitting**. Existing seams: `README.md:195–234`, `README.md:460–476`, `CONTRIBUTING.md:32–58`, `justfile:31–57`.

# Cockpit fork: sitting 1 handover

Date: 2026-09-09 (Asia/Singapore)
Status: setup, baseline gate, linked build and specification completed; pre-existing managed AGENTS.md change remains excluded (see repository status below).

## Outcome and commit

Spec/changelog commit: `1222c4b docs(spec): briain cockpit review flow`.

The contract is [spec.md](../../specs/2026-09-09-briain-cockpit-review/spec.md), sequenced in [plan.md](../../specs/2026-09-09-briain-cockpit-review/plan.md). Both implementer maps were read in full; the Claude map's section 13 sequence and design (A) govern disagreements. All five features are specified, including parser-first negative coverage and additive-only preservation of upstream exact-shape tests. No runtime code or packaging was changed. The Unreleased entry explicitly describes planned features, not a shipped cockpit UI.

The existing primary checkout is `/home/brian-isaac/Documents/personal/herdr-reviewr`, branch `main`, at `f3a0e4f205a90d1c6603fd6e6412314a8d8ab139`. It was clean before and after the gate/install/link. Its only tracked differences from upstream code baseline `4c090225af706bf3aaa24b39fea890a72994f40f` (v0.36.2) are the two research maps and their handovers. Source, tests, Cargo files, toolchain and justfile are unchanged. Origin is `https://github.com/BrianIsaac/herdr-reviewr.git`; upstream is `https://github.com/persiyanov/herdr-reviewr.git`. No fetch, merge, rebase or push was performed in this sitting.

## Toolchain

Installed through the [official rustup bootstrap](https://rustup.rs/) with `--no-modify-path --profile minimal --default-toolchain none`, setting `CARGO_HOME="$HOME/.cargo"` and `RUSTUP_HOME="$HOME/.rustup"`. `rustup show` in the primary checkout installed the toolchain from the unchanged `rust-toolchain.toml`. No system packages or shell startup files were changed. In future shells use:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Recorded versions:

```text
rustup 1.29.1 (d95a37b6a 2026-08-13)
rustc 1.97.0 (2d8144b78 2026-07-07)
cargo 1.97.0 (c980f4866 2026-06-30)
rustfmt 1.9.0-stable (2d8144b788 2026-07-07)
clippy 0.1.97 (2d8144b788 2026-07-07)
just 1.58.0
herdr 0.7.5
host: x86_64-unknown-linux-gnu
```

Installed components: cargo, clippy, rust-std, rustc, rustfmt for that host. `/usr/bin/just` was absent; `cargo install just --locked --root "$HOME/.cargo"` installed `~/.cargo/bin/just` (reported compilation time 2m 21s).

Setup detail: an early just install attempted while rustup was still downloading exited 101 with `error: missing manifest in toolchain '1.97.0-x86_64-unknown-linux-gnu'` from `rustc -vV`. Waiting for the active rustup installation to complete and rerunning just installation succeeded. This was before the baseline gate and was not an upstream code failure. One redundant version probe started by this sitting was terminated; no pre-existing process was stopped. Rustup warned that auto-installation through `rustup show` is deprecated; future scripts may use explicit `rustup install` while retaining the repository pin.

## First timed full gate

Command, from the clean primary checkout:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
RUSTFLAGS='-D warnings' just ci
```

`RUSTFLAGS` matches CI's environment; the justfile sequences fmt-check, clippy, tests, release build. Python `time.monotonic()` measured the subprocess wall clock independently of downloads/setup.

- Start: `2026-09-09T06:05:15.266205+00:00` (14:05:15 Singapore).
- End: `2026-09-09T06:15:06.455729+00:00` (14:15:06 Singapore).
- **Wall time: 591.190 seconds = 9m 51.190s. Exit code: 0.**
- Formatting passed; clippy passed with warnings as errors.
- Tests: **820 passed, 0 failed, 1 ignored**. Breakdown: 318 unit, 264 app_flow, 54 git_repo, 29 pane_actions, 24 pr_candidates, 130 render, 1 send_flow. The single pr_live test is ignored. Binary/doc harnesses had zero tests.
- Release build passed, reported `7m 49s`; package v0.36.2. This was a cold project build after installing just, on the live machine's existing load, not an isolated performance benchmark.
- No upstream failure to fix. No source edits made. PTY latency and smoke-edit were not triggered by setup/documentation changes and were not run.

Raw local logs (temporary, not required by later builds): `/tmp/reviewr-sitting-1-ci.log`, timing metadata `/tmp/reviewr-sitting-1-ci.json`, just installation `/tmp/reviewr-just-install.log`. Durable results are recorded above rather than relying on /tmp surviving.

## Linked build verification

Executed in the requested order:

```bash
herdr plugin uninstall persiyanov.reviewr
cd /home/brian-isaac/Documents/personal/herdr-reviewr
export PATH="$HOME/.cargo/bin:$PATH"
RUSTFLAGS='-D warnings' just install
herdr plugin link .
herdr plugin list
```

Same RUSTFLAGS reuses the gated release build. Install reported a 0.23s incremental release build and used `scripts/swap-binary.sh` for a fresh inode at `bin/herdr-reviewr`. `bin/` is ignored and no binary was committed. No tracked linked-build changes were needed.

Link response: `source.kind = local`, `plugin_root = /home/brian-isaac/Documents/personal/herdr-reviewr`, `plugin_id = persiyanov.reviewr`, version 0.36.2. List output:

```text
1 plugin installed:
- persiyanov.reviewr (reviewr) enabled [local:/home/brian-isaac/Documents/personal/herdr-reviewr]
  config: /home/brian-isaac/.config/herdr/plugins/config/persiyanov.reviewr
```

Both stable links still pointed into the removed GitHub installation and were dangling after link. Replaced only these symlinks, using temporary symlinks and rename, without invoking pane.sh or opening a pane:

```text
~/.local/bin/herdr-reviewr
~/.local/state/herdr/plugins/persiyanov.reviewr/bin/herdr-reviewr
  -> /home/brian-isaac/Documents/personal/herdr-reviewr/bin/herdr-reviewr
```

Strict realpath and file/executable checks passed for both. Installed binary and `target/release/herdr-reviewr` share SHA-256:

```text
2e42a167ef9eb7200c625ebff5c9764a2df62dc076adeb4ae4ab25f868849338
```

Protected files retained their original SHA-256:

```text
~/.config/herdr/config.toml
cf2c363df48fba7ad81b94deb4ca73af07924427b42c65caeed8f078d4c8c112
~/.config/herdr/plugins/config/persiyanov.reviewr/config.toml
c6eba14d878b6a8ad044adec08eb6cd99681fe916e1f4380c91e953cc6226e3a
```

No pane was opened or closed, no keybinding or reviewr config edited, no message sent to an agent, and no pre-existing process touched. Existing reviewr processes retain their old image until the operator closes/reopens them. The linked binary is still baseline v0.36.2; no cockpit feature is active yet. The primary link survives this worktree's reap. Future feature changes must reach that primary checkout before rebuilding there; rebuilding an isolated worktree does not update the linked plugin.

## Exact next step: sitting 2

Read the spec, plan ticket 1 and this handover. Begin **`src/briain.rs` registry module with tests against the real data layout, and the base chain in `src/git.rs`**. Do not begin the startup picker UI in this sitting.

1. Inspect `notes/projects/*.md`, `worktrees/<id>/`, and matched `runs/<id>/{status.json,job.yaml}` read-only. Current checks confirm astraweave is active at `/home/brian-isaac/Documents/personal/astraweave`; herdr-reviewr is active at the primary checkout. This run's status keys are backend, briefing_bytes, briefing_path, exit_reason, harness, phase, state, ts, window_target; project association is in `job.yaml` (`project: herdr-reviewr`, `mode: interactive`). Avoid dumping unrelated metadata into fixtures.
2. Add the module and pure root/scalar/project/run discovery functions with injectable paths. Test quoted/null paths, unrelated description colons and body keys, inactive/malformed/duplicate records, missing metadata, traversal ids, stray files and pruned linked worktrees. Walk worktrees first, join metadata by basename; do not scan all historical runs or infer project from job id. Keep git subprocesses in git.rs. Do not add serde_yaml.
3. Add and test the launch-only chain: local `refs/heads/main`, local `refs/heads/master`, branch tracking `@{upstream}`, then None. The selected launch will preserve explicit `--base` priority, and None leaves stock private-pick/origin-HEAD fallback alone. Use full ref spellings rather than remote-first picker spellings. Add actual HEAD branch helper as needed. Do not write a pick ref or alter the stock resolver.
4. Capture before/after PTY latency for git-path changes, rebuilding the old binary into a second target directory and interleaving same-load runs. Run targeted registry/git tests and the full gate. Preserve **No writes**, **Comments survive**, **Continuity** and all upstream exact-shape tests.
5. Sitting 3 starts with parser arms plus the negative fallthrough test before any callers emit `--project`, `--run`, `--pick`, or `--send-to`. Then implement `src/pick.rs` and pre-App startup design (A). Remaining sittings follow the committed plan.

Final operator acceptance is deferred to sitting 6: ctrl+a d, pick astraweave, comment, s, lands in cockpit input with header and no Enter. It was not tested here.

## Repository status and remaining exception

The task branch began with exactly one local change: AGENTS.md contained the briain-managed brief (99 added lines). The brief itself says it is never merged back, and the operator explicitly said not to rewrite AGENTS.md. It was preserved byte-for-byte and excluded from both commits; no assume-unchanged/skip-worktree flags were used to hide it. A question was sent asking whether this pre-existing managed change may be treated as the clean-status exception. Until the operator confirms, literal clean status is not satisfied, despite all authored deliverables being committed. Do not erase or commit the brief to manufacture clean status.

Primary main remains clean. This task's only expected final porcelain entry is ` M AGENTS.md`. The handover commit also marks the sitting-1 commit checklist complete in plan.md. All feature tickets remain pending.

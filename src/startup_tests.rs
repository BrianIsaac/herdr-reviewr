//! Startup harness: real temporary checkouts, injected registry/events, no review workers.
use super::*;
use ratatui::{Terminal, backend::TestBackend};
use std::{path::Path, process::Command};

fn git_at(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "t@example.test")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "t@example.test")
            .output()
            .unwrap()
            .status
            .success()
    );
}
fn fixture() -> (tempfile::TempDir, tempfile::TempDir) {
    let data = tempfile::tempdir().unwrap();
    let repo = tempfile::tempdir().unwrap();
    git_at(repo.path(), &["init", "-q", "-b", "main"]);
    git_at(repo.path(), &["commit", "--allow-empty", "-qm", "initial"]);
    std::fs::create_dir_all(data.path().join("notes/projects")).unwrap();
    std::fs::write(
        data.path().join("notes/projects/a.md"),
        format!("---\nid: a\nstatus: active\nworking_dir: {}\n---\n", repo.path().display()),
    )
    .unwrap();
    std::fs::create_dir_all(data.path().join("worktrees")).unwrap();
    git_at(
        repo.path(),
        &["worktree", "add", "-qb", "job", data.path().join("worktrees/job").to_str().unwrap()],
    );
    (data, repo)
}
fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}
fn terminal() -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(100, 30)).unwrap()
}

#[test]
fn startup_explicit_ids_bypass_input_and_resolve_before_app() {
    let (data, repo) = fixture();
    for (flag, id, root, scope) in [
        ("--project", "a", repo.path().to_path_buf(), Scope::Uncommitted),
        ("--run", "job", data.path().join("worktrees/job"), Scope::Branch),
    ] {
        let mut cfg = Config::parse([flag.into(), id.into()]);
        cfg.repo = data.path().to_path_buf(); // non-repository cockpit placeholder
        let plugin = resolve_launch(
            &mut terminal(),
            &mut cfg,
            || Ok(data.path().to_path_buf()),
            |_| panic!("explicit id opened menu"),
        )
        .unwrap()
        .unwrap();
        assert_eq!(cfg.repo, root.canonicalize().unwrap());
        assert_eq!(cfg.base.as_deref(), Some("refs/heads/main"));
        let app = app_for(&cfg, &Ok(plugin));
        assert_eq!(app.repo, cfg.repo);
        assert_eq!(app.scope, scope);
        assert_eq!(app.review_identity, cfg.selection.as_ref().map(|s| s.identity.clone()));
    }
}

#[test]
fn startup_non_git_menu_cancel_never_selects_or_seeds_baseline() {
    let (data, repo) = fixture();
    let mut cfg = Config::parse([data.path().display().to_string()]);
    let mut screen = terminal();
    let result = resolve_launch(
        &mut screen,
        &mut cfg,
        || Ok(data.path().to_path_buf()),
        |_| Ok(Some(key(KeyCode::Esc))),
    )
    .unwrap();
    assert!(result.is_none());
    assert!(cfg.selection.is_none());
    assert!(!repo.path().join(".git/refs/worktree/reviewr").exists());
    let text: String =
        screen.backend().buffer().content.iter().map(ratatui::buffer::Cell::symbol).collect();
    assert!(text.contains("projects"));
}

#[test]
fn startup_non_git_menu_selects_and_invalid_id_stays_visible_until_input() {
    let (data, repo) = fixture();
    for args in [
        vec![data.path().display().to_string()],
        vec!["--project".into(), "../a".into()],
        vec!["--run".into(), "absent".into()],
    ] {
        let mut cfg = Config::parse(args);
        let mut input = [KeyCode::Enter, KeyCode::Enter].into_iter();
        let plugin = resolve_launch(
            &mut terminal(),
            &mut cfg,
            || Ok(data.path().to_path_buf()),
            |_| Ok(Some(key(input.next().expect("two-level menu")))),
        )
        .unwrap();
        assert!(plugin.is_some());
        assert_eq!(cfg.repo, repo.path().canonicalize().unwrap());
    }
    let mut cfg = Config::parse(["--run".into(), "absent".into()]);
    let mut screen = terminal();
    resolve_launch(
        &mut screen,
        &mut cfg,
        || Ok(data.path().to_path_buf()),
        |_| Ok(Some(key(KeyCode::Esc))),
    )
    .unwrap();
    let text: String =
        screen.backend().buffer().content.iter().map(ratatui::buffer::Cell::symbol).collect();
    assert!(text.contains("run absent: expected one retained match"));
}

#[test]
fn startup_config_block_and_recovery_precede_discovery() {
    let (data, _repo) = fixture();
    let config_dir = tempfile::tempdir().unwrap();
    let path = config_dir.path().join("config.toml");
    std::fs::write(&path, "bad = true\n").unwrap();
    let mut cfg = Config::parse(["--project".into(), "a".into()]);
    cfg.plugin_config_dir = Some(config_dir.path().to_path_buf());
    let fixed = std::cell::Cell::new(false);
    let plugin = resolve_launch(
        &mut terminal(),
        &mut cfg,
        || {
            assert!(fixed.get(), "invalid config must gate discovery");
            Ok(data.path().to_path_buf())
        },
        |_| {
            std::fs::write(&path, "default_scope = \"last-turn\"\n").unwrap();
            assert!(!fixed.replace(true), "config repair must resolve on the next iteration");
            Ok(None)
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(ready_app(&cfg, plugin).scope, Scope::LastTurn);
}

#[test]
fn startup_recovery_and_config_directory_fallback_keep_resolved_context() {
    let (data, repo) = fixture();
    let mut cfg = Config::parse([
        "--run".into(),
        "job".into(),
        "--base".into(),
        "HEAD~0".into(),
        "--wrap".into(),
        "off".into(),
        "--theme".into(),
        "nord".into(),
        "--send-to".into(),
        "cockpit".into(),
    ]);
    resolve_launch(&mut terminal(), &mut cfg, || Ok(data.path().to_path_buf()), |_| panic!("menu"))
        .unwrap();
    let selected = cfg.selection.clone().unwrap();
    // A later config source and an edited project note cannot change the chosen checkout.
    let config_dir = tempfile::tempdir().unwrap();
    let path = config_dir.path().join("config.toml");
    cfg.plugin_config_dir = Some(config_dir.path().to_path_buf());
    std::fs::write(data.path().join("notes/projects/a.md"), "broken note").unwrap();
    std::fs::write(&path, "bad = true\n").unwrap();
    let blocked = app_for(&cfg, &config::plugin_config_in(config_dir.path()));
    assert_eq!(blocked.repo, selected.root);
    assert_eq!(blocked.review_identity, Some(selected.identity.clone()));
    std::fs::write(&path, "default_scope = \"uncommitted\"\ntheme = \"gruvbox\"\n").unwrap();
    let recovered = ready_app(&cfg.clone(), config::plugin_config_in(config_dir.path()).unwrap());
    assert_eq!(recovered.repo, selected.root);
    assert_ne!(recovered.repo, repo.path());
    assert_eq!(recovered.review_identity, Some(selected.identity));
    assert_eq!(recovered.scope, Scope::Branch);
    assert_eq!(recovered.base.as_deref(), Some("HEAD~0"));
    assert!(!recovered.wrap);
    assert_eq!(cfg.send_to.as_deref(), Some("cockpit"));
}

#[test]
fn startup_direct_path_keeps_stock_base_scope_and_identity() {
    let (data, repo) = fixture();
    let mut cfg = Config::parse([repo.path().display().to_string()]);
    let plugin = resolve_launch(
        &mut terminal(),
        &mut cfg,
        || panic!("direct paths do not discover"),
        |_| panic!("direct paths do not prompt"),
    )
    .unwrap()
    .unwrap();
    assert_eq!(cfg.base, None);
    assert_eq!(cfg.selection, None);
    let app = ready_app(&cfg, plugin);
    assert_eq!(app.scope, Scope::Uncommitted);
    assert_eq!(app.review_identity, None);
    assert_ne!(app.repo, data.path());
}

#[test]
fn startup_no_base_rung_preserves_none_and_selection_failure_is_atomic() {
    let (data, repo) = fixture();
    git_at(repo.path(), &["branch", "-m", "topic"]);
    let mut cfg = Config::parse(["--project".into(), "a".into()]);
    resolve_launch(&mut terminal(), &mut cfg, || Ok(data.path().to_path_buf()), |_| panic!("menu"))
        .unwrap();
    assert_eq!(cfg.base, None);
    let mut unavailable = cfg.selection.clone().unwrap();
    unavailable.root = data.path().join("gone");
    let old = cfg.selection.clone();
    assert!(apply_selection(&mut cfg, &unavailable).is_err());
    assert_eq!(cfg.selection, old);
}

use herdr_reviewr::config::Config;

fn parse(args: &[&str]) -> Config {
    Config::parse(args.iter().map(ToString::to_string))
}

#[test]
fn new_valued_flags_never_become_positional_paths() {
    for flag in ["--project", "--run", "--send-to"] {
        let cfg = parse(&[flag, "astraweave"]);
        assert_ne!(cfg.repo.to_str(), Some("astraweave"), "{flag} fell through");
    }
}

#[test]
fn new_missing_values_do_not_swallow_the_next_flag() {
    for flag in ["--project", "--run", "--send-to"] {
        let cfg = parse(&[flag, "--base", "main"]);
        assert_eq!(cfg.base.as_deref(), Some("main"));
        assert_ne!(cfg.repo.to_str(), Some("main"));
    }
}

#[test]
fn empty_missing_and_conflicting_selectors_are_refused() {
    // These must remain errors even if another token could otherwise supply a repo.
    for args in [
        vec!["--project"],
        vec!["--run"],
        vec!["--send-to"],
        vec!["--project", ""],
        vec!["--run", " "],
        vec!["--send-to", ""],
        vec!["--project", "a", "--run", "b"],
        vec!["--pick", "--project", "a"],
        vec!["--pick", "--pick"],
        vec!["--run", "a", "--run", "b"],
        vec!["--project", "a", "/tmp/repo"],
        vec!["/tmp/repo", "--pick"],
        vec!["--run", "a", "/tmp/repo"],
    ] {
        assert!(parse(&args).launch_error.is_some(), "{args:?}");
    }
}

#[test]
fn reserved_send_target_and_selector_values_are_carried() {
    use herdr_reviewr::config::Selector;
    let cfg = parse(&["--project", "a", "--send-to", "cockpit"]);
    assert_eq!(cfg.selector, Some(Selector::Project("a".into())));
    assert_eq!(cfg.send_to.as_deref(), Some("cockpit"));
    assert!(cfg.launch_error.is_none());
    assert_eq!(parse(&["--run", "r"]).selector, Some(Selector::Run("r".into())));
    assert_eq!(parse(&["--pick"]).selector, Some(Selector::Pick));
    for flag in ["--project", "--run", "--send-to"] {
        let cfg = parse(&[flag, "--base", "main"]);
        assert!(cfg.launch_error.is_some());
        assert_eq!(cfg.base.as_deref(), Some("main"));
    }
}

#[test]
fn legacy_unknown_flags_and_last_positional_assignment_are_unchanged() {
    assert_eq!(parse(&["first", "--unknown", "last"]).repo.to_str(), Some("last"));
    assert_eq!(parse(&["--base", "--theme", "dark"]).base.as_deref(), Some("--theme"));
    let cfg = parse(&["--send-to", "cockpit", "/tmp/repo"]);
    assert!(cfg.launch_error.is_none());
    assert_eq!(cfg.repo.to_str(), Some("/tmp/repo"));
}

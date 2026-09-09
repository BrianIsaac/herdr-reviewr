mod common;
use common::Repo;
use herdr_reviewr::{
    briain::{Availability, Project, Run},
    config::Selector,
    pick::{Outcome, Picker},
    theme,
};
use ratatui::{Terminal, backend::TestBackend};

fn project(repo: &Repo, id: &str) -> Project {
    Project {
        id: Some(id.into()),
        note: format!("/{id}.md").into(),
        status: Some("active".into()),
        working_dir: Some(repo.path_buf()),
        availability: Availability::Available,
    }
}
fn run(repo: &Repo, id: &str, project: Option<&str>) -> Run {
    Run {
        id: id.into(),
        project: project.map(str::to_string),
        working_dir: repo.path_buf(),
        state: None,
        phase: None,
        availability: Availability::Available,
        metadata_issues: vec![],
    }
}
fn paint(picker: &Picker, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|f| picker.render(f, &theme::resolve(None).palette)).unwrap();
    terminal.backend().buffer().content.iter().map(ratatui::buffer::Cell::symbol).collect()
}

#[test]
fn projects_then_checkouts_enter_and_cancel() {
    let repo = Repo::init();
    let mut picker = Picker::new(vec![project(&repo, "a")], vec![run(&repo, "job", Some("a"))]);
    assert_eq!(picker.enter(), Outcome::Continue);
    assert!(paint(&picker, 80, 30).contains("Project checkout"));
    picker.goto(8); // invalid digit doesn't move to the last row
    assert_eq!(picker.cursor, 0);
    let Outcome::Selected(chosen) = picker.enter() else {
        panic!("project checkout");
    };
    assert_eq!(chosen.root, repo.path().canonicalize().unwrap());
    assert_eq!(chosen.identity.project.as_deref(), Some("a"));
    assert_eq!(chosen.identity.run, None);
    picker.move_by(1);
    let Outcome::Selected(chosen) = picker.enter() else {
        panic!("run checkout");
    };
    assert_eq!(chosen.identity.run.as_deref(), Some("job"));
    assert_eq!(picker.escape(), Outcome::Continue);
    assert_eq!(picker.escape(), Outcome::Cancelled);
}

#[test]
fn unknown_project_runs_are_accessible_without_invention() {
    let repo = Repo::init();
    let mut picker = Picker::new(vec![], vec![run(&repo, "job", None)]);
    assert_eq!(picker.enter(), Outcome::Continue);
    let Outcome::Selected(chosen) = picker.enter() else {
        panic!("unknown metadata is reviewable");
    };
    assert_eq!(chosen.identity.project, None);
    assert_eq!(chosen.identity.run.as_deref(), Some("job"));
}

#[test]
fn explicit_ids_refuse_traversal_duplicates_and_unavailability() {
    let repo = Repo::init();
    let p = project(&repo, "a");
    let picker =
        Picker::new(vec![p.clone(), p], vec![run(&repo, "job", None), run(&repo, "job", None)]);
    for selector in [
        Selector::Project("a".into()),
        Selector::Run("job".into()),
        Selector::Project("../a".into()),
        Selector::Run("/job".into()),
        Selector::Run(String::new()),
        Selector::Project("absent".into()),
    ] {
        assert!(picker.explicit(&selector).is_err());
    }
    let mut p = project(&repo, "a");
    p.availability = Availability::Unavailable("inactive project".into());
    let mut picker = Picker::new(vec![p], vec![]);
    assert!(
        picker
            .explicit(&Selector::Project("a".into()))
            .unwrap_err()
            .to_string()
            .contains("inactive")
    );
    assert_eq!(picker.enter(), Outcome::Continue);
    assert!(picker.status.contains("inactive"));
}

#[test]
fn enter_rechecks_roots_after_discovery() {
    let repo = Repo::init();
    let mut picker = Picker::new(vec![project(&repo, "a")], vec![]);
    picker.enter();
    std::fs::remove_dir_all(repo.path().join(".git")).unwrap();
    assert_eq!(picker.enter(), Outcome::Continue);
    assert!(picker.status.contains("not a usable git worktree"));
}

#[test]
fn rescan_matches_identity_then_nearest_survivor() {
    let repo = Repo::init();
    let a = project(&repo, "a");
    let b = project(&repo, "b");
    let c = project(&repo, "c");
    let mut picker = Picker::new(vec![a.clone(), b.clone(), c.clone()], vec![]);
    picker.goto(1);
    picker.reconcile(vec![c.clone(), a.clone(), b], vec![]);
    assert_eq!(picker.cursor, 2);
    picker.reconcile(vec![c, a], vec![]);
    assert_eq!(picker.cursor, 1); // a was the nearest surviving predecessor
    picker.move_by(-99);
    assert_eq!(picker.cursor, 0);
    picker.move_by(99);
    assert_eq!(picker.cursor, 2);
}

#[test]
fn render_narrow_tall_empty_and_unavailable_rows() {
    let repo = Repo::init();
    let mut bad = project(&repo, "offline");
    bad.availability = Availability::Unavailable("checkout missing".into());
    let mut picker = Picker::new(vec![bad], vec![]);
    assert!(paint(&picker, 80, 50).contains("checkout missing"));
    assert!(paint(&picker, 22, 12).contains("offline"));
    paint(&picker, 1, 1);
    paint(&Picker::new(vec![], vec![]), 0, 0);
    let projects = (0..40).map(|n| project(&repo, &format!("project-{n}"))).collect();
    picker.reconcile(projects, vec![]);
    picker.goto(39);
    assert!(paint(&picker, 70, 12).contains("project-39"));
    assert!(paint(&picker, 70, 60).contains("project-0"));
}

#[test]
fn mouse_hits_only_painted_rows_with_scroll_offset() {
    let repo = Repo::init();
    let mut picker =
        Picker::new((0..40).map(|n| project(&repo, &format!("p{n}"))).collect(), vec![]);
    picker.goto(30);
    let area = ratatui::layout::Rect::new(0, 0, 40, 12);
    assert_eq!(picker.hit(area, 5, 0), None);
    assert_eq!(picker.hit(area, 5, 7), None);
    assert_eq!(picker.hit(area, 5, 6), Some(30));
    picker.goto(picker.hit(area, 5, 1).unwrap());
    assert_eq!(picker.cursor, 25);
}

#[test]
fn project_identity_survives_note_rename() {
    let repo = Repo::init();
    let a = project(&repo, "a");
    let mut b = project(&repo, "b");
    let mut picker = Picker::new(vec![a.clone(), b.clone()], vec![]);
    picker.goto(1);
    b.note = "/renamed-note.md".into();
    picker.reconcile(vec![b, a], vec![]);
    assert_eq!(picker.cursor, 0);
}

#[test]
fn inputs_follow_keymap_digits_mouse_and_non_destructive_enter() {
    use herdr_reviewr::keymap::{Action, Key, Keymap};
    use herdr_reviewr::pick::Input;
    use ratatui::crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
        MouseEventKind,
    };
    let repo = Repo::init();
    let mut picker = Picker::new(vec![project(&repo, "a"), project(&repo, "b")], vec![]);
    let keys = Keymap::resolve(&[(Action::Down, vec![Key::plain('x')])]).unwrap();
    let area = ratatui::layout::Rect::new(0, 0, 40, 12);
    let press = |code| Event::Key(KeyEvent::new(code, KeyModifiers::NONE));
    picker.input(&press(KeyCode::Char('x')), area, &keys);
    assert_eq!(picker.cursor, 1);
    picker.input(&press(KeyCode::Char('1')), area, &keys);
    assert_eq!(picker.cursor, 0);
    picker.input(&press(KeyCode::Char('9')), area, &keys);
    assert_eq!(picker.cursor, 0);
    picker.input(
        &Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 2,
            modifiers: KeyModifiers::NONE,
        }),
        area,
        &keys,
    );
    assert_eq!(picker.cursor, 1);
    let mut release = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    release.kind = KeyEventKind::Release;
    assert_eq!(picker.input(&Event::Key(release), area, &keys), Input::Outcome(Outcome::Continue));
    // Unlike Send's destructive Enter guard, modifiers do not suppress opening a checkout menu.
    picker.input(&Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT)), area, &keys);
    assert!(paint(&picker, 80, 20).contains("Project checkout"));
    assert_eq!(picker.input(&press(KeyCode::Char('r')), area, &keys), Input::Rescan);
}

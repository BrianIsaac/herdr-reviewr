//! Sanitized live-layout fixtures, instantiated only inside temporary directories.
mod common;

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use common::Repo;
use herdr_reviewr::briain::{self, Availability};
use tempfile::TempDir;

fn put(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn note(root: &Path, file: &str, text: &str) {
    put(root, &format!("notes/projects/{file}.md"), text);
}

fn active(id: &str, path: &Path) -> String {
    format!("---\nid: {id}\nstatus: active\nworking_dir: {}\n---\n", path.display())
}

fn available(value: &Availability) -> bool {
    *value == Availability::Available
}

#[test]
fn data_root_precedence_and_missing_environment_are_injectable() {
    assert_eq!(
        briain::resolve_data_root(Some(OsStr::new("/override")), Some(OsStr::new("/home/test")))
            .unwrap(),
        Path::new("/override")
    );
    assert_eq!(
        briain::resolve_data_root(None, Some(OsStr::new("/Users/test"))).unwrap(),
        Path::new("/Users/test/.local/share/briain")
    );
    assert!(briain::resolve_data_root(None, None).is_err());
    assert!(
        briain::resolve_data_root(Some(OsStr::new("")), Some(OsStr::new("/home/test"))).is_err()
    );
}

#[test]
fn explicit_ids_cannot_traverse_or_supply_absolute_paths() {
    for id in ["", ".", "..", "../other", "/tmp/other", "a/b", "a\\b", "a\nb", " id"] {
        assert!(briain::validate_id(id).is_err(), "{id:?}");
    }
    for id in ["astraweave", "opaque-run-20260909", "project.with.dots"] {
        briain::validate_id(id).unwrap();
    }
}

#[test]
fn real_project_shapes_ignore_descriptions_and_body_keys() {
    let root = TempDir::new().unwrap();
    let repo = Repo::init();
    for (name, fixture) in [
        ("plain", include_str!("fixtures/briain/notes/projects/plain.md")),
        ("quoted", include_str!("fixtures/briain/notes/projects/quoted.md")),
        ("null", include_str!("fixtures/briain/notes/projects/null.md")),
    ] {
        note(root.path(), name, &fixture.replace("@CHECKOUT@", repo.path().to_str().unwrap()));
    }
    let projects = briain::projects(root.path()).unwrap();
    assert_eq!(projects.len(), 3);
    for id in ["astraweave", "herdr-reviewr"] {
        let project = projects.iter().find(|p| p.id.as_deref() == Some(id)).unwrap();
        assert!(available(&project.availability));
        assert_eq!(project.working_dir.as_deref(), Some(repo.path()));
    }
    let null = projects.iter().find(|p| p.id.as_deref() == Some("absent-path")).unwrap();
    assert_eq!(null.working_dir, None);
    assert!(!available(&null.availability));
}

#[test]
fn quoted_paths_support_spaces_colons_apostrophes_and_comments() {
    let root = TempDir::new().unwrap();
    let repo = Repo::init();
    repo.write("a", "a");
    repo.commit_all("initial");
    let path = root.path().join("a space: it's a checkout");
    repo.git(&["worktree", "add", "-q", "-b", "quoted", path.to_str().unwrap()]);
    let single = path.to_str().unwrap().replace('\'', "''");
    note(
        root.path(),
        "single",
        &format!(
            "---\nid: single\nstatus: active # comment\nworking_dir: '{single}' # comment\n---\n"
        ),
    );
    note(
        root.path(),
        "double",
        &format!(
            "---\nid: double\nstatus: active\nworking_dir: {} # comment\n---\n",
            serde_json::to_string(&path).unwrap()
        ),
    );
    assert!(briain::projects(root.path()).unwrap().iter().all(|p| available(&p.availability)));
}

#[test]
fn malformed_inactive_duplicate_and_unsupported_notes_stay_visible() {
    let root = TempDir::new().unwrap();
    let repo = Repo::init();
    note(root.path(), "a", &active("duplicate", repo.path()));
    note(
        root.path(),
        "b",
        &active("duplicate", repo.path()).replace("status: active", "status: inactive"),
    );
    note(
        root.path(),
        "inactive",
        &active("inactive", repo.path()).replace("status: active", "status: archived"),
    );
    note(
        root.path(),
        "malformed-duplicate",
        &active("duplicate", repo.path()).replace("status: active", "status: [bad]"),
    );
    let invalid = [
        "id: ../escape\nstatus: active\nworking_dir: /tmp",
        "id: bad\nstatus: active\nworking_dir: [a, b]",
        "id: bad\nstatus: active\nworking_dir: 'unterminated",
        "id: bad\nstatus: active\nworking_dir: relative/path",
        "id: bad\nstatus: active\nworking_dir: &anchor /tmp",
        "id: bad\nstatus: active\nworking_dir: |\n  /tmp",
        "id: bad\nstatus: active\nworking_dir: true",
        "id: bad\nstatus: active\nworking_dir: 123",
        "id: bad\nstatus: active\nworking_dir: \"/tmp\" trailing",
        "id: bad\nstatus: active\nworking_dir: \"/tmp\\nother\"",
        "id: bad\nid: second\nstatus: active",
        "id: bad\nstatus: active\nworking_dir: /tmp\nworking_dir: /other",
        "id: bad\nstatus: active\nworking_dir: {path: /tmp}",
    ];
    for (i, fields) in invalid.iter().enumerate() {
        note(root.path(), &format!("bad-{i}"), &format!("---\n{fields}\n---\n"));
    }
    note(root.path(), "no-open", "id: fake\nstatus: active\n");
    note(root.path(), "no-close", "---\nid: incomplete\n");
    let projects = briain::projects(root.path()).unwrap();
    assert_eq!(projects.len(), invalid.len() + 6);
    assert!(projects.iter().all(|p| !available(&p.availability)));
    for project in projects.iter().filter(|p| p.id.as_deref() == Some("duplicate")) {
        assert_eq!(project.availability, Availability::Unavailable("duplicate project id".into()));
    }
}

#[test]
fn missing_registry_and_stray_files_are_empty() {
    let root = TempDir::new().unwrap();
    assert!(briain::projects(root.path()).unwrap().is_empty());
    assert!(briain::runs(root.path()).unwrap().is_empty());
    put(root.path(), "notes/projects/a.zip", "ignored");
    fs::create_dir_all(root.path().join("notes/projects/directory.md")).unwrap();
    put(root.path(), "worktrees/stray.zip", "ignored");
    put(root.path(), "worktrees/stray.log", "ignored");
    put(root.path(), "runs/historical/job.yaml", "project: old");
    assert!(briain::projects(root.path()).unwrap().is_empty());
    assert!(briain::runs(root.path()).unwrap().is_empty());
}

fn linked_run(repo: &Repo, root: &Path, id: &str) {
    fs::create_dir_all(root.join("worktrees")).unwrap();
    repo.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        id,
        root.join("worktrees").join(id).to_str().unwrap(),
    ]);
}

#[test]
fn retained_worktrees_join_exact_metadata_and_never_infer_project() {
    let root = TempDir::new().unwrap();
    let repo = Repo::init();
    repo.write("a", "a");
    repo.commit_all("initial");
    linked_run(&repo, root.path(), "opaque-run");
    linked_run(&repo, root.path(), "astraweave-missing-metadata");
    put(
        root.path(),
        "runs/opaque-run/job.yaml",
        include_str!("fixtures/briain/runs/opaque-run/job.yaml"),
    );
    put(
        root.path(),
        "runs/opaque-run/status.json",
        include_str!("fixtures/briain/runs/opaque-run/status.json"),
    );
    put(root.path(), "runs/historical/job.yaml", "project: historical");
    put(root.path(), "worktrees/archive.zip", "stray");
    let before = repo.git(&["show-ref"]);
    let runs = briain::runs(root.path()).unwrap();
    assert_eq!(runs.len(), 2);
    let known = runs.iter().find(|r| r.id == "opaque-run").unwrap();
    assert_eq!(known.project.as_deref(), Some("herdr-reviewr"));
    assert_eq!(known.state.as_deref(), Some("green"));
    assert_eq!(known.phase.as_deref(), Some("interactive"));
    assert!(known.metadata_issues.is_empty());
    let unknown = runs.iter().find(|r| r.id == "astraweave-missing-metadata").unwrap();
    assert_eq!((&unknown.project, &unknown.state, &unknown.phase), (&None, &None, &None));
    assert_eq!(unknown.metadata_issues.len(), 2);
    assert!(runs.iter().all(|r| available(&r.availability)));
    assert_eq!(repo.git(&["show-ref"]), before, "No writes");
}

#[test]
fn pruned_linked_worktree_is_retained_and_revalidation_refuses_it() {
    let root = TempDir::new().unwrap();
    let repo = Repo::init();
    repo.write("a", "a");
    repo.commit_all("initial");
    linked_run(&repo, root.path(), "pruned");
    let path = root.path().join("worktrees/pruned");
    assert!(available(&briain::checkout_availability(&path)));
    let link = fs::read_to_string(path.join(".git")).unwrap();
    let git_dir = link.trim().strip_prefix("gitdir: ").unwrap();
    fs::remove_dir_all(git_dir).unwrap();
    let runs = briain::runs(root.path()).unwrap();
    assert_eq!(runs.len(), 1);
    assert!(
        matches!(&runs[0].availability, Availability::Unavailable(reason) if reason.contains("pruned"))
    );
    assert!(!available(&briain::checkout_availability(&path)));
}

#[test]
fn malformed_run_metadata_is_unknown_without_disabling_valid_checkout() {
    let root = TempDir::new().unwrap();
    let repo = Repo::init();
    repo.write("a", "a");
    repo.commit_all("initial");
    linked_run(&repo, root.path(), "project-looking-id");
    for (job, status) in [
        ("project: ../escape", r#"{"state":42,"phase":[],"project":"false-association"}"#),
        ("project: [unsupported]", "not json"),
        ("project: null", r#"{"state":null,"phase":null}"#),
        ("description: 'project: false'\n  project: nested", "{}"),
    ] {
        put(root.path(), "runs/project-looking-id/job.yaml", job);
        put(root.path(), "runs/project-looking-id/status.json", status);
        let run = briain::runs(root.path()).unwrap().remove(0);
        assert!(available(&run.availability));
        assert_eq!((run.project, run.state, run.phase), (None, None, None));
    }
}

#[test]
fn checkout_must_be_root_and_metadata_is_bounded() {
    let root = TempDir::new().unwrap();
    let repo = Repo::init();
    fs::create_dir(repo.path().join("nested")).unwrap();
    assert!(!available(&briain::checkout_availability(&repo.path().join("nested"))));
    note(
        root.path(),
        "large",
        &format!("---\nid: large\ndescription: {}\n---\n", "x".repeat(256 * 1024)),
    );
    assert!(
        matches!(&briain::projects(root.path()).unwrap()[0].availability, Availability::Unavailable(reason) if reason.contains("exceeds"))
    );
}

#[test]
fn only_opening_frontmatter_is_read_and_null_variants_are_absent() {
    let root = TempDir::new().unwrap();
    let repo = Repo::init();
    note(
        root.path(),
        "large-body",
        &format!("{}{}", active("large-body", repo.path()), "x".repeat(300 * 1024)),
    );
    let project = briain::projects(root.path()).unwrap().remove(0);
    assert!(available(&project.availability));
    for value in ["", "~", "null", "NULL", "Null", "null # absent"] {
        note(
            root.path(),
            "absent",
            &format!("---\nid: absent\nstatus: active\nworking_dir: {value}\n---\n"),
        );
        let projects = briain::projects(root.path()).unwrap();
        let absent = projects.iter().find(|p| p.id.as_deref() == Some("absent")).unwrap();
        assert_eq!(absent.working_dir, None);
        assert!(!available(&absent.availability));
    }
}

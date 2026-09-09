//! Read-only briain registry discovery. Paths are injectable; no live registry is needed by tests.
//!
//! Only opening project frontmatter and top-level job scalars are supported, not general YAML.
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::git;

const MAX_METADATA_BYTES: u64 = 256 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Availability {
    Available,
    Unavailable(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    /// None for a malformed note whose identity cannot safely be read.
    pub id: Option<String>,
    pub note: PathBuf,
    pub status: Option<String>,
    pub working_dir: Option<PathBuf>,
    pub availability: Availability,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub id: String,
    pub working_dir: PathBuf,
    pub project: Option<String>,
    pub state: Option<String>,
    pub phase: Option<String>,
    pub availability: Availability,
    /// Missing or malformed metadata does not make a valid checkout unreviewable.
    pub metadata_issues: Vec<String>,
}

/// `BRIAIN_DATA_DIR`, otherwise HOME's Unix data location on both Linux and macOS.
/// No process environment mutation is needed to exercise precedence in tests.
pub fn resolve_data_root(override_dir: Option<&OsStr>, home: Option<&OsStr>) -> Result<PathBuf> {
    if let Some(path) = override_dir {
        if path.is_empty() {
            bail!("BRIAIN_DATA_DIR is empty");
        }
        return Ok(PathBuf::from(path));
    }
    let Some(home) = home.filter(|path| !path.is_empty()) else {
        bail!("HOME is unavailable; set BRIAIN_DATA_DIR");
    };
    Ok(PathBuf::from(home).join(".local/share/briain"))
}

pub fn data_root() -> Result<PathBuf> {
    resolve_data_root(
        std::env::var_os("BRIAIN_DATA_DIR").as_deref(),
        std::env::var_os("HOME").as_deref(),
    )
}

/// Validate before joining any explicit selector to the registry root.
pub fn validate_id(id: &str) -> Result<()> {
    if id.is_empty()
        || id == "."
        || id == ".."
        || id.contains(['/', '\\'])
        || id.chars().any(char::is_control)
        || id.trim() != id
    {
        bail!("id must be a non-empty basename");
    }
    Ok(())
}

/// Recheck on selection, not just discovery. A bare repository or a directory merely inside
/// another checkout is not a registered checkout root. Broken linked-worktree git files fail here.
pub fn checkout_availability(path: &Path) -> Availability {
    let Ok(canonical) = path.canonicalize() else {
        return Availability::Unavailable("checkout path is missing or unreadable".into());
    };
    match git::toplevel(path).and_then(|root| root.canonicalize().ok()) {
        Some(root) if root == canonical => Availability::Available,
        _ => Availability::Unavailable("not a usable git worktree root (possibly pruned)".into()),
    }
}

fn read_metadata(path: &Path) -> Result<String> {
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(MAX_METADATA_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_METADATA_BYTES {
        bail!("metadata exceeds {MAX_METADATA_BYTES} bytes");
    }
    Ok(String::from_utf8(bytes)?)
}

fn read_note(path: &Path) -> Result<String> {
    let reader = BufReader::new(fs::File::open(path)?.take(MAX_METADATA_BYTES + 1));
    let mut text = String::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        text.push_str(&line);
        text.push('\n');
        if text.len() as u64 > MAX_METADATA_BYTES {
            bail!("frontmatter exceeds {MAX_METADATA_BYTES} bytes");
        }
        if (i == 0 && line != "---") || (i > 0 && line == "---") {
            break;
        }
    }
    Ok(text)
}

/// A deliberately bounded YAML scalar subset: plain, single-quoted (doubled quotes),
/// JSON-compatible double-quoted, and null. Unsupported relevant values are errors.
fn scalar(raw: &str) -> Result<Option<String>> {
    let raw = raw.trim();
    let value = if raw.starts_with('"') {
        let mut stream = serde_json::Deserializer::from_str(raw).into_iter::<String>();
        let value = stream.next().transpose()?.ok_or_else(|| anyhow::anyhow!("missing scalar"))?;
        let tail = raw[stream.byte_offset()..].trim();
        if !tail.is_empty() && !tail.starts_with('#') {
            bail!("text after quoted scalar");
        }
        value
    } else if let Some(rest) = raw.strip_prefix('\'') {
        let mut chars = rest.char_indices().peekable();
        let mut value = String::new();
        let mut closed = false;
        while let Some((i, ch)) = chars.next() {
            if ch == '\'' {
                if chars.peek().is_some_and(|(_, next)| *next == '\'') {
                    chars.next();
                    value.push('\'');
                } else {
                    let tail = rest[i + 1..].trim();
                    if !tail.is_empty() && !tail.starts_with('#') {
                        bail!("text after quoted scalar");
                    }
                    closed = true;
                    break;
                }
            } else {
                value.push(ch);
            }
        }
        if !closed {
            bail!("unterminated quoted scalar");
        }
        value
    } else {
        let end = raw
            .char_indices()
            .find_map(|(i, ch)| {
                (ch == '#' && (i == 0 || raw[..i].ends_with(char::is_whitespace))).then_some(i)
            })
            .unwrap_or(raw.len());
        let plain = raw[..end].trim();
        if matches!(plain, "" | "~" | "null" | "Null" | "NULL") {
            return Ok(None);
        }
        if plain.starts_with([
            '[', ']', '{', '}', '&', '*', '!', '|', '>', '%', '@', '`', '#', ',', '?',
        ]) || plain == "-"
            || plain.starts_with("- ")
            || plain.contains(": ")
            || plain.ends_with(':')
            || matches!(plain, "true" | "false" | "True" | "False" | "TRUE" | "FALSE")
            || plain.parse::<f64>().is_ok()
        {
            bail!("unsupported scalar value");
        }
        plain.to_string()
    };
    if value.chars().any(char::is_control) {
        bail!("control character in scalar");
    }
    Ok(Some(value))
}

fn fields(
    text: &str,
    frontmatter: bool,
    keys: &[&str],
) -> Result<BTreeMap<String, Option<String>>> {
    let mut lines = text.lines();
    if frontmatter && lines.next() != Some("---") {
        bail!("missing opening frontmatter");
    }
    let mut values = BTreeMap::new();
    let mut closed = !frontmatter;
    for line in lines {
        if frontmatter && line == "---" {
            closed = true;
            break;
        }
        // Nested/body/description contents never become top-level keys.
        if line.starts_with(char::is_whitespace) {
            continue;
        }
        let Some((key, raw)) = line.split_once(':') else {
            if keys.iter().any(|key| line.trim() == *key) {
                bail!("malformed relevant field");
            }
            continue;
        };
        let key = key.trim();
        if !keys.contains(&key) {
            continue;
        }
        let value = scalar(raw).map_err(|err| anyhow::anyhow!("{key}: {err}"))?;
        if values.insert(key.into(), value).is_some() {
            bail!("duplicate field {key}");
        }
    }
    if !closed {
        bail!("unterminated opening frontmatter");
    }
    Ok(values)
}

fn entries(path: &Path) -> Result<Vec<fs::DirEntry>> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };
    let mut entries = entries.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    Ok(entries)
}

pub fn projects(root: &Path) -> Result<Vec<Project>> {
    let mut projects = Vec::new();
    for entry in entries(&root.join("notes/projects"))? {
        if !entry.file_type()?.is_file() || entry.path().extension() != Some(OsStr::new("md")) {
            continue;
        }
        let mut project = Project {
            id: None,
            note: entry.path(),
            status: None,
            working_dir: None,
            availability: Availability::Available,
        };
        let parsed = (|| -> Result<()> {
            let text = read_note(&entry.path())?;
            // Keep an unambiguous identity even if a different relevant field is malformed,
            // so an invalid duplicate cannot leave another row arbitrarily selectable.
            project.id = fields(&text, true, &["id"])?.remove("id").flatten();
            if let Some(id) = &project.id {
                validate_id(id)?;
            }
            let mut fields = fields(&text, true, &["id", "status", "working_dir"])?;
            let id = fields.remove("id").flatten().ok_or_else(|| anyhow::anyhow!("missing id"))?;
            validate_id(&id)?;
            project.id = Some(id);
            project.status = fields.remove("status").flatten();
            project.working_dir = fields.remove("working_dir").flatten().map(PathBuf::from);
            if project.status.as_deref() != Some("active") {
                bail!("project is not active");
            }
            let path = project
                .working_dir
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("missing working_dir"))?;
            if !path.is_absolute() {
                bail!("working_dir must be absolute");
            }
            project.availability = checkout_availability(path);
            Ok(())
        })();
        if let Err(err) = parsed {
            project.availability = Availability::Unavailable(err.to_string());
        }
        projects.push(project);
    }
    let mut counts = BTreeMap::new();
    for project in &projects {
        if let Some(id) = &project.id {
            *counts.entry(id.clone()).or_insert(0) += 1;
        }
    }
    for project in &mut projects {
        if project.id.as_ref().is_some_and(|id| counts[id] > 1) {
            project.availability = Availability::Unavailable("duplicate project id".into());
        }
    }
    Ok(projects)
}

pub fn runs(root: &Path) -> Result<Vec<Run>> {
    let mut runs = Vec::new();
    // Deliberately never enumerate runs/: it contains all historical jobs.
    for entry in entries(&root.join("worktrees"))? {
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let Ok(id) = entry.file_name().into_string() else {
            continue;
        };
        validate_id(&id)?;
        let metadata = root.join("runs").join(&id);
        let mut run = Run {
            id,
            working_dir: entry.path(),
            project: None,
            state: None,
            phase: None,
            availability: checkout_availability(&entry.path()),
            metadata_issues: Vec::new(),
        };
        match read_metadata(&metadata.join("job.yaml"))
            .and_then(|text| fields(&text, false, &["project"]))
        {
            Ok(mut fields) => {
                if let Some(project) = fields.remove("project").flatten() {
                    if validate_id(&project).is_ok() {
                        run.project = Some(project);
                    } else {
                        run.metadata_issues.push("job.yaml: invalid project id".into());
                    }
                }
            }
            Err(err) => run.metadata_issues.push(format!("job.yaml: {err}")),
        }
        match read_metadata(&metadata.join("status.json"))
            .and_then(|text| Ok(serde_json::from_str::<serde_json::Value>(&text)?))
        {
            Ok(value) if value.is_object() => {
                for (key, target) in [("state", &mut run.state), ("phase", &mut run.phase)] {
                    match value.get(key) {
                        Some(serde_json::Value::String(s))
                            if !s.is_empty() && !s.chars().any(char::is_control) =>
                        {
                            *target = Some(s.clone());
                        }
                        None | Some(serde_json::Value::Null) => (),
                        _ => run.metadata_issues.push(format!("status.json: invalid {key}")),
                    }
                }
            }
            Ok(_) => run.metadata_issues.push("status.json: expected object".into()),
            Err(err) => run.metadata_issues.push(format!("status.json: {err}")),
        }
        runs.push(run);
    }
    Ok(runs)
}

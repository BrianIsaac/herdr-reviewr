//! Pre-App selection. Discovery and validation are read-only; menu place reconciles by identity.
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::{
    briain::{self, Availability, Project, Run},
    config::Selector,
    theme::Palette,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewIdentity {
    pub project: Option<String>,
    pub run: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub root: PathBuf,
    pub identity: ReviewIdentity,
}

impl Selection {
    /// Validate the canonical root, then carry that exact root past config recovery.
    pub fn revalidate(&self) -> Result<Self> {
        let root = self.root.canonicalize()?;
        if let Availability::Unavailable(reason) = briain::checkout_availability(&root) {
            bail!("{reason}");
        }
        Ok(Self { root, identity: self.identity.clone() })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Level {
    Projects,
    Runs(Option<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Target {
    Project(String),
    Run(String),
    AllRuns,
}

#[derive(Clone, Debug)]
struct Row {
    target: Target,
    label: String,
    reason: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Picker {
    projects: Vec<Project>,
    runs: Vec<Run>,
    level: Level,
    rows: Vec<Row>,
    pub cursor: usize,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Selected(Selection),
    Cancelled,
}

impl Picker {
    pub fn new(projects: Vec<Project>, runs: Vec<Run>) -> Self {
        let mut picker = Self {
            projects,
            runs,
            level: Level::Projects,
            rows: Vec::new(),
            cursor: 0,
            status: String::new(),
        };
        picker.rebuild();
        picker
    }

    pub fn scan(root: &Path) -> Result<Self> {
        Ok(Self::new(briain::projects(root)?, briain::runs(root)?))
    }

    /// Continuity: identity first, nearest surviving neighbour second, clamp last.
    pub fn reconcile(&mut self, projects: Vec<Project>, runs: Vec<Run>) {
        let old = self.rows.iter().map(|row| row.target.clone()).collect::<Vec<_>>();
        self.projects = projects;
        self.runs = runs;
        self.rebuild();
        let mut neighbours: Vec<_> = old.iter().enumerate().collect();
        neighbours.sort_by_key(|(i, _)| i.abs_diff(self.cursor));
        self.cursor = neighbours
            .into_iter()
            .find_map(|(_, target)| self.rows.iter().position(|row| &row.target == target))
            .unwrap_or(self.cursor.min(self.rows.len().saturating_sub(1)));
    }

    fn rebuild(&mut self) {
        self.rows.clear();
        for project in &self.projects {
            if self.level != Level::Projects
                && (project.id.is_none() || self.level != Level::Runs(project.id.clone()))
            {
                continue;
            }
            let label = if self.level == Level::Projects {
                project.id.clone().unwrap_or_else(|| project.note.display().to_string())
            } else {
                "Project checkout".into()
            };
            self.rows.push(Row {
                target: project_target(project),
                label,
                reason: unavailable(&project.availability),
            });
        }
        match &self.level {
            Level::Projects => self.rows.push(Row {
                target: Target::AllRuns,
                label: "All retained runs".into(),
                reason: None,
            }),
            Level::Runs(project) => {
                for run in &self.runs {
                    if project.is_some() && &run.project != project {
                        continue;
                    }
                    let trail = run.state.as_deref().unwrap_or("unknown");
                    self.rows.push(Row {
                        target: Target::Run(run.id.clone()),
                        label: format!("{} · {trail}", run.id),
                        reason: unavailable(&run.availability),
                    });
                }
            }
        }
    }

    pub fn move_by(&mut self, delta: isize) {
        self.cursor =
            self.cursor.saturating_add_signed(delta).min(self.rows.len().saturating_sub(1));
    }

    /// Digits and clicks only highlight; Enter opens. Invalid digits are inert.
    pub fn goto(&mut self, index: usize) {
        if index < self.rows.len() {
            self.cursor = index;
        }
    }

    pub fn escape(&mut self) -> Outcome {
        if self.level == Level::Projects {
            return Outcome::Cancelled;
        }
        let previous = match &self.level {
            Level::Runs(Some(id)) => {
                self.projects.iter().find(|p| p.id.as_ref() == Some(id)).map(project_target)
            }
            _ => Some(Target::AllRuns),
        };
        self.level = Level::Projects;
        self.rebuild();
        self.cursor =
            self.rows.iter().position(|r| Some(&r.target) == previous.as_ref()).unwrap_or(0);
        self.status.clear();
        Outcome::Continue
    }

    pub fn enter(&mut self) -> Outcome {
        let Some(row) = self.rows.get(self.cursor).cloned() else {
            return Outcome::Continue;
        };
        if let Some(reason) = row.reason {
            self.status = reason;
            return Outcome::Continue;
        }
        let result = match row.target {
            Target::AllRuns => {
                self.level = Level::Runs(None);
                self.rebuild();
                self.cursor = 0;
                return Outcome::Continue;
            }
            Target::Project(identity) => {
                let Some(project) = self
                    .projects
                    .iter()
                    .find(|p| project_target(p) == Target::Project(identity.clone()))
                else {
                    return Outcome::Continue;
                };
                let Some(id) = project.id.clone() else {
                    return Outcome::Continue;
                };
                if self.level == Level::Projects {
                    self.level = Level::Runs(Some(id));
                    self.rebuild();
                    self.cursor = 0;
                    self.status.clear();
                    return Outcome::Continue;
                }
                self.explicit(&Selector::Project(id))
            }
            Target::Run(id) => self.explicit(&Selector::Run(id)),
        };
        match result {
            Ok(selection) => Outcome::Selected(selection),
            Err(error) => {
                self.status = error.to_string();
                Outcome::Continue
            }
        }
    }

    pub fn explicit(&self, selector: &Selector) -> Result<Selection> {
        match selector {
            Selector::Project(id) => {
                briain::validate_id(id)?;
                let matches: Vec<_> =
                    self.projects.iter().filter(|p| p.id.as_ref() == Some(id)).collect();
                if matches.len() != 1 {
                    bail!("project {id}: expected one registered match, found {}", matches.len());
                }
                let project = matches[0];
                if let Some(reason) = unavailable(&project.availability) {
                    bail!("project {id}: {reason}");
                }
                let root = project
                    .working_dir
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("project {id}: no checkout"))?;
                Selection {
                    root,
                    identity: ReviewIdentity { project: Some(id.clone()), run: None },
                }
                .revalidate()
            }
            Selector::Run(id) => {
                briain::validate_id(id)?;
                let matches: Vec<_> = self.runs.iter().filter(|r| &r.id == id).collect();
                if matches.len() != 1 {
                    bail!("run {id}: expected one retained match, found {}", matches.len());
                }
                let run = matches[0];
                if let Some(reason) = unavailable(&run.availability) {
                    bail!("run {id}: {reason}");
                }
                Selection {
                    root: run.working_dir.clone(),
                    identity: ReviewIdentity {
                        project: run.project.clone(),
                        run: Some(id.clone()),
                    },
                }
                .revalidate()
            }
            Selector::Pick => bail!("choose a project or run"),
        }
    }

    fn geometry(&self, area: Rect) -> (Rect, Rect, usize) {
        let width = self
            .rows
            .iter()
            .map(|r| Line::from(row_text(r, 0)).width() + 3)
            .max()
            .unwrap_or(34)
            .max(34)
            .min(usize::from(area.width)) as u16;
        let height = (self.rows.len() + 2).min(usize::from(area.height.saturating_sub(4))) as u16;
        let popup = Rect::new(area.x + area.width.saturating_sub(width) / 2, area.y, width, height);
        let inner = Block::default().borders(Borders::ALL).inner(popup);
        let rows = usize::from(inner.height);
        let first = if rows == 0 {
            0
        } else {
            (self.cursor + 1).saturating_sub(rows).min(self.rows.len().saturating_sub(rows))
        };
        (popup, inner, first)
    }

    pub fn hit(&self, area: Rect, col: u16, row: u16) -> Option<usize> {
        let (_, inner, first) = self.geometry(area);
        if !inner.contains((col, row).into()) {
            return None;
        }
        let index = first + usize::from(row - inner.y);
        (index < self.rows.len()).then_some(index)
    }

    pub fn render(&self, frame: &mut Frame, palette: &Palette) {
        self.render_with_keys(frame, palette, crate::keymap::default_keymap());
    }

    pub fn render_with_keys(
        &self,
        frame: &mut Frame,
        palette: &Palette,
        keys: &crate::keymap::Keymap,
    ) {
        use crate::keymap::Action;
        let up = keys.hint(Action::Up).label();
        let down = keys.hint(Action::Down).label();
        let refresh = keys.hint(Action::Refresh).label();
        let area = frame.area();
        let (popup, inner, first) = self.geometry(area);
        let title = match &self.level {
            Level::Projects => " projects ".into(),
            Level::Runs(Some(p)) => format!(" {p} · checkouts "),
            Level::Runs(None) => " retained runs ".into(),
        };
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(palette.purple)),
            popup,
        );
        let items: Vec<_> = self
            .rows
            .iter()
            .enumerate()
            .skip(first)
            .take(usize::from(inner.height))
            .map(|(i, row)| {
                let mut style = Style::default().fg(if row.reason.is_some() {
                    palette.dim0
                } else {
                    palette.text
                });
                if i == self.cursor {
                    style = style.bg(palette.surface2);
                }
                ListItem::new(row_text(row, i)).style(style)
            })
            .collect();
        frame.render_widget(List::new(items), inner);
        let reason = self.rows.get(self.cursor).and_then(|r| r.reason.as_deref()).unwrap_or("");
        let footer = Rect::new(
            area.x,
            popup.bottom(),
            area.width,
            area.bottom().saturating_sub(popup.bottom()),
        );
        frame.render_widget(Paragraph::new(format!("{up}/{down} move · 1–9 select · Enter open · Esc back/cancel · {refresh} rescan\n{reason}\n{}", self.status)).wrap(Wrap { trim: false }).style(Style::default().fg(palette.dim0)), footer);
    }
}

fn project_target(project: &Project) -> Target {
    Target::Project(
        project
            .id
            .as_ref()
            .map_or_else(|| format!("note:{}", project.note.display()), |id| format!("id:{id}")),
    )
}

fn unavailable(value: &Availability) -> Option<String> {
    match value {
        Availability::Available => None,
        Availability::Unavailable(reason) => Some(reason.clone()),
    }
}

fn row_text(row: &Row, index: usize) -> String {
    let lead = if index < 9 { format!(" {}  ", index + 1) } else { "    ".into() };
    let reason = row.reason.as_ref().map_or(String::new(), |reason| format!(" · {reason}"));
    // Registry metadata is text, never terminal control input.
    format!("{lead}{}{reason}", row.label)
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

/// Startup uses the same terminal ownership as review, with no placeholder App.
pub fn render_startup(frame: &mut Frame, message: &str) {
    frame.render_widget(
        Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title(" reviewr "))
            .wrap(Wrap { trim: false }),
        frame.area(),
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Outcome(Outcome),
    Rescan,
}

impl Picker {
    pub fn input(
        &mut self,
        event: &ratatui::crossterm::event::Event,
        area: Rect,
        keymap: &crate::keymap::Keymap,
    ) -> Input {
        use crate::keymap::{Action, Key, KeyCode as Code};
        use ratatui::crossterm::event::{
            Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
        };
        match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                match key.code {
                    KeyCode::Esc => return Input::Outcome(self.escape()),
                    KeyCode::Enter => return Input::Outcome(self.enter()),
                    KeyCode::Char(c @ '1'..='9') if key.modifiers.is_empty() => {
                        self.goto(c as usize - '1' as usize);
                        return Input::Outcome(Outcome::Continue);
                    }
                    _ => {}
                }
                let code = match key.code {
                    KeyCode::Char(c) => Some(Code::Char(c)),
                    KeyCode::Up => Some(Code::Up),
                    KeyCode::Down => Some(Code::Down),
                    KeyCode::PageUp => Some(Code::PageUp),
                    KeyCode::PageDown => Some(Code::PageDown),
                    _ => None,
                };
                let action = code.and_then(|code| {
                    keymap.action_for(Key {
                        code,
                        ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
                        alt: key.modifiers.contains(KeyModifiers::ALT),
                    })
                });
                match action {
                    Some(Action::Down) => self.move_by(1),
                    Some(Action::Up) => self.move_by(-1),
                    Some(Action::PageDown) => self.move_by(10),
                    Some(Action::PageUp) => self.move_by(-10),
                    Some(Action::Refresh) => return Input::Rescan,
                    _ => {}
                }
            }
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    if let Some(index) = self.hit(area, mouse.column, mouse.row) {
                        self.goto(index);
                    }
                }
                MouseEventKind::ScrollDown => self.move_by(1),
                MouseEventKind::ScrollUp => self.move_by(-1),
                _ => {}
            },
            _ => {}
        }
        Input::Outcome(Outcome::Continue)
    }
}

//! OpenSpec changes the agent works on, as plans: their documents in the order and with the
//! names of the change's schema, and their task progress (`viewer-views` design decision 3). Read
//! from the OpenSpec CLI on the host, else from OpenSpec's default layout.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use serde_json::Value;
use specdeck_protocol::timeline::{Entry, PlanDocument, TaskProgress};

use crate::environment::Environment;

/// How long one OpenSpec CLI call may take before the default layout is used instead.
const CLI_LIMIT: Duration = Duration::from_secs(5);
/// A burst of writes (as during `/opsx:propose`) is read once, this long after the last one.
pub const DEBOUNCE: Duration = Duration::from_millis(300);

/// An OpenSpec change: the folder holding `openspec/` and the change's name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChangeRef {
    pub root: PathBuf,
    pub name: String,
}

/// The change an absolute file path belongs to; none for archived changes or other files.
pub fn change_of(path: &Path) -> Option<ChangeRef> {
    let parts: Vec<Component> = path.components().collect();
    let at = parts.windows(2).position(|pair| {
        matches!(pair, [Component::Normal(a), Component::Normal(b)] if *a == "openspec" && *b == "changes")
    })?;
    let Some(Component::Normal(name)) = parts.get(at + 2) else { return None };
    // Only files inside the change's folder, not the folder itself.
    if *name == "archive" || parts.len() <= at + 3 {
        return None;
    }
    Some(ChangeRef {
        root: parts[..at].iter().collect(),
        name: name.to_string_lossy().into_owned(),
    })
}

/// Reads plans for the session in `folder`; paths in them are shown relative to it.
pub struct Plans {
    folder: PathBuf,
    env: Environment,
}

impl Plans {
    pub fn new(folder: &Path, env: &Environment) -> Self {
        Self { folder: folder.to_owned(), env: env.clone() }
    }

    /// The plan of `change`, from the CLI when it answers, else from the default layout.
    pub fn read(&self, change: &ChangeRef) -> Entry {
        let (documents, tasks) =
            self.read_cli(change).unwrap_or_else(|| self.read_default_layout(change));
        Entry::Plan { change: change.name.clone(), documents, tasks }
    }

    /// Reads plans for the changes sent on `changes` until the sender is gone, once per burst,
    /// and hands each to `emit`. Runs on its own thread, off the hook path.
    pub fn run(&self, changes: Receiver<ChangeRef>, emit: impl Fn(Entry)) {
        while let Ok(first) = changes.recv() {
            let mut pending = BTreeSet::from([first]);
            let mut quiet_until = Instant::now() + DEBOUNCE;
            loop {
                match changes.recv_timeout(quiet_until.saturating_duration_since(Instant::now())) {
                    Ok(change) => {
                        pending.insert(change);
                        quiet_until = Instant::now() + DEBOUNCE;
                    }
                    Err(RecvTimeoutError::Timeout) => break,
                    Err(RecvTimeoutError::Disconnected) => {
                        pending.iter().for_each(|change| emit(self.read(change)));
                        return;
                    }
                }
            }
            pending.iter().for_each(|change| emit(self.read(change)));
        }
    }

    fn read_cli(&self, change: &ChangeRef) -> Option<(Vec<PlanDocument>, TaskProgress)> {
        let status =
            self.openspec(&change.root, &["status", "--change", &change.name, "--json"])?;
        let paths = status.get("artifactPaths")?;
        let documents = status
            .get("artifacts")?
            .as_array()?
            .iter()
            .filter(|artifact| artifact.get("status").and_then(Value::as_str) != Some("skipped"))
            .filter_map(|artifact| {
                let id = artifact.get("id")?.as_str()?;
                let files: Vec<String> = paths
                    .get(id)
                    .and_then(|p| p.get("existingOutputPaths"))
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(|path| self.display(Path::new(path)))
                    .collect();
                Some(PlanDocument {
                    id: id.to_owned(),
                    name: readable(id),
                    exists: !files.is_empty(),
                    paths: files,
                })
            })
            .collect();
        let list = self.openspec(&change.root, &["list", "--json"])?;
        let entry = list
            .get("changes")?
            .as_array()?
            .iter()
            .find(|entry| entry.get("name").and_then(Value::as_str) == Some(&change.name))?;
        let count = |key: &str| entry.get(key).and_then(Value::as_u64).map(|n| n as u32);
        let tasks = TaskProgress { done: count("completedTasks")?, total: count("totalTasks")? };
        Some((documents, tasks))
    }

    /// Runs the OpenSpec CLI in `root` and parses its JSON output; `None` when it is missing,
    /// fails, takes longer than `CLI_LIMIT` or prints something else.
    fn openspec(&self, root: &Path, args: &[&str]) -> Option<Value> {
        let cli = self.env.find_executable("openspec")?;
        let mut child = Command::new(cli)
            .args(args)
            .current_dir(root)
            .env_clear()
            .envs(self.env.iter())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let mut stdout = child.stdout.take()?;
        let reader = std::thread::spawn(move || {
            let mut out = Vec::new();
            std::io::Read::read_to_end(&mut stdout, &mut out).map(|_| out)
        });
        let deadline = Instant::now() + CLI_LIMIT;
        let status = loop {
            if let Some(status) = child.try_wait().ok()? {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        let out = reader.join().ok()?.ok()?;
        status.success().then(|| serde_json::from_slice(&out).ok()).flatten()
    }

    /// OpenSpec's default schema: proposal, specs, design and tasks, with tasks counted from
    /// `tasks.md` the way OpenSpec counts them.
    fn read_default_layout(&self, change: &ChangeRef) -> (Vec<PlanDocument>, TaskProgress) {
        let dir = change.root.join("openspec").join("changes").join(&change.name);
        let single = |id: &str, file: &str| {
            let path = dir.join(file);
            let exists = path.is_file();
            PlanDocument {
                id: id.into(),
                name: readable(id),
                paths: if exists { vec![self.display(&path)] } else { Vec::new() },
                exists,
            }
        };
        let mut specs = Vec::new();
        collect_specs(&dir.join("specs"), &mut specs);
        specs.sort();
        let specs: Vec<String> = specs.iter().map(|path| self.display(path)).collect();
        let documents = vec![
            single("proposal", "proposal.md"),
            PlanDocument {
                id: "specs".into(),
                name: "Specs".into(),
                exists: !specs.is_empty(),
                paths: specs,
            },
            single("design", "design.md"),
            single("tasks", "tasks.md"),
        ];
        let text = std::fs::read_to_string(dir.join("tasks.md")).unwrap_or_default();
        (documents, count_tasks(&text))
    }

    /// Relative to the session folder when inside it, as file changes are shown.
    fn display(&self, path: &Path) -> String {
        let canonical_folder = self.folder.canonicalize().ok();
        let inside = path.strip_prefix(&self.folder).ok().or_else(|| {
            canonical_folder.as_deref().and_then(|folder| path.strip_prefix(folder).ok())
        });
        inside.unwrap_or(path).to_string_lossy().into_owned()
    }
}

/// Every `spec.md` below `dir`.
fn collect_specs(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_specs(&path, found);
        } else if path.file_name().is_some_and(|name| name == "spec.md") {
            found.push(path);
        }
    }
}

/// Checkbox tasks: `- [x]` or `- [X]` done, any other marker open.
fn count_tasks(text: &str) -> TaskProgress {
    let (mut done, mut total) = (0, 0);
    for line in text.lines() {
        let Some(rest) = line.trim_start().strip_prefix("- [") else { continue };
        let Some((marker, _)) = rest.split_once(']') else { continue };
        total += 1;
        if marker.trim().eq_ignore_ascii_case("x") {
            done += 1;
        }
    }
    TaskProgress { done, total }
}

/// "feature-brief" → "Feature brief".
fn readable(id: &str) -> String {
    let words = id.replace(['-', '_'], " ");
    let mut chars = words.chars();
    chars.next().map(|first| first.to_uppercase().chain(chars).collect()).unwrap_or_default()
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    struct Project {
        dir: tempfile::TempDir,
        bin: tempfile::TempDir,
    }

    impl Project {
        /// A project with change `x` in the default layout: a proposal, one spec, 1 of 3 tasks.
        fn new() -> Self {
            let project =
                Self { dir: tempfile::tempdir().unwrap(), bin: tempfile::tempdir().unwrap() };
            let change = project.dir.path().join("openspec/changes/x");
            std::fs::create_dir_all(change.join("specs/hosting")).unwrap();
            std::fs::write(change.join("proposal.md"), "# Proposal\n").unwrap();
            std::fs::write(change.join("specs/hosting/spec.md"), "## ADDED Requirements\n")
                .unwrap();
            std::fs::write(
                change.join("tasks.md"),
                "- [x] 1.1 a\n- [ ] 1.2 b\n  - [ ] 1.3 c\nnot a task\n",
            )
            .unwrap();
            project
        }

        fn change(&self) -> ChangeRef {
            ChangeRef { root: self.dir.path().to_owned(), name: "x".into() }
        }

        /// An environment whose PATH holds only this project's fake tools and the system's.
        fn plans(&self) -> Plans {
            let env: Environment =
                [("PATH", format!("{}:/usr/bin:/bin", self.bin.path().display()))]
                    .into_iter()
                    .collect();
            Plans::new(self.dir.path(), &env)
        }

        fn fake_openspec(&self, body: &str) {
            crate::test_support::fake_executable(self.bin.path(), "openspec", body);
        }
    }

    fn names(entry: &Entry) -> Vec<(String, bool)> {
        let Entry::Plan { documents, .. } = entry else { panic!("{entry:?}") };
        documents.iter().map(|doc| (doc.name.clone(), doc.exists)).collect()
    }

    fn tasks(entry: &Entry) -> TaskProgress {
        let Entry::Plan { tasks, .. } = entry else { panic!("{entry:?}") };
        *tasks
    }

    #[test]
    fn finds_the_change_of_a_file() {
        let change = change_of(Path::new("/repo/openspec/changes/viewer-views/specs/a/spec.md"));
        assert_eq!(change, Some(ChangeRef { root: "/repo".into(), name: "viewer-views".into() }));
        assert_eq!(change_of(Path::new("/repo/openspec/changes/archive/2026-x/tasks.md")), None);
        assert_eq!(change_of(Path::new("/repo/openspec/changes/x")), None);
        assert_eq!(change_of(Path::new("/repo/openspec/specs/a/spec.md")), None);
        assert_eq!(change_of(Path::new("/repo/src/main.rs")), None);
    }

    #[test]
    fn without_the_cli_the_default_layout_is_used() {
        let project = Project::new();
        let plan = project.plans().read(&project.change());
        assert_eq!(
            names(&plan),
            [
                ("Proposal".into(), true),
                ("Specs".into(), true),
                ("Design".into(), false),
                ("Tasks".into(), true)
            ]
        );
        assert_eq!(tasks(&plan), TaskProgress { done: 1, total: 3 });
        let Entry::Plan { documents, .. } = &plan else { unreachable!() };
        assert_eq!(documents[1].paths, ["openspec/changes/x/specs/hosting/spec.md"]);
    }

    #[test]
    fn the_cli_gives_the_schemas_own_documents() {
        let project = Project::new();
        let dir = project.dir.path().join("openspec/changes/x");
        project.fake_openspec(&format!(
            r#"case "$1" in
status) cat <<'JSON'
{{"artifacts":[{{"id":"feature-brief","status":"done"}},{{"id":"proposal","status":"done"}},{{"id":"specs","status":"skipped"}},{{"id":"tasks","status":"ready"}}],
 "artifactPaths":{{"feature-brief":{{"existingOutputPaths":["{dir}/brief.md"]}},"proposal":{{"existingOutputPaths":["{dir}/proposal.md"]}},"specs":{{"existingOutputPaths":[]}},"tasks":{{"existingOutputPaths":[]}}}}}}
JSON
;;
list) echo '{{"changes":[{{"name":"other","completedTasks":9,"totalTasks":9}},{{"name":"x","completedTasks":2,"totalTasks":8}}]}}' ;;
*) exit 1 ;;
esac"#,
            dir = dir.display()
        ));
        let plan = project.plans().read(&project.change());
        assert_eq!(
            names(&plan),
            [("Feature brief".into(), true), ("Proposal".into(), true), ("Tasks".into(), false)]
        );
        assert_eq!(tasks(&plan), TaskProgress { done: 2, total: 8 });
        let Entry::Plan { documents, .. } = &plan else { unreachable!() };
        assert_eq!(documents[0].paths, ["openspec/changes/x/brief.md"]);
    }

    #[test]
    fn a_failing_or_silent_cli_falls_back() {
        let project = Project::new();
        project.fake_openspec("echo broken >&2; exit 1");
        assert_eq!(
            tasks(&project.plans().read(&project.change())),
            TaskProgress { done: 1, total: 3 }
        );
        project.fake_openspec("echo 'not json'");
        assert_eq!(names(&project.plans().read(&project.change())).len(), 4);
    }

    #[test]
    fn a_burst_of_writes_reads_the_plan_once() {
        let project = Project::new();
        let calls = project.bin.path().join("calls");
        project.fake_openspec(&format!("echo \"$1\" >> '{}'; exit 1", calls.display()));
        let (send, receive) = std::sync::mpsc::channel();
        for _ in 0..5 {
            send.send(project.change()).unwrap();
        }
        drop(send);
        let emitted = std::sync::Mutex::new(Vec::new());
        project.plans().run(receive, |entry| emitted.lock().unwrap().push(entry));
        assert_eq!(emitted.lock().unwrap().len(), 1);
        assert_eq!(std::fs::read_to_string(&calls).unwrap(), "status\n");
    }

    #[test]
    fn ids_read_as_names() {
        assert_eq!(readable("feature-brief"), "Feature brief");
        assert_eq!(readable("specs"), "Specs");
        assert_eq!(readable("user_stories"), "User stories");
    }
}

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

struct TestRepo {
    dir: TempDir,
    state_file: TempDir,
}

impl TestRepo {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("repo tempdir");
        let state_file = tempfile::tempdir().expect("state tempdir");
        git(dir.path(), &["init", "--quiet", "-b", "main"]);
        git(dir.path(), &["config", "user.email", "tester@example.com"]);
        git(dir.path(), &["config", "user.name", "Tester"]);
        git(
            dir.path(),
            &["commit", "--allow-empty", "-m", "init", "--quiet"],
        );
        Self { dir, state_file }
    }

    fn ti(&self) -> assert_cmd::Command {
        let mut cmd = assert_cmd::Command::cargo_bin("ti").expect("ti binary");
        cmd.current_dir(self.dir.path());
        cmd.env(
            "TICGIT_STATE_FILE",
            self.state_file.path().join("state.json"),
        );
        cmd
    }
}

fn git(cwd: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .expect("spawn git");
    assert!(status.success(), "git {args:?} failed");
}

fn git_output(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn git");
    assert!(output.status.success(), "git {args:?} failed");
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn create_ticket(repo: &TestRepo, title: &str) -> String {
    let output = repo
        .ti()
        .args(["new", "--title", title, "--id-only"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap().trim().to_string()
}

#[cfg(unix)]
fn editor_script(repo: &TestRepo, contents: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = repo.state_file.path().join("editor.sh");
    fs::write(
        &path,
        format!("#!/bin/sh\ncat > \"$1\" <<'EOF'\n{contents}\nEOF\n"),
    )
    .expect("write editor script");

    let mut permissions = fs::metadata(&path).expect("editor metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("chmod editor script");
    path
}

#[cfg(unix)]
fn executable_script(dir: &Path, name: &str, contents: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    fs::create_dir_all(dir).expect("script dir");
    let path = dir.join(name);
    fs::write(&path, contents).expect("write executable script");

    let mut permissions = fs::metadata(&path).expect("script metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("chmod executable script");
    path
}

#[test]
fn init_is_idempotent() {
    let repo = TestRepo::new();
    repo.ti()
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("ticgit initialised"));

    repo.ti()
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("schema 1"));
}

#[test]
fn init_bootstraps_git_meta_defaults() {
    let repo = TestRepo::new();
    git(
        repo.dir.path(),
        &[
            "remote",
            "add",
            "origin",
            "https://example.invalid/repo.git",
        ],
    );

    repo.ti()
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Configured git-meta remote 'origin' with defaults.",
        ))
        .stdout(predicate::str::contains("ticgit initialised"));

    assert_eq!(
        git_output(repo.dir.path(), &["config", "--get", "meta.namespace"]),
        "meta",
    );
    assert_eq!(
        git_output(
            repo.dir.path(),
            &["config", "--bool", "--get", "remote.origin.meta"],
        ),
        "true",
    );
    let fetch = git_output(
        repo.dir.path(),
        &["config", "--get-all", "remote.origin.fetch"],
    );
    assert!(fetch.contains("+refs/meta/main:refs/meta/remotes/main"));
}

#[test]
fn new_show_and_list_round_trip() {
    let repo = TestRepo::new();
    let id = create_ticket(&repo, "first bug");

    let output = repo
        .ti()
        .args(["show", &id, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["id"], id);
    assert_eq!(json["title"], "first bug");
    assert_eq!(json["state"], "open");

    repo.ti()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("first bug"));

    repo.ti()
        .args(["show", &id, "--filter", ".title"])
        .assert()
        .success()
        .stdout(predicate::eq("first bug\n"));

    repo.ti()
        .args(["show", &id, "--filter"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Available filters:"))
        .stdout(predicate::str::contains("ti show <id> --filter '.title'"));
}

#[test]
#[cfg(unix)]
fn edit_updates_title_and_description() {
    let repo = TestRepo::new();
    let id = create_ticket(&repo, "old title");
    let editor = editor_script(&repo, "new title\n\nnew description\nsecond line\n");

    repo.ti()
        .env("EDITOR", editor)
        .args(["edit", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated"));

    let output = repo
        .ti()
        .args(["show", &id, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["title"], "new title");
    assert_eq!(json["description"], "new description\nsecond line");
}

#[test]
fn mutating_commands_update_ticket() {
    let repo = TestRepo::new();
    let id = create_ticket(&repo, "mutate me");

    repo.ti()
        .args(["tag", "-t", &id, "bug,ui"])
        .assert()
        .success();
    repo.ti()
        .args(["assign", "-t", &id, "tester@example.com"])
        .assert()
        .success();
    repo.ti()
        .args(["points", "-t", &id, "5"])
        .assert()
        .success();
    repo.ti()
        .args(["milestone", "-t", &id, "v1"])
        .assert()
        .success();
    repo.ti()
        .args(["state", "resolved", "-t", &id])
        .assert()
        .success();
    repo.ti()
        .args(["comment", "-t", &id, "fixed", "now"])
        .assert()
        .success();

    let output = repo
        .ti()
        .args(["show", &id, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["state"], "resolved");
    assert_eq!(json["assigned"], "tester@example.com");
    assert_eq!(json["points"], 5);
    assert_eq!(json["milestone"], "v1");
    assert_eq!(json["tags"].as_array().unwrap().len(), 2);
    assert_eq!(json["comments"][0]["body"], "fixed now");
}

#[test]
fn ticket_mutations_support_json_output() {
    let repo = TestRepo::new();

    let output = repo
        .ti()
        .args(["new", "--title", "json ticket", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    let id = json["id"].as_str().unwrap().to_string();
    assert_eq!(json["title"], "json ticket");

    let output = repo
        .ti()
        .args(["tag", "-t", &id, "bug", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert!(json["tags"].as_array().unwrap().iter().any(|t| t == "bug"));

    let output = repo
        .ti()
        .args(["assign", "-t", &id, "octocat", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["assigned"], "octocat");

    let output = repo
        .ti()
        .args(["points", "-t", &id, "8", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["points"], 8);

    let output = repo
        .ti()
        .args(["milestone", "-t", &id, "v2", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["milestone"], "v2");

    let output = repo
        .ti()
        .args(["comment", "-t", &id, "hello", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["comments"][0]["body"], "hello");

    let output = repo
        .ti()
        .args(["state", "hold", "-t", &id, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["state"], "hold");

    let output = repo
        .ti()
        .args(["checkout", &id, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["id"], id);
}

#[test]
fn checkout_makes_ticket_optional_for_show_and_comment() {
    let repo = TestRepo::new();
    let id = create_ticket(&repo, "selected ticket");

    repo.ti().args(["checkout", &id[..6]]).assert().success();
    repo.ti()
        .args(["comment", "from", "current"])
        .assert()
        .success();

    repo.ti()
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("selected ticket"))
        .stdout(predicate::str::contains("from current"));
}

#[test]
fn close_resolves_current_ticket_and_clears_checkout() {
    let repo = TestRepo::new();
    let id = create_ticket(&repo, "current close ticket");

    repo.ti().args(["checkout", &id]).assert().success();
    repo.ti()
        .arg("close")
        .assert()
        .success()
        .stdout(predicate::str::contains("cleared current ticket"));

    let output = repo
        .ti()
        .args(["show", &id, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["state"], "resolved");

    repo.ti()
        .arg("show")
        .assert()
        .failure()
        .stderr(predicate::str::contains("none checked out"));
}

#[test]
fn close_explicit_ticket_keeps_other_checkout() {
    let repo = TestRepo::new();
    let current = create_ticket(&repo, "current ticket");
    let other = create_ticket(&repo, "other ticket");

    repo.ti().args(["checkout", &current]).assert().success();
    let output = repo
        .ti()
        .args(["close", &other, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["id"], other);
    assert_eq!(json["state"], "resolved");

    repo.ti()
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("current ticket"));
}

#[test]
fn new_checkout_selects_created_ticket() {
    let repo = TestRepo::new();

    repo.ti()
        .args(["new", "--title", "checked out on create", "--checkout"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Checked out:"));

    let output = repo
        .ti()
        .args(["show", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["title"], "checked out on create");
}

#[test]
fn list_filters_and_saved_views_work() {
    let repo = TestRepo::new();
    let bug = create_ticket(&repo, "bug ticket");
    let docs = create_ticket(&repo, "docs ticket");

    repo.ti()
        .args(["tag", "-t", &bug, "bug"])
        .assert()
        .success();
    repo.ti()
        .args(["tag", "-t", &docs, "docs"])
        .assert()
        .success();

    repo.ti()
        .args(["list", "--tag", "bug"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bug ticket"))
        .stdout(predicate::str::contains("docs ticket").not());

    repo.ti()
        .args(["save-view", "bugs", "--tag", "bug"])
        .assert()
        .success();

    repo.ti()
        .args(["views"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bugs"));

    repo.ti()
        .args(["views", "bugs"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&bug))
        .stdout(predicate::str::contains(&docs).not());
}

#[test]
fn list_search_filters_title_description_and_comments() {
    let repo = TestRepo::new();
    let title = create_ticket(&repo, "parser panic");
    let file = repo.state_file.path().join("description-ticket.md");
    fs::write(
        &file,
        "description ticket\n\nThis ticket explains parser recovery.\n",
    )
    .unwrap();
    let output = repo
        .ti()
        .args(["new", "-F"])
        .arg(&file)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let description: Value = serde_json::from_slice(&output).unwrap();
    let description_id = description["id"].as_str().unwrap().to_string();
    let comment = create_ticket(&repo, "comment ticket");
    repo.ti()
        .args(["comment", "-t", &comment, "parser appears in a comment"])
        .assert()
        .success();

    let output = repo
        .ti()
        .args(["list", "--search", "parser", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    let tickets = json.as_array().unwrap();
    assert_eq!(tickets.len(), 3);

    let output = repo
        .ti()
        .args(["list", "--search", "title:parser", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    let tickets = json.as_array().unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0]["id"], title);

    let output = repo
        .ti()
        .args(["list", "--search", "description:recovery", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    let tickets = json.as_array().unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0]["id"], description_id);

    let output = repo
        .ti()
        .args(["list", "--search", "comments:appears", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    let tickets = json.as_array().unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0]["id"], comment);
}

#[test]
fn list_all_includes_non_open_tickets() {
    let repo = TestRepo::new();
    let id = create_ticket(&repo, "closed ticket");
    repo.ti()
        .args(["state", "resolved", "-t", &id])
        .assert()
        .success();

    repo.ti()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("closed ticket").not());

    repo.ti()
        .args(["list", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("closed ticket"));
}

#[test]
#[cfg(unix)]
fn import_gh_creates_tickets_and_skips_existing_issues() {
    let repo = TestRepo::new();
    let bin = repo.state_file.path().join("bin");
    executable_script(
        &bin,
        "gh",
        r#"#!/bin/sh
cat <<'JSON'
[
  {
    "number": 7,
    "title": "first gh issue",
    "body": "Imported body",
    "url": "https://github.com/owner/repo/issues/7",
    "author": {"login": "monalisa"},
    "labels": [{"name": "bug"}],
    "assignees": [{"login": "octocat"}, {"login": "hubot"}],
    "milestone": {"title": "v1"}
  },
  {
    "number": 8,
    "title": "second gh issue",
    "body": "",
    "url": "https://github.com/owner/repo/issues/8",
    "author": {"login": "hubot"},
    "labels": [],
    "assignees": [],
    "milestone": null
  }
]
JSON
"#,
    );
    let path = format!(
        "{}:{}",
        bin.display(),
        env::var_os("PATH").unwrap_or_default().to_string_lossy()
    );

    repo.ti()
        .env("PATH", &path)
        .args(["import", "gh", "--repo", "owner/repo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Imported 2 GitHub issue(s)."));

    let output = repo
        .ti()
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    let tickets = json.as_array().unwrap();
    assert_eq!(tickets.len(), 2);

    let first = tickets
        .iter()
        .find(|ticket| ticket["title"] == "first gh issue")
        .unwrap();
    assert_eq!(first["assigned"], "octocat");
    assert_eq!(first["milestone"], "v1");
    assert_eq!(
        first["description"],
        "GitHub issue: https://github.com/owner/repo/issues/7\nGitHub author: monalisa\nGitHub assignees: octocat, hubot\n\nImported body"
    );
    let tags = first["tags"].as_array().unwrap();
    assert!(tags.iter().any(|tag| tag == "github"));
    assert!(tags.iter().any(|tag| tag == "github-issue-7"));
    assert!(tags.iter().any(|tag| tag == "bug"));

    repo.ti()
        .env("PATH", &path)
        .args(["import", "gh", "--repo", "owner/repo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Imported 0 GitHub issue(s)."))
        .stdout(predicate::str::contains(
            "Skipped 2 issue(s) that were already imported.",
        ));
}

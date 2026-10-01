//! End-to-end tests: run the real binary the way a learner would.

use std::io::Write;
use std::process::{Command, Output, Stdio};

fn rust_std(args: &[&str], stdin: &str) -> Output {
    let progress = std::env::temp_dir().join(format!(
        "rust_std_cli_test_{}_{}",
        std::process::id(),
        args.join("_").replace(['/', ' '], "-")
    ));
    let mut child = Command::new(env!("CARGO_BIN_EXE_rust_std"))
        .args(args)
        .env("RUST_STD_PROGRESS", &progress)
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn rust_std");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let output = child.wait_with_output().expect("wait for rust_std");
    let _ = std::fs::remove_file(progress);
    output
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn list_shows_every_lesson() {
    let out = rust_std(&["list"], "");
    assert!(out.status.success());
    let text = stdout(&out);
    for (n, lesson) in rust_std::lessons::all() {
        assert!(text.contains(lesson.title), "missing lesson {n}");
    }
}

#[test]
fn show_prints_code_and_output() {
    let out = rust_std(&["show", "1"], "");
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("Hello, world!"));
    assert!(text.contains("▶ output"));
    assert!(!text.contains('\x1b'), "NO_COLOR must disable escapes");
}

#[test]
fn every_lesson_shows_without_errors() {
    for (n, _) in rust_std::lessons::all() {
        let out = rust_std(&["show", &n.to_string()], "");
        let text = stdout(&out);
        assert!(out.status.success(), "lesson {n} failed");
        assert!(
            !text.contains("[missing code region"),
            "lesson {n} has a missing region"
        );
        assert!(
            !text.contains("the demo panicked"),
            "lesson {n} has a panicking demo"
        );
    }
}

#[test]
fn unknown_lesson_is_an_error() {
    let out = rust_std(&["show", "9999"], "");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("no lesson 9999"));
}

#[test]
fn bad_flag_exits_with_usage_error() {
    let out = rust_std(&["--bogus"], "");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn menu_accepts_commands_until_exit() {
    let out = rust_std(&[], "list\nsearch format\nx\n");
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("Happy hacking!"));
}

#[test]
fn learn_then_quiz_records_progress() {
    let progress = std::env::temp_dir().join(format!("rust_std_progress_{}", std::process::id()));
    let sections = rust_std::lessons::find("1").unwrap().1.sections.len();
    // Press enter through every section, accept the quiz, answer "a" to all.
    let mut input = "\n".repeat(sections);
    input.push_str("y\n");
    input.push_str(&"a\n".repeat(20));

    let mut child = Command::new(env!("CARGO_BIN_EXE_rust_std"))
        .args(["learn", "1"])
        .env("RUST_STD_PROGRESS", &progress)
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    assert!(stdout(&out).contains("Score:"));

    let saved = std::fs::read_to_string(&progress).expect("progress file written");
    let _ = std::fs::remove_file(&progress);
    assert!(saved.contains("hello\tdone\t"), "got: {saved}");
}

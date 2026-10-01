//! Compiles every fenced code block in the lesson prose with `rustc`.
//!
//! - ```` ```rust ```` blocks must compile.
//! - ```` ```compile_fail,E0382 ```` blocks must fail, with that error code.
//! - ```` ```text ````, ```` ```output ```` and ```` ```ignore ```` are skipped.
//!
//! Blocks are wrapped in a function body unless tagged `nowrap`, so they may
//! mix statements and items. This keeps the "this does not compile" claims
//! in the lessons honest: if a future compiler accepts one, this test fails.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use rust_std::lessons;
use rust_std::snippet::dedent;

#[derive(Debug)]
struct Example {
    origin: String,
    info: String,
    code: String,
}

/// Pulls fenced blocks out of one prose string.
fn fenced_blocks(origin: &str, text: &str) -> Vec<Example> {
    let text = dedent(text);
    let mut out = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        let Some(info) = line.trim_start().strip_prefix("```") else {
            continue;
        };
        let code: Vec<&str> = lines
            .by_ref()
            .take_while(|l| !l.trim_start().starts_with("```"))
            .collect();
        out.push(Example {
            origin: origin.to_string(),
            info: info.trim().to_string(),
            code: code.join("\n"),
        });
    }
    out
}

fn all_examples() -> Vec<Example> {
    let mut examples = Vec::new();
    for (n, lesson) in lessons::all() {
        for section in lesson.sections {
            let origin = format!("lesson {n} ({}) › {}", lesson.id, section.title);
            examples.extend(fenced_blocks(&origin, section.text));
        }
        for (i, q) in lesson.quiz.iter().enumerate() {
            let origin = format!("lesson {n} ({}) › quiz {}", lesson.id, i + 1);
            examples.extend(fenced_blocks(&origin, q.prompt));
            examples.extend(fenced_blocks(&origin, q.explanation));
        }
    }
    examples
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

/// Compiles `code` as a library; returns (success, stderr).
fn compile(dir: &Path, id: usize, code: &str, wrap: bool) -> (bool, String) {
    let source = if wrap {
        format!("#![allow(warnings)]\nfn __example() {{\n{code}\n}}\n")
    } else {
        format!("#![allow(warnings)]\n{code}\n")
    };
    let src_path = dir.join(format!("ex{id}.rs"));
    std::fs::write(&src_path, source).expect("write example");
    let output = Command::new(rustc())
        .args([
            "--edition",
            "2024",
            "--crate-type",
            "lib",
            "--emit=metadata",
        ])
        .arg("--crate-name")
        .arg(format!("ex{id}"))
        .arg("-o")
        .arg(dir.join(format!("ex{id}.rmeta")))
        .arg(&src_path)
        .output()
        .expect("failed to run rustc; is it on PATH?");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// What went wrong with an example, if anything.
fn verdict(flags: &[&str], compiled: bool, stderr: &str) -> Option<String> {
    if flags[0] != "compile_fail" {
        return (!compiled).then(|| format!("failed to compile:\n{stderr}"));
    }
    if compiled {
        return Some("expected a compile error, but it compiled".to_string());
    }
    flags
        .iter()
        .filter(|f| f.starts_with('E'))
        .find(|code| !stderr.contains(&format!("error[{code}]")))
        .map(|missing| format!("expected error {missing}, got:\n{stderr}"))
}

fn scratch_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rust_std_examples_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

#[test]
fn prose_code_blocks_behave_as_claimed() {
    let examples: Vec<Example> = all_examples()
        .into_iter()
        .filter(|e| {
            let kind = e.info.split(',').next().unwrap_or("");
            matches!(kind, "" | "rust" | "compile_fail")
        })
        .collect();
    assert!(!examples.is_empty(), "no examples found");

    let dir = scratch_dir();
    let next = AtomicUsize::new(0);
    let failures = Mutex::new(Vec::new());
    let workers = thread::available_parallelism().map_or(4, |n| n.get());

    // Scoped threads may borrow `examples`, `next` and `failures` because the
    // scope guarantees every thread is joined before those locals die.
    thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(example) = examples.get(i) else {
                        break;
                    };
                    let flags: Vec<&str> = example.info.split(',').map(str::trim).collect();
                    let wrap = !flags.contains(&"nowrap");
                    let (ok, stderr) = compile(&dir, i, &example.code, wrap);
                    let problem = verdict(&flags, ok, &stderr);
                    if let Some(problem) = problem {
                        failures.lock().unwrap().push(format!(
                            "── {} [{}]\n{}\n{}",
                            example.origin, example.info, example.code, problem
                        ));
                    }
                }
            });
        }
    });

    let _ = std::fs::remove_dir_all(&dir);
    let failures = failures.into_inner().unwrap();
    assert!(
        failures.is_empty(),
        "{} of {} examples misbehaved:\n\n{}",
        failures.len(),
        examples.len(),
        failures.join("\n\n")
    );
}

#[test]
fn compile_fail_blocks_name_their_error() {
    for example in all_examples() {
        if example.info.starts_with("compile_fail") {
            assert!(
                example.info.split(',').any(|f| f.trim().starts_with('E')),
                "{}: compile_fail block should list its error code, e.g. compile_fail,E0382",
                example.origin
            );
        }
    }
}

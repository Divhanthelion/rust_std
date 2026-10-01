//! Printing lessons: prose, code, and live demo output.

use std::io::{self, Write};
use std::panic;

use crate::lesson::{Lesson, Section};
use crate::snippet;
use crate::ui::{self, Style};

pub fn print_header(number: usize, lesson: &Lesson) {
    println!();
    println!("{}", ui::title(number, lesson.title));
    print!("{}", ui::wrap(lesson.summary, "", ""));
    println!(
        "{}",
        ui::paint(
            &format!(
                "{} sections · {} quiz questions · id: {}",
                lesson.sections.len(),
                lesson.quiz.len(),
                lesson.id
            ),
            Style::Dim
        )
    );
}

/// Prints section `index` (0-based) of `lesson`, running its demo if any.
pub fn print_section(lesson: &Lesson, index: usize) {
    let section = &lesson.sections[index];
    println!();
    println!(
        "{}",
        ui::section_heading(&format!(
            "{}/{} · {}",
            index + 1,
            lesson.sections.len(),
            section.title
        ))
    );
    println!();
    print!("{}", ui::render_prose(section.text));
    print_code_and_output(lesson, section);
}

fn print_code_and_output(lesson: &Lesson, section: &Section) {
    if let Some(anchor) = section.anchor {
        match snippet::extract(lesson.source, anchor) {
            Some(code) => print!("{}", ui::code_block(&code, true)),
            None => println!(
                "{}",
                ui::paint(&format!("[missing code region `{anchor}`]"), Style::Bad)
            ),
        }
    }
    if let Some(run) = section.run {
        println!("{}", ui::paint("▶ output", Style::Good));
        run_demo(run);
        println!();
    }
}

/// Runs a demo function. A panicking demo must not take the whole program
/// down, so we catch the unwind and report it.
pub fn run_demo(demo: fn()) {
    let _ = io::stdout().flush();
    // `fn()` pointers are `UnwindSafe`, so this needs no `AssertUnwindSafe`.
    if panic::catch_unwind(demo).is_err() {
        println!(
            "{}",
            ui::paint("(the demo panicked — see the message above)", Style::Bad)
        );
    }
    let _ = io::stdout().flush();
}

pub fn print_exercises(lesson: &Lesson) {
    if lesson.exercises.is_empty() {
        return;
    }
    println!();
    println!("{}", ui::section_heading("Try it yourself"));
    println!();
    for (i, exercise) in lesson.exercises.iter().enumerate() {
        let label = format!("  {}. ", i + 1);
        let text = snippet::dedent(exercise).replace('\n', " ");
        print!("{}", ui::wrap(&text, &label, "     "));
    }
}

/// The whole lesson, without pauses.
pub fn print_lesson(number: usize, lesson: &Lesson) {
    print_header(number, lesson);
    for i in 0..lesson.sections.len() {
        print_section(lesson, i);
    }
    print_exercises(lesson);
    if !lesson.quiz.is_empty() {
        println!();
        println!(
            "{}",
            ui::paint(
                &format!(
                    "Quiz: {} questions — run `rust_std quiz {}`",
                    lesson.quiz.len(),
                    lesson.id
                ),
                Style::Accent
            )
        );
    }
}

/// Only the code and output of each section that has a demo.
pub fn run_lesson(number: usize, lesson: &Lesson) {
    println!("{}", ui::title(number, lesson.title));
    for section in lesson.sections.iter().filter(|s| s.run.is_some()) {
        println!();
        println!("{}", ui::section_heading(section.title));
        print_code_and_output(lesson, section);
    }
}

/// The lesson's complete source file.
pub fn print_source(number: usize, lesson: &Lesson) {
    println!("{}", ui::title(number, lesson.title));
    print!("{}", ui::code_block(lesson.source, true));
}

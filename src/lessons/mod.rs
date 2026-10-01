//! The course. Each module is one lesson; `PARTS` fixes their order.

// Demo structs are often only read through `{:?}`, which the dead-code lint
// ignores, so it is silenced here. The tests at the bottom of this file do
// its real job instead: every lesson file is registered, and every code
// region is shown by some section.
#![allow(dead_code)]
// Lesson code demonstrates behaviour on literal values (`NaN == NaN`,
// `f64::NAN as i32`, `Some(7).unwrap_or(0)`) and sometimes shows a less
// idiomatic form next to the better one. Clippy rightly flags those in
// production code, so these lints are relaxed for the lessons only.
#![allow(
    clippy::cast_nan_to_int,
    clippy::char_lit_as_u8,
    clippy::clone_on_copy,
    clippy::eq_op,
    clippy::iter_nth,
    clippy::let_unit_value,
    clippy::manual_is_ascii_check,
    clippy::manual_is_multiple_of,
    clippy::manual_range_patterns,
    clippy::needless_lifetimes,
    clippy::needless_range_loop,
    clippy::print_literal,
    clippy::to_string_in_format_args,
    clippy::type_complexity,
    clippy::unnecessary_fold,
    clippy::unnecessary_lazy_evaluations,
    clippy::unnecessary_literal_unwrap,
    clippy::unnecessary_min_or_max,
    clippy::useless_vec,
    clippy::vec_init_then_push,
    clippy::while_let_on_iterator,
    clippy::zero_divided_by_zero
)]

use crate::lesson::{Lesson, Part};

mod aggregates;
mod bindings;
mod borrowing;
mod closures;
mod enums;
mod errors;
mod flow;
mod generics;
mod hello;
mod iterators;
mod lifetimes;
mod numbers;
mod option_result;
mod ownership;
mod patterns;
mod std_traits;
mod strings;
mod structs;
mod traits;

pub static PARTS: &[Part] = &[
    Part {
        title: "Part I — Foundations",
        lessons: &[
            &hello::LESSON,
            &bindings::LESSON,
            &numbers::LESSON,
            &aggregates::LESSON,
            &flow::LESSON,
        ],
    },
    Part {
        title: "Part II — Ownership & borrowing",
        lessons: &[
            &ownership::LESSON,
            &borrowing::LESSON,
            &strings::LESSON,
            &lifetimes::LESSON,
        ],
    },
    Part {
        title: "Part III — Modeling data",
        lessons: &[
            &structs::LESSON,
            &enums::LESSON,
            &patterns::LESSON,
            &option_result::LESSON,
            &errors::LESSON,
        ],
    },
    Part {
        title: "Part IV — Abstraction",
        lessons: &[
            &generics::LESSON,
            &traits::LESSON,
            &std_traits::LESSON,
            &closures::LESSON,
            &iterators::LESSON,
        ],
    },
];

/// Every lesson with its 1-based number, in course order.
pub fn all() -> impl Iterator<Item = (usize, &'static Lesson)> {
    PARTS
        .iter()
        .flat_map(|part| part.lessons.iter().copied())
        .enumerate()
        .map(|(i, lesson)| (i + 1, lesson))
}

/// Looks a lesson up by number (`7`), id (`ownership`), unique id prefix
/// (`own`), or unique title fragment (`move`).
pub fn find(query: &str) -> Result<(usize, &'static Lesson), String> {
    let query = query.trim().to_lowercase();
    let count = all().count();

    if let Ok(n) = query.parse::<usize>() {
        return all()
            .find(|&(i, _)| i == n)
            .ok_or_else(|| format!("there is no lesson {n} (lessons are numbered 1 to {count})"));
    }
    if let Some(hit) = all().find(|(_, l)| l.id == query) {
        return Ok(hit);
    }

    // Prefer id prefixes; fall back to title substrings.
    let by_prefix: Vec<_> = all().filter(|(_, l)| l.id.starts_with(&query)).collect();
    let candidates = if by_prefix.is_empty() {
        all()
            .filter(|(_, l)| l.title.to_lowercase().contains(&query))
            .collect()
    } else {
        by_prefix
    };

    match candidates.as_slice() {
        [one] => Ok(*one),
        [] => Err(format!("no lesson matches `{query}` — try `list`")),
        many => {
            let names: Vec<String> = many
                .iter()
                .map(|(n, l)| format!("{n} ({})", l.id))
                .collect();
            Err(format!("`{query}` is ambiguous: {}", names.join(", ")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snippet;
    use std::collections::HashSet;

    #[test]
    fn ids_are_unique_and_simple() {
        let mut seen = HashSet::new();
        for (_, lesson) in all() {
            assert!(seen.insert(lesson.id), "duplicate id {}", lesson.id);
            assert!(
                lesson
                    .id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '_'),
                "id `{}` should be lowercase snake_case",
                lesson.id
            );
        }
    }

    #[test]
    fn every_anchor_exists() {
        for (n, lesson) in all() {
            for section in lesson.sections {
                if let Some(anchor) = section.anchor {
                    let code = snippet::extract(lesson.source, anchor);
                    assert!(
                        code.is_some_and(|c| !c.is_empty()),
                        "lesson {n} ({}) section `{}`: anchor `{anchor}` missing or empty",
                        lesson.id,
                        section.title
                    );
                }
            }
        }
    }

    #[test]
    fn quizzes_are_well_formed() {
        for (_, lesson) in all() {
            assert!(!lesson.quiz.is_empty(), "{} has no quiz", lesson.id);
            for q in lesson.quiz {
                assert!(q.choices.len() >= 2, "{}: `{}`", lesson.id, q.prompt);
                assert!(q.answer < q.choices.len(), "{}: `{}`", lesson.id, q.prompt);
            }
        }
    }

    #[test]
    fn lessons_have_substance() {
        for (_, lesson) in all() {
            assert!(lesson.sections.len() >= 4, "{} is too short", lesson.id);
            assert!(
                !lesson.exercises.is_empty(),
                "{} has no exercises",
                lesson.id
            );
        }
    }

    #[test]
    fn find_by_number_id_and_prefix() {
        let (n, first) = all().next().unwrap();
        assert_eq!(find("1").unwrap().1.id, first.id);
        assert_eq!(find(first.id).unwrap().0, n);
        assert!(find("0").is_err());
        assert!(find("no such lesson anywhere").is_err());
    }

    #[test]
    fn every_lesson_file_is_registered() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/lessons");
        let mut files: Vec<String> = std::fs::read_dir(dir)
            .expect("read src/lessons")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".rs") && name != "mod.rs")
            .collect();
        files.sort();
        let registered = all().count();
        assert_eq!(
            files.len(),
            registered,
            "lesson files {files:?} vs {registered} registered lessons"
        );
    }

    #[test]
    fn every_anchor_is_shown() {
        for (_, lesson) in all() {
            let used: HashSet<&str> = lesson.sections.iter().filter_map(|s| s.anchor).collect();
            for anchor in snippet::anchors(lesson.source) {
                assert!(
                    used.contains(anchor),
                    "{}: anchor `{anchor}` is never shown by a section",
                    lesson.id
                );
            }
        }
    }

    /// Every demo runs to completion without panicking.
    #[test]
    fn all_demos_run() {
        for (_, lesson) in all() {
            for section in lesson.sections {
                if let Some(run) = section.run {
                    run();
                }
            }
        }
    }
}

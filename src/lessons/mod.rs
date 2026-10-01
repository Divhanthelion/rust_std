//! The course. Each module is one lesson; `PARTS` fixes their order.

// Lesson code sometimes shows a less idiomatic form on purpose, next to the
// better one, so a few style lints are relaxed for the lessons only.
#![allow(clippy::print_literal)]

use crate::lesson::{Lesson, Part};

mod aggregates;
mod bindings;
mod borrowing;
mod flow;
mod hello;
mod lifetimes;
mod numbers;
mod ownership;
mod strings;

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

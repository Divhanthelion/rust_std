//! The data model for a lesson.
//!
//! Every lesson is a `static` value built at compile time. Prose lives in
//! string literals; code samples are *real functions* in the lesson's own
//! source file, so what you read is exactly what the compiler checked and
//! what the `run` command executes.

/// One lesson: a titled sequence of sections, a quiz, and exercises.
pub struct Lesson {
    /// Short, stable identifier used on the command line (`ownership`).
    pub id: &'static str,
    pub title: &'static str,
    /// One-line description shown in listings.
    pub summary: &'static str,
    /// The lesson's own source code, embedded with `include_str!`.
    /// Snippets are cut out of this text by anchor name.
    pub source: &'static str,
    pub sections: &'static [Section],
    pub quiz: &'static [Question],
    pub exercises: &'static [&'static str],
}

/// A titled chunk of a lesson: prose, then optionally code and its output.
#[derive(Clone, Copy)]
pub struct Section {
    pub title: &'static str,
    /// Prose in a tiny Markdown dialect (paragraphs, `- ` bullets,
    /// fenced code blocks, and `inline code`). See `ui::render_prose`.
    pub text: &'static str,
    /// Name of an `ANCHOR:` region in the lesson source to display.
    pub anchor: Option<&'static str>,
    /// A function to execute after showing the code.
    pub run: Option<fn()>,
}

impl Section {
    /// A section that is only prose.
    pub const fn new(title: &'static str, text: &'static str) -> Self {
        Section {
            title,
            text,
            anchor: None,
            run: None,
        }
    }

    /// Show the code in anchor region `anchor` (without running anything).
    pub const fn code(mut self, anchor: &'static str) -> Self {
        self.anchor = Some(anchor);
        self
    }

    /// Show the code in `anchor`, then execute `run` and show its output.
    pub const fn demo(mut self, anchor: &'static str, run: fn()) -> Self {
        self.anchor = Some(anchor);
        self.run = Some(run);
        self
    }
}

/// A multiple-choice question.
pub struct Question {
    pub prompt: &'static str,
    pub choices: &'static [&'static str],
    /// Index into `choices` of the correct answer.
    pub answer: usize,
    /// Shown after the learner answers, right or wrong.
    pub explanation: &'static str,
}

impl Question {
    pub const fn new(
        prompt: &'static str,
        choices: &'static [&'static str],
        answer: usize,
        explanation: &'static str,
    ) -> Self {
        Question {
            prompt,
            choices,
            answer,
            explanation,
        }
    }
}

/// A group of lessons shown together in the menu.
pub struct Part {
    pub title: &'static str,
    pub lessons: &'static [&'static Lesson],
}

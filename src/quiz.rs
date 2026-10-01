//! Multiple-choice quizzes.

use crate::console::Console;
use crate::lesson::Question;
use crate::ui::{self, Style};

pub struct Outcome {
    pub correct: u32,
    pub answered: u32,
    /// True if the learner stopped before the last question.
    pub quit: bool,
}

/// Asks each question in turn. Answers are letters (`a`, `b`, …) or
/// numbers (`1`, `2`, …); `q` stops early.
pub fn run(questions: &[&Question], console: &mut Console) -> Outcome {
    let mut outcome = Outcome {
        correct: 0,
        answered: 0,
        quit: false,
    };

    for (n, q) in questions.iter().enumerate() {
        println!();
        println!(
            "{}",
            ui::paint(
                &format!("Question {} of {}", n + 1, questions.len()),
                Style::Heading
            )
        );
        print!("{}", ui::render_prose(q.prompt));
        for (i, choice) in q.choices.iter().enumerate() {
            let label = format!("  {}) ", letter(i));
            print!("{}", ui::wrap(choice, &label, "     "));
        }

        let choice = loop {
            let Some(reply) = console.ask(&format!(
                "{} ",
                ui::paint(
                    &format!(
                        "Your answer [a-{}, q to stop]:",
                        letter(q.choices.len() - 1)
                    ),
                    Style::Dim
                )
            )) else {
                outcome.quit = true;
                return outcome;
            };
            if reply.eq_ignore_ascii_case("q") {
                outcome.quit = true;
                return outcome;
            }
            match parse_choice(&reply, q.choices.len()) {
                Some(i) => break i,
                None => println!(
                    "Please type a letter between a and {}.",
                    letter(q.choices.len() - 1)
                ),
            }
        };

        outcome.answered += 1;
        if choice == q.answer {
            outcome.correct += 1;
            println!("{}", ui::paint("✓ Correct!", Style::Good));
        } else {
            println!(
                "{} The answer is {}) {}",
                ui::paint("✗ Not quite.", Style::Bad),
                letter(q.answer),
                q.choices[q.answer]
            );
        }
        print!("{}", ui::render_prose(q.explanation));
    }
    outcome
}

fn letter(index: usize) -> char {
    // `char::from_u32` is fallible in general; `b'a' + n` stays ASCII here
    // because no question has more than 26 choices.
    char::from(b'a' + index as u8)
}

/// Accepts `b`, `B`, or `2` for the second choice.
fn parse_choice(reply: &str, count: usize) -> Option<usize> {
    let reply = reply.trim();
    let index = if let Ok(n) = reply.parse::<usize>() {
        n.checked_sub(1)?
    } else {
        let mut chars = reply.chars();
        let c = chars.next()?.to_ascii_lowercase();
        if chars.next().is_some() || !c.is_ascii_lowercase() {
            return None;
        }
        (c as u8 - b'a') as usize
    };
    (index < count).then_some(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_letters_and_numbers() {
        assert_eq!(parse_choice("a", 3), Some(0));
        assert_eq!(parse_choice("C", 3), Some(2));
        assert_eq!(parse_choice("2", 3), Some(1));
        assert_eq!(parse_choice("d", 3), None);
        assert_eq!(parse_choice("0", 3), None);
        assert_eq!(parse_choice("ab", 3), None);
        assert_eq!(parse_choice("", 3), None);
    }

    #[test]
    fn scripted_quiz() {
        static Q: Question = Question::new("1 + 1?", &["1", "2"], 1, "Arithmetic.");
        let mut console = Console::from_reader(&b"x\nb\n"[..]);
        let outcome = run(&[&Q], &mut console);
        assert_eq!(
            (outcome.correct, outcome.answered, outcome.quit),
            (1, 1, false)
        );
    }
}

//! Multiple-choice quizzes.

use crate::console::Console;
use crate::lesson::Question;
use crate::rng::Rng;
use crate::ui::{self, Style};

pub struct Outcome {
    pub correct: u32,
    pub answered: u32,
    /// True if the learner stopped before the last question.
    pub quit: bool,
}

/// Asks each question in turn. Answers are letters (`a`, `b`, …) or
/// numbers (`1`, `2`, …); `q` stops early. With an `rng`, the choices are
/// shown in a random order so the position of the answer gives nothing away.
pub fn run(questions: &[&Question], console: &mut Console, mut rng: Option<&mut Rng>) -> Outcome {
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
        // `order[shown position] = original index`
        let mut order: Vec<usize> = (0..q.choices.len()).collect();
        if let Some(rng) = rng.as_mut() {
            rng.shuffle(&mut order);
        }
        for (shown, &original) in order.iter().enumerate() {
            let label = format!("  {}) ", letter(shown));
            print!("{}", ui::wrap(q.choices[original], &label, "     "));
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
        if order[choice] == q.answer {
            outcome.correct += 1;
            println!("{}", ui::paint("✓ Correct!", Style::Good));
        } else {
            let shown = order
                .iter()
                .position(|&i| i == q.answer)
                .unwrap_or(q.answer);
            println!(
                "{} The answer is {}) {}",
                ui::paint("✗ Not quite.", Style::Bad),
                letter(shown),
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
    fn shuffled_choices_still_score_correctly() {
        static Q: Question = Question::new("pick c", &["a", "b", "c", "d"], 2, "");
        for seed in 1..20 {
            // Find where "c" will be shown for this seed, then answer that letter.
            let mut order: Vec<usize> = (0..4).collect();
            Rng::new(seed).shuffle(&mut order);
            let shown = order.iter().position(|&i| i == 2).unwrap();
            let reply = format!("{}\n", letter(shown));
            let mut console = Console::from_reader(reply.as_bytes());
            let outcome = run(&[&Q], &mut console, Some(&mut Rng::new(seed)));
            assert_eq!(outcome.correct, 1, "seed {seed}");
        }
    }

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
        let outcome = run(&[&Q], &mut console, None);
        assert_eq!(
            (outcome.correct, outcome.answered, outcome.quit),
            (1, 1, false)
        );
    }
}

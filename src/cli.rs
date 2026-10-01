//! Command-line parsing and the interactive menu.
//!
//! Arguments are parsed by hand from `std::env::args()`: a small enum of
//! commands, plus a few global flags. The interactive menu reuses the same
//! parser — each line you type there is split into words and parsed as if
//! you had typed it on the command line.

use std::io::IsTerminal;
use std::process::ExitCode;

use crate::console::Console;
use crate::lesson::{Lesson, Question};
use crate::lessons;
use crate::progress::Progress;
use crate::quiz;
use crate::rng::Rng;
use crate::ui::{self, Style};
use crate::viewer;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Menu,
    List,
    Learn(String),
    Show(String),
    Run(String),
    Code(String),
    Quiz(Option<String>),
    Search(String),
    Next,
    Progress,
    Reset,
    Help,
    Version,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// `Some(true)` for `--color`, `Some(false)` for `--no-color`.
    pub color: Option<bool>,
    pub width: Option<usize>,
}

/// Parses arguments (without the program name).
pub fn parse<I>(args: I) -> Result<(Command, Options), String>
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    let mut options = Options::default();
    let mut words: Vec<String> = Vec::new();
    let mut args = args.into_iter().map(Into::into);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--no-color" => options.color = Some(false),
            "--color" => options.color = Some(true),
            "-h" | "--help" => return Ok((Command::Help, options)),
            "-V" | "--version" => return Ok((Command::Version, options)),
            "--width" => {
                let value = args.next().ok_or("--width needs a number")?;
                options.width = Some(parse_width(&value)?);
            }
            _ if arg.starts_with("--width=") => {
                options.width = Some(parse_width(&arg["--width=".len()..])?);
            }
            _ if arg.starts_with('-') && arg.len() > 1 => {
                return Err(format!("unknown flag `{arg}` (try --help)"));
            }
            _ => words.push(arg),
        }
    }

    let command = parse_words(&words)?;
    Ok((command, options))
}

fn parse_width(value: &str) -> Result<usize, String> {
    value
        .parse()
        .map_err(|e| format!("invalid width `{value}`: {e}"))
}

fn parse_words(words: &[String]) -> Result<Command, String> {
    // Slice patterns make the shape of each command obvious.
    let target = |rest: &[String]| -> Result<String, String> {
        if rest.is_empty() {
            Err("which lesson? Give a number or a name, e.g. `7` or `ownership`".into())
        } else {
            Ok(rest.join(" "))
        }
    };
    let command = match words {
        [] => Command::Menu,
        [cmd, rest @ ..] => match cmd.as_str() {
            "list" | "ls" | "l" => Command::List,
            "learn" | "open" | "o" => Command::Learn(target(rest)?),
            "show" | "print" => Command::Show(target(rest)?),
            "run" | "r" => Command::Run(target(rest)?),
            "code" | "source" | "src" => Command::Code(target(rest)?),
            "quiz" | "q" => Command::Quiz((!rest.is_empty()).then(|| rest.join(" "))),
            "search" | "s" | "find" => Command::Search(target(rest)?),
            "next" | "n" | "continue" => Command::Next,
            "progress" | "p" => Command::Progress,
            "reset" => Command::Reset,
            "help" | "h" | "?" => Command::Help,
            "version" => Command::Version,
            // A bare lesson number or name opens that lesson.
            _ => Command::Learn(words.join(" ")),
        },
    };
    Ok(command)
}

/// Entry point used by `main.rs`.
pub fn main() -> ExitCode {
    let (command, options) = match parse(std::env::args().skip(1)) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::from(2);
        }
    };
    configure(&options);

    let mut progress = Progress::load();
    let mut console = Console::stdin();
    let result = match command {
        Command::Menu => {
            menu(&mut progress, &mut console);
            Ok(())
        }
        other => execute(other, &mut progress, &mut console),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn configure(options: &Options) {
    let no_color_env = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    let dumb_terminal = std::env::var("TERM").is_ok_and(|t| t == "dumb");
    let auto = std::io::stdout().is_terminal() && !no_color_env && !dumb_terminal;
    ui::set_color(options.color.unwrap_or(auto));

    let columns = options.width.or_else(|| {
        std::env::var("COLUMNS")
            .ok()
            .and_then(|c| c.trim().parse().ok())
    });
    ui::set_width(columns.unwrap_or(80));
}

/// Runs one command. Errors are messages for the learner.
pub fn execute(
    command: Command,
    progress: &mut Progress,
    console: &mut Console,
) -> Result<(), String> {
    match command {
        Command::Menu => menu(progress, console),
        Command::List => list(progress),
        Command::Learn(query) => {
            let (n, lesson) = lessons::find(&query)?;
            learn(n, lesson, progress, console);
        }
        Command::Show(query) => {
            let (n, lesson) = lessons::find(&query)?;
            viewer::print_lesson(n, lesson);
        }
        Command::Run(query) => {
            let (n, lesson) = lessons::find(&query)?;
            viewer::run_lesson(n, lesson);
        }
        Command::Code(query) => {
            let (n, lesson) = lessons::find(&query)?;
            viewer::print_source(n, lesson);
        }
        Command::Quiz(query) => quiz_command(query.as_deref(), progress, console)?,
        Command::Search(term) => search(&term),
        Command::Next => match lessons::all().find(|(_, l)| !progress.is_completed(l.id)) {
            Some((n, lesson)) => learn(n, lesson, progress, console),
            None => println!("You have finished every lesson. Try `quiz all` to review."),
        },
        Command::Progress => show_progress(progress),
        Command::Reset => {
            progress
                .reset()
                .map_err(|e| format!("could not reset progress: {e}"))?;
            println!("Progress cleared.");
        }
        Command::Help => print_help(),
        Command::Version => println!("rust_std {VERSION}"),
    }
    Ok(())
}

// ─── Menu ─────────────────────────────────────────────────────────────────

fn menu(progress: &mut Progress, console: &mut Console) {
    banner();
    list(progress);
    println!();
    println!(
        "{}",
        ui::paint(
            "Type a lesson number or name to start · n next · q <n> quiz · s <word> search · p progress · h help · x exit",
            Style::Dim
        )
    );

    let prompt = format!("\n{} ", ui::paint("rust_std ›", Style::Accent));
    while let Some(line) = console.ask(&prompt) {
        let words: Vec<String> = line.split_whitespace().map(String::from).collect();
        match words.first().map(String::as_str) {
            None => continue,
            Some("x" | "exit" | "quit") => break,
            _ => {}
        }
        match parse_words(&words) {
            Ok(Command::Menu) => {}
            Ok(command) => {
                if let Err(message) = execute(command, progress, console) {
                    println!("{}", ui::paint(&message, Style::Bad));
                }
            }
            Err(message) => println!("{}", ui::paint(&message, Style::Bad)),
        }
    }
    println!("Happy hacking!");
}

fn banner() {
    println!(
        "{}",
        ui::paint(
            "RUST STD — learn Rust with nothing but the standard library",
            Style::Title
        )
    );
    println!(
        "{}",
        ui::paint(
            "Every code sample below is real, compiled code from this program's own source.",
            Style::Dim
        )
    );
}

fn list(progress: &Progress) {
    let mut number = 0;
    for part in lessons::PARTS {
        println!();
        println!("{}", ui::paint(part.title, Style::Heading));
        for lesson in part.lessons {
            number += 1;
            let entry = progress.entry(lesson.id);
            let mark = if entry.completed {
                ui::paint("✓", Style::Good)
            } else {
                " ".to_string()
            };
            let quiz = entry
                .best_quiz
                .map(|(c, t)| ui::paint(&format!(" [quiz {c}/{t}]"), Style::Dim))
                .unwrap_or_default();
            println!(
                " {mark} {:>2}. {:<34}{}{quiz}",
                number,
                lesson.title,
                ui::paint(lesson.id, Style::Dim)
            );
        }
    }
}

// ─── Learning ─────────────────────────────────────────────────────────────

/// Steps through a lesson one section at a time.
fn learn(number: usize, lesson: &Lesson, progress: &mut Progress, console: &mut Console) {
    viewer::print_header(number, lesson);
    let mut index = 0;
    while index < lesson.sections.len() {
        viewer::print_section(lesson, index);
        let prompt = ui::paint(
            "[enter] next · b back · r repeat · c full source · m menu",
            Style::Dim,
        );
        let Some(reply) = console.ask(&format!("{prompt} ")) else {
            return;
        };
        match reply.as_str() {
            "b" | "back" => index = index.saturating_sub(1),
            "r" | "repeat" => {}
            "c" | "code" => viewer::print_source(number, lesson),
            "m" | "menu" | "q" | "x" => return,
            _ => index += 1,
        }
    }

    viewer::print_exercises(lesson);
    progress.mark_completed(lesson.id);
    save(progress);
    println!();
    println!(
        "{}",
        ui::paint(&format!("Lesson {number} complete!"), Style::Good)
    );

    if !lesson.quiz.is_empty() {
        let reply = console.ask(&format!(
            "Take the {}-question quiz now? [Y/n] ",
            lesson.quiz.len()
        ));
        if matches!(reply.as_deref(), Some("" | "y" | "Y" | "yes")) {
            let questions: Vec<&Question> = lesson.quiz.iter().collect();
            take_quiz(lesson.id, &questions, progress, console);
        }
    }
}

fn quiz_command(
    query: Option<&str>,
    progress: &mut Progress,
    console: &mut Console,
) -> Result<(), String> {
    match query {
        None | Some("all" | "review" | "random") => {
            // Review mode: random questions from finished lessons (or from
            // everything, if nothing is finished yet).
            let finished: Vec<&Lesson> = lessons::all()
                .map(|(_, l)| l)
                .filter(|l| progress.is_completed(l.id))
                .collect();
            let pool: Vec<&Lesson> = if finished.is_empty() {
                lessons::all().map(|(_, l)| l).collect()
            } else {
                finished
            };
            let mut questions: Vec<&Question> = pool.iter().flat_map(|l| l.quiz).collect();
            Rng::from_entropy().shuffle(&mut questions);
            questions.truncate(15);
            println!(
                "{}",
                ui::paint(
                    &format!("Review quiz: {} random questions", questions.len()),
                    Style::Title
                )
            );
            let outcome = quiz::run(&questions, console);
            report(outcome.correct, outcome.answered);
        }
        Some(query) => {
            let (n, lesson) = lessons::find(query)?;
            println!(
                "{}",
                ui::paint(
                    &format!("Quiz — Lesson {n}: {}", lesson.title),
                    Style::Title
                )
            );
            let questions: Vec<&Question> = lesson.quiz.iter().collect();
            take_quiz(lesson.id, &questions, progress, console);
        }
    }
    Ok(())
}

fn take_quiz(id: &str, questions: &[&Question], progress: &mut Progress, console: &mut Console) {
    let outcome = quiz::run(questions, console);
    report(outcome.correct, outcome.answered);
    if !outcome.quit && outcome.answered > 0 {
        progress.record_quiz(id, outcome.correct, outcome.answered);
        save(progress);
    }
}

fn report(correct: u32, answered: u32) {
    if answered == 0 {
        return;
    }
    let style = if correct * 10 >= answered * 8 {
        Style::Good
    } else {
        Style::Heading
    };
    println!();
    println!(
        "{}",
        ui::paint(&format!("Score: {correct}/{answered}"), style)
    );
}

fn save(progress: &Progress) {
    if let Err(e) = progress.save() {
        eprintln!("warning: could not save progress: {e}");
    }
}

// ─── Search & progress ────────────────────────────────────────────────────

fn search(term: &str) {
    let needle = term.to_lowercase();
    let mut hits = 0;
    for (n, lesson) in lessons::all() {
        let mut matched: Vec<&str> = lesson
            .sections
            .iter()
            .filter(|s| {
                let code = s
                    .anchor
                    .and_then(|a| crate::snippet::extract(lesson.source, a))
                    .unwrap_or_default();
                [s.title, s.text, code.as_str()]
                    .iter()
                    .any(|t| t.to_lowercase().contains(&needle))
            })
            .map(|s| s.title)
            .collect();
        let in_title = lesson.title.to_lowercase().contains(&needle)
            || lesson.summary.to_lowercase().contains(&needle);
        if matched.is_empty() && !in_title {
            continue;
        }
        hits += 1;
        println!("{:>3}. {}", n, ui::paint(lesson.title, Style::Heading));
        matched.dedup();
        for title in matched {
            println!("       › {title}");
        }
    }
    if hits == 0 {
        println!("No lessons mention “{term}”.");
    }
}

fn show_progress(progress: &Progress) {
    let total = lessons::all().count();
    let done = lessons::all()
        .filter(|(_, l)| progress.is_completed(l.id))
        .count();
    let bar_width = 30;
    let filled = done * bar_width / total.max(1);
    println!(
        "{}{} {done}/{total} lessons",
        ui::paint(&"█".repeat(filled), Style::Good),
        ui::paint(&"░".repeat(bar_width - filled), Style::Dim)
    );
    match progress.path() {
        Some(path) => println!(
            "{}",
            ui::paint(&format!("saved in {}", path.display()), Style::Dim)
        ),
        None => println!(
            "{}",
            ui::paint(
                "(progress is not being saved: no home directory)",
                Style::Dim
            )
        ),
    }
}

fn print_help() {
    let help = format!(
        "rust_std {VERSION} — an interactive course in standard Rust

USAGE:
    rust_std [FLAGS] [COMMAND]

COMMANDS:
    (none)              open the interactive menu
    list                list all lessons
    learn <lesson>      step through a lesson, section by section
    show <lesson>       print a whole lesson without pausing
    run <lesson>        show only the code and live output of each demo
    code <lesson>       print the lesson's complete source file
    quiz [<lesson>]     quiz on one lesson, or 15 random review questions
    search <text>       find lessons that mention <text>
    next                continue with the first unfinished lesson
    progress            show how far you've come
    reset               forget all progress
    help                show this message

    <lesson> is a number (7), an id (ownership), or a unique prefix (own).

FLAGS:
    --no-color, --color     force colors off / on (NO_COLOR is respected)
    --width <N>             wrap text at N columns (default: $COLUMNS or 80)
    -h, --help              show this message
    -V, --version           show the version

Progress is saved to $RUST_STD_PROGRESS or ~/.rust_std_progress."
    );
    println!("{help}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(args: &[&str]) -> Result<(Command, Options), String> {
        parse(args.iter().copied())
    }

    #[test]
    fn parses_commands() {
        assert_eq!(p(&[]).unwrap().0, Command::Menu);
        assert_eq!(p(&["list"]).unwrap().0, Command::List);
        assert_eq!(p(&["show", "7"]).unwrap().0, Command::Show("7".into()));
        assert_eq!(p(&["quiz"]).unwrap().0, Command::Quiz(None));
        assert_eq!(
            p(&["ownership"]).unwrap().0,
            Command::Learn("ownership".into())
        );
        assert_eq!(
            p(&["search", "drop", "order"]).unwrap().0,
            Command::Search("drop order".into())
        );
    }

    #[test]
    fn parses_flags_anywhere() {
        let (cmd, opts) = p(&["show", "--no-color", "3", "--width=60"]).unwrap();
        assert_eq!(cmd, Command::Show("3".into()));
        assert_eq!(
            opts,
            Options {
                color: Some(false),
                width: Some(60)
            }
        );
        assert_eq!(p(&["--width", "70"]).unwrap().1.width, Some(70));
    }

    #[test]
    fn rejects_bad_input() {
        assert!(p(&["--frobnicate"]).is_err());
        assert!(p(&["show"]).is_err());
        assert!(p(&["--width", "wide"]).is_err());
    }
}

//! Reading the learner's input.
//!
//! All interactive code reads through `Console` rather than touching
//! `stdin` directly, so tests can feed it scripted input from a byte slice.

use std::io::{self, BufRead, Write};

pub struct Console<'a> {
    input: Box<dyn BufRead + 'a>,
}

impl<'a> Console<'a> {
    pub fn stdin() -> Console<'static> {
        Console {
            input: Box::new(io::stdin().lock()),
        }
    }

    /// Reads from any buffered source — handy in tests:
    /// `Console::from_reader(&b"1\nq\n"[..])`.
    pub fn from_reader(reader: impl BufRead + 'a) -> Self {
        Console {
            input: Box::new(reader),
        }
    }

    /// Prints `prompt`, then reads one line. Returns `None` at end of input
    /// (Ctrl-D, or the end of a piped file) or on a read error.
    pub fn ask(&mut self, prompt: &str) -> Option<String> {
        print!("{prompt}");
        // `print!` doesn't flush: stdout is line-buffered, and the prompt
        // has no newline, so we flush by hand or it may never appear.
        io::stdout().flush().ok()?;
        let mut line = String::new();
        match self.input.read_line(&mut line) {
            Ok(0) | Err(_) => {
                println!();
                None
            }
            Ok(_) => Some(line.trim().to_string()),
        }
    }
}

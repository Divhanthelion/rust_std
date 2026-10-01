//! Saving which lessons you have finished, and your best quiz scores.
//!
//! The file is plain text, one lesson per line, with tab-separated fields
//! (shown here as `→`), so it is easy to read and easy to parse by hand:
//!
//! ```text
//! # rust_std progress v1
//! ownership → done → 4/5
//! borrowing → -    → 2/5
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;

const HEADER: &str = "# rust_std progress v1";

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Entry {
    pub completed: bool,
    /// Best quiz result as (correct, total).
    pub best_quiz: Option<(u32, u32)>,
}

#[derive(Debug, Default)]
pub struct Progress {
    path: Option<PathBuf>,
    entries: BTreeMap<String, Entry>,
}

impl Progress {
    /// Where progress is stored: `$RUST_STD_PROGRESS`, else
    /// `~/.rust_std_progress`. `None` if no home directory is known.
    pub fn default_path() -> Option<PathBuf> {
        if let Some(path) = std::env::var_os("RUST_STD_PROGRESS") {
            return Some(PathBuf::from(path));
        }
        let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
        Some(PathBuf::from(home).join(".rust_std_progress"))
    }

    /// Loads progress from the default location. A missing or unreadable
    /// file is not an error: you simply start fresh.
    pub fn load() -> Self {
        let path = Self::default_path();
        let entries = path
            .as_ref()
            .and_then(|p| fs::read_to_string(p).ok())
            .map(|text| parse(&text))
            .unwrap_or_default();
        Progress { path, entries }
    }

    /// An in-memory store that never touches the disk.
    pub fn in_memory() -> Self {
        Progress::default()
    }

    pub fn save(&self) -> io::Result<()> {
        match &self.path {
            Some(path) => fs::write(path, serialize(&self.entries)),
            None => Ok(()),
        }
    }

    pub fn entry(&self, id: &str) -> Entry {
        self.entries.get(id).cloned().unwrap_or_default()
    }

    pub fn is_completed(&self, id: &str) -> bool {
        self.entries.get(id).is_some_and(|e| e.completed)
    }

    pub fn mark_completed(&mut self, id: &str) {
        self.entries.entry(id.to_string()).or_default().completed = true;
    }

    /// Records a quiz result, keeping only the best ratio seen so far.
    pub fn record_quiz(&mut self, id: &str, correct: u32, total: u32) {
        let entry = self.entries.entry(id.to_string()).or_default();
        let better = match entry.best_quiz {
            None => true,
            // Compare fractions without floating point: a/b > c/d  ⇔  a·d > c·b
            Some((c, t)) => u64::from(correct) * u64::from(t) > u64::from(c) * u64::from(total),
        };
        if better {
            entry.best_quiz = Some((correct, total));
        }
    }

    pub fn reset(&mut self) -> io::Result<()> {
        self.entries.clear();
        match &self.path {
            Some(path) if path.exists() => fs::remove_file(path),
            _ => Ok(()),
        }
    }

    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }
}

fn parse(text: &str) -> BTreeMap<String, Entry> {
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let id = fields.next()?.trim();
            let completed = fields.next() == Some("done");
            let best_quiz = fields.next().and_then(|score| {
                let (correct, total) = score.split_once('/')?;
                Some((correct.parse().ok()?, total.parse().ok()?))
            });
            Some((
                id.to_string(),
                Entry {
                    completed,
                    best_quiz,
                },
            ))
        })
        .collect()
}

fn serialize(entries: &BTreeMap<String, Entry>) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    for (id, entry) in entries {
        let done = if entry.completed { "done" } else { "-" };
        let quiz = match entry.best_quiz {
            Some((c, t)) => format!("{c}/{t}"),
            None => "-".to_string(),
        };
        out.push_str(&format!("{id}\t{done}\t{quiz}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut p = Progress::in_memory();
        p.mark_completed("ownership");
        p.record_quiz("ownership", 3, 5);
        p.record_quiz("borrowing", 1, 4);
        let text = serialize(&p.entries);
        assert_eq!(parse(&text), p.entries);
    }

    #[test]
    fn keeps_best_score() {
        let mut p = Progress::in_memory();
        p.record_quiz("x", 4, 5);
        p.record_quiz("x", 2, 5);
        assert_eq!(p.entry("x").best_quiz, Some((4, 5)));
        p.record_quiz("x", 5, 5);
        assert_eq!(p.entry("x").best_quiz, Some((5, 5)));
    }

    #[test]
    fn ignores_garbage_lines() {
        let parsed = parse("# header\n\nok\tdone\t1/2\nbad\tdone\tx/y\n");
        assert_eq!(parsed["ok"].best_quiz, Some((1, 2)));
        assert_eq!(parsed["bad"].best_quiz, None);
        assert!(parsed["bad"].completed);
    }
}

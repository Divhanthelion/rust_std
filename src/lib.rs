//! # rust_std
//!
//! An interactive course in Rust that uses nothing but the standard library.
//!
//! The course content lives in [`lessons`]. Each lesson is a source file whose
//! demo functions are ordinary, compiled Rust; the CLI shows their code
//! (cut out of the file with [`snippet::extract`]) and then runs them.
//!
//! The rest of the crate is the machinery around the content — and is itself
//! written to be read: argument parsing, a Markdown-ish renderer, a syntax
//! highlighter, a PRNG, and file-based progress tracking, all with `std` only.

pub mod alloc_counter;
pub mod cli;
pub mod console;
pub mod lesson;
pub mod lessons;
pub mod progress;
pub mod quiz;
pub mod rng;
pub mod snippet;
pub mod ui;
pub mod viewer;

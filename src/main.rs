//! The `rust_std` binary. All the logic lives in the library crate
//! (`src/lib.rs`) so integration tests and doc tests can reach it.

fn main() -> std::process::ExitCode {
    rust_std::cli::main()
}

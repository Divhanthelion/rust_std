//! Lesson: I/O, files & processes.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "io",
    title: "I/O, files & processes",
    summary: "stdin/stdout, the Read/Write/BufRead traits, buffering, files and directories, paths, io::Error, environment variables and child processes.",
    source: include_str!("io.rs"),
    sections: &[
        Section::new(
            "Standard streams",
            r#"
            `io::stdin()`, `io::stdout()` and `io::stderr()` are handles to the
            process's standard streams. Reading a line looks like this:

            ```rust
            # fn demo() -> std::io::Result<()> {
            let mut line = String::new();
            let bytes = std::io::stdin().read_line(&mut line)?; // 0 means end of input
            let answer = line.trim();
            # let _ = (bytes, answer); Ok(()) }
            ```

            `print!` locks stdout on **every call**. For heavy output, lock
            once with `stdout().lock()` and use `writeln!`, which is much
            faster in loops. Stdout is line-buffered when it's a terminal, so a
            prompt without a newline needs an explicit `flush()`. This
            program's `Console::ask` does exactly that.
            "#,
        )
        .demo("stdio", stdio),
        Section::new(
            "Read, Write and BufRead: program to the traits",
            r#"
            I/O in Rust is built on three traits:

            - `Read` produces bytes, via `read`, `read_to_end`, `read_to_string`
              and `read_exact`;
            - `Write` consumes bytes, via `write`, `write_all` and `flush`, plus
              the `write!` macro;
            - `BufRead` is a `Read` with an internal buffer, which provides
              `read_line` and `lines()`.

            Files, sockets, stdin, `&[u8]`, `Vec<u8>` and `Cursor` all
            implement them. Write your functions against `impl Read` or
            `impl BufRead` instead of `File`, and you can test them with bytes
            in memory. This lesson's demos never touch a real terminal for
            input.
            "#,
        )
        .demo("traits", io_traits),
        Section::new(
            "Buffering",
            r#"
            Every `read` or `write` on a raw `File` is a system call. Wrap the
            file in `BufReader` or `BufWriter` to batch many small operations
            into a few large ones; on line-oriented work this is often a 10–100×
            speedup.

            A `BufWriter` flushes when dropped, but **errors during that flush
            are silently ignored**. Call `flush()` yourself, or `into_inner()`,
            to find out whether your data actually reached the file.
            "#,
        )
        .demo("buffering", buffering),
        Section::new(
            "Files",
            r#"
            The `fs` module has whole-file conveniences, `fs::read_to_string`,
            `fs::read` and `fs::write`, that are perfect for configuration and
            small data. For streaming or fine control:

            - `File::create(path)` truncates or creates a file for writing;
            - `File::open(path)` opens a file read-only;
            - `OpenOptions` chooses exactly: `.append(true)`,
              `.create_new(true)` (fail if the file exists), `.read`, `.write`.

            Files close automatically when the `File` is dropped (RAII again).
            "#,
        )
        .demo("files", files),
        Section::new(
            "Directories and metadata",
            r#"
            `fs::create_dir_all` makes a directory and its parents.
            `fs::read_dir` iterates over entries, each one an `io::Result`
            because reading a directory can fail midway. `fs::metadata` reports
            size, type and timestamps. `rename`, `copy`, `remove_file` and
            `remove_dir_all` do what they say.

            `read_dir` order is unspecified, so sort the names when you need
            stable output.
            "#,
        )
        .demo("dirs", directories),
        Section::new(
            "Paths",
            r#"
            `Path` (borrowed) and `PathBuf` (owned) are to file paths what
            `&str` and `String` are to text, but built on `OsStr`, because
            paths needn't be valid UTF-8. Build them with `join`, never by
            string concatenation, which gets separators wrong across platforms.
            Then take them apart with `file_name`, `file_stem`, `extension`,
            `parent` and `components`.
            "#,
        )
        .demo("paths", paths),
        Section::new(
            "I/O errors",
            r#"
            Every fallible I/O operation returns `io::Result<T>`, that is
            `Result<T, io::Error>`. Use `error.kind()` to react to the **kind**
            of failure, such as `NotFound`, `PermissionDenied`,
            `AlreadyExists`, `UnexpectedEof` or `InvalidData`, and not to the
            message text. `raw_os_error()` exposes the underlying OS code when
            there is one.
            "#,
        )
        .demo("errors", io_errors),
        Section::new(
            "Environment and arguments",
            r#"
            `std::env` connects you to the process environment:

            - `env::args()` gives the command-line arguments, with the program
              name first. `args_os()` doesn't panic on non-UTF-8 arguments.
            - `env::var("KEY")` returns `Result<String, VarError>`, which
              distinguishes `NotPresent` from `NotUnicode`.
            - `current_dir`, `temp_dir` and `current_exe` report locations.

            In edition 2024, `env::set_var` and `env::remove_var` are
            **`unsafe`**. On most platforms, changing the environment while
            another thread reads it is undefined behaviour in the C library.
            Pass configuration explicitly instead, for example through
            `Command::env` for child processes.
            "#,
        )
        .demo("env", environment),
        Section::new(
            "Running other programs",
            r#"
            `std::process::Command` builds and runs a child process:

            - `.output()` runs it to completion and captures stdout, stderr and
              the exit status;
            - `.status()` runs it with inherited streams and returns only the
              status;
            - `.spawn()` starts it and returns a `Child` whose
              `stdin`/`stdout` you can pipe to and from.

            Arguments are passed as a list, so no shell parses them, which
            rules out quoting bugs and injection. Run through `sh -c` only when
            you really need a shell. And always check the exit status: a child
            that ran isn't the same as a child that succeeded.
            "#,
        )
        .demo("process", processes),
    ],
    quiz: &[
        Question::new(
            "Why write `fn parse(input: impl BufRead)` instead of taking a `File`?",
            &[
                "It's faster",
                "The function then works with files, stdin, sockets and in-memory bytes, which makes testing easy",
                "File doesn't implement Read",
                "It avoids errors",
            ],
            1,
            "Programming against the I/O traits decouples logic from the data source.",
        ),
        Question::new(
            "What can go wrong if you rely on a BufWriter being flushed when it's dropped?",
            &[
                "Nothing",
                "The data is written twice",
                "Errors from the final flush are silently ignored",
                "It panics",
            ],
            2,
            "Drop can't return a Result, so flush errors are lost. Flush explicitly to observe them.",
        ),
        Question::new(
            "How should you build the path `logs/2026/can.log`?",
            &[
                "format!(\"{}/{}/{}\", a, b, c)",
                "Path::new(\"logs\").join(\"2026\").join(\"can.log\")",
                "String concatenation with '\\\\'",
                "Any of these is fine everywhere",
            ],
            1,
            "`join` uses the right separator for the platform and handles absolute components correctly.",
        ),
        Question::new(
            "In edition 2024, why is `std::env::set_var` an unsafe function?",
            &[
                "It can change PATH",
                "Concurrent environment access from other threads (e.g. in C code) is undefined behaviour on many platforms",
                "It allocates",
                "It's deprecated",
            ],
            1,
            "The C environment isn't thread-safe. The caller must guarantee no other thread reads or writes it concurrently.",
        ),
        Question::new(
            "Why prefer `Command::new(\"grep\").arg(pattern)` over `Command::new(\"sh\").arg(\"-c\").arg(format!(\"grep {pattern}\"))`?",
            &[
                "It's shorter",
                "Arguments are passed directly, with no shell parsing, which prevents quoting bugs and injection",
                "sh doesn't exist",
                "grep requires it",
            ],
            1,
            "Without a shell in the middle, each argument reaches the program exactly as given.",
        ),
    ],
    exercises: &[
        "Write `fn count_lines(r: impl BufRead) -> io::Result<(usize, usize)>` returning (lines, words). Test it with a `Cursor` and run it on a real file.",
        "Write a program that appends a timestamped line to `drive.log` on each run, using `OpenOptions::new().create(true).append(true)`.",
        "List all `.rs` files under a directory recursively with `fs::read_dir`, sorted, with their sizes.",
        "Run `rustc --version` with `Command::output`, parse the version number out of stdout, and handle the case where rustc is missing.",
    ],
};

/// A scratch directory for this lesson's demos, removed afterwards.
fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("rust_std_{name}_{}", std::process::id()))
}

fn stdio() {
    // ANCHOR: stdio
    let stdout = io::stdout();
    let mut out = stdout.lock(); // lock once for many writes
    for i in 1..=3 {
        writeln!(out, "line {i} via a locked stdout").unwrap();
    }
    write!(out, "a prompt without newline… ").unwrap();
    out.flush().unwrap(); // make it appear now
    writeln!(out, "flushed.").unwrap();
    drop(out);

    eprintln!("(diagnostics belong on stderr)");
    // ANCHOR_END: stdio
}

fn io_traits() {
    // ANCHOR: traits
    // Logic written against the traits...
    fn sum_numbers(input: impl BufRead) -> io::Result<i64> {
        let mut total = 0;
        for line in input.lines() {
            let line = line?;
            if let Ok(n) = line.trim().parse::<i64>() {
                total += n;
            }
        }
        Ok(total)
    }
    // ...works on in-memory data (and on files, sockets and stdin).
    let fake_file = Cursor::new("10\n20\nnot a number\n-5\n");
    println!("sum = {:?}", sum_numbers(fake_file));

    let mut bytes: &[u8] = b"\x01\x02\x03\x04rest";
    let mut header = [0u8; 4];
    bytes.read_exact(&mut header).unwrap(); // &[u8] advances as you read
    println!(
        "header {header:?}, remaining {:?}",
        std::str::from_utf8(bytes)
    );

    let mut sink: Vec<u8> = Vec::new(); // Vec<u8> implements Write
    write!(sink, "rpm={}", 3200).unwrap();
    sink.write_all(b";ok").unwrap();
    println!("written: {:?}", String::from_utf8_lossy(&sink));

    let copied = io::copy(&mut Cursor::new(vec![7u8; 1000]), &mut io::sink()).unwrap();
    println!("io::copy moved {copied} bytes into io::sink()");
    // ANCHOR_END: traits
}

fn buffering() {
    // ANCHOR: buffering
    let dir = scratch("buffering");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("samples.csv");

    {
        let file = File::create(&path).unwrap();
        let mut w = BufWriter::new(file); // batches small writes
        writeln!(w, "t_ms,rpm").unwrap();
        for t in 0..1000 {
            writeln!(w, "{},{}", t * 10, 800 + (t % 50) * 40).unwrap();
        }
        w.flush().unwrap(); // surface any error now, not silently in Drop
    }

    let reader = BufReader::new(File::open(&path).unwrap());
    let mut max_rpm = 0;
    let mut rows = 0;
    for line in reader.lines().skip(1) {
        let line = line.unwrap();
        if let Some((_, rpm)) = line.split_once(',') {
            max_rpm = max_rpm.max(rpm.parse::<u32>().unwrap_or(0));
            rows += 1;
        }
    }
    println!(
        "{rows} rows, max rpm {max_rpm}, file size {} bytes",
        fs::metadata(&path).unwrap().len()
    );
    fs::remove_dir_all(&dir).unwrap();
    // ANCHOR_END: buffering
}

fn files() {
    // ANCHOR: files
    let dir = scratch("files");
    fs::create_dir_all(&dir).unwrap();
    let config = dir.join("vehicle.conf");

    fs::write(&config, "model=RAV4\nyear=2026\n").unwrap(); // whole file at once
    let text = fs::read_to_string(&config).unwrap();
    println!("read back:\n{}", text.trim_end());

    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("trip.log"))
        .unwrap();
    writeln!(log, "trip start").unwrap();
    writeln!(log, "trip end").unwrap();
    drop(log); // closes the file

    let mut contents = String::new();
    File::open(dir.join("trip.log"))
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();
    println!("trip.log has {} lines", contents.lines().count());

    let again = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&config);
    println!(
        "create_new on an existing file → {:?}",
        again.map_err(|e| e.kind())
    );
    fs::remove_dir_all(&dir).unwrap();
    // ANCHOR_END: files
}

fn directories() {
    // ANCHOR: dirs
    let root = scratch("dirs");
    fs::create_dir_all(root.join("logs/2026")).unwrap();
    for (name, body) in [("a.log", "x"), ("b.log", "yyyy"), ("notes.txt", "zz")] {
        fs::write(root.join("logs/2026").join(name), body).unwrap();
    }

    let mut entries: Vec<(String, u64)> = fs::read_dir(root.join("logs/2026"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
        .map(|e| {
            (
                e.file_name().to_string_lossy().into_owned(),
                e.metadata().map(|m| m.len()).unwrap_or(0),
            )
        })
        .collect();
    entries.sort(); // read_dir order is unspecified
    println!(".log files: {entries:?}");

    let meta = fs::metadata(root.join("logs")).unwrap();
    println!("logs is_dir {} is_file {}", meta.is_dir(), meta.is_file());

    fs::rename(root.join("logs/2026/a.log"), root.join("logs/2026/a.old")).unwrap();
    println!(
        "a.log exists after rename? {}",
        root.join("logs/2026/a.log").exists()
    );
    fs::remove_dir_all(&root).unwrap();
    println!("cleaned up: {}", !root.exists());
    // ANCHOR_END: dirs
}

fn paths() {
    // ANCHOR: paths
    let base = Path::new("logs");
    let full: PathBuf = base.join("2026").join("can0.trace.log");
    println!("joined:    {}", full.display());
    println!("file_name: {:?}", full.file_name());
    println!("file_stem: {:?}", full.file_stem());
    println!("extension: {:?}", full.extension());
    println!("parent:    {:?}", full.parent());
    println!("with_ext:  {}", full.with_extension("bin").display());
    let parts: Vec<_> = full
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    println!("components {parts:?}");
    println!("absolute?  {}", full.is_absolute());
    // ANCHOR_END: paths
}

fn io_errors() {
    // ANCHOR: errors
    fn load(path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    match load(Path::new("/definitely/not/here.conf")) {
        Ok(_) => println!("unexpectedly found it"),
        Err(e) if e.kind() == io::ErrorKind::NotFound => println!("not found → use defaults ({e})"),
        Err(e) => println!("other error: {e}"),
    }

    let mut short: &[u8] = &[1, 2];
    let mut buf = [0u8; 4];
    let err = short.read_exact(&mut buf).unwrap_err();
    println!("read_exact on 2 bytes → {:?}", err.kind());

    let custom = io::Error::new(io::ErrorKind::InvalidData, "checksum mismatch");
    println!("custom error: {custom} (kind {:?})", custom.kind());
    let other = io::Error::other("bus off"); // Rust 1.74+
    println!("io::Error::other: {other}");
    // ANCHOR_END: errors
}

fn environment() {
    // ANCHOR: env
    let args: Vec<String> = std::env::args().collect();
    println!(
        "program: {:?} ({} extra args)",
        Path::new(&args[0]).file_name().unwrap_or_default(),
        args.len() - 1
    );

    match std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        Ok(home) => println!("home directory is {} chars long", home.len()),
        Err(e) => println!("no home: {e}"),
    }
    println!(
        "RUST_STD_DEMO_UNSET → {:?}",
        std::env::var("RUST_STD_DEMO_UNSET")
    );
    println!(
        "PATH has {} entries",
        std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).count())
            .unwrap_or(0)
    );
    println!("temp dir exists: {}", std::env::temp_dir().exists());
    // ANCHOR_END: env
}

fn processes() {
    // ANCHOR: process
    // A shell is used here only to have a portable command to run.
    fn shell(script: &str) -> Command {
        let mut cmd = if cfg!(windows) {
            Command::new("cmd")
        } else {
            Command::new("sh")
        };
        cmd.arg(if cfg!(windows) { "/C" } else { "-c" }).arg(script);
        cmd
    }

    match shell("echo hello from a child process").output() {
        Ok(out) => println!(
            "stdout: {:?}, success: {}",
            String::from_utf8_lossy(&out.stdout).trim(),
            out.status.success()
        ),
        Err(e) => println!("could not start the child: {e}"),
    }

    match shell("exit 3").status() {
        Ok(status) => println!("exit code: {:?}", status.code()),
        Err(e) => println!("could not start the child: {e}"),
    }

    // Pipe data into a child's stdin and read its stdout:
    let child = shell(if cfg!(windows) { "sort" } else { "sort -n" })
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();
    if let Ok(mut child) = child {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(b"30\n4\n200\n").unwrap();
        } // stdin dropped → the child sees end of input
        let out = child.wait_with_output().unwrap();
        let sorted: Vec<&str> = std::str::from_utf8(&out.stdout)
            .unwrap_or("")
            .lines()
            .collect();
        println!("sorted by the child: {sorted:?}");
    }

    let missing = Command::new("definitely-not-a-real-program-xyz").output();
    println!("missing program → {:?}", missing.map_err(|e| e.kind()));
    // ANCHOR_END: process
}

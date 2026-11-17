use azuki::experimental::formatter::{FormatOptions, format};

use std::{
    env, fs,
    io::{self, Read, Write},
    path::PathBuf,
    process::ExitCode,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("azuki-fmt: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = FormatOptions::default();
    let mut path = None;
    let mut args = env::args_os().skip(1);
    let mut positional = false;
    while let Some(arg) = args.next() {
        if !positional && arg == "--help" {
            println!("Usage: azuki-fmt [--max-width COLUMNS] [--] [FILE]");
            println!("Reads FILE or standard input and writes formatted text to standard output.");
            return Ok(());
        }
        if !positional && arg == "--" {
            positional = true;
        } else if !positional && arg == "--max-width" {
            let width = args.next().ok_or("--max-width requires a number")?;
            options.max_width = width
                .to_str()
                .ok_or("--max-width requires a number")?
                .parse()?;
        } else if !positional && arg.to_str().is_some_and(|arg| arg.starts_with('-')) {
            return Err(format!("unknown option: {}", arg.to_string_lossy()).into());
        } else if path.is_some() {
            return Err("expected at most one input file".into());
        } else {
            path = Some(PathBuf::from(arg));
        }
    }

    let source = match path {
        Some(path) => fs::read_to_string(path)?,
        None => {
            let mut source = String::new();
            io::stdin().read_to_string(&mut source)?;
            source
        }
    };
    io::stdout()
        .lock()
        .write_all(format(&source, options).as_bytes())?;
    Ok(())
}

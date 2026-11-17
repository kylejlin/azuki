mod analysis;
mod diagnostics;
mod index;

use std::io::{self, Read, Write};

fn main() {
    if let Err(error) = run() {
        eprintln!("azuki_ide: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut source = String::new();
    io::stdin().read_to_string(&mut source)?;
    let analysis = analysis::analyze(&source);
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    serde_json::to_writer(&mut stdout, &analysis)?;
    stdout.flush()?;
    Ok(())
}

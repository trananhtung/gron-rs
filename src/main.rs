//! `gron` command-line tool: make JSON greppable, and turn it back.
//!
//! Usage:
//!   gron [FILE]            flatten JSON (from FILE or stdin) into assignment lines
//!   gron -u | --ungron     reconstruct JSON from gron lines on stdin
//!   gron --root NAME       use NAME instead of `json` as the root identifier

use std::io::{self, Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("gron: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut ungron_mode = false;
    let mut root = String::from("json");
    let mut file: Option<String> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-u" | "--ungron" => ungron_mode = true,
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-V" | "--version" => {
                println!("gron {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--root" => {
                root = args.next().ok_or("--root requires a value")?;
            }
            other if other.starts_with('-') && other != "-" => {
                return Err(format!("unknown option: {other}").into());
            }
            other => file = Some(other.to_string()),
        }
    }

    let input = read_input(file.as_deref())?;
    let stdout = io::stdout();
    let mut out = stdout.lock();

    if ungron_mode {
        let value = gron::ungron(&input)?;
        serde_json::to_writer_pretty(&mut out, &value)?;
        out.write_all(b"\n")?;
    } else {
        let value: serde_json::Value = serde_json::from_str(input.trim())?;
        out.write_all(gron::gron_with_root(&value, &root).as_bytes())?;
    }
    Ok(())
}

fn read_input(file: Option<&str>) -> io::Result<String> {
    match file {
        Some(path) if path != "-" => std::fs::read_to_string(path),
        _ => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

fn print_help() {
    println!(
        "gron {} — make JSON greppable\n\n\
         USAGE:\n  \
         gron [FILE]            flatten JSON into assignment lines\n  \
         gron -u | --ungron     reconstruct JSON from gron lines (stdin)\n  \
         gron --root NAME       root identifier (default: json)\n\n\
         With no FILE, reads from stdin.",
        env!("CARGO_PKG_VERSION")
    );
}

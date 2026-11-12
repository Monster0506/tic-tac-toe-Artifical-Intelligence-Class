//! `cargo ranking`: the ranking study (see `src/study.rs`) as its own program, so the
//! app's executable is free to be rebuilt while it runs. Ctrl+C stops it; running it
//! again carries on where it stopped.

// Shared with the app; this program uses only part of them.
#![allow(dead_code)]

#[path = "../roster.rs"]
mod roster;
#[path = "../sizes.rs"]
mod sizes;
#[path = "../study.rs"]
mod study;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(study::main(&args));
}

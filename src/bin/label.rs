use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use c4::board::{B87, Board};
use c4::solver::{Outcome, Solver};

const fn symbol(outcome: Option<Outcome>) -> char {
    match outcome {
        None => '.',
        Some(Outcome::Win) => '+',
        Some(Outcome::Draw) => '=',
        Some(Outcome::Loss) => '-',
    }
}

fn main() -> io::Result<ExitCode> {
    let mut solver = Solver::<B87>::default();
    let mut out = io::stdout().lock();
    for (index, line) in io::stdin().lock().lines().enumerate() {
        let moves = line?;
        let Some(board) = Board::<B87>::from_moves(&moves).filter(|_| !moves.is_empty()) else {
            eprintln!("line {}: rejected move string: {moves:?}", index + 1);
            return Ok(ExitCode::FAILURE);
        };
        let label = solver.label(board);
        let symbols: String = label
            .columns
            .iter()
            .map(|&outcome| symbol(outcome))
            .collect();
        writeln!(out, "{moves} {} {symbols}", symbol(Some(label.result)))?;
        out.flush()?;
    }
    Ok(ExitCode::SUCCESS)
}

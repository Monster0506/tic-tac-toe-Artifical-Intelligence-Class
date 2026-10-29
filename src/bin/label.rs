use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use c4::board::{B87, Board};
use c4::outcome::Outcome;
use c4::solver::Solver;

const TABLE_BITS: u32 = 21;

fn main() -> io::Result<ExitCode> {
    let mut solver = Solver::<B87>::new(TABLE_BITS);
    let mut out = io::stdout().lock();
    for (index, line) in io::stdin().lock().lines().enumerate() {
        let moves = line?;
        let Some(board) = Board::<B87>::from_moves(&moves).filter(|_| !moves.is_empty()) else {
            eprintln!("line {}: rejected move string: {moves:?}", index + 1);
            return Ok(ExitCode::FAILURE);
        };
        let Some(label) = solver.label(board, None) else {
            eprintln!("line {}: search aborted", index + 1);
            return Ok(ExitCode::FAILURE);
        };
        let symbols: String = label
            .columns
            .iter()
            .map(|&outcome| outcome.map_or('.', Outcome::symbol))
            .collect();
        writeln!(out, "{moves} {} {symbols}", label.result.symbol())?;
        out.flush()?;
    }
    Ok(ExitCode::SUCCESS)
}

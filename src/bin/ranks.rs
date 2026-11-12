//! The ranking study behind the app's agent order: a full round robin of all eight
//! agents for every board size and every time per move the app offers, from empty
//! boards and from random openings.
//!
//! Every game is appended to the output CSV as it finishes, and a restart skips the
//! games already there, so the study can be stopped and resumed. Cheap cells (small
//! boards, short times) run first.
//!
//! usage: ranks <out.csv> <workers> <games per pair and colour> [sizes] [ms list]
//!   sizes: e.g. 4x4,9x8 (default: all six); ms list: e.g. 1,10,100 (default: the app's)

use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::ops::ControlFlow;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use c4::board::{Board, Geometry, Grid};
use c4::dataset::unique_samples;
use c4::outcome::Outcome;
use c4::tournament::{Fixture, PERFECT_TABLE_BITS, ROSTER, Seat, Settings, run};

const USAGE: &str = "usage: ranks <out.csv> <workers> <games per pair and colour> [sizes] [ms list]";
const SIZES: [(u32, u32); 6] = [(4, 4), (5, 4), (6, 5), (7, 6), (8, 7), (9, 8)];
const TIMES_MS: [u64; 10] = [1, 10, 50, 100, 250, 500, 1000, 2000, 5000, 10_000];
const STARTS: [&str; 2] = ["empty", "random"];
const HEADER: &str = "start,size,ms,game,opening,first,second,first_result,plies";

struct Request {
    path: String,
    workers: usize,
    games: usize,
    sizes: Vec<(u32, u32)>,
    times: Vec<u64>,
}

fn parse(args: &[String]) -> Option<Request> {
    let [path, workers, games, rest @ ..] = args else {
        return None;
    };
    let sizes = match rest.first() {
        Some(list) => list
            .split(',')
            .map(|size| {
                let (cols, rows) = size.split_once('x')?;
                Some((cols.parse().ok()?, rows.parse().ok()?))
            })
            .collect::<Option<Vec<_>>>()?,
        None => SIZES.to_vec(),
    };
    let times = match rest.get(1) {
        Some(list) => list.split(',').map(|ms| ms.parse().ok()).collect::<Option<Vec<u64>>>()?,
        None => TIMES_MS.to_vec(),
    };
    Some(Request {
        path: path.clone(),
        workers: workers.parse().ok().filter(|count| *count >= 1)?,
        games: games.parse().ok().filter(|count| *count >= 1)?,
        sizes,
        times,
    })
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(request) = parse(&args) else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    match execute(&request) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ranks: {error}");
            ExitCode::FAILURE
        }
    }
}

/// The games already in the file: (start, size, ms, game).
fn done(path: &str) -> HashSet<(String, String, u64, u64)> {
    let Ok(file) = File::open(path) else {
        return HashSet::new();
    };
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<&str> = line.split(',').collect();
            let [start, size, ms, game, ..] = fields.as_slice() else {
                return None;
            };
            Some(((*start).to_owned(), (*size).to_owned(), ms.parse().ok()?, game.parse().ok()?))
        })
        .collect()
}

fn execute(request: &Request) -> Result<(), Box<dyn Error>> {
    let finished = done(&request.path);
    let fresh = finished.is_empty() && File::open(&request.path).is_err();
    let mut out = OpenOptions::new().create(true).append(true).open(&request.path)?;
    if fresh {
        writeln!(out, "{HEADER}")?;
    }
    // Cheap first: by time, then by board size.
    let mut cells: Vec<(u64, (u32, u32), &str)> = Vec::new();
    for &ms in &request.times {
        for &size in &request.sizes {
            for start in STARTS {
                cells.push((ms, size, start));
            }
        }
    }
    let begun = Instant::now();
    for (ms, size, start) in cells {
        let cell = Cell {
            start,
            ms,
            games: request.games,
            workers: request.workers,
        };
        let played = match size {
            (4, 4) => cell.play::<Grid<4, 4, u64>>(&finished, &mut out)?,
            (5, 4) => cell.play::<Grid<5, 4, u64>>(&finished, &mut out)?,
            (6, 5) => cell.play::<Grid<6, 5, u64>>(&finished, &mut out)?,
            (7, 6) => cell.play::<Grid<7, 6, u64>>(&finished, &mut out)?,
            (8, 7) => cell.play::<Grid<8, 7, u64>>(&finished, &mut out)?,
            (9, 8) => cell.play::<Grid<9, 8, u128>>(&finished, &mut out)?,
            (cols, rows) => return Err(format!("no board {cols}x{rows}").into()),
        };
        eprintln!(
            "{start:6} {}x{} {ms:>5} ms: {played} games  ({:.0} s so far)",
            size.0,
            size.1,
            begun.elapsed().as_secs_f64()
        );
    }
    Ok(())
}

struct Cell<'a> {
    start: &'a str,
    ms: u64,
    games: usize,
    workers: usize,
}

impl Cell<'_> {
    /// Plays the cell's games not yet in the file; returns how many it played.
    fn play<G: Geometry + 'static>(
        &self,
        finished: &HashSet<(String, String, u64, u64)>,
        out: &mut File,
    ) -> Result<usize, Box<dyn Error>> {
        let size = format!("{}x{}", G::COLUMNS, G::ROWS);
        let per_move = Duration::from_millis(self.ms);
        let cells = usize::try_from(G::COLUMNS * G::ROWS)?;
        // The app's random start: 4 to 8 stones, at most a quarter of the board.
        let high = 8.min(cells / 4).max(2);
        let low = 4.min(high);
        let seed = 0x00C4_5EED ^ u64::from(G::COLUMNS * 16 + G::ROWS);
        let openings: Vec<Board<G>> = if self.start == "random" {
            unique_samples::<G>(seed, self.games, low, high)
                .into_iter()
                .map(|sample| sample.board)
                .collect()
        } else {
            vec![Board::default(); self.games]
        };
        let mut fixtures = Vec::new();
        let mut index: u64 = 0;
        for (opening, &board) in openings.iter().enumerate() {
            for &first in &ROSTER {
                for &second in &ROSTER {
                    if first == second {
                        continue;
                    }
                    index += 1;
                    if finished.contains(&(self.start.to_owned(), size.clone(), self.ms, index)) {
                        continue;
                    }
                    fixtures.push(Fixture {
                        // The index seeds the random agents, so repeats from an empty
                        // board still differ.
                        index: index + self.ms * 1_000_003,
                        opening,
                        start: board,
                        first: Seat { entrant: first, per_move },
                        second: Seat { entrant: second, per_move },
                    });
                }
            }
        }
        let offset = self.ms * 1_000_003;
        let mut played = 0;
        let mut failure: Option<Box<dyn Error>> = None;
        run(&fixtures, Settings { perfect_table_bits: PERFECT_TABLE_BITS }, self.workers, |record| {
            let result = match record.outcome {
                Outcome::Win => "W",
                Outcome::Draw => "D",
                Outcome::Loss => "L",
            };
            let line = format!(
                "{},{size},{},{},{},{},{},{result},{}",
                self.start,
                self.ms,
                record.game - offset,
                record.opening,
                record.first.name(),
                record.second.name(),
                record.moves.len()
            );
            if let Err(error) = writeln!(out, "{line}").and_then(|()| out.flush()) {
                failure = Some(error.into());
                return ControlFlow::Break(());
            }
            played += 1;
            ControlFlow::Continue(())
        })
        .ok_or("could not build the network agents")?;
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(played)
    }
}

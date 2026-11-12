//! The ranking study behind the arena's agent order: a full round robin of the eight
//! agents, every pair once with each colour, for every board size, every time per move
//! and both starts (empty board, random opening).
//!
//! Run it with `cargo ranking` (its own small program, so the app can still be rebuilt
//! while it runs). It plays in rounds, so every setting gets data early, and more
//! rounds where games are cheap: 20 up to 0.25 s a move, 6 up to 2 s, 3 at 5 s, 2 at
//! 10 s. Every game is appended to `ranking/games.csv` (tracked by git) and flushed as
//! it finishes, so Ctrl+C loses at most the games being played, and the next run
//! carries on where this one stopped. The random 9x8 cells at 0.5, 1, 5 and 10 s hold
//! the original tournament's 2,800 games each (`cargo ranking --import-tournament`
//! added them), so the study skips those.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::sync::mpsc;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use c4::arena::{Pace, play};
use c4::board::{Board, Geometry, Grid};
use c4::dataset::unique_samples;
use c4::outcome::Outcome;
use c4::tournament::{Entrant, PERFECT_TABLE_BITS};

use crate::roster::ROSTER;
use crate::sizes::{LADDER, TIMES};

/// Where the games go, in the repository so they are kept.
const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/ranking/games.csv");
/// The same file, read when the program is built.
const GAMES: &str = include_str!("../ranking/games.csv");
const HEADER: &str = "start,size,ms,round,first,second,first_result,plies";
/// Rounds of the original tournament start here, so they never clash with the study's.
const IMPORTED_ROUND: usize = 1000;

/// How many rounds each time per move gets.
const fn rounds(ms: u64) -> usize {
    match ms {
        0..=250 => 20,
        251..=2000 => 6,
        2001..=5000 => 3,
        _ => 2,
    }
}

fn times_ms() -> Vec<u64> {
    TIMES.iter().map(|seconds| (seconds * 1000.0).round() as u64).collect()
}

fn size_name(index: usize) -> String {
    format!("{}x{}", LADDER[index].columns, LADDER[index].rows)
}

// ================================================================== the table

/// Scores by (random start, board index, time index): half-points per roster agent.
pub type Scores = HashMap<(bool, usize, usize), [u32; 8]>;

pub fn scores() -> &'static Scores {
    static SCORES: OnceLock<Scores> = OnceLock::new();
    SCORES.get_or_init(|| {
        let times = times_ms();
        let mut scores = Scores::new();
        for line in GAMES.lines().skip(1) {
            let Some(game) = Game::parse(line) else { continue };
            let (Some(size), Some(time)) = (
                (0..LADDER.len()).find(|&index| size_name(index) == game.size),
                times.iter().position(|&ms| ms == game.ms),
            ) else {
                continue;
            };
            let (Some(first), Some(second)) = (roster_index(&game.first), roster_index(&game.second)) else {
                continue;
            };
            let cell = scores.entry((game.start == "random", size, time)).or_insert([0; 8]);
            let (to_first, to_second) = match game.result.as_str() {
                "W" => (2, 0),
                "L" => (0, 2),
                _ => (1, 1),
            };
            cell[first] += to_first;
            cell[second] += to_second;
        }
        scores
    })
}

fn roster_index(name: &str) -> Option<usize> {
    ROSTER.iter().position(|info| info.entrant.name() == name)
}

// ================================================================== the study

struct Game {
    start: String,
    size: String,
    ms: u64,
    round: usize,
    first: String,
    second: String,
    result: String,
}

impl Game {
    fn parse(line: &str) -> Option<Self> {
        let fields: Vec<&str> = line.trim().split(',').collect();
        let [start, size, ms, round, first, second, result, plies] = fields.as_slice() else {
            return None;
        };
        plies.parse::<u32>().ok()?;
        if !matches!(*result, "W" | "D" | "L") {
            return None;
        }
        Some(Self {
            start: (*start).to_owned(),
            size: (*size).to_owned(),
            ms: ms.parse().ok()?,
            round: round.parse().ok()?,
            first: (*first).to_owned(),
            second: (*second).to_owned(),
            result: (*result).to_owned(),
        })
    }

    fn key(&self) -> Key {
        (self.start.clone(), self.size.clone(), self.ms, self.round, self.first.clone(), self.second.clone())
    }
}

type Key = (String, String, u64, usize, String, String);

/// One game to play.
#[derive(Clone)]
struct Job {
    random: bool,
    size: usize,
    ms: u64,
    round: usize,
    first: Entrant,
    second: Entrant,
}

impl Job {
    fn key(&self) -> Key {
        (
            if self.random { "random" } else { "empty" }.to_owned(),
            size_name(self.size),
            self.ms,
            self.round,
            self.first.name().to_owned(),
            self.second.name().to_owned(),
        )
    }

    fn seed(&self) -> u64 {
        let mut seed = 0x9E37_79B9_7F4A_7C15_u64;
        for part in [u64::from(self.random), self.size as u64, self.ms, self.round as u64] {
            seed = (seed ^ part).wrapping_mul(0x0100_0000_01B3);
        }
        seed ^ (ROSTER.iter().position(|info| info.entrant == self.first).unwrap_or(0) as u64) << 8
            ^ (ROSTER.iter().position(|info| info.entrant == self.second).unwrap_or(0) as u64) << 16
    }

    /// Plays it: the line for the CSV.
    fn play(&self) -> Option<String> {
        let (result, plies) = match (LADDER[self.size].columns, LADDER[self.size].rows) {
            (4, 4) => self.play_on::<Grid<4, 4, u64>>(),
            (5, 4) => self.play_on::<Grid<5, 4, u64>>(),
            (6, 5) => self.play_on::<Grid<6, 5, u64>>(),
            (7, 6) => self.play_on::<Grid<7, 6, u64>>(),
            (8, 7) => self.play_on::<Grid<8, 7, u64>>(),
            (9, 8) => self.play_on::<Grid<9, 8, u128>>(),
            _ => None,
        }?;
        let (start, size, ms, round, first, second) = self.key();
        Some(format!("{start},{size},{ms},{round},{first},{second},{result},{plies}"))
    }

    fn play_on<G: Geometry + 'static>(&self) -> Option<(&'static str, usize)> {
        let start = if self.random {
            // The app's random start: 4 to 8 stones, at most a quarter of the board.
            // Round r of every pair plays from the same opening.
            let cells = usize::try_from(G::COLUMNS * G::ROWS).ok()?;
            let high = 8.min(cells / 4).max(2);
            let openings = unique_samples::<G>(0x00C4_0BE4 ^ self.size as u64, 20, 4.min(high), high);
            openings.get(self.round)?.board
        } else {
            Board::<G>::default()
        };
        let seed = self.seed();
        let mut first = self.first.build::<G>(seed, PERFECT_TABLE_BITS)?;
        let mut second = self.second.build::<G>(seed ^ 0x5EED, PERFECT_TABLE_BITS)?;
        let per_move = Duration::from_millis(self.ms);
        let game = play(start, first.as_mut(), second.as_mut(), Pace { first: per_move, second: per_move });
        let result = match game.outcome {
            Outcome::Win => "W",
            Outcome::Draw => "D",
            Outcome::Loss => "L",
        };
        Some((result, game.moves.len()))
    }
}

/// Opens the CSV for appending, with its header, and on a fresh line (a game cut off
/// by Ctrl+C mid-write is ignored when read).
fn open_for_append() -> std::io::Result<File> {
    if let Some(folder) = std::path::Path::new(PATH).parent() {
        std::fs::create_dir_all(folder)?;
    }
    let mut file = OpenOptions::new().create(true).read(true).append(true).open(PATH)?;
    let length = file.metadata()?.len();
    if length == 0 {
        writeln!(file, "{HEADER}")?;
    } else {
        let mut last = [0_u8];
        file.seek(SeekFrom::End(-1))?;
        file.read_exact(&mut last)?;
        if last[0] != b'\n' {
            writeln!(file)?;
        }
    }
    Ok(file)
}

fn played() -> HashSet<Key> {
    let Ok(file) = File::open(PATH) else {
        return HashSet::new();
    };
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
        .filter_map(|line| Game::parse(&line))
        .map(|game| game.key())
        .collect()
}

/// `--ranking`: plays the study (or `--import-tournament`: adds the original 9x8
/// tournament's games). Returns the process exit code.
pub fn main(args: &[String]) -> i32 {
    let result = if args.iter().any(|arg| arg == "--import-tournament") { import() } else { study() };
    match result {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("ranking: {error}");
            1
        }
    }
}

/// Lowers this process's priority, so whatever else the computer is doing comes first
/// and it stays responsive; the study only uses the CPU time left over. (It never uses
/// the GPU: every agent, the network included, runs on the CPU.)
fn be_polite() {
    let pid = std::process::id().to_string();
    let lowered = if cfg!(windows) {
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &format!("(Get-Process -Id {pid}).PriorityClass = 'BelowNormal'")])
            .status()
    } else {
        std::process::Command::new("renice").args(["-n", "10", "-p", &pid]).status()
    };
    if !lowered.is_ok_and(|status| status.success()) {
        eprintln!("(could not lower the study's priority; it may slow other programs down)");
    }
}

fn study() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("C4_STUDY_PRIORITY").is_ok_and(|value| value == "normal") {
        eprintln!("running at normal priority (C4_STUDY_PRIORITY=normal)");
    } else {
        be_polite();
    }
    let workers = std::env::var("C4_WORKERS")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |cores| cores.get().saturating_sub(2).max(1)));
    let done = played();
    // Cells the original tournament already covers.
    let imported: HashSet<(String, u64)> = done
        .iter()
        .filter(|key| key.3 >= IMPORTED_ROUND)
        .map(|key| (key.1.clone(), key.2))
        .collect();
    // C4_STUDY_MS=1,10 plays only those times (handy for a quick check, or for
    // splitting the study between machines).
    let only: Option<Vec<u64>> =
        std::env::var("C4_STUDY_MS").ok().map(|list| list.split(',').filter_map(|ms| ms.trim().parse().ok()).collect());
    let times: Vec<u64> = times_ms().into_iter().filter(|ms| only.as_ref().is_none_or(|only| only.contains(ms))).collect();
    let most = times.iter().map(|&ms| rounds(ms)).max().unwrap_or(1);
    let total: usize = times.iter().map(|&ms| rounds(ms)).sum::<usize>() * LADDER.len() * 2 * 56;
    let mut finished = done.iter().filter(|key| key.3 < IMPORTED_ROUND).count();
    eprintln!(
        "ranking study: {finished} of about {total} games already in {PATH}; {workers} workers (C4_WORKERS to change). Ctrl+C to stop, run again to resume. Running at below-normal priority: the computer stays usable, and the study uses whatever CPU is spare."
    );
    let mut out = open_for_append()?;
    let begun = Instant::now();
    for round in 0..most {
        let mut jobs = Vec::new();
        for &ms in &times {
            if rounds(ms) <= round {
                continue;
            }
            for size in 0..LADDER.len() {
                for random in [false, true] {
                    if random && imported.contains(&(size_name(size), ms)) {
                        continue;
                    }
                    for first in &ROSTER {
                        for second in &ROSTER {
                            if first.entrant == second.entrant {
                                continue;
                            }
                            let job = Job { random, size, ms, round, first: first.entrant, second: second.entrant };
                            if !done.contains(&job.key()) {
                                jobs.push(job);
                            }
                        }
                    }
                }
            }
        }
        if jobs.is_empty() {
            continue;
        }
        // Longest first, so the round does not end waiting on one slow game.
        jobs.sort_by_key(|job| std::cmp::Reverse((job.ms, job.size)));
        let count = jobs.len();
        let queue = Mutex::new(jobs.into_iter().collect::<VecDeque<_>>());
        let (send, receive) = mpsc::channel::<Option<String>>();
        std::thread::scope(|scope| -> std::io::Result<()> {
            for _ in 0..workers {
                let send = send.clone();
                let queue = &queue;
                scope.spawn(move || {
                    loop {
                        let job = queue.lock().ok().and_then(|mut queue| queue.pop_front());
                        let Some(job) = job else { break };
                        if send.send(job.play()).is_err() {
                            break;
                        }
                    }
                });
            }
            drop(send);
            for line in receive {
                if let Some(line) = line {
                    // One write and a flush per game: Ctrl+C loses only games in play.
                    out.write_all(format!("{line}\n").as_bytes())?;
                    out.flush()?;
                    finished += 1;
                }
            }
            Ok(())
        })?;
        let hours = begun.elapsed().as_secs_f64() / 3600.0;
        eprintln!("round {} of {most} done ({count} games); {finished} of about {total} in all; {hours:.2} h this run", round + 1);
    }
    eprintln!("ranking study complete: rebuild the app to use it.");
    Ok(())
}

/// Adds the original 9x8 tournament (`../results/games.csv`) as random-start games.
fn import() -> Result<(), Box<dyn std::error::Error>> {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../results/games.csv");
    let done = played();
    let mut out = open_for_append()?;
    let mut added = 0;
    for line in BufReader::new(File::open(source)?).lines().map_while(Result::ok).skip(1) {
        let fields: Vec<&str> = line.split(',').collect();
        let [experiment, board, _, opening, first, second, first_ms, _, _, plies, result, ..] = fields.as_slice() else {
            continue;
        };
        if !experiment.starts_with("tournament_") || *board != "9x8" {
            continue;
        }
        let Ok(opening) = opening.parse::<usize>() else { continue };
        let key: Key = (
            "random".to_owned(),
            "9x8".to_owned(),
            first_ms.parse()?,
            IMPORTED_ROUND + opening,
            (*first).to_owned(),
            (*second).to_owned(),
        );
        if done.contains(&key) {
            continue;
        }
        writeln!(out, "random,9x8,{},{},{first},{second},{result},{plies}", key.2, key.3)?;
        added += 1;
    }
    out.flush()?;
    eprintln!("imported {added} tournament games into {PATH}");
    Ok(())
}

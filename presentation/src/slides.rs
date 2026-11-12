//! The presentation's content: what we did, how, and why the results came out the way
//! they did. Every number comes from `results/` (measurements.csv, accuracy.csv,
//! DEVIATIONS.md) or `midterm_proposal.md`.
//!
//! Pacing: about six seconds per press of Next, and every press moves something on
//! screen (a bar grows or lights up, the board grows, a tile or a layer arrives). A
//! press never only drops discs: games on the board slides play out by themselves.
//! `every_beat_moves_something` checks that.

use std::time::{Duration, Instant};

use c4::board::{B76, Board};
use c4::outcome::Outcome;
use c4::solver::Solver;

pub struct Slide {
    pub kicker: &'static str,
    pub title: &'static str,
    pub visual: Visual,
    /// One entry per press of Next: each brings in the next part of the visual and has
    /// about six seconds of things to say (shown in the presenter window, never on the
    /// slide).
    pub beats: &'static [Beat],
}

impl Slide {
    /// The time budget for this slide: six seconds for every 26 words (at least one
    /// lot per beat).
    pub fn seconds(&self) -> u32 {
        self.beats.iter().map(Beat::seconds).sum()
    }
}

pub const SECONDS_PER_BEAT: u32 = 6;
/// About what can be said in `SECONDS_PER_BEAT`.
pub const WORDS_PER_BEAT: usize = 26;

/// The whole talk, in seconds.
pub fn talk_seconds() -> u32 {
    SLIDES.iter().map(Slide::seconds).sum()
}

/// One step of a slide.
pub struct Beat {
    /// An optional short line under the visual while this step is showing.
    pub caption: &'static str,
    pub say: &'static str,
}

impl Beat {
    pub fn seconds(&self) -> u32 {
        let words = self.say.split_whitespace().count();
        u32::try_from(words.div_ceil(WORDS_PER_BEAT).max(1)).unwrap_or(1) * SECONDS_PER_BEAT
    }
}

const fn say(say: &'static str) -> Beat {
    Beat { caption: "", say }
}

const fn show(caption: &'static str, say: &'static str) -> Beat {
    Beat { caption, say }
}

pub enum Visual {
    /// One board that physically grows, a stage per beat, its size written beside it.
    Expand(&'static [Stage]),
    /// One board, centred, whose game plays itself out as the slide opens (one stage,
    /// one beat).
    Game(&'static [Stage]),
    /// A horizontal bar chart.
    Bars(Bars),
    /// The eight agents; the beat each roster entry shows up on.
    Agents(&'static [usize]),
    /// The convolutional network: board, layers, column scores, the move it picks, and
    /// the search tree it guides.
    Network,
    /// A 7x6 position with every column solved live by our solver, then its checks.
    Solved,
}

/// Beats the network slide uses (board, layers, scores, its move, the tree).
#[cfg(test)]
pub const NETWORK_BEATS: usize = 5;
/// Beats the solver slide uses (board, labels, best move, checks).
#[cfg(test)]
pub const SOLVED_BEATS: usize = 4;

pub struct Stage {
    pub columns: u32,
    pub rows: u32,
    pub name: &'static str,
    pub note: &'static str,
    /// Discs dropped in when this stage arrives (0-based columns; colours alternate on
    /// from the moves before).
    pub moves: &'static [u32],
}

/// A stage of a `Game`: just discs.
const fn drop(columns: u32, rows: u32, moves: &'static [u32]) -> Stage {
    Stage { columns, rows, name: "", note: "", moves }
}

pub struct Bars {
    pub caption: &'static str,
    /// Value at the left edge of the bars (so close values still look different).
    pub floor: f32,
    pub ceiling: f32,
    pub items: &'static [Bar],
}

pub struct Bar {
    pub label: &'static str,
    /// The plotted value (between the chart's floor and ceiling).
    pub value: f32,
    /// The value as written next to the bar.
    pub text: &'static str,
    /// The beat the bar grows in on.
    pub beat: usize,
    /// The beat it lights up orange on: the same beat means it grows in orange, a later
    /// one sweeps orange along it then.
    pub lit: Option<usize>,
}

const fn bar(label: &'static str, value: f32, text: &'static str, beat: usize) -> Bar {
    Bar { label, value, text, beat, lit: None }
}

const fn lit(label: &'static str, value: f32, text: &'static str, beat: usize, lit: usize) -> Bar {
    Bar {
        label,
        value,
        text,
        beat,
        lit: Some(lit),
    }
}

/// A 7x6 game, 1-based columns as `Board::from_moves` expects. Solved at startup.
pub const SOLVED_MOVES: &str = "4453432233554";

pub const SLIDES: &[Slide] = &[
    Slide {
        kicker: "group 3  \u{b7}  TJ Raklovits and Andrew Roddy",
        title: "Connect Four AI",
        visual: Visual::Game(&[drop(9, 8, &[4, 4, 3, 5, 5, 3, 4, 2, 6, 4, 3, 3, 5, 2, 6, 6, 2])]),
        beats: &[say(
            "Hey everyone, we're TJ and Andrew from group 3. Our project is Connect Four, played by eight \
             different AI agents. We trained them on an 8 by 7 board and tested them on 9 by 8. And \
             everything you'll see today, these slides included, is running live in our Rust code.",
        )],
    },
    Slide {
        kicker: "the question",
        title: "A board nobody has solved",
        visual: Visual::Expand(&[
            Stage { columns: 7, rows: 6, name: "7 \u{d7} 6", note: "solved\nfirst player wins", moves: &[3, 3, 2, 4, 4, 2, 3, 5] },
            Stage { columns: 8, rows: 7, name: "8 \u{d7} 7", note: "solved by John Tromp\nour training board", moves: &[7, 6] },
            Stage { columns: 9, rows: 8, name: "9 \u{d7} 8", note: "never solved\nour test board", moves: &[8, 3, 4, 4, 5, 1] },
        ]),
        beats: &[
            say("This is the normal 7 by 6 board. It's solved: with perfect play the first player wins."),
            say("Add a column and a row and you get 8 by 7. John Tromp solved it too. This is where we trained."),
            show(
                "does what they learn carry over, and why?",
                "One more of each and you're at 9 by 8. Nobody has solved it; on our hardware it would take something \
                 like 15 years. So our question: if we train agents on a board we can solve, do they still play well \
                 on one we can't? And if some methods carry over better than others, why?",
            ),
        ],
    },
    Slide {
        kicker: "scale",
        title: "How big is the game?",
        visual: Visual::Bars(Bars {
            caption: "positions (log scale, powers of ten)",
            floor: 0.0,
            ceiling: 35.0,
            items: &[
                bar("7x6", 12.65, "10^12.7", 0),
                bar("8x7", 26.7, "10^26.7", 1),
                lit("9x8", 34.3, "10^34.3", 2, 3),
            ],
        }),
        beats: &[
            say("To give you a feel for the size: 7 by 6 has about 4.5 trillion legal positions."),
            say("For 8 by 7 the upper bound is around 10 to the 26."),
            say("And 9 by 8 is around 10 to the 34."),
            say("This is a log scale, so every step up is millions of times bigger. You just can't brute force that."),
        ],
    },
    Slide {
        kicker: "step 1",
        title: "An exact solver",
        visual: Visual::Solved,
        beats: &[
            say("Step one was an exact solver: alpha-beta search on bitboards with a transposition table, like the published solvers."),
            say("Give it any position and it says whether each column wins, draws or loses."),
            say("These labels were worked out live when the app started. Here, the marked column wins."),
            show(
                "0 wrong",
                "We checked it against all 67,557 UCI positions and 6,000 of Pascal Pons' tests. It got every one right.",
            ),
        ],
    },
    Slide {
        kicker: "the wall",
        title: "Why not just solve 9x8?",
        visual: Visual::Bars(Bars {
            caption: "seconds to solve a position (log scale), by stones on the board",
            floor: -1.5,
            ceiling: 3.4,
            items: &[
                bar("48 stones", -1.03, "0.09 s", 0),
                bar("36 stones", -0.9, "0.13 s", 0),
                bar("30 stones", -0.5, "0.32 s", 1),
                bar("26 stones", 0.34, "2.2 s", 2),
                bar("24 stones", 1.07, "11.7 s", 3),
                bar("20 stones", 1.9, "80 s", 4),
                lit("16 stones", 2.9, "> 600 s", 5, 5),
                lit("8x7, 6 stones", 3.26, "> 30 min", 6, 6),
            ],
        }),
        beats: &[
            say("So why not just solve 9 by 8? Late in the game it's quick, about a tenth of a second."),
            say("With fewer stones it slows down: a third of a second at 30."),
            say("Two seconds at 26 stones."),
            say("At 24 the median is 12 seconds, and the worst one we saw took 31 minutes."),
            say("At 20 stones it's over a minute."),
            say("And at 16 stones every probe we ran went past ten minutes."),
            say("Even 8 by 7 with six stones took over half an hour. That wall is why we needed learning."),
        ],
    },
    Slide {
        kicker: "step 2",
        title: "The data",
        visual: Visual::Bars(Bars {
            caption: "positions labelled by the solver",
            floor: 0.0,
            ceiling: 110_000.0,
            items: &[
                lit("8x7 training", 106_898.0, "106,898", 0, 0),
                bar("8x7 openings", 2_000.0, "2,000", 1),
                bar("9x8 late game", 9_990.0, "9,990", 2),
            ],
        }),
        beats: &[
            say("We used the solver to build our data: about 107 thousand 8 by 7 positions, every column labelled."),
            say("Plus 2,000 harder openings, 10 to 12 stones, just for testing."),
            show(
                "192 cores, one position each",
                "And about 10 thousand late game 9 by 8 positions, to test transfer, solved on a 192 core cloud machine.",
            ),
        ],
    },
    Slide {
        kicker: "step 3",
        title: "Eight agents",
        // MCTS, AB learned, AB handcrafted, network MCTS, Allis, solver, network, safe random.
        visual: Visual::Agents(&[0, 2, 1, 3, 4, 1, 3, 5]),
        beats: &[
            say("Then we built eight agents. First, Monte Carlo tree search."),
            say("Alpha-beta with a handcrafted evaluation, and the exact solver itself, limited to one second."),
            say("Then the ones that learn: alpha-beta with a learned evaluation."),
            say("A convolutional neural network, and that network guiding MCTS."),
            say("Victor Allis' hand written strategy rules."),
            say("And safe random: random, but it always takes a win and blocks a loss. Every agent gets one second a move."),
        ],
    },
    Slide {
        kicker: "search",
        title: "Alpha-beta: how deep?",
        visual: Visual::Bars(Bars {
            caption: "8x7 move accuracy (%) by search depth",
            floor: 85.0,
            ceiling: 100.0,
            items: &[
                bar("depth 1", 90.8, "90.8%", 0),
                bar("depth 2", 93.8, "93.8%", 1),
                bar("depth 4", 95.6, "95.6%", 2),
                bar("depth 6", 96.3, "96.3%", 2),
                lit("depth 8", 97.5, "97.5%", 3, 3),
                lit("depth 12", 97.8, "97.8%", 4, 5),
            ],
        }),
        beats: &[
            say("Alpha-beta with a handcrafted evaluation of threats and row parity. At depth 1 it matches the solver 91 percent of the time."),
            say("Depth 2 gets to about 94."),
            say("Each extra bit of depth helps a little."),
            say("Depth 8 reaches 97.5 percent."),
            say("Depth 12 barely adds anything, and it looks at 35 times as many nodes."),
            show(
                "deeper still wins 2 of 3 games on 9x8",
                "But in real games on 9 by 8 the deeper search still wins two out of three, so depth does pay off.",
            ),
        ],
    },
    Slide {
        kicker: "search + learning",
        title: "Learning the evaluation",
        visual: Visual::Bars(Bars {
            caption: "held-out 8x7 result accuracy (%)",
            floor: 50.0,
            ceiling: 75.0,
            items: &[
                bar("constant guess", 56.6, "56.6%", 0),
                bar("handcrafted", 67.8, "67.8%", 1),
                lit("learned", 70.1, "70.1%", 2, 2),
            ],
        }),
        beats: &[
            say("Next we learned the evaluation instead of tuning it by hand. Guessing the most common result gets 56.6 percent."),
            say("Our hand tuned evaluation gets 67.8."),
            show(
                "on 9x8: Elo 722 learned vs 690 handcrafted",
                "A logistic regression on the same features, trained on solver labels, gets 70.1, and it holds up on 9 by 8: 722 Elo against 690.",
            ),
        ],
    },
    Slide {
        kicker: "search",
        title: "Monte Carlo tree search",
        visual: Visual::Bars(Bars {
            caption: "9x8 tournament Elo by time per move",
            floor: 600.0,
            ceiling: 780.0,
            items: &[
                bar("0.5 s", 694.5, "695", 0),
                lit("1 s", 739.9, "740", 1, 1),
                bar("5 s", 754.4, "754", 2),
                lit("10 s", 760.7, "761", 3, 3),
            ],
        }),
        beats: &[
            say("MCTS needs no evaluation at all. It plays random games to the end, about 740 thousand a second on 9 by 8."),
            show(
                "+ proven wins and losses",
                "It grows its tree toward the moves that win most, and never misses a proven win or loss. At one second it was our strongest agent.",
            ),
            say("Give it more time and it keeps improving."),
            say("Up to 761 Elo at ten seconds a move."),
        ],
    },
    Slide {
        kicker: "learning",
        title: "A network for any board",
        visual: Visual::Network,
        beats: &[
            say("For the neural network we went fully convolutional, so the same weights run on any board size."),
            say("Six layers of 64 channels, trained for 12 epochs on the CPU after our GPU setup failed."),
            say("Out comes a score for each column."),
            show(
                "92.4% of 8x7 moves  \u{b7}  91% of 9x8 moves",
                "It plays the solver's move 92.4 percent of the time on 8 by 7, and still 91 percent on 9 by 8.",
            ),
            show(
                "alone: Elo 37  \u{b7}  guiding MCTS: Elo 647",
                "But alone, one bad move loses a game. Used to guide MCTS's search, it reaches 647 Elo.",
            ),
        ],
    },
    Slide {
        kicker: "rules",
        title: "Victor Allis' strategy",
        visual: Visual::Bars(Bars {
            caption: "score vs safe random (%)",
            floor: 50.0,
            ceiling: 90.0,
            items: &[
                lit("all rules, 9x8", 78.2, "78.2%", 0, 0),
                bar("all rules, 8x7", 83.0, "83.0%", 1),
                bar("9x8, no claimeven", 83.5, "83.5%", 2),
                bar("9x8, parity flipped", 75.2, "75.2%", 3),
            ],
        }),
        beats: &[
            show(
                "win  \u{b7}  block  \u{b7}  play safe  \u{b7}  claimeven  \u{b7}  threats on good rows",
                "We also coded part of Victor Allis' strategy rules, with no search. On 9 by 8 they beat safe random 78 percent of the time.",
            ),
            say("And 83 percent on 8 by 7, so the ideas carry over."),
            say("Funny enough, turning off claimeven didn't hurt at all."),
            say("Flipping the parity hurt a little. Our set of rules is just too small for a strong game."),
        ],
    },
    Slide {
        kicker: "transfer",
        title: "Does it carry over?",
        visual: Visual::Bars(Bars {
            caption: "move accuracy against the solver (%): 8x7, then 9x8 late game",
            floor: 70.0,
            ceiling: 100.0,
            items: &[
                bar("MCTS", 99.1, "98.7 \u{2192} 99.1%", 0),
                bar("alpha-beta learned", 99.3, "98.9 \u{2192} 99.3%", 0),
                bar("exact solver (1 s)", 97.8, "97.8 \u{2192} 97.8%", 1),
                lit("network alone", 91.0, "92.4 \u{2192} 91.0%", 2, 2),
                bar("Allis rules", 89.7, "88.3 \u{2192} 89.7%", 3),
                bar("safe random", 77.8, "76.4 \u{2192} 77.8%", 3),
            ],
        }),
        beats: &[
            say("So does it carry over? The search agents play almost the same on 9 by 8 as on 8 by 7."),
            say("The time limited solver stays at 97.8."),
            say("The network drops only a little, from 92.4 to 91, even on a board it never saw."),
            say("And the rules and the baseline barely move. What they learned transfers."),
        ],
    },
    Slide {
        kicker: "results",
        title: "The 9x8 tournament",
        visual: Visual::Bars(Bars {
            caption: "9x8 Elo at 1 s per move",
            floor: 0.0,
            ceiling: 760.0,
            items: &[
                lit("MCTS", 739.9, "740", 4, 4),
                bar("alpha-beta learned", 722.1, "722", 3),
                bar("alpha-beta handcrafted", 690.3, "690", 2),
                bar("network MCTS", 647.1, "647", 2),
                bar("Allis rules", 160.4, "160", 1),
                bar("exact solver", 77.9, "78", 1),
                bar("network alone", 36.5, "37", 0),
                bar("safe random", 0.0, "0", 0),
            ],
        }),
        beats: &[
            show(
                "every pair  \u{b7}  50 openings  \u{b7}  both colours  \u{b7}  2,800 games",
                "The main result: a 9 by 8 round robin, 2,800 games, with safe random pinned at Elo 0 and the network alone just above.",
            ),
            say("The rules, and the time limited solver, come next."),
            say("Then network guided MCTS, and the handcrafted alpha-beta."),
            say("The learned alpha-beta at 722."),
            say("And at the top, MCTS at 740."),
        ],
    },
    Slide {
        kicker: "why",
        title: "Why it came out this way",
        visual: Visual::Bars(Bars {
            caption: "8x7: won positions kept against the solver (%)",
            floor: 50.0,
            ceiling: 100.0,
            items: &[
                bar("alpha-beta learned", 97.0, "97.0%", 3),
                lit("MCTS", 96.5, "96.5%", 3, 4),
                bar("network MCTS", 96.0, "96.0%", 3),
                lit("exact solver (1 s)", 89.4, "89.4%", 0, 1),
                bar("network alone", 80.3, "80.3%", 2),
                bar("Allis rules", 77.8, "77.8%", 2),
                bar("safe random", 54.5, "54.5%", 2),
            ],
        }),
        beats: &[
            say("So why? Search wins on a board you can't solve. The exact solver is perfect, but only if it finishes."),
            say("In one second it can't finish a 9 by 8 position, so it falls back to playing near the middle."),
            say("Learned knowledge transfers, but without search one mistake costs you the game."),
            say("Put search on top of that knowledge and most of the problem goes away."),
            show("search + knowledge wins", "This is the same story on 8 by 7, measured against the solver itself."),
        ],
    },
    Slide {
        kicker: "takeaways",
        title: "What we learned",
        // Red builds the bottom row, yellow stacks on it, and red's last disc wins.
        visual: Visual::Game(&[drop(7, 6, &[3, 3, 2, 2, 1, 1, 0])]),
        beats: &[show(
            "knowledge guides search",
            "First: on a board you can't solve, search is what wins. Second: what agents learn on 8 by 7 carries \
             over to 9 by 8 almost unchanged. And third: knowledge is best used to guide search, not to replace it.",
        )],
    },
    Slide {
        kicker: "demo",
        title: "Now watch them play",
        visual: Visual::Game(&[drop(8, 7, &[3, 4, 3, 3, 4, 2, 5, 5, 2, 4, 1, 6])]),
        beats: &[say(
            "Alright, let's actually watch them play. I'll pick two agents and a board size. These are the real \
             agents from our results, playing live, even on the 9 by 8 test board.",
        )],
    },
];

/// Solves every column of `SOLVED_MOVES` (7x6) with the project's solver.
/// `None` for a full column; the whole thing is `None` if it does not finish in time.
pub fn solve_position(limit: Duration) -> Option<Vec<Option<Outcome>>> {
    let board = Board::<B76>::from_moves(SOLVED_MOVES)?;
    let mut solver = Solver::<B76>::new(22);
    solver.label(board, Some(Instant::now() + limit)).map(|label| label.columns)
}

/// `SOLVED_MOVES` as 0-based columns.
pub fn solved_columns() -> Vec<u32> {
    SOLVED_MOVES.chars().filter_map(|ch| ch.to_digit(10)).map(|col| col - 1).collect()
}

/// Which beats of a slide bring something new or moving onto the screen.
#[cfg(test)]
pub fn moving_beats(slide: &Slide) -> Vec<bool> {
    let mut moves = vec![false; slide.beats.len()];
    let mut mark = |beat: usize| {
        if let Some(slot) = moves.get_mut(beat) {
            *slot = true;
        }
    };
    match &slide.visual {
        Visual::Expand(stages) | Visual::Game(stages) => {
            let mut size = None;
            for (beat, stage) in stages.iter().enumerate() {
                let resized = size.is_some_and(|size| size != (stage.columns, stage.rows));
                // Discs alone do not count: a press must do more than drop chips.
                if beat == 0 || resized {
                    mark(beat);
                }
                size = Some((stage.columns, stage.rows));
            }
        }
        Visual::Bars(bars) => {
            for item in bars.items {
                mark(item.beat);
                if let Some(lit) = item.lit {
                    mark(lit);
                }
            }
        }
        Visual::Agents(list) => list.iter().copied().for_each(&mut mark),
        Visual::Network => (0..NETWORK_BEATS).for_each(&mut mark),
        Visual::Solved => (0..SOLVED_BEATS).for_each(&mut mark),
    }
    moves
}

/// The speaker notes, as `SPEAKER_NOTES.md` should read.
#[cfg(test)]
pub fn speaker_notes() -> String {
    let total = talk_seconds();
    let mut out = format!(
        "# Speaker notes\n\nAbout {} minutes in total ({} presses of Next, most about {SECONDS_PER_BEAT} seconds). The\n\
         slides only show a title and a picture, so everything worth saying is here. Each\n\
         numbered step is one press of Next (right arrow, space or click): it moves the next\n\
         piece of the picture in. The presenter window (press N) shows the current step and\n\
         the next one.\n\n\
         Generated from `src/slides.rs`: after changing a slide, run\n\
         `C4_WRITE_NOTES=1 cargo test speaker_notes` to rewrite this file.\n",
        (total + 30) / 60,
        SLIDES.iter().map(|slide| slide.beats.len()).sum::<usize>(),
    );
    let mut start = 0;
    for (index, slide) in SLIDES.iter().enumerate() {
        out.push_str(&format!(
            "\n## {}. {}  ({} s, starts at {}:{:02})\n\n",
            index + 1,
            slide.title,
            slide.seconds(),
            start / 60,
            start % 60
        ));
        for (step, beat) in slide.beats.iter().enumerate() {
            out.push_str(&format!("{}. {}\n", step + 1, beat.say));
        }
        start += slide.seconds();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_live_position_solves_quickly() {
        let started = Instant::now();
        let columns = solve_position(Duration::from_secs(60)).expect("solves within a minute");
        eprintln!("solved in {:?}: {columns:?}", started.elapsed());
        assert_eq!(columns.len(), 7);
    }

    #[test]
    fn board_games_are_legal() {
        for slide in SLIDES {
            if let Visual::Expand(stages) | Visual::Game(stages) = slide.visual {
                let mut heights = vec![0; 16];
                for stage in stages {
                    for &col in stage.moves {
                        assert!(col < stage.columns, "{}", slide.title);
                        heights[col as usize] += 1;
                        assert!(heights[col as usize] <= stage.rows, "{}: column {col} overflows", slide.title);
                    }
                }
            }
        }
    }

    #[test]
    fn beats_line_up() {
        for slide in SLIDES {
            let beats = slide.beats.len();
            assert!(beats >= 1, "{}", slide.title);
            for beat in slide.beats {
                assert!(!beat.say.contains('\u{2014}') && !beat.say.contains('\u{2013}'));
            }
            match &slide.visual {
                Visual::Bars(bars) => {
                    for item in bars.items {
                        assert!(item.beat < beats, "{}", slide.title);
                        assert!(item.lit.is_none_or(|lit| lit >= item.beat && lit < beats), "{}", slide.title);
                    }
                }
                Visual::Agents(list) => assert!(list.iter().all(|&beat| beat < beats)),
                Visual::Expand(stages) => assert_eq!(stages.len(), beats, "{}", slide.title),
                Visual::Game(stages) => assert!(stages.len() == 1 && beats == 1, "{}: a game is one stage", slide.title),
                Visual::Network => assert_eq!(beats, NETWORK_BEATS),
                Visual::Solved => assert_eq!(beats, SOLVED_BEATS),
            }
        }
    }

    /// Every press of Next moves something: no beat only swaps a line of text.
    #[test]
    fn every_beat_moves_something() {
        for slide in SLIDES {
            for (beat, moves) in moving_beats(slide).into_iter().enumerate() {
                assert!(moves, "{}: beat {} brings nothing new on screen", slide.title, beat + 1);
            }
        }
    }

    /// Each beat is about six seconds of talking (at most 26 words), except where
    /// steps that would only drop discs were folded into one: there, at most three lots.
    #[test]
    fn beats_are_about_six_seconds() {
        for slide in SLIDES {
            for beat in slide.beats {
                let words = beat.say.split_whitespace().count();
                assert!((4..=3 * WORDS_PER_BEAT).contains(&words), "{}: {words} words in {:?}", slide.title, beat.say);
            }
            let long = slide.beats.iter().filter(|beat| beat.say.split_whitespace().count() > WORDS_PER_BEAT).count();
            assert!(long <= 1, "{}: only one long step per slide", slide.title);
        }
    }

    /// `SPEAKER_NOTES.md` matches the slides. `C4_WRITE_NOTES=1` rewrites it.
    #[test]
    fn speaker_notes_are_current() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/SPEAKER_NOTES.md");
        let wanted = speaker_notes();
        if std::env::var("C4_WRITE_NOTES").is_ok() {
            std::fs::write(path, &wanted).expect("writes the notes");
        }
        let current = std::fs::read_to_string(path).unwrap_or_default().replace("\r\n", "\n");
        assert_eq!(current, wanted, "SPEAKER_NOTES.md is stale: run C4_WRITE_NOTES=1 cargo test speaker_notes");
    }
}

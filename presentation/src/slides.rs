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

/// An agent's slide: the same four numbers for every agent, so the eight slides compare
/// at a glance. Games won first, then both accuracies, then wins kept against the
/// solver.
macro_rules! spotlight {
    ($won:literal, $won_text:literal, $acc87:literal, $acc87_text:literal, $acc98:literal, $acc98_text:literal, $kept:literal, $kept_text:literal) => {
        Visual::Bars(Bars {
            caption: "the same four numbers for every agent (%)",
            floor: 0.0,
            ceiling: 100.0,
            items: &[
                bar("9x8 games won", $won, $won_text, 0),
                bar("8x7 move accuracy", $acc87, $acc87_text, 1),
                bar("9x8 late-game accuracy", $acc98, $acc98_text, 1),
                bar("8x7 wins kept vs solver", $kept, $kept_text, 2),
            ],
        })
    };
}

pub const SLIDES: &[Slide] = &[
    Slide {
        kicker: "group 3  \u{b7}  TJ Raklovits and Andrew Roddy",
        title: "Connect Four AI",
        visual: Visual::Game(&[drop(9, 8, &[4, 4, 3, 5, 5, 3, 4, 2, 6, 4, 3, 3, 5, 2, 6, 6, 2])]),
        beats: &[say(
            "Hi everyone, we're TJ and Andrew, group 3. Our project: eight different AI agents playing Connect \
             Four. Here's the twist: we trained them on a board we can solve, 8 by 7, and tested them on one \
             nobody has solved, 9 by 8. Everything you see today, these slides included, runs live in our Rust code.",
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
            say("Start with the board you know: 7 by 6. It's solved. With perfect play, the first player always wins."),
            say("Add a column and a row: 8 by 7. John Tromp solved this one too, so that's where we trained."),
            show(
                "does what they learn carry over, and why?",
                "One more of each gets you 9 by 8, and that has never been solved. On our hardware it would take \
                 roughly 15 years. So here's our question: if agents learn on a board we can solve, do they still \
                 play well on one we can't? And if some methods carry over better, why?",
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
            say("How big are we talking? Standard Connect Four has about 4.5 trillion positions. Already a lot."),
            say("8 by 7 jumps to roughly 10 to the 26."),
            say("And 9 by 8, roughly 10 to the 34."),
            say("Remember, this is a log scale: each step is millions of times bigger. Brute force is simply off the table."),
        ],
    },
    Slide {
        kicker: "step 1",
        title: "An exact solver",
        visual: Visual::Solved,
        beats: &[
            say("So step one: build a perfect player anyway. Alpha-beta on bitboards with a transposition table, the same tricks published solvers use."),
            say("Hand it any position and it tells you, for every column: win, draw or loss."),
            say("These letters were computed live when the app launched. And watch: it plays the winning column."),
            show(
                "0 wrong",
                "Can you trust it? We checked all 67,557 UCI positions and 6,000 of Pascal Pons' tests. Zero mistakes.",
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
            say("So why not just solve 9 by 8? Near the end of a game it's easy: about a tenth of a second."),
            say("Take stones away and it slows down: a third of a second at 30."),
            say("Two seconds at 26."),
            say("At 24 stones the median is 12 seconds, and our worst case took 31 minutes."),
            say("At 20 stones, over a minute each."),
            say("At 16, every single probe blew past ten minutes."),
            say("Even 8 by 7 with only six stones took over half an hour. That wall is exactly why we turned to learning."),
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
            say("Instead, the solver became our teacher: about 107 thousand 8 by 7 positions, every column labelled."),
            say("Plus 2,000 tricky openings, 10 to 12 stones in, held back just for testing."),
            show(
                "192 cores, one position each",
                "And to test transfer, about 10 thousand late game 9 by 8 positions, solved on a 192 core cloud machine.",
            ),
        ],
    },
    Slide {
        kicker: "step 3",
        title: "Eight agents",
        // MCTS, AB learned, AB handcrafted, network MCTS, Allis, solver, network, safe random.
        visual: Visual::Agents(&[0, 2, 1, 3, 4, 1, 3, 5]),
        beats: &[
            say("Step three, the players: eight agents, coloured by family. First, Monte Carlo tree search."),
            say("Two more pure search agents: handcrafted alpha-beta, and our solver capped at one second."),
            say("Then learning comes in: alpha-beta with a learned evaluation."),
            say("A convolutional network on its own, and that same network guiding MCTS."),
            say("Victor Allis' strategy rules: pure knowledge, no search."),
            say("And safe random as the baseline. Every agent gets one second per move. Let's meet them one at a time."),
        ],
    },
    // ---------------------------------------------------------------- the agents, weakest first
    Slide {
        kicker: "agent 1 of 8  \u{b7}  baseline",
        title: "Safe random",
        visual: spotlight!(16.7, "16.7%", 76.4, "76.4%", 77.8, "77.8%", 54.5, "54.5%"),
        beats: &[
            say("Weakest first. Safe random plays at random, except it always takes a winning move and blocks an obvious loss."),
            show(
                "+ never misses a one-move win or block",
                "It's our yardstick, pinned at Elo zero. It still matches the solver on 76 percent of moves, because many positions have several fine moves.",
            ),
            show(
                "\u{2212} no plan beyond the next move",
                "But it has no plan, so it keeps barely half its won positions against the solver. Every other agent should beat this.",
            ),
        ],
    },
    Slide {
        kicker: "agent 2 of 8  \u{b7}  knowledge",
        title: "Allis rules",
        visual: spotlight!(30.7, "30.7%", 88.3, "88.3%", 89.7, "89.7%", 77.8, "77.8%"),
        beats: &[
            say("Allis rules is pure knowledge: win, block, avoid squares that hand over a win, and build threats on the right rows."),
            show(
                "+ instant, and the ideas transfer",
                "It moves instantly, and its accuracy actually rises on 9 by 8, from 88 to 90 percent. Good strategy doesn't care about board size.",
            ),
            show(
                "\u{2212} no lookahead, so tactics catch it",
                "But with no lookahead it walks into tactics. It keeps only 78 percent of won positions and wins 31 percent of its games.",
            ),
        ],
    },
    Slide {
        kicker: "agent 3 of 8  \u{b7}  search",
        title: "Exact solver, 1 s a move",
        visual: spotlight!(23.4, "23.4%", 97.8, "97.8%", 97.8, "97.8%", 89.4, "89.4%"),
        beats: &[
            say("Our exact solver as a player. When it finishes, its move is perfect. Guaranteed."),
            show(
                "+ perfect whenever it finishes",
                "Late in the game it finishes easily, and that's where its 97.8 percent accuracy comes from.",
            ),
            show(
                "\u{2212} one second is nowhere near enough",
                "The catch: early 9 by 8 positions take far longer than a second, so it's left guessing. It wins only 23 percent of games.",
            ),
        ],
    },
    Slide {
        kicker: "agent 4 of 8  \u{b7}  search",
        title: "Alpha-beta, handcrafted",
        visual: spotlight!(76.0, "76.0%", 98.8, "98.8%", 99.3, "99.3%", 97.0, "97.0%"),
        beats: &[
            say("Alpha-beta looks ahead as deep as one second allows, pruning branches that can't matter, and scores the leaves with our evaluation."),
            show(
                "+ deep lookahead, sharp tactics",
                "The evaluation counts threats and rewards them on the right rows, Allis' odd and even rule. That makes it tactically razor sharp.",
            ),
            show(
                "\u{2212} only as good as our hand-picked weights",
                "Its weakness is us: the weights are our guesses. Still, it wins 76 percent, and from an empty board it often beats MCTS.",
            ),
        ],
    },
    Slide {
        kicker: "agent 5 of 8  \u{b7}  search + learning",
        title: "Alpha-beta, learned",
        visual: spotlight!(78.9, "78.9%", 98.9, "98.9%", 99.3, "99.3%", 97.0, "97.0%"),
        beats: &[
            say("Same search, same features, but learned weights: a logistic regression on 107 thousand solver-labelled positions."),
            show(
                "+ learned weights beat our guesses",
                "It predicts results better than our hand tuning, 70 versus 68 percent, and that edge survives the jump to 9 by 8.",
            ),
            show(
                "\u{2212} a linear model can't find new ideas",
                "The limit: it only reweighs the features we gave it, so it can't discover anything new. Still, second place overall.",
            ),
        ],
    },
    Slide {
        kicker: "agent 6 of 8  \u{b7}  search",
        title: "Monte Carlo tree search",
        visual: spotlight!(80.6, "80.6%", 98.7, "98.7%", 99.1, "99.1%", 96.5, "96.5%"),
        beats: &[
            say("MCTS needs no evaluation at all. It plays out thousands of random games and grows its tree toward the moves that win."),
            show(
                "+ simple, and better with every second",
                "It's simple, it scales with time, and it never misses a proven win or loss. At one second, it tops the tournament.",
            ),
            show(
                "\u{2212} random playouts miss long-term strategy",
                "Its blind spot: random games don't understand row parity, so in quiet positions, like an empty board, alpha-beta can outplay it.",
            ),
        ],
    },
    Slide {
        kicker: "agent 7 of 8  \u{b7}  learning",
        title: "Convolutional network",
        visual: spotlight!(20.1, "20.1%", 92.4, "92.4%", 91.0, "91.0%", 80.3, "80.3%"),
        beats: &[
            say("The network looks at the whole board at once and scores every column. No search, just one pass."),
            show(
                "+ one set of weights for any board size",
                "Because it's fully convolutional, the same weights run on 9 by 8, and it still plays the solver's move 91 percent of the time.",
            ),
            show(
                "\u{2212} one bad move and the game is gone",
                "But 91 percent per move isn't enough. Over a 40 move game a slip is almost certain, so it wins only 20 percent.",
            ),
        ],
    },
    Slide {
        kicker: "agent 8 of 8  \u{b7}  search + learning",
        title: "Network-guided MCTS",
        visual: spotlight!(72.1, "72.1%", 98.2, "98.2%", 98.5, "98.5%", 96.0, "96.0%"),
        beats: &[
            say("Last, the AlphaZero idea: the network tells MCTS which moves look promising and how good a position is."),
            show(
                "+ knowledge fixes the network's blunders",
                "Search on top of knowledge fixes the network's slips: it keeps 96 percent of won positions and climbs to 647 Elo.",
            ),
            show(
                "\u{2212} each network call is slow on a CPU",
                "The price: each network call costs far more than a random playout, so it searches much less. That's why plain MCTS edges it out.",
            ),
        ],
    },
    // ---------------------------------------------------------------- what we found
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
            say("Now, what we found. Alpha-beta first: at depth 1 it already agrees with the solver 91 percent of the time."),
            say("Depth 2, about 94."),
            say("Depth 4 and 6: slow, steady gains."),
            say("Depth 8 reaches 97.5 percent."),
            say("Depth 12 adds almost nothing, while searching 35 times as many nodes. Diminishing returns."),
            show(
                "deeper still wins 2 of 3 games on 9x8",
                "But don't be fooled: in real 9 by 8 games the deeper search still wins two out of three. Depth matters.",
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
            say("Can we learn the evaluation instead of hand tuning it? Baseline: always guess the most common result, 56.6 percent."),
            say("Our handcrafted evaluation gets 67.8."),
            show(
                "on 9x8: Elo 722 learned vs 690 handcrafted",
                "Logistic regression on the same features, trained on solver labels: 70.1. And it transfers: 722 Elo on 9 by 8, versus 690.",
            ),
        ],
    },
    Slide {
        kicker: "search",
        title: "MCTS loves time",
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
            say("Back to MCTS, because it has a superpower: time. At half a second a move, 695 Elo."),
            say("At one second, 740: top of the tournament, playing about 740 thousand random games a second."),
            say("At five seconds, 754."),
            say("At ten, 761. Its win rate keeps climbing with time, while alpha-beta's stays flat or slips."),
        ],
    },
    Slide {
        kicker: "learning",
        title: "A network for any board",
        visual: Visual::Network,
        beats: &[
            say("The network. We made it fully convolutional, so the exact same weights run on any board size."),
            say("Six layers, 64 channels, trained for 12 epochs on the CPU, after our GPU setup gave up on us."),
            say("Out come scores, one per column. The highest is its move."),
            show(
                "92.4% of 8x7 moves  \u{b7}  91% of 9x8 moves",
                "It picks the solver's move 92.4 percent of the time on 8 by 7, and 91 on a 9 by 8 board it never trained on.",
            ),
            show(
                "alone: Elo 37  \u{b7}  guiding MCTS: Elo 647",
                "Alone, though, it scores just 37 Elo, because one slip loses. Let it guide MCTS and it jumps to 647.",
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
                "Allis' rules, with no search at all, beat safe random 78 percent of the time on 9 by 8.",
            ),
            say("83 percent on 8 by 7: same rules, nearly the same strength."),
            say("Plot twist: turning off claimeven, a famous rule, didn't hurt. It actually scored higher."),
            say("Flipping the parity hurt a bit. Lesson: a handful of rules isn't a full strategy."),
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
            say("Now the big question: does it carry over? The search agents play just as well on 9 by 8, even slightly better."),
            say("The one second solver holds at 97.8."),
            say("The network slips only from 92.4 to 91, on a board it has never seen."),
            say("The rules and the baseline barely move. So yes: what they learned transfers."),
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
                "The main event: a full round robin on 9 by 8, 2,800 games. Safe random anchors at zero; the network alone sits just above.",
            ),
            say("Allis' rules and the time limited solver come next."),
            say("Then a big jump: network guided MCTS and handcrafted alpha-beta."),
            say("Learned alpha-beta takes second, at 722."),
            say("And the winner: plain MCTS, at 740."),
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
            say("So why did it shake out this way? The solver is perfect, but only when it finishes."),
            say("In one second it can't finish an early 9 by 8 position, so it's left guessing."),
            say("Knowledge alone transfers well, but without search a single mistake costs the game."),
            say("Add search on top of knowledge, and most of those mistakes disappear."),
            show(
                "search + knowledge wins",
                "Same story on 8 by 7, measured against the solver itself: the search agents keep about 97 percent of their wins.",
            ),
        ],
    },
    Slide {
        kicker: "takeaways",
        title: "What we learned",
        // Red builds the bottom row, yellow stacks on it, and red's last disc wins.
        visual: Visual::Game(&[drop(7, 6, &[3, 3, 2, 2, 1, 1, 0])]),
        beats: &[show(
            "knowledge guides search",
            "Three takeaways. One: on a board you can't solve, search is what wins. Two: what agents learn on 8 by 7 \
             carries over to 9 by 8 almost unchanged. Three: knowledge works best guiding search, not replacing it.",
        )],
    },
    Slide {
        kicker: "demo",
        title: "Now watch them play",
        visual: Visual::Game(&[drop(8, 7, &[3, 4, 3, 3, 4, 2, 5, 5, 2, 4, 1, 6])]),
        beats: &[say(
            "Enough slides, let's watch them play. I'll pick two agents and a board. Heads up: from an empty board, \
             handcrafted alpha-beta often beats MCTS. Switch Start to Random for the tournament's conditions.",
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

#set document(title: "Connect Four: Train on 8x7, Test on 9x8", author: ("TJ Raklovits", "Andrew Roddy"))
#set page(paper: "us-letter", margin: (x: 0.75in, y: 0.5in))
#set text(font: "Bell MT", size: 9.5pt)
//#set par(justify: false, leading: 0.5em, spacing: 0.6em)
//#set list(indent: 0.6em, spacing: 0.75em)
#show heading.where(level: 2): it => block(above: 0.75em, below: 0.75em, text(size: 11pt, weight: "bold", it.body))
#show link: underline

#align(center)[
  #text(size: 15pt, weight: "bold")[Connect Four: Train on 8x7, Test on 9x8] \
  Midterm Report #h(0.6em)|#h(0.6em) AI #h(0.6em)|#h(0.6em) Group 3 #h(0.6em)|#h(0.6em) TJ Raklovits, Andrew Roddy
]

== Problem Statement and Objective

We will make AI agents for Connect Four on boards that are larger than the standard 7x6 board. Many
published studies use 7x6. We will train the agents on an 8x7 board (8 columns, 7 rows), then we will
use a 9x8 board for the test.

John Tromp solved 8x7 to determine that with perfect play, the first player wins
(#link("https://tromp.github.io/c4/c4.html")[tromp.github.io/c4]). As a result, an exact solver can give the
correct result for each 8x7 position. No published result exists for 9x8, and we will not try to solve 9x8.
Our question is: can agents that we train on 8x7 play well on 9x8? We also want to know why each method
works or fails.

The task is game playing, a search and planning task. The learned agents also do supervised prediction:
for each column, they predict win, draw, or loss. The output of each agent is one column for a given
board. The output of the solver is the result of each legal column.

== Dataset Description

- *8x7 training set:* 8x7 positions with labels from an exact solver. We will write this solver. It will
  use alpha-beta search on bitboards with a transposition table, like published Connect Four solvers.
  - The positions will come from games with random moves that do not lose at once, with no duplicates
    or mirror images. Each sample is a board state with 56 cells (the features). Each cell is empty, a
    stone of the player to move, or a stone of the other player.
  - Labels: 3 classes (win, draw, or loss) for each legal column, so up to 8 labels for each position.
  - Target size: about 100,000 positions from the middle and the end of the game. A separate
    *8x7 opening set* of about 2,000 positions (10 to 12 moves played) is only for the test.
- *Solver check:* the UCI Connect-4 dataset (John Tromp, UCI Machine Learning Repository,
  #link("https://archive.ics.uci.edu/dataset/26/connect+4")[archive.ics.uci.edu/dataset/26]) has 67,557
  7x6 positions with win, draw, or loss labels. Pascal Pons published 6,000 more 7x6 test positions, in
  the sets Test_L3_R1 to Test_L1_R3 (#link("http://blog.gamesolver.org")[blog.gamesolver.org]).
- *Test board:* 9x8. This board has no labels. The agents play games against each other from random
  opening positions.
- *Environment and scale:* a state is a board and the player to move; an action is a column. 7x6 has
  about 4.5 × 10#super[12] legal positions (Tromp). An 8x7 board has at most 3#super[56] ≈ 5 × 10#super[26]
  cell patterns, and 9x8 at most 3#super[72] ≈ 2 × 10#super[34]. A 9x8 game has at most 72 moves.
- *Tools:* we will write as much as possible in Rust. This includes the game engine for both board sizes,
  the solver, all agents, the match program, and the tournament. The neural network will use a Rust deep
  learning library (for example Burn).

== Proposed Methods and Individual Responsibilities (can change)

#table(
  columns: (1.05in, 3.25in, 1fr),
  inset: (x: 4pt, y: 2.5pt),
  stroke: 0.4pt + luma(180),
  fill: (_, y) => if y == 0 { luma(232) },
  table.header([*Member*], [*Agent*], [*Key question*]),
  [TJ Raklovits], [Exact solver as an agent, with an opening book],
    [How strong is it on 9x8 with a time limit?],
  [TJ Raklovits], [Alpha-beta, handcrafted evaluation (threats, row parity)],
    [How deep must it search? Does it work on 9x8?],
  [TJ Raklovits], [Alpha-beta, logistic-regression evaluation from 8x7 data],
    [Is the learned evaluation better on 9x8?],
  [TJ Raklovits], [Rule agent with the strategy rules of Victor Allis, no search],
    [Do the strategy rules work on 9x8?],
  [Andrew Roddy], [Monte Carlo Tree Search (UCT)],
    [How many simulations does it need?],
  [Andrew Roddy], [Fully convolutional network, trained on 8x7 labels],
    [Does its knowledge transfer to 9x8?],
  [Andrew Roddy], [MCTS with the network as a guide (like AlphaZero)],
    [Is it better than search alone?],
  [Andrew Roddy], [Safe random baseline: random, but takes wins and blocks threats],
    [It is a low bar and the Elo anchor.],
)

== Evaluation

- *8x7 (training board):* we measure move accuracy against a solver. We use positions that we keep
  out of training, and the opening set. We report the accuracy for each phase of the game. The agents
  also play matches against the solver.
- *9x8 (test board): agent tournament.* Each agent plays each other agent (28 pairs), with 1 second for
  each move. Each pair plays from the same 50 random opening positions, once as each side: 100 games
  for each pair, about 2,800 games in total. We rank the agents by win rate and Elo.

== Limitations

- We cannot find perfect play on 9x8 with our compute, so the tournament gives only relative strength.
  The solver can possibly solve late 9x8 positions, which allows exact checks near the end of the game.
- The learned weights and the opening book depend on the 8x7 board. The neural network will be fully
  convolutional, so that it can use a 9x8 board.

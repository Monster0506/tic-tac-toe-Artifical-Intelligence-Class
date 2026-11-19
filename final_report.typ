#set document(
  title: "Train on 8×7, Test on 9×8: Connect Four Agents on an Unsolved Board",
  author: ("TJ Raklovits", "Andrew Roddy"),
)
#set page(paper: "us-letter", margin: (x: 0.7in, y: 0.75in), columns: 2, numbering: "1")
#set text(font: "New Computer Modern", size: 10pt, hyphenate: true)
#set par(justify: true, leading: 0.56em, spacing: 0.8em)
#set heading(numbering: "1.1")
#set math.equation(numbering: "(1)")
#set list(indent: 0.8em, spacing: 0.5em)
#set enum(indent: 0.8em, spacing: 0.5em)
#show heading.where(level: 1): it => block(above: 1.15em, below: 0.65em, text(size: 12.5pt, it))
#show heading.where(level: 2): it => block(above: 1em, below: 0.55em, text(size: 10.5pt, it))
#show figure.where(kind: table): set figure.caption(position: top)
#show figure.where(kind: table): set text(size: 9pt)
#show figure.where(kind: table): set par(justify: false)
#show figure.caption: set text(size: 9pt)
#show raw: set text(size: 8.5pt)
#show link: set text(fill: rgb("#1a4f8b"))


#let meas = csv("results/measurements.csv", row-type: dictionary)
#let acc = csv("results/accuracy.csv", row-type: dictionary)
#let games = csv("results/games.csv", row-type: dictionary)
#let study = csv("presentation/ranking/games.csv", row-type: dictionary)

#let pick(study, board, agent, setting, metric, notes: none) = {
  let hits = meas.filter(r => (
    r.study == study and r.board == board and r.agent == agent and r.setting == setting
      and r.metric == metric and (notes == none or r.notes.contains(notes))
  ))
  assert(
    hits.len() == 1,
    message: "expected one row for " + study + " / " + agent + " / " + setting + " / " + metric,
  )
  hits.first()
}
#let val(..args) = float(pick(..args).value)

#let accuracy(dataset, agent, phase: "total") = {
  let hits = acc.filter(r => r.dataset == dataset and r.agent == agent and r.phase_stones == phase)
  assert(hits.len() == 1, message: "expected one accuracy row for " + dataset + " / " + agent)
  hits.first()
}

#let fix(x, d: 1) = {
  let s = str(calc.round(x, digits: d))
  if d == 0 { return s }
  if not s.contains(".") { s += "." }
  let frac = s.split(".").at(1)
  s + "0" * (d - frac.len())
}
#let pct(x, d: 1) = fix(100 * x, d: d)

#let agents = (
  (id: "mcts", name: "MCTS", short: "MCTS", who: "Roddy", color: rgb("#1f6fb4")),
  (id: "alpha-beta-learned", name: "Alpha-beta, learned", short: "AB-L", who: "Raklovits", color: rgb("#d1495b")),
  (id: "alpha-beta-handcrafted", name: "Alpha-beta, handcrafted", short: "AB-H", who: "Raklovits", color: rgb("#e8912d")),
  (id: "network-mcts", name: "Network-guided MCTS", short: "N-MCTS", who: "Roddy", color: rgb("#2a9d8f")),
  (id: "allis", name: "Allis rules", short: "Allis", who: "Raklovits", color: rgb("#8d6a9f")),
  (id: "perfect", name: "Solver, 1 s", short: "Solver", who: "Raklovits", color: rgb("#222222")),
  (id: "network", name: "Network alone", short: "Network", who: "Roddy", color: rgb("#6a994e")),
  (id: "safe-random", name: "Safe random", short: "Random", who: "Roddy", color: rgb("#8a8a8a")),
)
#let agent(id) = agents.find(a => a.id == id)
#let top-four = ("mcts", "alpha-beta-learned", "alpha-beta-handcrafted", "network-mcts")


#let small = 7.5pt

#let lineplot(
  w: 6cm,
  h: 3.6cm,
  xr: (0, 1),
  yr: (0, 1),
  xticks: (),
  yticks: (),
  xlabel: none,
  ylabel: none,
  ylog: false,
  series: (),
  marks: (),
  legend-columns: 2,
) = {
  set par(justify: false)
  let fy(y) = if ylog { calc.log(y) } else { y }
  let px(x) = (x - xr.at(0)) / (xr.at(1) - xr.at(0)) * w
  let py(y) = h - (fy(y) - fy(yr.at(0))) / (fy(yr.at(1)) - fy(yr.at(0))) * h
  let axis = 0.5pt + black
  let canvas = box(width: w, height: h, {
    for (y, label) in yticks {
      place(line(start: (0pt, py(y)), end: (w, py(y)), stroke: 0.4pt + luma(215)))
      place(dx: -33pt, dy: py(y) - 3.5pt, box(width: 30pt, align(right, text(size: small, label))))
    }
    for (x, label) in xticks {
      place(line(start: (px(x), h), end: (px(x), h + 2.5pt), stroke: axis))
      place(dx: px(x) - 22pt, dy: h + 4.5pt, box(width: 44pt, align(center, text(size: small, label))))
    }
    for (y, label) in marks {
      place(line(start: (0pt, py(y)), end: (w, py(y)), stroke: (paint: luma(90), thickness: 0.6pt, dash: "dashed")))
      place(dx: w - 82pt, dy: py(y) - 9.5pt, box(width: 80pt, align(right, text(size: small, fill: luma(60), label))))
    }
    place(line(start: (0pt, 0pt), end: (0pt, h), stroke: axis))
    place(line(start: (0pt, h), end: (w, h), stroke: axis))
    for s in series {
      let pts = s.pts.map(p => (px(p.at(0)), py(p.at(1))))
      let stroke = (paint: s.color, thickness: 1pt, dash: s.at("dash", default: "solid"))
      for i in range(pts.len() - 1) {
        place(line(start: pts.at(i), end: pts.at(i + 1), stroke: stroke))
      }
      for p in pts {
        place(dx: p.at(0) - 1.5pt, dy: p.at(1) - 1.5pt, circle(radius: 1.5pt, fill: s.color))
      }
    }
    if xlabel != none {
      place(dx: 0pt, dy: h + 15pt, box(width: w, align(center, text(size: small, xlabel))))
    }
  })
  let legend = grid(
    columns: legend-columns,
    column-gutter: 10pt,
    row-gutter: 4pt,
    ..series.map(s => box({
      box(width: 12pt, height: 5pt, place(dy: 2.5pt, line(
        length: 12pt,
        stroke: (paint: s.color, thickness: 1.2pt, dash: s.at("dash", default: "solid")),
      )))
      std.h(3pt)
      text(size: small, s.label)
    }))
  )
  align(center, stack(
    dir: ttb,
    spacing: 5pt,
    if ylabel != none { align(left, pad(left: 6pt, text(size: small, ylabel))) },
    pad(left: 35pt, right: 8pt, bottom: if xlabel != none { 26pt } else { 15pt }, canvas),
    legend,
  ))
}

#let cell-table(..args) = table(
  inset: (x: 4.5pt, y: 2.6pt),
  stroke: none,
  ..args,
)
#let rule = table.hline(stroke: 0.5pt)
#let heavy = table.hline(stroke: 0.9pt)

#let pair(columns: (1fr, 1fr), at: auto, left, right) = {
  place(at, float: true, clearance: 1.2em, left)
  place(at, float: true, clearance: 1.2em, right)
}


#place(top, scope: "parent", float: true, clearance: 1.2em)[
#align(center)[
  #text(size: 16pt, weight: "bold")[
    Train on 8×7, Test on 9×8: \ Connect Four Agents on an Unsolved Board
  ]
  #v(0.4em)
  Group 3 #h(1.2em) TJ Raklovits #h(1.2em) Andrew Roddy \
  CS 44201/54201 Artificial Intelligence, Fall 2026
]

#pad(x: 2.2em, top: 0.2em, bottom: 0.1em)[
  #set text(size: 9.5pt)
  *Abstract.* Connect Four is solved on the standard board of 7 columns and 6 rows and on the larger 
8×7 board, but there is not a published solution for the 9×8 board. We ask whether agents that learn 
on a board with known answers can still play well on a larger board. To study this, we wrote a solver, 
checked it against 73,557 published positions without a single error, and used it to label 106,898 
positions on 8×7. We then compared agents from four families (search, search with a learned 
evaluation, a neural network, and strategy rules) by how often they pick a best move according to 
the solver and by a tournament of 2,800 games on 9×8. We found that there was almost no loss in 
knowledge transfer. A convolutional network that saw only 8×7 positions picked a best move in 92.4% 
of the 8×7 test positions and in 91.0% of the 9×8 test positions. However, the accuracy of single 
moves did not decide the games. The search agents won the tournament, with Monte Carlo tree 
search at 740 Elo and alpha-beta search with a learned evaluation at 722 Elo, while the network 
alone finished near the random baseline at 37 Elo. The same network reached 647 Elo when it guided a 
search.
]
]


= Introduction

Connect Four is a game for two players on an upright board. The players take turns, and on each turn a 
player drops one stone into a column, where it falls to the lowest empty cell. The first player to 
make a line of four stones, in a row, column, or diagonal, wins, and a full board with no such line is 
a draw. In this project the input is a position, which is the board together with the player to move, 
and the output is one column to play. For example, if the opponent has three stones in a line and the 
fourth cell can be filled on the next turn, the correct output is the column of that cell.

The standard board has 7 columns and 6 rows. It is a win for the first player under perfect play
as solved by Allis and, independently, Allen in 1988 @allis1988 @tromp. Tromp later solved the larger
8×7 board, which is also a win for the first player @tromp, but there is not a published solution for 
the 9×8 board. This is because the 7×6 board has about $4.5 times 10^12$ legal positions @tromp, and a 
simple upper limit on the number of cell patterns is $3^56 approx 5 times 10^26$ for 8×7 and $3^72 
approx 2 times 10^34$ for 9×8. Our solver needed more than 30 minutes for some 8×7 positions that had 
only six stones on the board, and we estimate that a full solution of 9×8 would take about 15 years on 
our hardware. This makes 9×8 a useful test board because an agent cannot look up the answer there.

If agents learn on a board where a solver gives the correct answer, do they also play well on a larger 
board where no solver can help? And if some methods transfer better than others, why? To find out,
we trained on 8×7 and tested on 9×8. We found that every agent kept its move accuracy on the new 
board, that the agents that search were far stronger in games than the agents that do not, and that 
the three best agents were so close to each other that their order changed with the start positions. 
Our contributions are:

+ An exact solver for any board size, with 0 errors on 73,557 published positions, and three labeled
data sets that it made: 106,898 training positions and 2,000 opening positions on 8×7, and 9,990
late positions on 9×8.
+ Eight agents from four families on one shared interface, which include a logistic regression and a
convolutional network that we trained on 8×7 labels only.
+ A transfer study with two measures, move accuracy against exact labels and a 9×8 tournament,

= Related Work

*Solvers for Connect Four.* Allis solved 7×6 with a program that applies nine strategy rules, such
as claimeven, together with conspiracy-number search and search tables @allis1988. Tromp later solved 
many board sizes by search and published a data set of 67,557 labeled 7×6 positions @tromp @uci. Pons 
described the design of a modern solver step by step: alpha-beta search @knuth1975, a board stored as 
bits, a good move order, and a table of positions that the search has already seen @pons. Our solver 
follows this design. All of this work measures play on a single board size, whereas we ask what 
transfers when the board grows.

*Search for game playing.* The standard method for playing games at this type is Alpha-beta search 
with an evaluation function at a depth limit @russell2020. Monte Carlo tree search (MCTS) needs 
no evaluation function. Its most common form, UCT @kocsis2006, selects moves in the tree with the UCB1 
rule @auer2002, and many variants exist @browne2012. Winands et al. showed how to keep proven wins and 
losses in the tree @winands2008, and we use a small form of this idea.

*Networks that guide a search.* AlphaZero trains a network by self-play and uses it to guide MCTS
@silver2018. MuZero uses a similar rule to select moves in the tree @schrittwieser2020. Our
guided agent learns from solver labels instead of from self-play to guide search. A fully 
convolutional network has no layer with a fixed input size @long2015, so it can read a board of any 
size. KataGo uses this property to train one network on many Go board sizes @wu2019, while we train on 
one size only and test on a size that the network never saw. We rank the agents with Elo ratings @elo1978, which we fit with the Bradley-Terry model @bradley1952.


= Methods

@tab:members gives the work of each member. Section 3.1 describes the shared parts, and Sections
3.2 and 3.3 describe the four agents of each member.

#figure(
  placement: auto,
  scope: "parent",
  cell-table(
    columns: (0.9in, 1fr, 1.45in, 1.75in),
    align: left,
    heavy,
    table.header([*Member*], [*Methods*], [*Implementation*], [*Experiments*]),
    rule,
    [TJ Raklovits],
    [Exact solver and solver agent. Alpha-beta with a handcrafted evaluation. Alpha-beta with a
      learned evaluation (logistic regression). Allis rule agent. (Section 3.2)],
    [`solver.rs`, `search.rs`, `learn.rs`, `rules.rs`, label programs],
    [Solver check, solve times, depth study, evaluation fit, rule ablation (Section 4.3)],
    rule,
    [Andrew Roddy],
    [Safe random baseline. Monte Carlo tree search. Fully convolutional network. Network-guided
      MCTS. (Section 3.3)],
    [`mcts.rs`, `net.rs`, training program, board size study],
    [MCTS speed and time, time scaling, network size and training, board size study (Section 4.4)],
    rule,
    [Both],
    [Problem formulation, board engine, data sets, accuracy and tournament programs. (Section 3.1)],
    [`board.rs`, `dataset.rs`, `accuracy.rs`, `tournament.rs`],
    [Main results (Section 4.2)],
    heavy,
  ),
  caption: [Overview of individual contributions.],
) <tab:members>

== Problem Formulation and Shared Components

*Task.* A position $s$ is a board together with the player to move, and the legal actions $A(s)$ are
the columns that are not full. With perfect play from both players, each position has an exact
result for the player to move: loss, draw, or win. For each legal column $c$, let $y_c (s)$ be the
exact result for the player to move after a stone in column $c$, with the order loss $<$ draw $<$
win. An agent is a function $pi$ that takes a position and a time limit and returns one legal
column. The time limit is 1 second for each move unless we say otherwise. We are evaluating these 
tasks:

+ *Game playing.* Each agent must select moves that win games against the other agents on 9×8.
+ *Supervised learning.* The learned agents predict exact results from solver labels. The logistic
  regression predicts the result of a position from four features (Section 3.2). The network
  predicts one of three classes (win, draw, loss) for each column (Section 3.3).
+ *Transfer.* All training uses 8×7 labels only, and all tests on 9×8 use the same weights with no
  further training.

We measure how well an agent agrees with the solver by its move accuracy on a set $D$ of labeled
positions:
$ "Acc"(pi) = 1 / abs(D) sum_(s in D) bb(1) [ y_(pi(s)) (s) = max_(c in A(s)) y_c (s) ]. $ <eq:acc>
A move counts as correct if no other legal move has a better exact result. Many positions have more
than one correct move, so a random legal move is already correct in about 70% of our positions.

*Shared software.* The board engine stores a position as two bit patterns, one for the stones of the
player to move and one for all stones. It uses 64 bits for boards up to 8×7 and 128 bits for 9×8,
and the same code runs for every size. All agents use the same interface, where we input a position and
output a column.

*Data.* We made positions by playing random games in which a player never makes a move that lets the
opponent win at once, and we removed duplicates and mirror images. The solver from Section 3.2 then
labeled each position on a cloud computer with 192 virtual CPUs, with a limit of 3,600 s for each
position. @tab:data lists the three sets. Each line of a data file holds the moves, the result for
the player to move, and the result of each column. In the line `4673247653511662 + --+-+-+-`, for
example, the player to move wins, and columns 3, 5 and 7 are the winning moves.

#figure(
  placement: auto,
  scope: "parent",
  cell-table(
    columns: 6,
    align: (left, center, center, right, center, left),
    heavy,
    table.header([*Set*], [*Board*], [*Stones*], [*Positions*], [*Win / draw / loss*], [*Use*]),
    rule,
    [Training], [8×7], [14--44], [106,898], [60,555 / 3,888 / 42,455],
    [Training (96,209), held-out (10,689)],
    [Openings], [8×7], [10--12], [2,000], [1,236 / 81 / 683], [Test only],
    [Late 9×8], [9×8], [24--60], [9,990], [5,702 / 148 / 4,140], [Test only],
    heavy,
  ),
  caption: [The labeled data sets. The results are for the player to move.],
) <tab:data>

== Exact Solver and Knowledge-Based Agents (TJ Raklovits)

*Exact solver.* The solver is a depth-first search with alpha-beta pruning @knuth1975 over the three
possible results. Pruning lets the search skip branches that cannot change the result. The solver
has no evaluation function and always searches until the game ends. We used existing methods from 
published solvers @tromp @pons to make it fast. Bit operations detect a win in one move before the 
search goes deeper. The search tries the center columns first and then sorts the moves by the number of 
threats that each move creates, where a threat is an empty cell that would complete a line of four for 
one player. Finally, a table with about two million slots stores a lower and an upper limit for the 
result of each position that the search has already seen. Each slot holds two positions, the one that 
needed the most work and the most recent one. To label a position, the solver runs one search for each 
legal column.

*Solver agent.* this agent asks the solver for the result of each legal column until the
deadline and plays the column with the best proven result. An unfinished column ranks above a proven 
loss, and the nearest column to the center wins a tie. So when the solver cannot finish, this agent 
plays the column nearest to the center that is not a proven loss.

*Alpha-beta with a handcrafted evaluation.* This agent searches with alpha-beta pruning to a depth
limit and scores the positions at the limit. It searches to depth 1, then 2, then 3, and so on until
the deadline, and it plays the best move of the last depth that it completed. A win scores higher the
sooner it comes. All other positions get a score that is a linear function of four threat counts:
$ V_w (s) = w^top f(s), \ f(s) = (t_"own"^"good", t_"own"^"other", t_"opp"^"good", t_"opp"^"other"). $ <eq:eval>
Here $t_"own"$ counts the threats of the player to move and $t_"opp"$ counts the threats of the
opponent. The split into good rows and other rows comes from the parity (odd or even) rule of Allis
@allis1988. Near the end of a game the players must fill the columns in turn, so the row of a threat
decides which player gets to play there. A row is good for a player if that player gets its cells
when all other columns are filled first. On a board with $C$ columns and $R$ rows, a cell in row $r$
(counted from 0 at the bottom) of the last open column is filled after $(C - 1) R + r$ stones, so it
belongs to the first player when this number is even. The agent computes this rule from the board
size, which means that the evaluation needs no change for 9×8. The handcrafted weights are a tested 
estimate: $w = (4, 1, -4, -1)$.

*Alpha-beta with a learned evaluation.* This agent uses the same search and the same four features,
but it learns the weights from the 8×7 training set. A logistic regression predicts the result of a
position:
$ p_w (s) = sigma(w^top f(s) + b), \
  L(w, b) = - sum_s [ t_s log p_w (s) \
  + (1 - t_s) log(1 - p_w (s)) ], $ <eq:logit>
where $sigma$ is the logistic function and the target $t_s$ is 1 for a win, 0.5 for a draw and 0 for
a loss. We wrote the fit with the Newton method and used no library. The learned weights are
$w = (1.010, 0.662, -1.152, -0.895)$. Compared with our estimate, the data give less weight to good
rows (a ratio of 1.5 to 1 instead of 4 to 1) and weigh the threats of the opponent more heavily than
the threats of the player to move.

*Allis rule agent.* This agent uses rules only and never looks more than one move ahead. It applies
the first rule that gives a move: (1) play a move that wins at once, (2) block a win of the
opponent, (3) play the claimeven follow-up, which is to play on top of the last stone of the
opponent if that cell is on a good row and the move is safe, (4) play the safe move with the best
balance of good-row threats, and (5) play the column nearest to the center. A move is safe if it
does not let the opponent win at once. The agent uses only this subset of the nine rules of Allis.

== Sampling and Neural Agents (Andrew Roddy)

*Safe random.* This baseline plays a move that wins at once if one exists, and otherwise a move that
blocks a win of the opponent. In all other positions it plays a random legal move. We based our Elo scale on this as the zero value.

*Monte Carlo tree search.* MCTS builds a tree of positions in memory, and each simulation has four
steps. It first goes down the tree from the current position, and at each node it selects the child
$c$ with the largest value of
$ Q(c) + k sqrt((ln N) / n_c), quad k = sqrt(2), $ <eq:uct>
where $Q(c)$ is the mean result of the simulations through $c$, $n_c$ is their count, and $N$ is the
count for the parent @auer2002 @kocsis2006. At the leaf that it reaches, the search adds the
children, plays random legal moves until the game ends (a playout), and adds the result to each
node on the path. When the time ends, the agent plays the move with the most simulations. This agent
needs no evaluation function and no training. We added one rule that plain UCT does not have: a move
that wins at once is a proven win, and the search always selects it, while a move that lets the
opponent win at once is a proven loss, and the search never selects it @winands2008.

*Fully convolutional network.* The input is three planes with the size of the board: the stones of
the player to move, the stones of the opponent, and the cells where a stone can land next. Six
convolution layers with 3×3 filters, 64 channels and ReLU keep the size of the board. A last layer
with 1×1 filters gives three numbers for each cell, and the maximum over the rows of a column gives
three numbers $z_c$ for each column. A softmax turns these into the probability of a win, a draw and
a loss for that column. No layer has a fixed board size, so the same 186,627 weights run on any
board @long2015, and six 3×3 layers see a window of 13×13 cells, which is larger than the 9×8 board.
The training loss is the cross-entropy over the legal columns of all positions:
$ L(theta) = - 1 / M sum_s sum_(c in A(s)) log "softmax"(z_c (s; theta))_(y_c (s)), $ <eq:net>
where $M$ is the number of legal columns in the training set. We trained on the 96,209 training
positions with Adam @kingma2015 (learning rate 0.001, batches of 256, 12 epochs) in the Burn library
@burn. The agent plays the column with the highest expected score, $P("win") + 0.5 P("draw")$, using
one pass through the network and no search.

*Network-guided MCTS.* This agent uses the same tree code as MCTS, but the network replaces the
random playout. When the search reaches a new leaf, one pass through the network gives the expected
score of each column. The value of the leaf is the highest of these scores, and the prior $P(c)$ of
column $c$ is its expected score divided by the sum over the legal columns. The search then selects
the child with the largest value of
$ Q(c) + P(c) sqrt(N) / (1 + n_c) (1.25 + ln((N + 19653) / 19652)), $ <eq:puct>
which is the rule of MuZero @schrittwieser2020. The proven-result rule of the MCTS agent applies
here too. One pass through the network takes much more time than one random playout, so this agent
runs far fewer simulations in the same time.


= Experiments

== Experimental Setup

*Data and splits.* We used the three sets in @tab:data. We chose the shape of the network on a
validation part of the training lines, and we did not use the held-out lines, the opening set, or the
9×8 set to make any decision.

*Metrics.* We use three measures. The first is move accuracy (@eq:acc) with 95% Wilson intervals
@wilson1927. The second is wins kept: the agent plays 8×7 games against the exact solver, starting
from 200 held-out positions with 16 to 20 stones, once on each side. The solver gets 600 s for each
move and the agent gets 1 s, and we report the share of the 198 games in which the agent started
from a won position and won the game. The third is the win rate and the Elo rating in the
tournament. We fit the ratings $theta$ by maximum likelihood with the model
$ P(a "beats" b) = sigma(theta_a - theta_b), \ "Elo"_a = 400 / (ln 10) dot theta_a, $ <eq:elo>
where a draw counts as half a win and safe random is fixed at 0. A difference of 400 Elo means odds
of 10 to 1. To give a 95% interval for the difference between two agents, we use a bootstrap
@efron1979: we fit the ratings again on 1,000 random samples of the 50 openings.

*Tournament and baselines.* The eight agents play on 9×8. Each ordered pair of agents (56 pairs)
plays from the same 50 random openings with 4 to 6 stones, so each pair plays 100 games and the
tournament has 2,800 games. Safe random is the baseline agent. For move accuracy, the baseline is a
random legal move, and for the evaluation fit, it is a model that always predicts the most common
result.

== Main Results

#let main-rows = agents.map(a => (
  a,
  float(accuracy("train_8x7 held-out", a.id).accuracy),
  float(accuracy("openings_8x7", a.id).accuracy),
  float(accuracy("late_9x8", a.id).accuracy),
  val("versus_solver", "8x7", a.id, "agent value win", "kept_value_rate"),
  val("tournament_time_scaling", "9x8", a.id, "1000 ms/move", "win_rate"),
  val("tournament_time_scaling", "9x8", a.id, "1000 ms/move", "elo"),
))
#let best(j) = calc.max(..main-rows.map(r => r.at(j)))
#let mark(r, j, shown) = if r.at(j) == best(j) { strong(shown) } else { shown }

#figure(
  placement: auto,
  scope: "parent",
  cell-table(
    columns: (auto, auto, 1fr, 1fr, 1fr, 1.1fr, 1.1fr, 0.8fr),
    align: (left, left, center, center, center, center, center, center),
    heavy,
    table.header(
      table.cell(rowspan: 2, align: bottom + left)[*Agent*],
      table.cell(rowspan: 2, align: bottom + left)[*Member*],
      table.cell(colspan: 3)[*Move accuracy (%)*],
      [*Wins kept*],
      table.cell(colspan: 2)[*9×8 tournament*],
      table.hline(start: 2, end: 5, stroke: 0.4pt),
      table.hline(start: 6, end: 8, stroke: 0.4pt),
      [8×7 \ held-out], [8×7 \ openings], [9×8 \ late], [8×7 (%)], [Wins (%)], [Elo],
    ),
    rule,
    ..main-rows
      .map(r => (
        r.at(0).name,
        r.at(0).who,
        mark(r, 1, pct(r.at(1))),
        mark(r, 2, pct(r.at(2))),
        mark(r, 3, pct(r.at(3))),
        mark(r, 4, pct(r.at(4))),
        mark(r, 5, pct(r.at(5))),
        mark(r, 6, fix(r.at(6), d: 0)),
      ))
      .flatten(),
    rule,
    [_Random legal move_], [---],
    pct(float(accuracy("train_8x7 held-out", "mcts").random_baseline)),
    pct(float(accuracy("openings_8x7", "mcts").random_baseline)),
    pct(float(accuracy("late_9x8", "mcts").random_baseline)),
    [---], [---], [---],
    heavy,
  ),
  caption: [Main results with 1 s for each move. The best value in each column is in *bold*. Each 
95% interval is at most ±2.0 points. Each agent played 700 tournament games.],
) <tab:main>

*Transfer.* @tab:main gives the main results. All agents kept their accuracy on the 9×8 board. The
network changed from 92.4% to 91.0% on a board size that was not part of its training, and the two
alpha-beta agents and MCTS stayed at 99% on both boards. The learned evaluation, which learned its
four weights on 8×7, was a little better than the handcrafted evaluation on 8×7 and equal to it on
9×8. Because the two boards have different position sets, small differences between the columns do
not show a real change.

*Accuracy is not strength.* The order in the tournament is very different from the order by
accuracy. The solver agent had 97.8% accuracy on late 9×8 positions but won only 23% of its games,
and the network had 91.0% accuracy but won only 20%. In contrast, the four agents that search with a
time limit (MCTS, the two alpha-beta agents, and network-guided MCTS) each scored at least 92 points
out of 100 against each of the other four agents (@tab:h2h).

*The top of the table.* MCTS and alpha-beta with the learned evaluation were almost equal in their
own games (51 to 49 points). Their difference in Elo is 18 points with a 95% interval from −22 to
+57, so the data do not show which of the two is stronger. MCTS is above handcrafted alpha-beta by
50 Elo (+7 to +93) and above network-guided MCTS by 93 Elo (+50 to +137). The learned evaluation is
ahead of the handcrafted evaluation by 32 Elo, but its interval (−10 to +73) includes zero.

#let score = (W: 1.0, D: 0.5, L: 0.0)
#let h2h = {
  let d = (:)
  for r in games.filter(r => r.experiment == "tournament_1000ms") {
    let s = score.at(r.first_result)
    let forward = r.first + "|" + r.second
    let backward = r.second + "|" + r.first
    d.insert(forward, d.at(forward, default: 0.0) + s)
    d.insert(backward, d.at(backward, default: 0.0) + 1.0 - s)
  }
  d
}
#let h2h-row(id) = (
  agent(id).short,
  ..top-four.map(col => if id == col { [---] } else { fix(h2h.at(id + "|" + col)) }),
)
#let phases = ("10-12", "14-15", "16-20", "21-26", "27-32", "33-38", "39-44")
#let phase-points(id) = phases.enumerate().map(((i, phase)) => (
  i,
  float(accuracy(if i == 0 { "openings_8x7" } else { "train_8x7 held-out" }, id, phase: phase).accuracy),
))

#pair(
  columns: (1fr, 1.05fr),
  [
    #figure(
      cell-table(
        columns: (auto, 1fr, 1fr, 1fr, 1.3fr),
        inset: (x: 3pt, y: 2.6pt),
        align: (left, center, center, center, center),
        heavy,
        table.header([], ..top-four.map(id => strong(agent(id).short))),
        rule,
        ..top-four.map(h2h-row).flatten(),
        rule,
        ..("allis", "perfect", "network", "safe-random").map(h2h-row).flatten(),
        heavy,
      ),
      caption: [Points of the row agent against the column agent in 100 games on 9×8 (a win is 1, a
        draw is 0.5).],
    ) <tab:h2h>
  ],
  [
    #figure(
      lineplot(
        w: 6.3cm,
        h: 3.3cm,
        xr: (-0.3, 6.3),
        yr: (0.68, 1.0),
        xticks: phases.enumerate().map(((i, p)) => (i, p.replace("-", "–"))),
        yticks: ((0.7, "70"), (0.8, "80"), (0.9, "90"), (1.0, "100")),
        xlabel: [stones on the board],
        ylabel: [move accuracy on 8×7 (%)],
        series: ("perfect", "mcts", "alpha-beta-learned", "network-mcts", "network", "allis").map(id => (
          label: if id == "perfect" { "Solver, 1 s" } else { agent(id).short },
          color: agent(id).color,
          pts: phase-points(id),
        )),
        legend-columns: 3,
      ),
      caption: [Move accuracy on 8×7 by the number of stones (opening set at 10-12 stones, held-out
        set for the others).],
    ) <fig:phase>
  ],
)

*Accuracy by phase.* @fig:phase shows the accuracy by the number of stones on the board. When 27 or more 
stones are on the board, the solver agent is correct, but it falls to 80.7% in the openings, where
it cannot finish in 1 s. The agents that search with a time limit lose only a few points in the
openings. The network is the only agent whose accuracy does not rise late in the game, and in the
openings network-guided MCTS has the best accuracy of all agents (94.6%).

*The bottom of the table.* The four weak agents do not have a clear order. The network alone lost to
safe random (42 to 58 points) although its accuracy is much better, because it has no rule for a win
in one move or a block, and safe random has both.

== Experiments by TJ Raklovits

*Solver check.* The solver gave the published result for all 67,557 positions of the UCI set @uci
and all 6,000 test positions of Pons @pons, with 0 errors. We also calculated 306 of the 8×7 opening
labels two times, on the laptop and on the cloud computer, and found 0 differences.

#let wall = range(24, 61).map(stones => {
  let row = meas.find(r => (
    r.study == "solve_time" and r.board == "9x8" and r.setting.starts-with(str(stones) + " stones")
  ))
  (stones: stones, median: float(row.value), max: float(row.notes.split(" ").at(1)))
})
#let depths = (1, 2, 4, 6, 8, 10, 12)
#let depth-points(board, id) = depths.map(d => (
  d,
  val("alphabeta_depth", board, id, "depth " + str(d), "move_accuracy"),
))

#pair(
  [
    #figure(
      lineplot(
        w: 5.9cm,
        h: 3.3cm,
        xr: (23, 61),
        yr: (0.05, 5000),
        ylog: true,
        xticks: (24, 30, 36, 42, 48, 54, 60).map(x => (x, str(x))),
        yticks: ((0.1, "0.1"), (1, "1"), (10, "10"), (100, "100"), (1000, "1,000")),
        marks: ((1, [1 s time limit]),),
        xlabel: [stones on the 9×8 board],
        ylabel: [seconds for one position (log scale)],
        series: (
          (label: "median", color: rgb("#222222"), pts: wall.map(p => (p.stones, p.median))),
          (label: "slowest position", color: rgb("#d1495b"), dash: "dashed", pts: wall.map(p => (p.stones, p.max))),
        ),
      ),
      caption: [Time of the exact solver for late 9×8 positions (about 270 positions for each number
        of stones, one core each).],
    ) <fig:wall>
  ],
  [
    #figure(
      lineplot(
        w: 5.9cm,
        h: 3.3cm,
        xr: (0.5, 12.5),
        yr: (0.90, 1.0),
        xticks: depths.map(d => (d, str(d))),
        yticks: ((0.90, "90"), (0.925, "92.5"), (0.95, "95"), (0.975, "97.5"), (1.0, "100")),
        xlabel: [search depth (moves)],
        ylabel: [move accuracy (%)],
        series: (
          (label: "8×7, handcrafted", color: agent("alpha-beta-handcrafted").color, pts: depth-points("8x7", "alpha-beta-handcrafted")),
          (label: "8×7, learned", color: agent("alpha-beta-learned").color, pts: depth-points("8x7", "alpha-beta-learned")),
          (label: "9×8, handcrafted", color: agent("alpha-beta-handcrafted").color, dash: "dashed", pts: depth-points("9x8", "alpha-beta-handcrafted")),
          (label: "9×8, learned", color: agent("alpha-beta-learned").color, dash: "dashed", pts: depth-points("9x8", "alpha-beta-learned")),
        ),
        legend-columns: 2,
      ),
      caption: [Move accuracy of alpha-beta at a fixed depth, on the first 1,000 positions of the
        held-out set (8×7) and of the late set (9×8).],
    ) <fig:depth>
  ],
)

*Solve times.* @fig:wall shows why we cannot solve 9×8 and why the solver agent is weak. With 37 or
more stones on the board, the solver needs about 0.1 s. With fewer stones the time grows fast: the
median is above 1 s at 27 stones and 11.7 s at 24 stones, and the slowest position at 24 stones took
31 minutes. On the laptop the median was 80 s at 20 stones, and at 16 stones all six test positions
took more than 10 minutes. A game starts with only 4 to 6 stones, so the solver agent plays most of
the important moves without an answer.

*Search depth.* @fig:depth shows the accuracy of alpha-beta at a fixed depth. One move of search
already gives 91% on 8×7, and the accuracy rises to 97.5% at depth 8 and then stays almost flat,
although depth 12 visits 35 times as many positions as depth 8 (379,324 against 10,896 for each
move). The curves for 9×8 have the same shape. More depth still wins games, however. In 200 games on
9×8 for each pair of depths, the deeper search scored
#pct(val("depth_vs_depth", "9x8", "alpha-beta-handcrafted", "depth 4 vs depth 2", "deeper_score"), d: 0)%
(depth 4 against 2),
#pct(val("depth_vs_depth", "9x8", "alpha-beta-handcrafted", "depth 6 vs depth 4", "deeper_score"), d: 0)%
(6 against 4),
#pct(val("depth_vs_depth", "9x8", "alpha-beta-handcrafted", "depth 8 vs depth 6", "deeper_score"), d: 0)%
(8 against 6) and
#pct(val("depth_vs_depth", "9x8", "alpha-beta-handcrafted", "depth 10 vs depth 8", "deeper_score"), d: 0)%
(10 against 8) of the points.

#let fit(setting, metric) = val("logistic_fit", "8x7", "alpha-beta-learned", setting, metric)
#let duel(depth) = pick("handcrafted_vs_learned_duel", "9x8", "alpha-beta-handcrafted", "both depth " + str(depth), "score")
#let ablation(board, setting) = pick("allis_ablation", board, "allis", setting, "score")
#let interval(row) = box[#pct(float(row.value)) #text(size: 7pt)[(#pct(float(row.ci_low))--#pct(float(row.ci_high)))]]

#pair(
  columns: (1fr, 1.45fr),
  [
    #figure(
      cell-table(
        columns: (auto, 1fr, 1fr),
        align: (left, center, center),
        heavy,
        table.header([*Model*], [*Acc. (%)*], [*Log loss*]),
        rule,
        [Most common result], pct(fit("constant", "held_out_accuracy")), fix(fit("constant", "held_out_log_loss"), d: 3),
        [Handcrafted weights], pct(fit("handcrafted (calibrated)", "held_out_accuracy")), fix(fit("handcrafted (calibrated)", "held_out_log_loss"), d: 3),
        [Learned weights], strong(pct(fit("learned", "held_out_accuracy"))), strong(fix(fit("learned", "held_out_log_loss"), d: 3)),
        heavy,
      ),
      caption: [Prediction of the result of a position (win or not) on the 10,689 held-out
        positions.],
    ) <tab:fit>
  ],
  [
    #figure(
      cell-table(
        columns: (auto, 1fr, 1fr),
        align: (left, center, center),
        heavy,
        table.header([*Version*], [*8×7*], [*9×8*]),
        rule,
        [All rules],
        interval(ablation("8x7", "full rules vs safe-random")),
        interval(ablation("9x8", "full rules vs safe-random")),
        [No claimeven],
        interval(ablation("8x7", "without claimeven follow-up vs safe-random")),
        interval(ablation("9x8", "without claimeven follow-up vs safe-random")),
        [Inverted parity],
        interval(ablation("8x7", "follow-up parity inverted vs safe-random")),
        interval(ablation("9x8", "follow-up parity inverted vs safe-random")),
        [All rules vs no claimeven],
        interval(ablation("8x7", "full rules vs without claimeven")),
        interval(ablation("9x8", "full rules vs without claimeven")),
        heavy,
      ),
      caption: [Ablation of the Allis rule agent: score (%) in 400 games with 95% intervals, against
        safe random except in the last row.],
    ) <tab:allis>
  ],
)

*Learned evaluation.* @tab:fit compares the two sets of weights as predictors of the result. The
learned weights are better than our estimate on the held-out positions (70.1% against 67.8%), and
both are better than the baseline. In games on 9×8 with the same depth for both agents, the
handcrafted agent scored
#pct(float(duel(2).value), d: 0)%,
#pct(float(duel(4).value), d: 0)%,
#pct(float(duel(6).value), d: 0)% and
#pct(float(duel(8).value), d: 0)%
of the points against the learned agent at depths 2, 4, 6 and 8 (200 games each). The learned agent
is ahead at three of the four depths, but each 95% interval includes 50%.

*Rule ablation.* @tab:allis shows the Allis rule agent with one rule changed. The rules transfer,
because the agent beat safe random on both boards. The claimeven follow-up did not help, however.
Without it the agent scored a little more against safe random on both boards, and in direct games the
two versions were equal. When we inverted the parity of the follow-up, the score was a little lower.

== Experiments by Andrew Roddy

#let times = (500, 1000, 5000, 10000)
#let time-points(id) = times.enumerate().map(((i, ms)) => (
  i,
  val("tournament_time_scaling", "9x8", id, str(ms) + " ms/move", "elo"),
))
#let budget(board, id, ms) = pct(val("mcts_budget", board, id, str(ms) + " ms/move", "move_accuracy", notes: "after"))
#let sweep(shape) = calc.min(
  val("network_sweep", "8x7", "network", "width x depth " + shape + ", lr 0.001", "best_validation_loss"),
  val("network_sweep", "8x7", "network", "width x depth " + shape + ", lr 0.003", "best_validation_loss"),
)

*MCTS speed and time.* On an empty board, MCTS runs about 990,000 simulations in one second on 7×6,
933,000 on 8×7 and 737,000 on 9×8. @tab:budget shows that MCTS is already accurate with 0.1 s for
each move and gains at most one more point with 1 s. The proven-result rule changed the accuracy by
less than its 95% interval.

*Time scaling.* We ran the full tournament four times, with 0.5 s, 1 s, 5 s and 10 s for each move
(@fig:time). MCTS is the only agent that gained at every step, with 695, 740, 754 and 761 Elo, and
its win rate rose from 79.4% to 81.3%. Network-guided MCTS also gained from more time (617 to 686
Elo), while the two alpha-beta agents stayed in the same range. Each tournament has its own Elo
scale, so small changes between time limits are not certain.

#pair(
  columns: (1fr, 1.05fr),
  at: bottom,
  [
    #figure(
      lineplot(
        w: 5.6cm,
        h: 3.5cm,
        xr: (-0.3, 3.3),
        yr: (600, 780),
        xticks: ((0, "0.5 s"), (1, "1 s"), (2, "5 s"), (3, "10 s")),
        yticks: ((600, "600"), (650, "650"), (700, "700"), (750, "750")),
        xlabel: [time for each move],
        ylabel: [Elo on 9×8 (safe random = 0)],
        series: top-four.map(id => (
          label: agent(id).short,
          color: agent(id).color,
          pts: time-points(id),
        )),
        legend-columns: 4,
      ),
      caption: [Elo of the four best agents in four tournaments of 2,800 games with different time
        limits.],
    ) <fig:time>
  ],
  [
    #figure(
      cell-table(
        columns: (auto, 1fr, 1fr, 1fr, 1fr),
        align: (left, center, center, center, center),
        heavy,
        table.header(
          table.cell(rowspan: 2, align: bottom + left)[*Agent*],
          table.cell(colspan: 2)[*8×7*],
          table.cell(colspan: 2)[*9×8*],
          [0.1 s], [1 s], [0.1 s], [1 s],
        ),
        rule,
        [MCTS], budget("8x7", "mcts", 100), budget("8x7", "mcts", 1000), budget("9x8", "mcts", 100), budget("9x8", "mcts", 1000),
        [Network-guided MCTS], budget("8x7", "network-mcts", 100), budget("8x7", "network-mcts", 1000), budget("9x8", "network-mcts", 100), budget("9x8", "network-mcts", 1000),
        [Network alone], table.cell(colspan: 2, budget("8x7", "network", 100)), table.cell(colspan: 2, budget("9x8", "network", 100)),
        heavy,
      ),
      caption: [Move accuracy (%) by time for each move, on 600 held-out positions (8×7) and 300
        late positions (9×8).],
    ) <tab:budget>
    #v(0.2em)
    #figure(
      cell-table(
        columns: (auto, 1fr, 1fr, 1fr),
        align: (left, center, center, center),
        heavy,
        table.header([*Channels*], [*2 layers*], [*4 layers*], [*6 layers*]),
        rule,
        [32], fix(sweep("32x2"), d: 3), fix(sweep("32x4"), d: 3), fix(sweep("32x6"), d: 3),
        [64], fix(sweep("64x2"), d: 3), fix(sweep("64x4"), d: 3), strong(fix(sweep("64x6"), d: 3)),
        heavy,
      ),
      caption: [Best validation loss of the network by size (the better of two learning rates). By
        Andrew Roddy.],
    ) <tab:sweep>
  ],
)

*Network size and training.* @tab:sweep shows the validation loss for six network sizes. More layers
helped much more than more channels: with 32 channels, the move accuracy on the validation part rose
from 82% with 2 layers to 91% with 6 layers. We chose 64 channels and 6 layers and trained this
network for 12 epochs, which took 37 minutes. The training loss went from 0.653 in the first epoch
to 0.329 in the last. The sweep ran on an earlier version of the data split, so its values are good
only for comparing the sizes.

*Network alone and network as a guide.* The same weights give two very different agents. Alone, the
network keeps 80% of its won positions against the solver and has 37 Elo. As a guide for MCTS it
keeps 96% and has 647 Elo (@tab:main). The search raised the move accuracy from 92.4% to 98.2% on
8×7 and from 91.0% to 98.5% on 9×8, although plain MCTS is still a little more accurate on these
sets.

#let sizes = ("4x4", "5x4", "6x5", "7x6", "8x7", "9x8")
#let size-order = ("mcts", "alpha-beta-learned", "alpha-beta-handcrafted", "network-mcts", "perfect", "allis", "network", "safe-random")
#let shares = {
  let points = (:)
  let count = (:)
  for r in study.filter(r => r.start == "random" and r.ms == "250") {
    let s = score.at(r.first_result)
    for (id, gained) in ((r.first, s), (r.second, 1.0 - s)) {
      let key = r.size + "|" + id
      points.insert(key, points.at(key, default: 0.0) + gained)
      count.insert(key, count.at(key, default: 0) + 1)
    }
    let key = r.size + "|draws"
    points.insert(key, points.at(key, default: 0.0) + if r.first_result == "D" { 1.0 } else { 0.0 })
    count.insert(key, count.at(key, default: 0) + 1)
  }
  let out = (:)
  for (key, total) in points {
    out.insert(key, total / count.at(key))
  }
  out
}
#let heat(x) = table.cell(fill: color.mix((rgb("#ff922f"), x * 100), (white, 100 - x * 100)), pct(x, d: 0))

#figure(
  placement: auto,
  scope: "parent",
  cell-table(
    columns: (auto,) + (1fr,) * 9,
    inset: (x: 2pt, y: 2.6pt),
    align: center,
    heavy,
    table.header([*Board*], ..size-order.map(id => strong(agent(id).short)), [*Draws*]),
    rule,
    ..sizes
      .map(size => (
        size.replace("x", "×"),
        ..size-order.map(id => heat(shares.at(size + "|" + id))),
        pct(shares.at(size + "|draws"), d: 0),
      ))
      .flatten(),
    heavy,
  ),
  caption: [Board size study: share of the points (%) of each agent in a full tournament on each
    board size, with 0.25 s for each move and random openings (1,120 games for each board). The
    last column is the share of games that ended in a draw.],
) <tab:sizes>

*Board size.* To see how the order of the agents depends on the board, we ran the tournament on six
board sizes (@tab:sizes). On the three smallest boards the solver agent finishes most of its searches
and belongs to the top group with 59% to 70% of the points, but on 8×7 and 9×8 it falls to 31% to
32%. MCTS and the two alpha-beta agents stay in the top group on every size. The network alone,
which trained on 8×7 only, is weak on all sizes, and network-guided MCTS comes nearest to the top on
the two largest boards. We also ran this study from an empty board. There, on 9×8, alpha-beta with
the learned evaluation was first (89% of the points) and MCTS was third (78%), so the order of the
three best agents depends on the start positions.


= Discussion

*Why accuracy did not decide the games.* A game is a chain of moves, and one bad move can lose it.
An agent that is correct in 91% of its moves makes no error in 20 moves with a probability of only
$0.91^20 approx 0.15$. The errors of the network are also not random: it has no rule for a win in
one move, so it sometimes misses a win or a block that a search of depth 2 always finds. This
explains why safe random, with only 76% accuracy, beat the network in their own games. Search
corrects these errors, which is why the same network gained 610 Elo as a guide for MCTS. A second
reason lies in our measure. The 9×8 labels exist only for positions with 24 or more stones, but the
games are decided earlier. The solver agent shows this limit most clearly, because it is almost
perfect where we can measure it and weak where we cannot.

*What transferred, and why.* All of the knowledge in our agents is local. A threat, the parity of a
row, and a 3×3 filter do not depend on the width or the height of the board, and MCTS and the solver
have no knowledge of the board size at all. This is why the differences between 8×7 and 9×8 in
@tab:main are small. The logistic regression learned only four numbers, and these numbers describe
threats, so they are as valid on 9×8 as on 8×7. Its limit is that it can only reweigh the features
that we chose. The network can find new features, but after 12 epochs of training on positions from
random games it was only about as accurate as a search of depth 1 with four features (@fig:depth).

*MCTS, alpha-beta, and time.* MCTS and alpha-beta reached almost the same strength in different
ways. Alpha-beta is exact to its depth and then trusts a simple evaluation. Two extra moves of depth
cost about six times as many positions, so more time adds little depth, and @fig:time shows no clear
gain for alpha-beta from 1 s to 10 s. MCTS turns each extra second into more playouts and kept
improving. But MCTS was not better in all conditions: in the board size study, the first place
changed between MCTS and the two alpha-beta agents with the board and with the start position. The
right conclusion is that the three agents form one group, and not that one of them is the best.

*Why the guided search did not beat plain MCTS.* The idea of AlphaZero did work, because the guide
turned a weak network into a strong agent. Three things, however, kept the guided agent below plain
MCTS. First, one pass through our network costs much more than a random playout, so the guided
search runs far fewer simulations. Second, the network trained on positions with 14 to 44 stones,
while the tournament starts at 4 to 6 stones, so the first moves are outside its training data.
Third, the network learned from positions of random games, not from positions that strong agents
reach. The opening set hints at what is possible: at 10 to 12 stones, the guided agent was the most
accurate of all agents.

*The claimeven surprise.* We expected the claimeven rule to help, because it is the best-known rule
of Allis, but it did not (@tab:allis). In the work of Allis the rules are parts of a proof, and a set
of rules is valid only if together they answer all possible threats of the opponent. Our agent
applies one rule alone, before it looks at threats, so the rule sometimes replaces a better move. A
small number of rules is not a full strategy.

*Limitations.* First, the tournament gives only relative strength on 9×8, because no perfect player
exists for this board. Second, the Elo scale depends on safe random, which almost never wins against
the best agents, so the order of the Elo values is more certain than their size. Third, the solver
limit of 3,600 s removed 17 opening positions and ten 9×8 positions, so the most difficult positions
are a little under-represented. Fourth, the depth study, the rule ablation, and the network sweep
have summary values only, with no file for each game. Finally, the rule agent uses a subset of the
rules of Allis, the solver agent has no opening book (some 8×7 positions with six stones took more
than 30 minutes each), and the network is small and trained for only 12 epochs.


= Conclusion

We trained Connect Four agents on the solved 8×7 board and tested them on the 9×8 board, which
is not solved. Three results are clear. First, on a board that we cannot solve, search wins: the
four agents that search with a time limit were far above all agents that do not. Second, the
knowledge from 8×7 transferred to 9×8 with almost no loss, both for four learned weights and for a
network with 186,627 weights. Third, knowledge is most useful as a guide for search, since the same
network went from 37 to 647 Elo when a tree search used it. For future work, the most important
steps are a larger network trained for longer, training positions from self-play that include the
opening, and the full rule set of Allis. An opening book for 8×7 would also help, because it would
let us measure the early game, where the games are decided.

#bibliography("final_report.bib", style: "ieee", title: "References")

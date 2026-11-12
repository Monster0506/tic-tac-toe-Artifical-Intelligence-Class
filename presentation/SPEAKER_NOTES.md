# Speaker notes

About 10 minutes in total (90 presses of Next, most about 6 seconds). The
slides only show a title and a picture, so everything worth saying is here. Each
numbered step is one press of Next (right arrow, space or click): it moves the next
piece of the picture in. The presenter window (press N) shows the current step and
the next one.

Generated from `src/slides.rs`: after changing a slide, run
`C4_WRITE_NOTES=1 cargo test speaker_notes` to rewrite this file.

## 1. Connect Four AI  (18 s, starts at 0:00)

1. Hi everyone, we're TJ and Andrew, group 3. Our project: eight different AI agents playing Connect Four. Here's the twist: we trained them on a board we can solve, 8 by 7, and tested them on one nobody has solved, 9 by 8. Everything you see today, these slides included, runs live in our Rust code.

## 2. A board nobody has solved  (30 s, starts at 0:18)

1. Start with the board you know: 7 by 6. It's solved. With perfect play, the first player always wins.
2. Add a column and a row: 8 by 7. John Tromp solved this one too, so that's where we trained.
3. One more of each gets you 9 by 8, and that has never been solved. On our hardware it would take roughly 15 years. So here's our question: if agents learn on a board we can solve, do they still play well on one we can't? And if some methods carry over better, why?

## 3. How big is the game?  (24 s, starts at 0:48)

1. How big are we talking? Standard Connect Four has about 4.5 trillion positions. Already a lot.
2. 8 by 7 jumps to roughly 10 to the 26.
3. And 9 by 8, roughly 10 to the 34.
4. Remember, this is a log scale: each step is millions of times bigger. Brute force is simply off the table.

## 4. An exact solver  (24 s, starts at 1:12)

1. So step one: build a perfect player anyway. Alpha-beta on bitboards with a transposition table, the same tricks published solvers use.
2. Hand it any position and it tells you, for every column: win, draw or loss.
3. These letters were computed live when the app launched. And watch: it plays the winning column.
4. Can you trust it? We checked all 67,557 UCI positions and 6,000 of Pascal Pons' tests. Zero mistakes.

## 5. Why not just solve 9x8?  (42 s, starts at 1:36)

1. So why not just solve 9 by 8? Near the end of a game it's easy: about a tenth of a second.
2. Take stones away and it slows down: a third of a second at 30.
3. Two seconds at 26.
4. At 24 stones the median is 12 seconds, and our worst case took 31 minutes.
5. At 20 stones, over a minute each.
6. At 16, every single probe blew past ten minutes.
7. Even 8 by 7 with only six stones took over half an hour. That wall is exactly why we turned to learning.

## 6. The data  (18 s, starts at 2:18)

1. Instead, the solver became our teacher: about 107 thousand 8 by 7 positions, every column labelled.
2. Plus 2,000 tricky openings, 10 to 12 stones in, held back just for testing.
3. And to test transfer, about 10 thousand late game 9 by 8 positions, solved on a 192 core cloud machine.

## 7. Eight agents  (36 s, starts at 2:36)

1. Step three, the players: eight agents, coloured by family. First, Monte Carlo tree search.
2. Two more pure search agents: handcrafted alpha-beta, and our solver capped at one second.
3. Then learning comes in: alpha-beta with a learned evaluation.
4. A convolutional network on its own, and that same network guiding MCTS.
5. Victor Allis' strategy rules: pure knowledge, no search.
6. And safe random as the baseline. Every agent gets one second per move. Let's meet them one at a time.

## 8. Safe random  (18 s, starts at 3:12)

1. Weakest first. Safe random plays at random, except it always takes a winning move and blocks an obvious loss.
2. It's our yardstick, pinned at Elo zero. It still matches the solver on 76 percent of moves, because many positions have several fine moves.
3. But it has no plan, so it keeps barely half its won positions against the solver. Every other agent should beat this.

## 9. Allis rules  (18 s, starts at 3:30)

1. Allis rules is pure knowledge: win, block, avoid squares that hand over a win, and build threats on the right rows.
2. It moves instantly, and its accuracy actually rises on 9 by 8, from 88 to 90 percent. Good strategy doesn't care about board size.
3. But with no lookahead it walks into tactics. It keeps only 78 percent of won positions and wins 31 percent of its games.

## 10. Exact solver, 1 s a move  (18 s, starts at 3:48)

1. Our exact solver as a player. When it finishes, its move is perfect. Guaranteed.
2. Late in the game it finishes easily, and that's where its 97.8 percent accuracy comes from.
3. The catch: early 9 by 8 positions take far longer than a second, so it's left guessing. It wins only 23 percent of games.

## 11. Alpha-beta, handcrafted  (18 s, starts at 4:06)

1. Alpha-beta looks ahead as deep as one second allows, pruning branches that can't matter, and scores the leaves with our evaluation.
2. The evaluation counts threats and rewards them on the right rows, Allis' odd and even rule. That makes it tactically razor sharp.
3. Its weakness is us: the weights are our guesses. Still, it wins 76 percent, and from an empty board it often beats MCTS.

## 12. Alpha-beta, learned  (18 s, starts at 4:24)

1. Same search, same features, but learned weights: a logistic regression on 107 thousand solver-labelled positions.
2. It predicts results better than our hand tuning, 70 versus 68 percent, and that edge survives the jump to 9 by 8.
3. The limit: it only reweighs the features we gave it, so it can't discover anything new. Still, second place overall.

## 13. Monte Carlo tree search  (18 s, starts at 4:42)

1. MCTS needs no evaluation at all. It plays out thousands of random games and grows its tree toward the moves that win.
2. It's simple, it scales with time, and it never misses a proven win or loss. At one second, it tops the tournament.
3. Its blind spot: random games don't understand row parity, so in quiet positions, like an empty board, alpha-beta can outplay it.

## 14. Convolutional network  (18 s, starts at 5:00)

1. The network looks at the whole board at once and scores every column. No search, just one pass.
2. Because it's fully convolutional, the same weights run on 9 by 8, and it still plays the solver's move 91 percent of the time.
3. But 91 percent per move isn't enough. Over a 40 move game a slip is almost certain, so it wins only 20 percent.

## 15. Network-guided MCTS  (18 s, starts at 5:18)

1. Last, the AlphaZero idea: the network tells MCTS which moves look promising and how good a position is.
2. Search on top of knowledge fixes the network's slips: it keeps 96 percent of won positions and climbs to 647 Elo.
3. The price: each network call costs far more than a random playout, so it searches much less. That's why plain MCTS edges it out.

## 16. Alpha-beta: how deep?  (36 s, starts at 5:36)

1. Now, what we found. Alpha-beta first: at depth 1 it already agrees with the solver 91 percent of the time.
2. Depth 2, about 94.
3. Depth 4 and 6: slow, steady gains.
4. Depth 8 reaches 97.5 percent.
5. Depth 12 adds almost nothing, while searching 35 times as many nodes. Diminishing returns.
6. But don't be fooled: in real 9 by 8 games the deeper search still wins two out of three. Depth matters.

## 17. Learning the evaluation  (18 s, starts at 6:12)

1. Can we learn the evaluation instead of hand tuning it? Baseline: always guess the most common result, 56.6 percent.
2. Our handcrafted evaluation gets 67.8.
3. Logistic regression on the same features, trained on solver labels: 70.1. And it transfers: 722 Elo on 9 by 8, versus 690.

## 18. MCTS loves time  (24 s, starts at 6:30)

1. Back to MCTS, because it has a superpower: time. At half a second a move, 695 Elo.
2. At one second, 740: top of the tournament, playing about 740 thousand random games a second.
3. At five seconds, 754.
4. At ten, 761. Its win rate keeps climbing with time, while alpha-beta's stays flat or slips.

## 19. A network for any board  (30 s, starts at 6:54)

1. The network. We made it fully convolutional, so the exact same weights run on any board size.
2. Six layers, 64 channels, trained for 12 epochs on the CPU, after our GPU setup gave up on us.
3. Out come scores, one per column. The highest is its move.
4. It picks the solver's move 92.4 percent of the time on 8 by 7, and 91 on a 9 by 8 board it never trained on.
5. Alone, though, it scores just 37 Elo, because one slip loses. Let it guide MCTS and it jumps to 647.

## 20. Victor Allis' strategy  (24 s, starts at 7:24)

1. Allis' rules, with no search at all, beat safe random 78 percent of the time on 9 by 8.
2. 83 percent on 8 by 7: same rules, nearly the same strength.
3. Plot twist: turning off claimeven, a famous rule, didn't hurt. It actually scored higher.
4. Flipping the parity hurt a bit. Lesson: a handful of rules isn't a full strategy.

## 21. Does it carry over?  (24 s, starts at 7:48)

1. Now the big question: does it carry over? The search agents play just as well on 9 by 8, even slightly better.
2. The one second solver holds at 97.8.
3. The network slips only from 92.4 to 91, on a board it has never seen.
4. The rules and the baseline barely move. So yes: what they learned transfers.

## 22. The 9x8 tournament  (30 s, starts at 8:12)

1. The main event: a full round robin on 9 by 8, 2,800 games. Safe random anchors at zero; the network alone sits just above.
2. Allis' rules and the time limited solver come next.
3. Then a big jump: network guided MCTS and handcrafted alpha-beta.
4. Learned alpha-beta takes second, at 722.
5. And the winner: plain MCTS, at 740.

## 23. Why it came out this way  (30 s, starts at 8:42)

1. So why did it shake out this way? The solver is perfect, but only when it finishes.
2. In one second it can't finish an early 9 by 8 position, so it's left guessing.
3. Knowledge alone transfers well, but without search a single mistake costs the game.
4. Add search on top of knowledge, and most of those mistakes disappear.
5. Same story on 8 by 7, measured against the solver itself: the search agents keep about 97 percent of their wins.

## 24. What we learned  (12 s, starts at 9:12)

1. Three takeaways. One: on a board you can't solve, search is what wins. Two: what agents learn on 8 by 7 carries over to 9 by 8 almost unchanged. Three: knowledge works best guiding search, not replacing it.

## 25. Now watch them play  (12 s, starts at 9:24)

1. Enough slides, let's watch them play. I'll pick two agents and a board. Heads up: from an empty board, handcrafted alpha-beta often beats MCTS. Switch Start to Random for the tournament's conditions.

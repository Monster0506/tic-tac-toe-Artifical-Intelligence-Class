# Speaker notes

About 7 minutes in total (66 presses of Next, most about 6 seconds). The
slides only show a title and a picture, so everything worth saying is here. Each
numbered step is one press of Next (right arrow, space or click): it moves the next
piece of the picture in. The presenter window (press N) shows the current step and
the next one.

Generated from `src/slides.rs`: after changing a slide, run
`C4_WRITE_NOTES=1 cargo test speaker_notes` to rewrite this file.

## 1. Connect Four AI  (12 s, starts at 0:00)

1. Hey everyone, we're TJ and Andrew from group 3. Our project is Connect Four, played by eight different AI agents. We trained them on an 8 by 7 board and tested them on 9 by 8. And everything you'll see today, these slides included, is running live in our Rust code.

## 2. A board nobody has solved  (30 s, starts at 0:12)

1. This is the normal 7 by 6 board. It's solved: with perfect play the first player wins.
2. Add a column and a row and you get 8 by 7. John Tromp solved it too. This is where we trained.
3. One more of each and you're at 9 by 8. Nobody has solved it; on our hardware it would take something like 15 years. So our question: if we train agents on a board we can solve, do they still play well on one we can't? And if some methods carry over better than others, why?

## 3. How big is the game?  (24 s, starts at 0:42)

1. To give you a feel for the size: 7 by 6 has about 4.5 trillion legal positions.
2. For 8 by 7 the upper bound is around 10 to the 26.
3. And 9 by 8 is around 10 to the 34.
4. This is a log scale, so every step up is millions of times bigger. You just can't brute force that.

## 4. An exact solver  (24 s, starts at 1:06)

1. Step one was an exact solver: alpha-beta search on bitboards with a transposition table, like the published solvers.
2. Give it any position and it says whether each column wins, draws or loses.
3. These labels were worked out live when the app started. Here, the marked column wins.
4. We checked it against all 67,557 UCI positions and 6,000 of Pascal Pons' tests. It got every one right.

## 5. Why not just solve 9x8?  (42 s, starts at 1:30)

1. So why not just solve 9 by 8? Late in the game it's quick, about a tenth of a second.
2. With fewer stones it slows down: a third of a second at 30.
3. Two seconds at 26 stones.
4. At 24 the median is 12 seconds, and the worst one we saw took 31 minutes.
5. At 20 stones it's over a minute.
6. And at 16 stones every probe we ran went past ten minutes.
7. Even 8 by 7 with six stones took over half an hour. That wall is why we needed learning.

## 6. The data  (18 s, starts at 2:12)

1. We used the solver to build our data: about 107 thousand 8 by 7 positions, every column labelled.
2. Plus 2,000 harder openings, 10 to 12 stones, just for testing.
3. And about 10 thousand late game 9 by 8 positions, to test transfer, solved on a 192 core cloud machine.

## 7. Eight agents  (36 s, starts at 2:30)

1. Then we built eight agents. First, Monte Carlo tree search.
2. Alpha-beta with a handcrafted evaluation, and the exact solver itself, limited to one second.
3. Then the ones that learn: alpha-beta with a learned evaluation.
4. A convolutional neural network, and that network guiding MCTS.
5. Victor Allis' hand written strategy rules.
6. And safe random: random, but it always takes a win and blocks a loss. Every agent gets one second a move.

## 8. Alpha-beta: how deep?  (36 s, starts at 3:06)

1. Alpha-beta with a handcrafted evaluation of threats and row parity. At depth 1 it matches the solver 91 percent of the time.
2. Depth 2 gets to about 94.
3. Each extra bit of depth helps a little.
4. Depth 8 reaches 97.5 percent.
5. Depth 12 barely adds anything, and it looks at 35 times as many nodes.
6. But in real games on 9 by 8 the deeper search still wins two out of three, so depth does pay off.

## 9. Learning the evaluation  (18 s, starts at 3:42)

1. Next we learned the evaluation instead of tuning it by hand. Guessing the most common result gets 56.6 percent.
2. Our hand tuned evaluation gets 67.8.
3. A logistic regression on the same features, trained on solver labels, gets 70.1, and it holds up on 9 by 8: 722 Elo against 690.

## 10. Monte Carlo tree search  (24 s, starts at 4:00)

1. MCTS needs no evaluation at all. It plays random games to the end, about 740 thousand a second on 9 by 8.
2. It grows its tree toward the moves that win most, and never misses a proven win or loss. At one second it was our strongest agent.
3. Give it more time and it keeps improving.
4. Up to 761 Elo at ten seconds a move.

## 11. A network for any board  (30 s, starts at 4:24)

1. For the neural network we went fully convolutional, so the same weights run on any board size.
2. Six layers of 64 channels, trained for 12 epochs on the CPU after our GPU setup failed.
3. Out comes a score for each column.
4. It plays the solver's move 92.4 percent of the time on 8 by 7, and still 91 percent on 9 by 8.
5. But alone, one bad move loses a game. Used to guide MCTS's search, it reaches 647 Elo.

## 12. Victor Allis' strategy  (24 s, starts at 4:54)

1. We also coded part of Victor Allis' strategy rules, with no search. On 9 by 8 they beat safe random 78 percent of the time.
2. And 83 percent on 8 by 7, so the ideas carry over.
3. Funny enough, turning off claimeven didn't hurt at all.
4. Flipping the parity hurt a little. Our set of rules is just too small for a strong game.

## 13. Does it carry over?  (24 s, starts at 5:18)

1. So does it carry over? The search agents play almost the same on 9 by 8 as on 8 by 7.
2. The time limited solver stays at 97.8.
3. The network drops only a little, from 92.4 to 91, even on a board it never saw.
4. And the rules and the baseline barely move. What they learned transfers.

## 14. The 9x8 tournament  (30 s, starts at 5:42)

1. The main result: a 9 by 8 round robin, 2,800 games, with safe random pinned at Elo 0 and the network alone just above.
2. The rules, and the time limited solver, come next.
3. Then network guided MCTS, and the handcrafted alpha-beta.
4. The learned alpha-beta at 722.
5. And at the top, MCTS at 740.

## 15. Why it came out this way  (30 s, starts at 6:12)

1. So why? Search wins on a board you can't solve. The exact solver is perfect, but only if it finishes.
2. In one second it can't finish a 9 by 8 position, so it falls back to playing near the middle.
3. Learned knowledge transfers, but without search one mistake costs you the game.
4. Put search on top of that knowledge and most of the problem goes away.
5. This is the same story on 8 by 7, measured against the solver itself.

## 16. What we learned  (12 s, starts at 6:42)

1. First: on a board you can't solve, search is what wins. Second: what agents learn on 8 by 7 carries over to 9 by 8 almost unchanged. And third: knowledge is best used to guide search, not to replace it.

## 17. Now watch them play  (12 s, starts at 6:54)

1. Alright, let's actually watch them play. I'll pick two agents and a board size. These are the real agents from our results, playing live, even on the 9 by 8 test board.

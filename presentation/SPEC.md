# Connect Four Arena 3D — Presentation App Spec

A Bevy app for the class presentation. It shows the project in 3D: pick two of
our eight agents, grow or shrink the board, and watch the agents play.

## Design requests (from the owner, in order)

These are the owner's requests, word for word. They override anything below that
disagrees, and later requests win over earlier ones.

1. "Look at the liquid glass style in quiz catch. Please apply that styling to this
   project. Also apply the light theme and fonts used in that light theme to this
   project as well. Do a complete re-design. Also, instructions and text are not
   needed all of the time and having a lot of buttons isn't needed all of the time.
   Keep that in mind."
2. "Remove this text from the bottom: All 121 catches, back to the first." (This was
   in quiz-catch's collection page, and it was removed there.)
3. "Add back the chips falling in the background in the presentation. Remove the
   explanations for the algorithms. Remove the thinking bar. Have the like selecting
   which models play against eachother always visible. Remove the words "Connect
   Four AI" from the top left, and the little icon. Remove hte 'red moves first' and
   'yellow moves second' text."
4. "Remove the 'start from' option, it shuold just always start from an empty board."
5. "Remove the step mode also"
6. "Also, always have selcting the algorithms always visible you shouldn't need to
   click for it to appear haha. Also make it left click to rotate and right click to
   pan."
7. "Then continue working on pacing out the presentation for about 6 seconds per
   click. It should always move something or have a graphic available or something,
   it shuold never like just change some text or something on click. Make it
   informative and engaging with good animations. Also remember that #ff922f is in
   the pallette as well. Test things to keep animations consistent and correct."
8. "Put all of these prompts I sent in this chat into the SPEC.md then create a loop
   to work towards this every hour incase you accidentally stop for some reason."
9. "The controls at the bottom should always be vidible as well, as a bottom bar. And
   look nice. Also remember the cool liquid glass animations and stuff"
10. "Remove the popup when someone wins under the scoreboard"
11. "Remove the elo from the side of the algorithm select buttons."
12. "Remove the word 'search' under the algorithms but above the stats'"
13. "Going back one in the presentation should only go back one like 'tick' or
    whatever. Right now I can go forward three bubbles and going back takes me back to
    0 bubbles on that slide or whatever. Have going back go as far back and going
    forward goes."
14. "Don't have the next slide button just put chips in the board or whatever I don't
    like that. Pressing r on the slides should reset us back to the start"
15. "Make the main colors of the project #ff922f and #b61b0d"
16. "No, don't change the color of the red chips. I am saying make the main accent
    #ff922f and the red hex I gave your earlier. Keep the old red chip color as it was."
17. "Be more creative with this, maybe make gradients or something to make these both
    work as accent colors throughout the application. Make it look nice. Maybe even add
    more elements with the new color as a cool accent. Maybe the first half of the
    presentation or the second half. Somethingg like this. Think of a cool way to
    incorporate it better."
18. "Remove the progress bar from the bottom of the slides"
19. "Pressing r should also reset the speaker notes timer"
20. "Okay, add an option to have like at the bottom either empty board or start from
    random beginning (like the tournament did). Also, have a settings menu to enable or
    disbable having the second red color be an accent."
21. "'stopped coult not find random opening' shouldn't happen haha. Please fix that"
22. "Also, in the bottom toolbar remove the random vertical line"
23. "Remove the settings thing and remove red as a second accent color entirely. Go back
    to how it was just #ff922f as the accent before adn remove the setting to enable the
    gradient."
24. "Also, improve the presenter notes content to make it a bit smarter and more
    presenter'ey also include a slide about each specific algorithm and a bump about
    like why its good or bad or just some info about it"
25. "Instead of elo show 'rank' like who would win the tournament. Add a 10s option,
    and run the tournament better to get real numbers. Like real tournament. Then have
    it where when the settings change, the order of the agents changes too. Show the
    'rank' of the agent on the left of the agent and when I change the settings have it
    change the order. But do a real study first to get this data for real for all of
    the different sizes. If two literally tie, just don't change where they are then I
    guess."
26. (Study size) "Do the overnight one but make it so that if I do like cargo run
    --ranking it runs it and saves it somewhere where git can remember. If I do Ctrl+C
    it shouldn't lose too much progress. This is important so that I can save the
    results. I can then run this on my desktop or something like that if I need."
27. "The user should still be able to use their device while this runs as well. So
    base it off of what they have as space on their GPU and CPU"

What they mean in practice:

- Look: quiz-catch's Liquid Glass in its light theme, Inter type, `#ff922f` as the
  one accent (with `#b4530a` when orange is text on cream). No second accent colour
  and no accent gradients (requests 15 to 17 were undone by request 23). The discs
  keep their own red (`#e04a2f`) and yellow.
- Arena: both agent pickers always visible; no brand, no algorithm explanations, no
  "moves first/second" headers, no thinking bar, no Step, no starting-position
  option (every match starts empty). Left-drag rotates the camera, right-drag pans.
- Slides: falling chips visible behind them; about 6 seconds per click; every click
  moves or reveals something visual, never only text; animations consistent,
  checked with screenshots.

## Goals

1. **Choose the agents.** Pick any of the 8 agents (same names as the paper) for Red
   (moves first) and for Yellow. Each one shows a short "how it works" line and
   our measured results (9×8 Elo at 1 s, 8×7 and 9×8 accuracy).
2. **Grow and shrink the board.** Click **−** / **+** to step through a ladder of sizes:
   `4×4 → 5×4 → 6×5 → 7×6 (standard) → 8×7 (training) → 9×8 (test)`.
   The 3D board rebuilds with a short scale animation, and the camera moves to fit.
3. **Watch a match.** Pressing Play runs the real agents from the `c4` crate in a
   background thread with the chosen time per move. Each disc drops into its
   column with gravity and a bounce. When someone wins, the four winning discs glow.

Not goals: training models, running the full tournament, network play, saving games.

## Architecture

```
presentation/           standalone Cargo package (own workspace, own lockfile)
  Cargo.toml            bevy 0.20 + c4 = { path = ".." }
  src/main.rs           App setup, plugins, resources, states
  src/roster.rs         agent metadata: names, descriptions, results from results/*.csv
  src/sizes.rs          board size ladder + type erasure over Geometry (macro)
  src/engine.rs         background worker thread that owns the agents and board
  src/scene.rs          3D: board mesh with round holes, discs, lights, table, camera
  src/ui.rs             Bevy UI panels, buttons, status text, keyboard shortcuts
```

The root `c4` crate is **not changed**. The app uses only its public API:
`tournament::Entrant` (`build::<G>()`, `name()`), `board::{Board, Grid, Cell}`,
`dataset::sample`, and `random::SplitMix`.

### Type erasure over board size

Agents are generic over `G: Geometry` (sizes fixed at compile time). A macro
instantiates one `Session<Grid<W, H, Bits>>` for each size on the ladder and puts it
behind a `trait Match` object (`legal`, `play`, `choose`, `wins`, `is_full`). Sizes
up to 64 bits use `u64`, and 9×8 uses `u128`, matching `B76`/`B87`/`B98`.

### Engine thread

`Box<dyn Agent<G>>` is not `Send`, so the agents are **built inside** the worker
thread and never leave it.

- UI → worker: `Think { per_move, count }` (play up to `count` moves, each with
  `per_move` to think) or the channel is dropped (stop). The time travels with each
  request, so it can change mid-match.
- `per_move` is the agent's deadline and also the **minimum** time a move takes: a
  fast agent waits out the rest, a slow one overruns and the overrun is not carried
  over. At 1 ms every agent effectively plays as fast as it can.
- Below 10 ms per move the UI asks for about a frame's worth of moves at once (up to
  16), so fast games are not limited to one move per frame.
- Worker → UI: `Opened { moves }`, `Moved { col, millis, verdict }`, `Failed(msg)`.
- The UI asks for a move only after the previous disc has landed and the match is
  not paused. So Pause works, and the animation never falls behind the game.
- Reset or a new match drops the channel. An old worker stops after its current
  think, which is at most the time per move. Messages carry a match id, so stale
  messages are ignored.

### Match settings

| Setting                  | Values                                        | Default                         |
| ------------------------ | --------------------------------------------- | ------------------------------- |
| Red agent / Yellow agent | the 8 roster agents                           | mcts vs alpha-beta-learned      |
| Board size               | the ladder above                              | 9×8 (test board)                |
| Time per move            | 1 ms, 10 ms, 50 ms, 0.1, 0.25, 0.5, 1, 2, 5 s | 1 s (as in the main tournament) |

Every match starts from an empty board. Agent seeds come from the clock, so a match
is different each time.

## Presentation mode

- **present (p)** in the top-right corner (or the P key) opens a 15-slide deck that walks
  through the project: the question, the scale, the solver, the data, each agent family,
  the tournament, and why the results came out as they did. Every number is from
  `results/` or `midterm_proposal.md`; the content lives in `src/slides.rs`.
- **Visuals first, words in the notes:** each slide shows only a small kicker, a
  title and the page number. Everything else is in the speaker notes
  (`SPEAKER_NOTES.md` and the presenter window). The visual is large and centred
  under the title: boards with dropping discs, growing bar charts, agent tiles, and a
  layer-stack picture of the network.
- **The talk:** title, the question, scale, the solver, the wall, the data, the eight
  agents, then **one slide per agent** (weakest first: safe random, Allis rules, the
  1 s solver, handcrafted alpha-beta, learned alpha-beta, MCTS, the network,
  network-guided MCTS), then what we found (depth, learned evaluation, MCTS and time,
  the network, Allis' rules, transfer, the tournament, why), takeaways and the demo.
  - **Agent slides** (`spotlight!` in `src/slides.rs`): the kicker says "agent n of 8"
    and its family; the visual is the same four bars for every agent, so they compare
    at a glance: 9x8 games won, 8x7 and 9x8 late-game move accuracy, and 8x7 wins
    kept against the solver. Three presses: how it works; "+" a strength; "−" a
    weakness (each as a caption, the detail in the notes).
- **Notes written to be spoken:** each beat opens with a hook or a transition, ties
  the number on screen to what it means, and the agent slides say why each one is
  good or bad.
- **About six seconds a click, and every click moves something:** 25 slides, 90
  beats, about 10 minutes (`slides::talk_seconds`: six seconds for every 26 spoken
  words). Each press of Next brings something visual: a bar grows, a bar lights up
  (orange sweeps along it and its value turns orange, `Bar::lit`), the board grows, an
  agent tile or a network layer arrives, the network's move is played, a search tree
  grows. A press never only drops chips into a board, and never only changes text:
  `every_beat_moves_something` fails if a beat brings nothing new on screen (discs
  alone do not count). `beats_are_about_six_seconds` caps each beat at 26 spoken
  words, except one folded step per slide (at most 78), where steps that used to only
  drop chips were merged.
  Dots over the page number fill in per beat. Back undoes exactly one press: only
  the pieces that press brought in fly back out (each piece remembers its beat,
  `present::Step`), anything it had taken away comes back, and a grown board shrinks
  back, while the rest of the slide stays put. From a slide's first beat Back goes to
  the previous slide's last beat.
- **The growing board** (slide 2, `Visual::Expand`): one board that physically grows
  7x6, then 8x7, then 9x8, one beat each. It is the arena's morphing board
  (`scene::spawn_shaped_board` / `morph_shaped_board`): the frame slides outward while
  the new column unfolds, then the top rises while the new row grows in. The discs
  already played slide along with their columns, and a couple of new ones drop into
  the new space once it is done. The size (`7 x 6`, ...) is written large beside it
  and swaps on each beat; the 9x8 beat's caption asks the question.
- **Boards that play out** (`Visual::Game`): the title, takeaways and demo slides show
  one centred board whose game plays itself out as the slide opens, a disc about every
  0.45 s. They are one press each. On the takeaways slide red's last disc completes
  four in a row.
- **The solver slide:** the board, then its live W/D/L labels, then the winning move
  dropped into the board, then two green bars filling to 100% (UCI and Pons checks).
- **The network slide:** the board, the six layers, the column scores, then its
  top-scoring move dropped into the board, then a small search tree growing over the
  scores with its best line in orange (the network guiding MCTS).
- **3D visuals, 2D text:** all words (titles, bar labels, values, captions, the W/D/L
  labels) are flat 2D text pinned to points on the stage. They follow the camera's
  drift, always face the screen, and are sized from the window height so they stay
  sharp and readable. They fade and slide in with the slide's pieces.
- **Live solve:** the solver slide's 7x6 position is solved by the project's own solver in a
  background thread at startup (about 70 ms), and its W/D/L column labels are shown.
- **Keys:** Right / Space / click next beat, Left back a beat, R or Home back to the
  first slide (R also restarts the presenter window's talk and slide clocks), Esc back to the
  arena (also after the last slide). The slides sit on a stage far from the arena table,
  so the arena's match keeps its state.
- **Transitions:** opening the deck flies the camera from the arena table to the stage.
  On every slide change the old pieces fly off the way you are going and the new ones
  sweep in from the other side, staggered; the camera settles at a slightly different
  angle for each slide and drifts gently. Each change also gives a quick camera
  push-in and a flash of the stage light, so every click has some impact.
- **Always moving** (`src/backdrop.rs`): red and yellow chips fall far behind
  everything, self-lit and three-quarters opaque so they read on the cream stage.
  The see-through backdrop board that used to play behind the slides is hidden, so
  nothing sits behind the centred visuals.
- Dev aid: `C4_SLIDE=n` with `C4_SCREENSHOT` opens slide n for a screenshot; `a` in `C4_KEYS` returns to the arena; `>` in
  `C4_KEYS` goes one beat on (`>>>` three beats), `C4_KEYS_AT=secs` delays the keys,
  `C4_NAV=>>><` presses Next (`>`) and Back (`<`) one at a time, 0.6 s apart, from
  `C4_KEYS_AT`,
  `C4_AFTER_WIN=secs` shoots that long after the first game ends, `C4_YELLOW=n`
  picks Yellow's agent.

### Presenter window and speaker notes

- Every beat has a line to say (`src/slides.rs`); each slide's budget is six seconds
  a beat. `SPEAKER_NOTES.md` is generated from the slides: after changing one, run
  `C4_WRITE_NOTES=1 cargo test speaker_notes` (the `speaker_notes_are_current` test
  fails while it is stale).
- **N** during the slides opens a second window (drag it to your own screen): this
  beat's lines, which step of the slide you are on, the talk clock (red past the planned length),
  time on this slide against its budget, whether you are ahead or behind plan, and
  what the next press brings (the next beat's lines, or the next slide's title). N again or
  closing the window puts it away. The projector only ever shows the slides.
- Notes are written to be read aloud, in plain spoken language, with no em dashes.
- Dev aid: `C4_NOTES_SHOT=notes.png` (with `C4_SCREENSHOT`) opens the window by itself
  and saves a screenshot of it too.

## Robustness and performance

Measured on the presentation laptop (Intel Iris Xe, 2400 x 1350 at 150% scaling):

- Bevy 0.20's GPU light clustering made the Intel driver lose the device (a crash)
  when games restart many times a second; it is turned off (CPU clustering is free
  with our few lights).
- **Memory leaks found and fixed** (they caused stalls, then a lost device, after a
  minute or two, even idle):
  - Bevy's `system_font_discovery` leaked on every 2D text change (hundreds of MB a
    second in fast games). It is off; the UI's Inter fonts (`fonts/`, four static
    weights, OFL) are built into the binary and added as font assets at startup instead.
  - Adding a mesh to an entity after it was spawned, and swapping materials on live
    entities, also grew render memory. Pooled discs are spawned with their mesh and material, keep one colour per pool
    slot (slot k is always move k), and the win glow is a halo spawned over each disc.
  - Unused pool discs are parked below the table instead of hidden.
  - After the fixes: 9 boards at 1 ms for 90 s holds about 1.3 GB at about 40 fps with
    no stalls; normal play holds about 1.6 GB.
- MSAA is off with FXAA instead; shadows use 2 cascades.
- Systems that need the window size ask for the primary window, so opening the
  presenter window cannot stall the slides or the camera.
- With more than 2 boards, or below 0.1 s per move, discs use a plain 20-sided mesh
  instead of the ridged one (hundreds of full-detail discs are too much for the GPU).
- `C4_FPS=1` logs the frame rate.
- More than 9 boards: no win beams, lights or confetti (the glow still marks wins),
  and the screen-edge flash only fires at animated speeds, at most every 0.8 s.

## 3D scene

- **Board:** a custom mesh, a blue plate with a round hole in every cell (front face,
  back face, hole walls, outer rim), plus two feet. Built again on every resize.
- **Discs:** cylinders turned to face the camera. Red = first player, yellow = second.
  Animated fall (gravity, one damped bounce), sped up to 4x at short move times;
  below 0.1 s per move discs appear in place with no drop.
- **"Thinking" disc:** the next player's disc hovers and bobs above the board.
- **Win** (`src/effects.rs`): the 4+ winning discs glow and pulse, pop forward, a
  glowing beam draws itself through them, confetti bursts out in the winner's colours
  and a coloured light flashes. A banner names the winner and their score number
  bumps. The effects (not the glow) are skipped below 0.1 s per move.
- **Quality:** pieces are lathed meshes like the real ones (raised outer rim, a
  recessed face with two concentric ridges, rounded edges) in glossy clear-coated
  plastic; the board's holes are bevelled on both faces; the camera has soft bloom so
  glows, the win beam and flashes shine. Used in the arena and on the slides.
- **Landing:** each disc gives the board a small rock back when it lands.
- **Resize morph:** arena boards are one tile per cell plus a stretchable frame. On a
  resize the frame slides outward while new columns unfold, then the top rises while
  new rows grow in (shrinking runs height first, then width), about 1.3 s. Agents wait
  for the morph before moving; with several boards each one glides to its new spot.
- **Camera:** left-drag orbits, right-drag pans (the view follows the pointer), the
  scroll wheel zooms; a drag that starts on the UI never moves the camera. Also a slow idle sway, and
  automatic framing when the board size changes. Directional light with shadows,
  ambient light, a cream table under a cream wallpaper (the clear colour is drawn
  brighter than the palette so it survives tonemapping).

## Arena UI (2D)

- **Liquid Glass, light theme** (`src/theme.rs`, `src/ui.rs`), after quiz-catch's
  `glass.css`: every control is a drop of tinted glass floating over the scene, with a
  one-pixel rim of light (brightest top left and bottom right, a `BorderGradient`), a
  sheen of light under its top edge, a hairline and a soft warm shadow. Capsules for
  buttons, concentric rounded panes (24 to 28 px) for everything else. Colours are
  quiz-catch's light palette: cream wallpaper `#f7f3e8`, surface `#fffdf7`, ink
  `#282820`, muted `#5f5b49`, faint `#8f8a75`, orange `#ff922f` (and `#b4530a` when
  orange is text on cream). Type is Inter (what quiz-catch shows where SF is missing) on the
  system scale, tracked tighter as it grows.
- **Only what matters now is on screen.** Always there: the score capsule naming
  both agents, `? / Slides`, an agent picker down each side, and the control bar
  along the bottom.
  - **Control bar** (always visible): Play, Reset, then Start (Empty or Random, one
    click switches it, `O`), then a stepper each for Board, Per move (1 ms to 10 s) and Boards
    (minus, a small caption over the value, plus, in a faint capsule).
  - **Start:** Empty starts every match from an empty board. Random starts it from a
    random opening of 4 to 8 stones (at most a quarter of the board; no immediate
    wins, `c4::dataset::sample`, drawn again when a draw runs into a dead end), like the tournament's random openings. Switching it
    restarts the match (the score is kept). From an empty board the handcrafted
    alpha-beta usually beats MCTS; from random openings MCTS wins about half, as in the
    tournament.
  - **Agent pickers** (always visible): the eight agents as rows, **in the order they
    would finish a tournament at the current settings** (board, time per move, start),
    each with its place on the left. Changing a setting re-ranks them and the rows
    spring to their new places (agents with equal scores keep their rows); the pill
    follows the chosen agent. Under them three tiles: Rank (`#1`), 8x7 and 9x8
    accuracy. No family label, no explanations of the algorithms, no "moves
    first/second" header.
  - **The ranking study** (`src/study.rs`, `cargo ranking`): a full round robin of all
    eight agents, every pair once with each colour, for each of the 6 boards, each of
    the 10 times per move (1 ms to 10 s) and both starts. Rounds: 20 up to 0.25 s, 6 up
    to 2 s, 3 at 5 s, 2 at 10 s (about 14 hours on the laptop, overnight). Every game
    is appended to `ranking/games.csv` (tracked by git) and flushed, so Ctrl+C loses
    only the games in play and the next run resumes. It runs as its own program
    (`ranking.exe`) so the app can be rebuilt meanwhile; `cargo run --release --
    --ranking` works too. It runs at below-normal priority (`C4_STUDY_PRIORITY=normal`
    to override), so the computer stays usable and the study takes only spare CPU; it
    never uses the GPU (every agent, the network too, runs on the CPU). `C4_WORKERS`
    sets the threads (default: logical cores minus two), `C4_STUDY_MS=1,10` limits
    the times (to split the study between machines). The random 9x8 cells at 0.5, 1,
    5 and 10 s hold the original tournament's 2,800 games each
    (`--import-tournament`), so the study skips them. The app reads the CSV when it
    is built (`src/ranking.rs`): place = tournament score (1 a win, 1/2 a draw); a
    setting with no games yet borrows the nearest time that has some.
  - **Shortcut list** (`?` or `H`), the only pane on demand. It springs in over the
    bar; starting a series, or `Esc`, puts it away. No brand, status line, move list,
  thinking bar, Step button or starting-position option: the hovering disc shows who
  is thinking, and a win or draw shows on the board and in the score (no result
  popup); only an error gets a toast under the score. The games/draws count appears under the score once there is one.
- **Fits any window:** the layout is drawn for 1600 x 920 and `ui::fit_ui` sets
  `UiScale` to fit the actual window. Slide text divides by the same scale so it
  keeps its size. F11 toggles borderless fullscreen. `C4_WINDOW=1536x816` starts at
  another size, for screenshots.
- **Shared readout** (`src/readout.rs`): every string the UI shows is computed once a
  frame from the game state.
- **The boards are never behind the fixed glass:** the UI measures its top bar,
  pickers and control bar every frame and hands the sizes to the camera
  (`scene::UiMargins`). The shortcut list floats over the scene and does not move the
  camera.
- **Colour:** the scene camera has no tonemapping (so the cream and `#ff922f` come out
  exactly) and an exposure one stop under Bevy's default (so lit surfaces stay below
  white).
- **Animations:** the fixed glass slides in whenever the arena opens; buttons ease
  their fills, spring (overshooting a little) on hover, press in, flash and send out a
  ripple when clicked; panes spring open and close quickly; the picker's pill stretches
  as it flows; changed values roll up into place and pop; a score that goes up bumps
  and floats a "+1"; a win flashes the window edge in the winner's colour.
  - **The lens:** a glass drop flows under whichever bar control the pointer is on.
    Its two edges spring separately, the leading one stiffer, so it stretches toward
    where it is going and catches up behind; it squashes a little in flight and fades
    when the pointer leaves the bar.
  - **Light:** every pane of glass gathers a soft white glow under the pointer, which
    follows it and fades when the pointer leaves.
- Dev aids: `C4_OPEN=help` opens the shortcut list at startup; `C4_HOVER=x,y;x,y`
  moves the pointer (window pixels) to each point a quarter second apart from
  `C4_KEYS_AT`, for screenshots of the hover animations.

## UI layout

```
                        ( ● MCTS (UCT)   2 : 1   Alpha-beta (learned) ● )   ( ?  Slides )
 .-----------------.                3 games · 1 draw                  .-----------------.
 | MCTS (UCT)      | <- pill                                           | MCTS (UCT)      |
 | Alpha-beta      |                                                  | Alpha-beta      |
 | ...             |                 3D board(s)                      | ...             |
 | [740][98.7][99] |        .------------------------------.          | [722][98.9][99] |
 '-----------------'                                                  '-----------------'
                                START    BOARD          PER MOVE       BOARDS
              ( [ Play ]  Reset  Empty  ( -  9 x 8  + ) ( -  1 s  + ) ( -  1  + ) )  <- control bar
```

### Series and score

- **Play** starts a series: when a match ends the winner's score goes up (or draws),
  the result shows briefly (6x the move time, 0.1 s to 1.5 s), and the next match
  starts by itself. Space on the result screen stops the series there.
- The score (Red wins : Yellow wins) is the middle of the score capsule; games and
  draws show faintly under it once a game has been played.
- Every control works mid-match:
  - Picking a different agent **resets the score** and restarts the match.
  - Changing the board size restarts the match on the new board (score kept).
  - Time per move applies from the next move.
- Reset stops play, clears the board and the score.

### Several boards at once

- **Boards at once - / +** (or `,` / `.`) steps through 1 to 9, then 12, 16, 20, 25,
  30, 36, 42, 49, 64, 81, 100, 121, 144, 196 and 256 matches between the same two
  agents. Up to 9 they stand in rows going back (one row up to 3, then 2 x 2, 3 x 2,
  3 x 3); past 9 they stack into a flat wall facing the camera, a little wider than
  tall. Each board is a table
  entity with its own `Game` and engine thread; Play, Pause and Reset apply to
  all of them, and every result goes into the one score.
- With several rows going back the camera tilts down so the back rows show over the
  front ones; a wall is seen straight on. Every board runs its own engine thread, so
  hundreds of boards share the CPU and each agent gets less thinking per move.
- The hovering "thinking" disc is shown only with a single board.

### Keyboard shortcuts (for presenting)

| Key               | Action                    |
| ----------------- | ------------------------- |
| Space             | Play / Pause              |
| R                 | Reset                     |
| `+` / `-`         | Grow / shrink the board   |
| `[` / `]`         | Less / more time per move |
| `,` / `.`         | Fewer / more boards       |
| O                 | Empty or random start     |
| P                 | Slides                    |
| H or ?            | Shortcut list             |
| Esc               | Put the panes away        |
| F11               | Full screen               |
| Left-drag         | Orbit                     |
| Right-drag        | Pan                       |
| Wheel             | Zoom                      |

## Results shown on agent cards

From `results/accuracy.csv` and `results/measurements.csv` (1 s per move):

| Agent                  | 9×8 Elo    | 8×7 held-out acc. | 9×8 late acc. |
| ---------------------- | ---------- | ----------------- | ------------- |
| mcts                   | 740        | 98.7%             | 99.1%         |
| alpha-beta-learned     | 722        | 98.9%             | 99.3%         |
| alpha-beta-handcrafted | 690        | 98.8%             | 99.3%         |
| network-mcts           | 647        | 98.2%             | 98.5%         |
| allis                  | 160        | 88.3%             | 89.7%         |
| perfect                | 78         | 97.8%             | 97.8%         |
| network                | 37         | 92.4%             | 91.0%         |
| safe-random            | 0 (anchor) | 76.4%             | 77.8%         |

## Build and run

```
cd presentation
cargo run --release
```

Dev builds use `opt-level = 3` for dependencies (including `c4`), so the agents are
not slowed down in debug builds.

## Acceptance checks

- `cargo build --release` and `cargo clippy` succeed with no warnings.
- The rules glue has unit tests: size ladder ↔ geometry, win-line detection, and a
  headless match that runs safe-random vs safe-random to the end on every ladder size
  (legal moves only, and the result agrees with `Board::wins`).
- Manual check: every agent pair can start on 9×8; resizing rebuilds the board; and
  Pause and Reset work while a worker is thinking.

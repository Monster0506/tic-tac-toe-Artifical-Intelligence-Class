//! Who would win the tournament at the arena's current settings: the agents' order and
//! places, from the ranking study's games (`src/study.rs`, read when the app is built).
//!
//! An agent's strength at a setting is its tournament score rate (1 a win, 1/2 a draw,
//! over games played) there, steadied by the neighbouring times per move on the same
//! board and start: their rates count as `PRIOR_GAMES` extra games, each time step
//! away counting `NEIGHBOUR_DECAY` as much as the one before. The slow settings have
//! only two or three rounds (28 to 42 games an agent), so their raw order is mostly
//! noise; checked on held-out rounds of the study, this predicts the order of unseen
//! games better at every speed (Kendall tau 0.75 -> 0.79 at 2 to 10 s, 0.856 -> 0.864
//! at the fast times) without blurring the well-played settings. Other board sizes are
//! not borrowed from: that made the predictions worse, the agents really do change
//! order with the board. A setting with no games of its own uses its neighbours alone.

use bevy::prelude::*;

use crate::game::{Settings, TIMES};
use crate::study::{Scores, scores};

/// The arena's agents in tournament order for the current settings.
#[derive(Resource, Clone, PartialEq, Eq)]
pub struct Ranking {
    /// The row each agent stands in (0 = top).
    pub slot: [usize; 8],
    /// Each agent's place (1 = would win the tournament); tied scores share a place.
    /// `None` until the study has games for this board and start.
    pub place: [Option<usize>; 8],
}

impl Default for Ranking {
    fn default() -> Self {
        Self {
            slot: [0, 1, 2, 3, 4, 5, 6, 7],
            place: [None; 8],
        }
    }
}

/// Re-ranks when the board, the time or the start changes. Agents with the same score
/// keep the order they were in.
pub fn update_ranking(settings: Res<Settings>, mut ranking: ResMut<Ranking>) {
    if !settings.is_changed() {
        return;
    }
    let next = ranked(&settings, &ranking);
    if next != *ranking {
        *ranking = next;
    }
}

/// How many games the neighbouring times' rate counts for.
const PRIOR_GAMES: f64 = 40.0;
/// Each time step further away counts this much as the one before.
const NEIGHBOUR_DECAY: f64 = 0.5;

/// Each agent's strength (expected score, 0 to 1) at this start, board and time;
/// `None` when the study has no games for the board and start.
pub fn strengths(scores: &Scores, random: bool, size: usize, time: usize) -> Option<[f64; 8]> {
    let mut strength = [0.0; 8];
    for (agent, slot) in strength.iter_mut().enumerate() {
        *slot = strength_of(scores, (random, size), time, agent)?;
    }
    Some(strength)
}

/// One agent's score rate in one setting, if it played there.
fn rate(scores: &Scores, key: (bool, usize, usize), agent: usize) -> Option<(f64, f64)> {
    let tally = scores.get(&key).filter(|tally| tally.games[agent] > 0)?;
    let games = f64::from(tally.games[agent]);
    Some((f64::from(tally.points[agent]) / 2.0 / games, games))
}

/// `agent`'s own rate at `time`, steadied by the other times on the same board and start.
fn strength_of(scores: &Scores, (random, size): (bool, usize), time: usize, agent: usize) -> Option<f64> {
    let (mut sum, mut weights) = (0.0, 0.0);
    for other in (0..TIMES.len()).filter(|&other| other != time) {
        if let Some((other_rate, _)) = rate(scores, (random, size, other), agent) {
            #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap, reason = "ten times")]
            let weight = NEIGHBOUR_DECAY.powi(other.abs_diff(time) as i32);
            sum += weight * other_rate;
            weights += weight;
        }
    }
    let prior = (weights > 0.0).then(|| sum / weights);
    match (rate(scores, (random, size, time), agent), prior) {
        (Some((own, games)), Some(prior)) => Some((own * games + PRIOR_GAMES * prior) / (games + PRIOR_GAMES)),
        (Some((own, _)), None) => Some(own),
        (None, prior) => prior,
    }
}

/// Strengths this close count as a tie.
const TIE: f64 = 1e-9;

/// Higher strength first; within `TIE` counts as equal.
fn stronger_first(a: f64, b: f64) -> std::cmp::Ordering {
    if (a - b).abs() < TIE { std::cmp::Ordering::Equal } else { b.total_cmp(&a) }
}

/// The ranking for `settings`, starting from the order in `ranking`.
fn ranked(settings: &Settings, ranking: &Ranking) -> Ranking {
    let mut next = ranking.clone();
    match strengths(scores(), settings.random_opening, settings.size, settings.time) {
        Some(strength) => {
            let mut order: Vec<usize> = (0..8).collect();
            order.sort_by_key(|&agent| ranking.slot[agent]);
            // Stable: tied agents keep their current rows.
            order.sort_by(|&a, &b| stronger_first(strength[a], strength[b]));
            for (slot, &agent) in order.iter().enumerate() {
                next.slot[agent] = slot;
                next.place[agent] = Some(1 + strength.iter().filter(|&&other| other > strength[agent] + TIE).count());
            }
        }
        None => next.place = [None; 8],
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roster::ROSTER;

    /// The original tournament's games give its published order: 9x8, random
    /// openings, 1 s a move.
    #[test]
    fn imported_tournament_ranks_as_published() {
        let settings = Settings {
            size: 5,
            time: 6,
            random_opening: true,
            ..Settings::default()
        };
        let ranking = ranked(&settings, &Ranking::default());
        let order: Vec<&str> = (0..8)
            .map(|slot| ROSTER[ranking.slot.iter().position(|&at| at == slot).unwrap_or(0)].entrant.name())
            .collect();
        assert_eq!(
            order,
            ["mcts", "alpha-beta-learned", "alpha-beta-handcrafted", "network-mcts", "allis", "perfect", "network", "safe-random"]
        );
        assert_eq!(ranking.place[0], Some(1));
    }

    /// Equal scores keep the rows they were in.
    #[test]
    fn ties_keep_their_rows() {
        let settings = Settings {
            size: 5,
            time: 6,
            random_opening: true,
            ..Settings::default()
        };
        let once = ranked(&settings, &Ranking::default());
        assert!(ranked(&settings, &once) == once);
    }

    fn tally(rate: [u32; 8], games: u32) -> crate::study::Tally {
        // `rate` in percent, as half-points over `games` games.
        crate::study::Tally { points: rate.map(|percent| percent * games * 2 / 100), games: [games; 8] }
    }

    /// A thinly played time is pulled toward its neighbours, a well played one hardly.
    #[test]
    fn neighbours_steady_thin_settings() {
        let mut scores = Scores::new();
        let even = [50; 8];
        let mut odd = even;
        odd[0] = 100;
        scores.insert((true, 0, 0), tally(even, 1000));
        scores.insert((true, 0, 1), tally(odd, 20));
        scores.insert((true, 0, 2), tally(odd, 10_000));
        let thin = strengths(&scores, true, 0, 1).unwrap_or_default()[0];
        // Own games say 1.0, the neighbours (equal weight) say 0.75.
        assert!((thin - (20.0 + 40.0 * 0.75) / 60.0).abs() < 1e-9, "{thin}");
        let thick = strengths(&scores, true, 0, 2).unwrap_or_default()[0];
        assert!(thick > 0.99, "{thick}");
        // No games of its own: the neighbours alone, nearest first.
        scores.remove(&(true, 0, 1));
        let borrowed = strengths(&scores, true, 0, 1).unwrap_or_default()[0];
        assert!((borrowed - 0.75).abs() < 1e-9, "{borrowed}");
        assert!(strengths(&scores, false, 0, 1).is_none());
    }

    /// Every setting of the study has a full ranking.
    #[test]
    fn study_covers_every_setting() {
        let sizes = crate::sizes::LADDER.len();
        for cell in 0..2 * sizes * TIMES.len() {
            let key = (cell % 2 == 1, cell / 2 % sizes, cell / 2 / sizes);
            assert!(scores().contains_key(&key), "{key:?}");
        }
    }
}

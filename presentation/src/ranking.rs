//! Who would win the tournament at the arena's current settings: the agents' order and
//! places, from the ranking study's games (`src/study.rs`, read when the app is built).
//! For the chosen board, time and start, each agent's tournament score (1 a win, 1/2 a
//! draw) gives its place; with no games at that time, the nearest time with some is
//! used.

use bevy::prelude::*;

use crate::game::{Settings, TIMES};
use crate::study::scores;

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

/// The ranking for `settings`, starting from the order in `ranking`.
fn ranked(settings: &Settings, ranking: &Ranking) -> Ranking {
    let scores = scores();
    let wanted = settings.random_opening;
    // This time, or the nearest one with games.
    let cell = (0..TIMES.len())
        .filter(|&time| scores.contains_key(&(wanted, settings.size, time)))
        .min_by_key(|&time| time.abs_diff(settings.time))
        .and_then(|time| scores.get(&(wanted, settings.size, time)));
    let mut next = ranking.clone();
    match cell {
        Some(points) => {
            let mut order: Vec<usize> = (0..8).collect();
            order.sort_by_key(|&agent| ranking.slot[agent]);
            // Stable: equal scores keep their current rows.
            order.sort_by(|&a, &b| points[b].cmp(&points[a]));
            for (slot, &agent) in order.iter().enumerate() {
                next.slot[agent] = slot;
                next.place[agent] = Some(1 + points.iter().filter(|&&other| other > points[agent]).count());
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
}


use std::collections::HashSet;

use c4::board::{B76, B87, B98, Board, Geometry, Grid};

// Positions reachable in play, from John Tromp (tromp.github.io/c4/c4.html).
const POSITIONS_4X4: usize = 161_029;
const POSITIONS_4X5: usize = 1_706_255;
const POSITIONS_5X4: usize = 3_945_711;

fn count<G: Geometry>() -> usize {
    let mut seen = HashSet::new();
    visit(Board::<G>::default(), &mut seen);
    seen.len()
}

fn visit<G: Geometry>(board: Board<G>, seen: &mut HashSet<G::Bits>) {
    if !seen.insert(board.key()) {
        return;
    }

    for col in 0..G::COLUMNS {
        if !board.can_play(col) {
            continue;
        }

        let mut child = board;
        child.play(col);
        if board.wins(col) {
            seen.insert(child.key());
            continue;
        }
        visit(child, seen);
    }
}

#[test]
fn narrow_counts_match_tromp() {
    assert_eq!(count::<Grid<4, 4, u64>>(), POSITIONS_4X4);
    assert_eq!(count::<Grid<4, 5, u64>>(), POSITIONS_4X5);
    assert_eq!(count::<Grid<5, 4, u64>>(), POSITIONS_5X4);
}

#[test]
fn wide_counts_match_tromp() {
    assert_eq!(count::<Grid<4, 4, u128>>(), POSITIONS_4X4);
    assert_eq!(count::<Grid<4, 5, u128>>(), POSITIONS_4X5);
    assert_eq!(count::<Grid<5, 4, u128>>(), POSITIONS_5X4);
}

fn fill_last_column<G: Geometry>() {
    let last = G::COLUMNS - 1;
    let mut board = Board::<G>::default();
    let mut keys = vec![board.key()];

    for _ in 0..G::ROWS {
        assert!(board.can_play(last));
        assert!(!board.wins(last));
        board.play(last);
        keys.push(board.key());
    }

    assert!(!board.can_play(last));
    assert!((0..last).all(|col| board.can_play(col)));
    assert_eq!(keys.iter().collect::<HashSet<_>>().len(), keys.len());
}

#[test]
fn last_column_fills() {
    fill_last_column::<B76>();
    fill_last_column::<B87>();
    fill_last_column::<B98>();
}

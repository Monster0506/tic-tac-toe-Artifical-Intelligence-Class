use std::collections::HashSet;

use c4::board::{B76, B87, B98, Board, Geometry, Grid};

/// Positions reachable in play, from John Tromp (tromp.github.io/c4/c4.html).
const POSITIONS_4X4: usize = 161_029;
const POSITIONS_4X5: usize = 1_706_255;
const POSITIONS_5X4: usize = 3_945_711;

fn count<G: Geometry>() -> usize {
    let mut seen = HashSet::new();
    visit(Board::<G>::new(), &mut seen);
    seen.len()
}

fn visit<G: Geometry>(board: Board<G>, seen: &mut HashSet<G::Bits>) {
    if !seen.insert(board.key()) {
        return;
    }

    for col in 0..G::W {
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
    let last = G::W - 1;
    let mut board = Board::<G>::new();
    let mut keys = HashSet::from([board.key()]);

    for _ in 0..G::H {
        assert!(board.can_play(last));
        assert!(!board.wins(last));
        board.play(last);
        keys.insert(board.key());
    }

    assert!(!board.can_play(last));
    assert!((0..last).all(|col| board.can_play(col)));
    assert_eq!(keys.len(), G::H as usize + 1);
}

#[test]
fn last_column_fills_on_spec_boards() {
    fill_last_column::<B76>();
    fill_last_column::<B87>();
    fill_last_column::<B98>();
}

use c4::board::{B76, Board, Geometry};

// Pascal Pons, end-game test set Test_L3_R1: http://blog.gamesolver.org/data/Test_L3_R1
const PONS: &str = include_str!("data/Test_L3_R1");
const PONS_POSITIONS: usize = 1000;

fn replay<G: Geometry>(moves: &str) -> Option<Board<G>> {
    let mut board = Board::<G>::default();
    for ch in moves.chars() {
        let col = ch.to_digit(10)?.checked_sub(1)?;
        if col >= G::COLUMNS || !board.can_play(col) || board.wins(col) {
            return None;
        }
        board.play(col);
    }
    Some(board)
}

#[test]
fn pons_positions_replay_legally() {
    assert_eq!(PONS.lines().count(), PONS_POSITIONS);
    for line in PONS.lines() {
        let board = line
            .split_once(' ')
            .and_then(|(moves, _)| replay::<B76>(moves));
        assert!(board.is_some(), "{line}");
    }
}

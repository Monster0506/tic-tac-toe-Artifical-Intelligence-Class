use std::collections::HashSet;

pub type Cells = HashSet<(i64, i64)>;

#[must_use]
pub fn grid_wins(own: &Cells, (column, row): (i64, i64)) -> bool {
    let run = |(dcol, drow): (i64, i64)| {
        (1..4)
            .take_while(|&step| own.contains(&(column + step * dcol, row + step * drow)))
            .count()
    };
    [(1, 0), (0, 1), (1, 1), (1, -1)]
        .into_iter()
        .any(|(dcol, drow)| 1 + run((dcol, drow)) + run((-dcol, -drow)) >= 4)
}

use crate::board::Geometry;
use crate::dataset::Labelled;
use crate::outcome::Outcome;
use crate::search::{Features, Weights};

const MAX_ITERATIONS: u32 = 50;
const STEP_TOLERANCE: f64 = 1e-9;

pub struct Row<const K: usize> {
    pub inputs: [f64; K],
    pub outcome: Outcome,
}

pub struct Model<const K: usize> {
    pub weights: [f64; K],
}

impl<const K: usize> Model<K> {
    fn logit(&self, inputs: [f64; K]) -> f64 {
        dot(self.weights, inputs)
    }
}

#[must_use]
pub const fn constant_row<G: Geometry>(sample: &Labelled<G>) -> Row<1> {
    Row {
        inputs: [1.0],
        outcome: sample.result,
    }
}

#[must_use]
pub fn handcrafted_row<G: Geometry>(sample: &Labelled<G>) -> Row<2> {
    let score = dot(Weights::DEFAULT.0.map(f64::from), features(sample));
    Row {
        inputs: [score, 1.0],
        outcome: sample.result,
    }
}

#[must_use]
pub fn learned_row<G: Geometry>(sample: &Labelled<G>) -> Row<5> {
    let [mover_good, mover_other, opponent_good, opponent_other] = features(sample);
    Row {
        inputs: [mover_good, mover_other, opponent_good, opponent_other, 1.0],
        outcome: sample.result,
    }
}

fn features<G: Geometry>(sample: &Labelled<G>) -> [f64; 4] {
    Features::of(sample.board).0.map(f64::from)
}

fn dot<const K: usize>(left: [f64; K], right: [f64; K]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(weight, input)| weight * input)
        .sum()
}

struct Derivatives<const K: usize> {
    gradient: [f64; K],
    hessian: [[f64; K]; K],
}

#[must_use]
pub fn fit<const K: usize>(rows: &[Row<K>]) -> Option<Model<K>> {
    let mut weights = [0.0; K];
    for _ in 0..MAX_ITERATIONS {
        let totals = derivatives(rows, weights);
        let step = solve(totals.hessian, totals.gradient)?;
        for (weight, delta) in weights.iter_mut().zip(step) {
            *weight += delta;
        }
        if step.iter().all(|delta| delta.abs() < STEP_TOLERANCE) {
            return Some(Model { weights });
        }
    }
    None
}

fn derivatives<const K: usize>(rows: &[Row<K>], weights: [f64; K]) -> Derivatives<K> {
    let mut totals = Derivatives {
        gradient: [0.0; K],
        hessian: [[0.0; K]; K],
    };
    for row in rows {
        let probability = sigmoid(dot(weights, row.inputs));
        let residual = target(row.outcome) - probability;
        let curvature = probability * (1.0 - probability);
        accumulate(&mut totals, row.inputs, residual, curvature);
    }
    totals
}

const fn target(outcome: Outcome) -> f64 {
    match outcome {
        Outcome::Win => 1.0,
        Outcome::Draw => 0.5,
        Outcome::Loss => 0.0,
    }
}

fn accumulate<const K: usize>(
    totals: &mut Derivatives<K>,
    inputs: [f64; K],
    residual: f64,
    curvature: f64,
) {
    for (slot, input) in totals.gradient.iter_mut().zip(inputs) {
        *slot += residual * input;
    }
    for (hessian_row, outer) in totals.hessian.iter_mut().zip(inputs) {
        for (cell, inner) in hessian_row.iter_mut().zip(inputs) {
            *cell += curvature * outer * inner;
        }
    }
}

fn sigmoid(logit: f64) -> f64 {
    1.0 / (1.0 + (-logit).exp())
}

fn solve<const K: usize>(mut matrix: [[f64; K]; K], mut rhs: [f64; K]) -> Option<[f64; K]> {
    for column in 0..K {
        let pivot = (column..K).max_by(|&left, &right| {
            matrix[left][column]
                .abs()
                .total_cmp(&matrix[right][column].abs())
        })?;
        matrix.swap(column, pivot);
        rhs.swap(column, pivot);
        if !matrix[column][column].is_normal() {
            return None;
        }
        eliminate(&mut matrix, &mut rhs, column);
    }
    let mut solution = [0.0; K];
    for row in (0..K).rev() {
        let tail: f64 = (row + 1..K)
            .map(|inner| matrix[row][inner] * solution[inner])
            .sum();
        solution[row] = (rhs[row] - tail) / matrix[row][row];
    }
    Some(solution)
}

fn eliminate<const K: usize>(matrix: &mut [[f64; K]; K], rhs: &mut [f64; K], column: usize) {
    let pivot_row = matrix[column];
    for row in column + 1..K {
        let factor = matrix[row][column] / pivot_row[column];
        for (cell, pivot_cell) in matrix[row].iter_mut().zip(pivot_row).skip(column) {
            *cell -= factor * pivot_cell;
        }
        rhs[row] -= factor * rhs[column];
    }
}

pub struct Metrics {
    pub log_loss: f64,
    pub accuracy: f64,
}

#[must_use]
pub fn evaluate<const K: usize>(model: &Model<K>, rows: &[Row<K>]) -> Metrics {
    let mut loss_total = 0.0;
    let mut correct = 0.0;
    let mut count = 0.0;
    for row in rows {
        let logit = model.logit(row.inputs);
        loss_total += log_loss(logit, target(row.outcome));
        if predicted_correctly(logit, row.outcome) {
            correct += 1.0;
        }
        count += 1.0;
    }
    Metrics {
        log_loss: loss_total / count,
        accuracy: correct / count,
    }
}

fn log_loss(logit: f64, expected: f64) -> f64 {
    expected.mul_add(-logit, softplus(logit))
}

fn softplus(value: f64) -> f64 {
    value.max(0.0) + (-value.abs()).exp().ln_1p()
}

fn predicted_correctly(logit: f64, outcome: Outcome) -> bool {
    match outcome {
        Outcome::Win => logit > 0.0,
        Outcome::Draw | Outcome::Loss => logit <= 0.0,
    }
}

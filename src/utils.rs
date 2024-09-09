//! Utilities that are useful in many places.
use crate::point::Point2D;
use itertools::Itertools;
use num_traits::{Float, NumCast, One, ToPrimitive};

/// Generates a 2D grid of n equally spaced points
/// that fits nicely within the given bounds.
pub fn spread_grid<P>(
    n: usize,
    width: P::Value,
    height: P::Value,
    reserve_boundary: P::Value,
) -> impl Iterator<Item = P>
where
    P: Point2D,
{
    let aspect_ratio = (width / height).to_f64().unwrap();
    let mut best_rows = 1;
    let mut best_cols = n;
    let mut min_diff = f64::infinity();
    for rows in 1..=n {
        let cols = (n as f64 / rows as f64).ceil() as usize;
        let actual_ratio = cols as f64 / rows as f64;
        let diff = (actual_ratio - aspect_ratio).abs();
        if diff < min_diff {
            min_diff = diff;
            best_rows = rows;
            best_cols = cols;
        }
    }
    (0..best_rows)
        .cartesian_product(0..best_cols)
        .map(move |(i, j)| {
            let x = <P::Value as NumCast>::from(j).unwrap() * width * reserve_boundary
                / <P::Value as NumCast>::from(best_cols).unwrap()
                + (width * (P::Value::one() - reserve_boundary)
                    / (P::Value::one() + P::Value::one()))
                + width * reserve_boundary
                    / <P::Value as NumCast>::from(best_cols).unwrap()
                    / (P::Value::one() + P::Value::one());
            let y = <P::Value as NumCast>::from(i).unwrap() * height * reserve_boundary
                / <P::Value as NumCast>::from(best_rows).unwrap()
                + (height * (P::Value::one() - reserve_boundary)
                    / (P::Value::one() + P::Value::one()))
                + height * reserve_boundary
                    / <P::Value as NumCast>::from(best_rows).unwrap()
                    / (P::Value::one() + P::Value::one());
            P::from_xy(x, y)
        })
        .take(n)
}

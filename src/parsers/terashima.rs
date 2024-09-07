//! Parser for Terashima test data
//! From the README of Terashima2
//! For each instance we report:
//! - first line: the number N of pieces;
//! - second line: the width and height of the rectangular objects where pieces are placed.
//! - each of next N lines: number of vertices and coordinates x1 y1 x2 y2 x3 y3 ... xN yN.  
//!   Coordinates are counterclockwise.
//!
//! Optimum is known and it is given.  
//! Optimum placing for <INSTANCE>.txt is in file Op<INSTANCE>.txt
//!
//! For each file Op<INSTANCE>.txt we report:
//! - first line: the number of objects followed by how many pieces are in each object.
//! - second line: the width and height of the rectangular objects where pieces are placed.
//! - each of next N lines: number of vertices and coordinates x1 y1 x2 y2 x3 y3 ... xN yN
//!   where each piece is placed in the optimal solution.

use crate::point::Point2D;
use crate::polygon::Polygon;
use anyhow::{anyhow, Result};
use num_traits::Zero;
use std::{
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

#[derive(Debug)]
pub struct TerashimaInstance<P: Polygon> {
    pub bin: P,
    pub pieces: Vec<P>,
}

pub fn parse_terashima<P: Polygon, R: Read>(reader: R) -> Result<TerashimaInstance<P>>
where
    P: From<Vec<P::Point>>,
    P::Point: From<(
        <<P as Polygon>::Point as Point2D>::Value,
        <<P as Polygon>::Point as Point2D>::Value,
    )>,
    <<P as Polygon>::Point as Point2D>::Value: FromStr,
    <<<P as Polygon>::Point as Point2D>::Value as FromStr>::Err:
        std::error::Error + Send + Sync + 'static,
{
    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    // Parse number of pieces
    let n_pieces: usize = lines
        .next()
        .ok_or(anyhow!("Missing number of pieces"))??
        .trim()
        .parse()?;

    // Parse bin dimensions
    let bin_dims: Vec<<<P as Polygon>::Point as Point2D>::Value> = lines
        .next()
        .ok_or(anyhow!("Missing bin dimensions"))??
        .trim()
        .split_whitespace()
        .map(|s| s.parse())
        .collect::<Result<Vec<_>, _>>()?;

    if bin_dims.len() != 2 {
        return Err(anyhow!("Invalid bin dimensions"));
    }

    let bin = P::from(vec![
        (Zero::zero(), Zero::zero()).into(),
        (Zero::zero(), bin_dims[1]).into(),
        (bin_dims[0], bin_dims[1]).into(),
        (bin_dims[0], Zero::zero()).into(),
    ]);

    let mut pieces = Vec::with_capacity(n_pieces);

    // Parse pieces
    for _ in 0..n_pieces {
        let line = lines.next().ok_or(anyhow!("Missing piece data"))??;
        let values: Vec<<<P as Polygon>::Point as Point2D>::Value> = line
            .trim()
            .split_whitespace()
            .skip(1) // Skip the number of vertices
            .map(|s| s.parse())
            .collect::<Result<Vec<_>, _>>()?;

        if values.len() % 2 != 0 {
            return Err(anyhow!("Invalid number of coordinates for a piece"));
        }

        let vertices: Vec<P::Point> = values
            .chunks_exact(2)
            .map(|chunk| (chunk[0], chunk[1]).into())
            .collect();

        pieces.push(P::from(vertices));
    }

    Ok(TerashimaInstance { bin, pieces })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernelf64::Polygon;
    use std::fs::File;
    use std::path::PathBuf;

    #[test]
    fn test_parse_terashima() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test_data/Terashima2/TV001C5.txt");
        let file = File::open(path).unwrap();
        let result = parse_terashima::<Polygon, _>(file);
        let instance = result.unwrap();
        assert_eq!(instance.pieces.len(), 15);
    }
}

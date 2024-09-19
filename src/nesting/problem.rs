//! A "problem" is a complete input description for a nesting problem.
//! It includes the parts to be nested/packed, the shape they are packed in
//! and any constraints on the packing.

use crate::multi_polygon::MultiPolygon;
use crate::point::Point2D;
use crate::polygon::Polygon;
use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct PieceDescription<P: Polygon> {
    pub piece: MultiPolygon<P>,
    pub allowed_rotations: Vec<<P::Point as Point2D>::Value>,
    pub instances: usize,
}

impl<P: Polygon> PieceDescription<P> {
    pub fn new(
        piece: MultiPolygon<P>,
        allowed_rotations: Vec<<P::Point as Point2D>::Value>,
        instances: usize,
    ) -> Self {
        Self {
            piece,
            allowed_rotations,
            instances,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IrregularBinPackingProblem<P: Polygon> {
    bin: MultiPolygon<P>,
    piece_descriptions: Vec<PieceDescription<P>>,
}

impl<P: Polygon> IrregularBinPackingProblem<P> {
    pub fn new(bin: MultiPolygon<P>, piece_descriptions: Vec<PieceDescription<P>>) -> Self {
        Self {
            bin,
            piece_descriptions
        }
    }

    /// Returns a new builder instance for constructing a problem
    pub fn builder() -> IrregularBinPackingProblemBuilder<P> {
        IrregularBinPackingProblemBuilder::new()
    }

    /// Get the shape to pack all pieces into
    pub fn bin(&self) -> &MultiPolygon<P> {
        &self.bin
    }

    /// Get the pieces that should be packed into the bin
    pub fn piece_descriptions(&self) -> &Vec<PieceDescription<P>> {
        &self.piece_descriptions
    }

}

#[derive(Debug, Clone)]
pub struct IrregularBinPackingProblemBuilder<P: Polygon> {
    bin: Option<MultiPolygon<P>>,
    piece_descriptions: Vec<PieceDescription<P>>,
}

impl<P: Polygon> IrregularBinPackingProblemBuilder<P> {
    /// Creates a new builder instance.
    /// use the build method to consume the builder and return a problem
    pub fn new() -> Self {
        Self {
            bin: None,
            piece_descriptions: vec![],
        }
    }

    /// Sets the bin to pack the pieces into
    pub fn bin(mut self, bin: MultiPolygon<P>) -> Self {
        self.bin = Some(bin);
        self
    }

    /// Sets the pieces to be packed into the bin
    pub fn piece_descriptions<I>(mut self, piece_descriptions: I) -> Self
    where
        I: IntoIterator<Item = PieceDescription<P>>,
    {
        self.piece_descriptions = piece_descriptions.into_iter().collect();
        self
    }

    /// Adds a piece to pack into the bin
    pub fn piece_description<I>(
        mut self,
        piece: MultiPolygon<P>,
        allowed_rotations: I,
        instances: usize,
    ) -> Self
    where
        I: IntoIterator<Item = <P::Point as Point2D>::Value>,
    {
        self.piece_descriptions.push(PieceDescription::new(
            piece,
            allowed_rotations.into_iter().collect(),
            instances,
        ));
        self
    }

    /// Returns a problem
    pub fn build(self) -> Result<IrregularBinPackingProblem<P>> {
        Ok(IrregularBinPackingProblem {
            bin: self.bin.ok_or(anyhow!("No bin set"))?,
            piece_descriptions: self.piece_descriptions,
        })
    }
}

//! 2D Polygon Clipping library based on Vatti 1992

use crate::polygon::Polygon;

mod lmt;

pub trait Clippable: Polygon {
    fn get_vertex(&self, index: usize) -> <Self as Polygon>::Point;
}
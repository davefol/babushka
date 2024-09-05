use super::{Point2D, Segment};
use crate::no_fit_polygon::ComputeNoFitPolygon;
#[derive(Clone, Debug)]
pub struct Polygon {
    pub vertices: Vec<Point2D>,
    pub offset: Point2D,
}

impl Polygon {
    pub fn iter_mut_vertices_local(
        &mut self,
    ) -> impl Iterator<Item = &mut <Self as crate::polygon::Polygon>::Point> {
        self.vertices.iter_mut()
    }
}

impl<I> From<I> for Polygon
where
    I: IntoIterator<Item = <Self as crate::polygon::Polygon>::Point>,
{
    /// Creates a new polygon from an iterator over vertices.
    /// Vertices should be in order, clockwise for positive area
    /// and counter-clockwise for negative area.
    /// Vertices should have no offset.
    /// Do not repeat the first vertex at the end.
    fn from(vertices: I) -> Self {
        Self {
            vertices: vertices.into_iter().collect(),
            offset: Point2D { x: 0.0, y: 0.0 },
        }
    }
}

impl crate::polygon::Polygon for Polygon {
    type Point = Point2D;
    type Segment = Segment;

    fn iter_vertices_local(
        &self,
    ) -> impl Iterator<Item = &<Self as crate::polygon::Polygon>::Point> {
        self.vertices.iter()
    }

    fn iter_segments_local(&self) -> impl Iterator<Item = Segment> + Clone {
        self.vertices
            .iter()
            .zip(self.vertices.iter().cycle().skip(1))
            .take(self.vertices.len())
            .map(|window| Segment {
                start: *window.0,
                end: *window.1,
            })
    }

    fn offset(&self) -> Self::Point {
        self.offset
    }

    fn set_offset(&mut self, offset: Self::Point) {
        self.offset = offset;
    }

    fn length(&self) -> usize {
        self.vertices.len()
    }
}

impl ComputeNoFitPolygon for Polygon {
    fn get_vertex(&self, index: usize) -> <Self as crate::polygon::Polygon>::Point {
        self.vertices[index] + self.offset
    }
}

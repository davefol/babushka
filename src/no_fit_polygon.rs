use crate::point::Point2D;
use crate::polygon::Polygon;
use num_traits::{Float, One, Zero};

pub trait ComputeNoFitPolygon: Clone {
    type Polygon: Polygon;
    fn polygon(&self) -> &Self::Polygon;
    fn polygon_mut(&mut self) -> &mut Self::Polygon;
    fn get_vertex(
        &self,
        index: usize,
    ) -> <<Self as ComputeNoFitPolygon>::Polygon as Polygon>::Point;
    fn mark_vertex(&mut self, index: usize);
    fn unmark_vertex(&mut self, index: usize);
    fn is_vertex_marked(&self, index: usize) -> bool;

    fn no_fit_polygon(
        &mut self,
        other: &mut Self,
        inside: bool,
        search_edges: bool,
    ) -> Option<Vec<<<Self as ComputeNoFitPolygon>::Polygon as Polygon>::Point>> {
        // define some constants to avoid the disgusting generic type annotations
        let zero_point = <<Self as ComputeNoFitPolygon>::Polygon as Polygon>::Point::zero();

        self.polygon_mut().set_offset(zero_point);
        let min_self_by_y = self
            .polygon()
            .iter_vertices_local()
            .min_by(|a, b| a.y().partial_cmp(&b.y()).unwrap())
            .unwrap().clone();

        let max_other_by_y = other
            .polygon()
            .iter_vertices_local()
            .max_by(|a, b| a.y().partial_cmp(&b.y()).unwrap())
            .unwrap().clone();

        for i in 0..self.polygon().length() {
            self.unmark_vertex(i);
        }
        for i in 0..other.polygon().length() {
            self.unmark_vertex(i);
        }

        if !inside {
            min_self_by_y - max_other_by_y;
        } else {

        }


        Some(vec![])
    }

    fn search_start_point(&self, other: &Self, inside: bool) {
        let a = self.clone();
        let b = self.clone();

        for a_segment in a.polygon().iter_segments() {
        }
    }
}

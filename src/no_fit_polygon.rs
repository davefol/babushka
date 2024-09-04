use crate::point::Point2D;
use crate::polygon::Polygon;
use crate::segment::Segment;
use approx::abs_diff_eq;
use num_traits::{Float, One, Zero};

pub trait ComputeNoFitPolygon: Polygon {
    fn get_vertex(&self, index: usize) -> <Self as Polygon>::Point;

    fn no_fit_polygon(
        &mut self,
        other: &mut Self,
        inside: bool,
        search_edges: bool,
    ) -> Option<Vec<<Self as Polygon>::Point>> {
        // keep track of visited vertices
        let mut self_marked = vec![false; self.length()];
        let mut other_marked = vec![false; other.length()];

        self.set_offset(Zero::zero());
        let min_self_by_y = self
            .iter_vertices_local()
            .min_by(|a, b| a.y().partial_cmp(&b.y()).unwrap())
            .unwrap()
            .clone();

        let max_other_by_y = other
            .iter_vertices_local()
            .max_by(|a, b| a.y().partial_cmp(&b.y()).unwrap())
            .unwrap()
            .clone();

        let start_point = if !inside {
            Some(min_self_by_y - max_other_by_y)
        } else {
            self.search_start_point(other, &self_marked, true, None)
        };

        Some(vec![])
    }

    fn search_start_point(
        &self,
        other: &Self,
        self_marked: &Vec<bool>,
        inside: bool,
        nfp: Option<Vec<Vec<<Self as Polygon>::Point>>>,
    ) -> Option<<Self as Polygon>::Point> {
        //let self_clone = self.clone();
        let mut other = other.clone();
        let mut self_marked = self_marked.clone();

        // since we are iterating over every segment, the index i will be the index of
        // the starting point of that segment
        for (i, self_segment) in self.iter_segments().enumerate() {
            if !self_marked[i] {
                self_marked[i] = true;

                for j in 0..other.length() {
                    other.set_offset(*self_segment.start() - other.get_vertex(j));

                    // TODO: This kinda looks suspicious
                    let mut other_inside = None::<bool>;
                    for k in 0..other.length() {
                        if let Some(in_poly) = other.get_vertex(k).in_polygon(self) {
                            other_inside = Some(in_poly);
                            break;
                        }
                    }

                    // A and B are the same
                    let Some(mut other_inside) = other_inside else {
                        return None;
                    };

                    let mut start_point = other.offset();
                    if (other_inside && inside || !other_inside && !inside)
                        && self.intersects_polygon(&other)
                        && !Self::in_nfp(&start_point, &nfp)
                    {
                        return Some(start_point);
                    }

                    // Slide other along vector
                    let mut v = *self_segment.end() - *self_segment.start();
                    let d1 = self.project_distance_on_polygon(&other, v);
                    let d2 = self.project_distance_on_polygon(&other, -v);

                    let d = if d1.is_none() && d2.is_none() {
                        None
                    } else if d1.is_none() {
                        d2
                    } else if d2.is_none() {
                        d1
                    } else {
                        Some(d1.unwrap().min(d2.unwrap()))
                    };

                    // only slide until no longer negative
                    let Some(d) = d else {
                        continue;
                    };
                    if !(!abs_diff_eq!(d, Zero::zero()) && d > Zero::zero()) {
                        continue;
                    }

                    let vd2 = v.dot(&v);
                    if d*d < vd2 && !abs_diff_eq!(d*d, vd2) {
                        let vd = v.dot(&v);
                        v.set_x(v.x() * d / vd);
                        v.set_y(v.y() * d / vd);
                    }

                    other.set_offset(other.offset() + v);


                    // TODO: This kinda looks suspicious
                    for k in 0..other.length() {
                        if let Some(in_poly) = other.get_vertex(k).in_polygon(self) {
                            other_inside = in_poly;
                            break;
                        }
                    }
                    start_point = other.offset();
                    if (other_inside && inside || !other_inside && !inside) && self.intersects_polygon(&other) && !Self::in_nfp(&start_point, &nfp) {
                        return Some(start_point);
                    }
                }
            }
        }

        None
    }

    fn in_nfp(p: &<Self as Polygon>::Point, nfp: &Option<Vec<Vec<<Self as Polygon>::Point>>>) -> bool {
        let Some(nfp) = nfp else {
            return false;
        };

        if nfp.is_empty() {
            return false;
        }

        for poly in nfp {
            for point in poly {
                if abs_diff_eq!(p.x(), point.x()) && abs_diff_eq!(p.y(), point.y()) {
                    return true;
                }
            }
        }

        false
    }
}

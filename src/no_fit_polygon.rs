use crate::point::Point2D;
use crate::polygon::Polygon;
use crate::segment::Segment;
use approx::abs_diff_eq;
use itertools::Itertools;
use num_traits::{Float, One, Zero};

enum TouchingType {
    A,
    B,
    C,
}

struct Touching {
    tt: TouchingType,
    a: usize,
    b: usize,
}

struct Vector<P> {
    point: P,
    start: usize,
    end: usize,
}
pub trait ComputeNoFitPolygon: Polygon {
    /// Return the vertex at the given index after transformations.
    fn get_vertex(&self, index: usize) -> <Self as Polygon>::Point;

    fn no_fit_polygon(
        &self,
        other: &Self,
        inside: bool,
        search_edges: bool,
    ) -> Option<Vec<<Self as Polygon>::Point>> {
        // we will be mucking with the offset of other so clone it
        let mut other = other.clone();

        // keep track of visited vertices
        let mut self_marked = vec![false; self.length()];
        let mut other_marked = vec![false; other.length()];

        let min_self_by_y = self
            .iter_vertices_local()
            .min_by(|a, b| a.y().partial_cmp(&b.y()).unwrap())
            .unwrap()
            .clone();

        let max_other_by_y = other
            .iter_vertices()
            .max_by(|a, b| a.y().partial_cmp(&b.y()).unwrap())
            .unwrap()
            .clone();

        let mut start_point = if !inside {
            Some(min_self_by_y - max_other_by_y)
        } else {
            self.search_start_point(&other, &self_marked, true, None)
        };

        let nfp_list = vec![];

        while let Some(current_start_point) = start_point {
            other.set_offset(current_start_point);

            // Touching Type, A index, B index
            let mut touchings: Vec<Touching> = vec![];
            let mut prev_vector = None::<Vector<<Self as Polygon>::Point>>;
            let mut nfp: Vec<<Self as Polygon>::Point> = vec![other.get_vertex(0)];

            let mut reference = other.get_vertex(0);
            let start = reference;
            let mut counter = 0;

            // Sanity check, prevent infinite loop
            while counter < 10 * (self.length() + other.length()) {
                touchings = vec![];

                // find touching vertices / edges
                // we need to carry around indices into self and other
                // to avoid dealing with lots of mutable refernces
                for ((idx_self_start, self_segment), (idx_other_start, other_segment)) in self
                    .iter_segments_local()
                    .enumerate()
                    .cartesian_product(other.iter_segments().enumerate())
                {
                    let idx_self_end = if idx_self_start == self.length() {
                        0
                    } else {
                        idx_self_start + 1
                    };
                    let idx_other_end = if idx_other_start == other.length() {
                        0
                    } else {
                        idx_other_start + 1
                    };

                    if abs_diff_eq!(self_segment.start(), other_segment.start()) {
                        touchings.push(Touching {
                            tt: TouchingType::A,
                            a: idx_self_start,
                            b: idx_other_start,
                        });
                    } else if other_segment.start().on_segment(&self_segment) {
                        touchings.push(Touching {
                            tt: TouchingType::B,
                            a: idx_self_end,
                            b: idx_other_start,
                        });
                    } else if self_segment.start().on_segment(&other_segment) {
                        touchings.push(Touching {
                            tt: TouchingType::C,
                            a: idx_self_start,
                            b: idx_other_end,
                        });
                    }
                }

                // generate translation vectors from touching vertices / edges
                let mut vectors: Vec<Vector<<Self as Polygon>::Point>> = vec![];
                for touching in touchings {
                    let vertex_self = self.get_vertex(touching.a);
                    self_marked[touching.a] = true;

                    // adjacent self vertices
                    let prev_self_index = if touching.a == 0 {
                        self.length() - 1
                    } else {
                        touching.a - 1
                    };
                    let next_self_index = if touching.a == self.length() - 1 {
                        0
                    } else {
                        touching.a + 1
                    };

                    let prev_vertex_self = self.get_vertex(prev_self_index);
                    let next_vertex_self = self.get_vertex(next_self_index);

                    // adjacent B vertices
                    let vertex_other = other.get_vertex(touching.b);
                    let prev_other_index = if touching.b == 0 {
                        other.length() - 1
                    } else {
                        touching.b - 1
                    };
                    let next_other_index = if touching.b == other.length() - 1 {
                        0
                    } else {
                        touching.b + 1
                    };
                    let prev_vertex_other = other.get_vertex(prev_other_index);
                    let next_vertex_other = other.get_vertex(next_other_index);

                    match touching.tt {
                        TouchingType::A => {
                            vectors.push(Vector {
                                point: prev_vertex_self - vertex_self,
                                start: touching.a,
                                end: prev_self_index,
                            });
                            vectors.push(Vector {
                                point: next_vertex_self - vertex_self,
                                start: touching.a,
                                end: next_self_index,
                            });

                            // other's vectors need to be inverted
                            vectors.push(Vector {
                                point: vertex_other - prev_vertex_other,
                                start: prev_other_index,
                                end: touching.b,
                            });
                            vectors.push(Vector {
                                point: vertex_other - next_vertex_other,
                                start: next_other_index,
                                end: touching.b,
                            });
                        }
                        TouchingType::B => {
                            vectors.push(Vector {
                                point: vertex_self - vertex_b,
                                start: touching.a,
                                end: prev_self_index,
                            });
                            vectors.push(Vector {
                                point: next_vertex_self - vertex_self,
                                start: touching.a,
                                end: next_self_index,
                            });
                        }
                        TouchingType::C => {}
                    }
                }
            }
        }

        Some(nfp_list)
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
                    if d * d < vd2 && !abs_diff_eq!(d * d, vd2) {
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
                    if (other_inside && inside || !other_inside && !inside)
                        && self.intersects_polygon(&other)
                        && !Self::in_nfp(&start_point, &nfp)
                    {
                        return Some(start_point);
                    }
                }
            }
        }

        None
    }

    fn in_nfp(
        p: &<Self as Polygon>::Point,
        nfp: &Option<Vec<Vec<<Self as Polygon>::Point>>>,
    ) -> bool {
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

use std::ops::Add;

use approx::abs_diff_eq;
use num_traits::{Float, Zero};

use crate::point::Point2D;
use crate::polygon::Polygon;
use crate::segment::Segment;

impl<'a, P> Polygon for Vec<MarkableVertex<'a, P>>
where
    P: Point2D,
{
    type Point = P;
    type Segment = MarkableVertexSegment<P>;

    fn iter_vertices_local(
        &self,
    ) -> impl Iterator<Item = &<Self as crate::polygon::Polygon>::Point> {
        self.iter().map(|vertex| vertex.vertex)
    }

    fn iter_segments_local(&self) -> impl Iterator<Item = MarkableVertexSegment<P>> + Clone {
        self.iter()
            .zip(self.iter().cycle().skip(1))
            .take(self.len())
            .map(|window| MarkableVertexSegment {
                start: *window.0.vertex,
                end: *window.1.vertex,
            })
    }

    fn offset(&self) -> Self::Point {
        Self::Point::zero()
    }

    fn set_offset(&mut self, _: Self::Point) {}

    fn length(&self) -> usize {
        self.len()
    }
}

#[derive(Clone, Copy)]
struct MarkableVertex<'a, P> {
    vertex: &'a P,
    marked: bool,
}

#[derive(Clone, Copy)]
struct MarkableVertexSegment<P> {
    start: P,
    end: P,
}

impl<P> Add<P> for MarkableVertexSegment<P>
where
    P: Point2D,
{
    type Output = Self;
    fn add(self, rhs: P) -> Self {
        Self {
            start: self.start + rhs,
            end: self.end + rhs,
        }
    }
}

impl<P> Segment for MarkableVertexSegment<P>
where
    P: Point2D,
{
    type Point = P;
    fn start(&self) -> &Self::Point {
        &self.start
    }
    fn end(&self) -> &Self::Point {
        &self.end
    }
}

pub trait SearchStartPoint<'a, P, N> {
    fn search_start_point(
        &mut self,
        other: &mut Self,
        inside: bool,
        nfp: Option<Vec<N>>,
    ) -> Option<P>;
}

impl<'a, P, N> SearchStartPoint<'a, P, N> for Vec<MarkableVertex<'a, P>>
where
    P: Point2D,
    N: Polygon<Point = P>,
{
    fn search_start_point(
        &mut self,
        other: &mut Self,
        inside: bool,
        nfp: Option<Vec<N>>,
    ) -> Option<P> {
        let mut other_offset = P::zero();

        for i in 0..self.len() - 1 {
            if !self[i].marked {
                self[i].marked = true;
                for j in 0..other.len() {
                    other_offset.set_x(self[i].vertex.x() - other[j].vertex.x());
                    other_offset.set_y(self[i].vertex.y() - other[j].vertex.y());

                    let mut other_inside = None::<bool>;
                    for k in 0..other.len() {
                        let in_poly = (*other[k].vertex + other_offset).in_polygon(other);
                        if in_poly.is_some() {
                            other_inside = in_poly;
                            break;
                        }
                    }

                    let Some(mut other_inside) = other_inside else {
                        return None;
                    };

                    let mut start_point = other_offset.clone();
                    let in_nfp = {
                        if let Some(ref nfp) = nfp {
                            let mut is_in = false;
                            for poly in nfp {
                                for point in poly.iter_vertices_local() {
                                    if abs_diff_eq!(start_point.x(), point.x())
                                        && abs_diff_eq!(start_point.y(), point.y())
                                    {
                                        is_in = true;
                                        break;
                                    }
                                    if is_in {
                                        break;
                                    }
                                }
                            }
                            is_in
                        } else {
                            false
                        }
                    };
                    if ((other_inside && inside) || (!other_inside && !inside))
                        && !self.intersects_polygon(other)
                        && !in_nfp
                    {
                        return Some(start_point);
                    }

                    // slide other along vector
                    let vx = self[i + 1].vertex.x() - self[i].vertex.x();
                    let vy = self[i + 1].vertex.y() - self[i].vertex.y();
                    let mut v_forward = <P as Point2D>::from_xy(vx, vy);
                    let v_reverse = <P as Point2D>::from_xy(-vx, -vy);
                    let d1 = self.project_distance_on_polygon(other, v_forward);
                    let d2 = self.project_distance_on_polygon(other, v_reverse);

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
                    if (d.is_some() && !abs_diff_eq!(d.unwrap(), <P as Point2D>::Value::zero()))
                        && d.unwrap() > <P as Point2D>::Value::zero()
                    {
                    } else {
                        continue;
                    }

                    let vd2 = vx * vx + vy * vy;

                    // TODO: check that this isn't bullshit
                    // is d really guaranteed to be Some?
                    let d = d.unwrap();
                    if d * d < vd2 && !abs_diff_eq!(d * d, vd2) {
                        v_forward = v_forward.normalized().unwrap();
                    }

                    other_offset.set_x(other_offset.x() + v_forward.x());
                    other_offset.set_y(other_offset.y() + v_forward.y());

                    for k in 0..other.len() {
                        let in_poly = (*other[k].vertex + other_offset).in_polygon(self);
                        if let Some(in_poly) = in_poly {
                            other_inside = in_poly;
                            break;
                        }
                    }

                    start_point = other_offset.clone();
                    let in_nfp = {
                        if let Some(ref nfp) = nfp {
                            let mut is_in = false;
                            for poly in nfp {
                                for point in poly.iter_vertices_local() {
                                    if abs_diff_eq!(start_point.x(), point.x())
                                        && abs_diff_eq!(start_point.y(), point.y())
                                    {
                                        is_in = true;
                                        break;
                                    }
                                    if is_in {
                                        break;
                                    }
                                }
                            }
                            is_in
                        } else {
                            false
                        }
                    };
                    if ((other_inside && inside) || (!other_inside && !inside))
                        && self.intersects_polygon(other)
                        && !in_nfp
                    {
                        return Some(start_point);
                    }
                }
            }
        }
        None
    }
}

pub trait NoFitPolygon {
    fn no_fit_polygon(&self, other: &Self, inside: bool) -> Self;
}

impl<T: Polygon> NoFitPolygon for T
where
    T: FromIterator<<T as Polygon>::Point>,
{
    /// Compute a no fit polygon by orbiting other around this polygon.
    /// if the inside flag is true, other is orbitted inside of this polygon
    /// if search_edges is true, all edges of this polygon are explored for NFPs
    fn no_fit_polygon(&self, other: &Self, inside: bool) -> Self {
        enum TouchingType {
            A,
            B,
            C
        }
        let mut a: Vec<MarkableVertex<<Self as Polygon>::Point>> = self
            .iter_vertices_local()
            .map(|vertex| MarkableVertex {
                vertex,
                marked: false,
            })
            .collect();

        let mut b: Vec<MarkableVertex<<Self as Polygon>::Point>> = other
            .iter_vertices_local()
            .map(|vertex| MarkableVertex {
                vertex,
                marked: false,
            })
            .collect();

        let (min_a_index, min_a) = a
            .iter()
            .enumerate()
            .map(|(idx, v)| (idx, v.vertex.y()))
            .min_by(|i, j| i.1.partial_cmp(&j.1).unwrap())
            .unwrap();
        let (max_b_index, max_b) = b
            .iter()
            .enumerate()
            .map(|(idx, v)| (idx, v.vertex.y()))
            .max_by(|i, j| i.1.partial_cmp(&j.1).unwrap())
            .unwrap();

        let start_point = if !inside {
            Some(<Self as Polygon>::Point::from_xy(
                a[min_a_index].vertex.x() - b[max_b_index].vertex.x(),
                a[min_a_index].vertex.y() - b[max_b_index].vertex.y(),
            ))
        } else {
            a.search_start_point(&mut b, true, None::<Vec<Self>>)
        };

        let mut nfp_list = vec![];

        let a_offset = <Self as Polygon>::Point::zero();
        let mut b_offset = <Self as Polygon>::Point::zero();
        while let Some(start_point) = start_point {
            b_offset.set_x(start_point.x());
            b_offset.set_y(start_point.y());

            let mut touching = vec![];
            let mut prev_vector = None;

            let mut nfp = vec![b[0].vertex.clone() + b_offset];

            let mut reference = b[0].vertex.clone() + b_offset;
            let mut start = reference.clone();
            let mut counter = 0;

            while counter < 10 * (a.len() + b.len()) {
                touching = vec![];
                for i in 0..a.len() {
                    let next_i = if i == a.len() - 1 { 0 } else { i + 1 };
                    for j in 0..b.len() {
                        let next_j = if j == b.len() - 1 { 0 } else { j + 1 };
                        let a_segment = MarkableVertexSegment {
                            start: a[i].vertex.clone(),
                            end: a[next_i].vertex.clone()
                        };
                        let b_segment = MarkableVertexSegment {
                            start: b[j].vertex.clone() + b_offset,
                            end: b[next_j].vertex.clone() + b_offset
                        };
                        if abs_diff_eq!(a[i].vertex.x(), b[j].vertex.x() + b_offset.x())
                            && abs_diff_eq!(a[i].vertex.y(), b[j].vertex.y() + b_offset.y())
                        {
                            touching.push((TouchingType::A, i, j));
                        } else if (*b[j].vertex + b_offset).on_segment(&a_segment) {
                            touching.push((TouchingType::B, next_i, j));
                        } else if (a[i].vertex).on_segment(&b_segment) {
                            touching.push((TouchingType::C, i, next_j));
                        }
                    }
                }
                // generate translation vectors from touching vertices/edges
                let mut vectors = vec![];
                for i in 0..touching.len() {
                    let vertex_a = &mut a[touching[i].1];
                    vertex_a.marked = true;

                    // adjacent a vertices
                    let prev_a_index = touching[i].1 - 1;
                    let next_a_index = touching[i].1 + 1;

                    let prev_a_index = if prev_a_index < 0 {
                        a.len() - 1
                    } else {
                        prev_a_index
                    };
                    let next_a_index = if next_a_index >= a.len() {
                        0
                    } else {
                        next_a_index
                    };
                    let prev_a = &mut a[prev_a_index];
                    let next_a = &mut a[next_a_index];

                    let vertex_b = &mut b[touching[i].2];
                    let prev_b_index = touching[i].2 - 1;
                    let next_b_index = touching[i].2 + 1;
                    let prev_b_index = if prev_b_index < 0 {
                        b.len() - 1
                    } else {
                        prev_b_index
                    };
                    let next_b_index = if next_b_index >= b.len() {
                        0
                    } else {
                        next_b_index
                    };
                    let prev_b = &mut b[prev_b_index];
                    let next_b = &mut b[next_b_index];

                    match touching[i].0 {
                        TouchingType::A => {
                            vectors.push(prev_a)
                        }
                        TouchingType::B => {

                        }
                        TouchingType::C => {

                        }
                    }
                    
                }
            }
        }

        Self::from_iter(vec![])
    }
}

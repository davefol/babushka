use crate::point::Point2D;
use crate::polygon::Polygon;
use approx::abs_diff_eq;
use num_traits::{Float, One, Zero};
use std::ops::Add;

#[derive(Debug)]
pub enum SegmentSegmentIntersection<T: Point2D> {
    None,
    Equal,
    Touching(T),
    Intersection(T),
    Overlap(T, T),
}

pub trait Segment:
    Clone + Copy + Add<Self::Point, Output = Self> + From<(Self::Point, Self::Point)>
{
    type Point: Point2D;

    fn start(&self) -> &Self::Point;
    fn end(&self) -> &Self::Point;

    fn rotate(&self, angle: <Self::Point as Point2D>::Value) -> Self {
        Self::from((self.start().rotate(angle), self.end().rotate(angle)))
    }

    /// Returns the intersection of this segment with a polygon.
    /// The intersections are ordered by distance from the start of this segment.
    fn intersects_polygon<P>(&self, other: &P) -> Vec<Self::Point>
    where
        P: Polygon<Segment = Self>,
    {
        let mut intersections = vec![];
        for segment in other.iter_segments() {
            let intersection = self.intersects_segment(&segment, false);
            match intersection {
                SegmentSegmentIntersection::None => {}
                SegmentSegmentIntersection::Equal => {}
                SegmentSegmentIntersection::Touching(point) => {
                    intersections.push(point);
                }
                SegmentSegmentIntersection::Intersection(point) => {
                    intersections.push(point);
                }
                SegmentSegmentIntersection::Overlap(start, end) => {
                    intersections.push(start);
                    intersections.push(end);
                }
            }
            // if let Some(intersection) = intersection {
            //     intersections.push(intersection);
        }
        // order intersections by distance from self.start()
        intersections.sort_by(|a, b| {
            let dist_a = self.start().dot(&(*a - *self.start()));
            let dist_b = self.start().dot(&(*b - *self.start()));
            dist_a.partial_cmp(&dist_b).unwrap()
        });
        // remove duplicates
        intersections.dedup_by(|a, b| abs_diff_eq!(a, b));
        intersections
    }

    /// Returns the intersection of this segment with another segment.
    /// Returns None if there is no intersection.
    fn intersects_segment(
        &self,
        other: &Self,
        infinite: bool,
    ) -> SegmentSegmentIntersection<Self::Point> {
        if abs_diff_eq!(self.start(), other.start()) && abs_diff_eq!(self.end(), other.end()) {
            return SegmentSegmentIntersection::Equal;
        }

        let a = self.start();
        let b = self.end();
        let c = other.start();
        let d = other.end();

        let a1 = b.y() - a.y();
        let b1 = a.x() - b.x();
        // 2D cross product
        let c1 = b.x() * a.y() - a.x() * b.y();
        let a2 = d.y() - c.y();
        let b2 = c.x() - d.x();
        // 2D cross product
        let c2 = d.x() * c.y() - c.x() * d.y();

        // Determinant of the position vectors of the two segments
        let denominator = a1 * b2 - a2 * b1;

        let x = (b1 * c2 - b2 * c1) / denominator;
        let y = (a2 * c1 - a1 * c2) / denominator;

        // Segments are parallel
        if abs_diff_eq!(
            denominator,
            Zero::zero(),
            epsilon = <Self as Segment>::Point::value_epsilon()
        ) {
            let mut overlaps = vec![];
            if a.on_segment(other) {
                overlaps.push(a);
            }
            if b.on_segment(other) {
                overlaps.push(b);
            }
            if c.on_segment(self) {
                overlaps.push(c);
            }
            if d.on_segment(self) {
                overlaps.push(d);
            }

            if overlaps.len() == 0 {
                return SegmentSegmentIntersection::None;
            } else if overlaps.len() == 1 {
                return SegmentSegmentIntersection::Touching(overlaps[0].clone());
            } else if overlaps.len() == 2 {
                println!("Segments overlap");
                return SegmentSegmentIntersection::Overlap(
                    overlaps[0].clone(),
                    overlaps[1].clone(),
                );
            }
        }

        if !infinite {
            if (a.x() - b.x()).abs() > Self::Point::epsilon()
                && if a.x() < b.x() {
                    x < a.x() || x > b.x()
                } else {
                    x > a.x() || x < b.x()
                }
            {
                return SegmentSegmentIntersection::None;
            }

            if (a.y() - b.y()).abs() > Self::Point::epsilon()
                && if a.y() < b.y() {
                    y < a.y() || y > b.y()
                } else {
                    y > a.y() || y < b.y()
                }
            {
                return SegmentSegmentIntersection::None;
            }

            if (c.x() - d.x()).abs() > Self::Point::epsilon()
                && if c.x() < d.x() {
                    x < c.x() || x > d.x()
                } else {
                    x > c.x() || x < d.x()
                }
            {
                return SegmentSegmentIntersection::None;
            }

            if (c.y() - d.y()).abs() > Self::Point::epsilon()
                && if c.y() < d.y() {
                    y < c.y() || y > d.y()
                } else {
                    y > c.y() || y < d.y()
                }
            {
                return SegmentSegmentIntersection::None;
            }
        }

        let out = Self::Point::from_xy(x, y);
        SegmentSegmentIntersection::Intersection(out)
    }

    /// Returns the distance from this segment to another segment
    /// if the other segment travelled along direction.
    fn distance_to_segment_along_direction(
        &self,
        other: &Self,
        direction: Self::Point,
    ) -> Option<<<Self as Segment>::Point as Point2D>::Value> {
        let Some(direction) = direction.normalized() else {
            return None;
        };

        let normal = <<Self as Segment>::Point as Point2D>::from_xy(direction.y(), -direction.x());
        let reverse = -direction.clone();

        let a = self.start();
        let b = self.end();
        let e = other.start();
        let f = other.end();

        let dot_a = a.dot(&normal);
        let dot_b = b.dot(&normal);
        let dot_e = e.dot(&normal);
        let dot_f = f.dot(&normal);

        let cross_a = a.dot(&direction);
        let cross_b = b.dot(&direction);
        let cross_e = e.dot(&direction);
        let cross_f = f.dot(&direction);

        let ab_min = dot_a.min(dot_b);
        let ab_max = dot_a.max(dot_b);
        let ef_min = dot_e.min(dot_f);
        let ef_max = dot_e.max(dot_f);

        // segments that will merely touch at one point
        if abs_diff_eq!(
            ab_max,
            ef_min,
            epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
        ) || abs_diff_eq!(
            ab_min,
            ef_max,
            epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
        ) {
            return None;
        }

        // segments miss each other completely
        if ab_max < ef_min || ab_min > ef_max {
            return None;
        }

        let overlap =
            if (ab_max > ef_max && ab_min < ef_min) || (ef_max > ab_max && ef_min < ab_min) {
                One::one()
            } else {
                let min_max = ab_max.min(ef_max);
                let max_min = ab_min.max(ef_min);
                let max_max = ab_max.max(ef_max);
                let min_min = ab_min.min(ef_min);

                (min_max - max_min) / (max_max - min_min)
            };

        let cross_abe = (e.y() - a.y()) * (b.x() - a.x()) - (e.x() - a.x()) * (b.y() - a.y());
        let cross_abf = (f.y() - a.y()) * (b.x() - a.x()) - (f.x() - a.x()) * (b.y() - a.y());

        // lines are colinear
        if abs_diff_eq!(
            cross_abe,
            Zero::zero(),
            epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
        ) && abs_diff_eq!(
            cross_abf,
            Zero::zero(),
            epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
        ) {
            let ab_norm =
                <<Self as Segment>::Point as Point2D>::from_xy(b.y() - a.y(), a.x() - b.x());
            let ef_norm =
                <<Self as Segment>::Point as Point2D>::from_xy(f.y() - e.y(), e.x() - f.x());

            let Some(ab_norm) = ab_norm.normalized() else {
                return None;
            };

            let Some(ef_norm) = ef_norm.normalized() else {
                return None;
            };

            // segment normals must point in opposite directions
            if (ab_norm.y() * ef_norm.x() - ab_norm.x() * ef_norm.y()).abs() < Float::epsilon()
                && ab_norm.y() * ef_norm.y() + ab_norm.x() * ef_norm.x() < Zero::zero()
            {
                // normal of AB segment must point in same direction as given direction vector
                let norm_dot = ab_norm.y() * direction.y() + ab_norm.x() * direction.x();
                if abs_diff_eq!(
                    norm_dot,
                    Zero::zero(),
                    epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
                ) {
                    return None;
                }
                if norm_dot < Zero::zero() {
                    return Some(Zero::zero());
                }
            }
            return None;
        }

        let mut distances = vec![];

        // coincident points
        if abs_diff_eq!(
            dot_a,
            dot_e,
            epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
        ) {
            distances.push(cross_a - cross_e);
        } else if abs_diff_eq!(
            dot_a,
            dot_f,
            epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
        ) {
            distances.push(cross_a - cross_f);
        } else if dot_a > ef_min && dot_a < ef_max {
            let mut d = a.distance_to_segment(other, reverse, false);
            if let Some(dhat) = d {
                if abs_diff_eq!(
                    dhat,
                    Zero::zero(),
                    epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
                ) {
                    let db = b.distance_to_segment(other, reverse, true);
                    if let Some(db) = db {
                        if db < Zero::zero()
                            || abs_diff_eq!(
                                db * overlap,
                                Zero::zero(),
                                epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
                            )
                        {
                            d = None;
                        }
                    }
                }
            }
            if let Some(d) = d {
                distances.push(d);
            }
        }

        if abs_diff_eq!(
            dot_b,
            dot_e,
            epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
        ) {
            distances.push(cross_b - cross_e);
        } else if abs_diff_eq!(
            dot_b,
            dot_f,
            epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
        ) {
            distances.push(cross_b - cross_f);
        } else if dot_b > ef_min && dot_b < ef_max {
            let mut d = b.distance_to_segment(other, reverse, false);
            if let Some(dhat) = d {
                if abs_diff_eq!(
                    dhat,
                    Zero::zero(),
                    epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
                ) {
                    let da = a.distance_to_segment(other, reverse, true);
                    if let Some(da) = da {
                        if da < Zero::zero()
                            || abs_diff_eq!(
                                da * overlap,
                                Zero::zero(),
                                epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
                            )
                        {
                            d = None;
                        }
                    }
                }
            }
            if let Some(d) = d {
                distances.push(d);
            }
        }

        if dot_e > ab_min && dot_e < ab_max {
            let mut d = e.distance_to_segment(self, direction, false);
            if let Some(dhat) = d {
                if abs_diff_eq!(
                    dhat,
                    Zero::zero(),
                    epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
                ) {
                    let df = f.distance_to_segment(self, direction, true);
                    if let Some(df) = df {
                        if df < Zero::zero()
                            || abs_diff_eq!(
                                df * overlap,
                                Zero::zero(),
                                epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
                            )
                        {
                            d = None;
                        }
                    }
                }
            }
            if let Some(d) = d {
                distances.push(d);
            }
        }

        if dot_f > ab_min && dot_f < ab_max {
            let mut d = f.distance_to_segment(self, direction, false);
            if let Some(dhat) = d {
                if abs_diff_eq!(
                    dhat,
                    Zero::zero(),
                    epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
                ) {
                    let de = e.distance_to_segment(self, direction, true);
                    if let Some(de) = de {
                        if de < Zero::zero()
                            || abs_diff_eq!(
                                de * overlap,
                                Zero::zero(),
                                epsilon = <<Self as Segment>::Point as Point2D>::value_epsilon()
                            )
                        {
                            d = None;
                        }
                    }
                }
            }
            if let Some(d) = d {
                distances.push(d);
            }
        }

        if distances.is_empty() {
            return None;
        }

        return distances
            .into_iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap());
    }
}

mod tests {
    use crate::segment::SegmentSegmentIntersection;

    #[test]
    fn test_segment_intersects_segment() {
        use super::Segment as _;
        use crate::kernelf64::Point2D;
        use crate::kernelf64::Segment;
        use crate::point::Point2D as _;
        use approx::abs_diff_eq;
        let segment1 = Segment {
            start: Point2D { x: 0.0, y: 0.0 },
            end: Point2D { x: 5.0, y: 5.0 },
        };
        let segment2 = Segment {
            start: Point2D { x: 0.0, y: 5.0 },
            end: Point2D { x: 5.0, y: 0.0 },
        };
        let segment3 = Segment {
            start: Point2D { x: 6.0, y: 6.0 },
            end: Point2D { x: 7.0, y: 7.0 },
        };

        // Test intersecting segments
        let intersection = segment1.intersects_segment(&segment2, false);
        match intersection {
            SegmentSegmentIntersection::Intersection(point) => {
                assert!(abs_diff_eq!(point.x(), 2.5));
                assert!(abs_diff_eq!(point.y(), 2.5));
            }
            _ => panic!("Expected intersection, got something else"),
        }

        // Test non-intersecting segments
        let no_intersection = segment1.intersects_segment(&segment3, false);
        match no_intersection {
            SegmentSegmentIntersection::None => {}
            _ => panic!("Expected no intersection, got something else"),
        }

        // Test parallel segments
        let segment4 = Segment {
            start: Point2D { x: 1.0, y: 1.0 },
            end: Point2D { x: 6.0, y: 6.0 },
        };
        let parallel_intersection = segment1.intersects_segment(&segment4, false);
        match parallel_intersection {
            SegmentSegmentIntersection::Overlap(point1, point2) => {
                assert!(abs_diff_eq!(segment1.end(), &point1));
                assert!(abs_diff_eq!(segment4.start(), &point2));
            }
            _ => panic!("Expected Overlap, got something else"),
        }

        // Test with infinite flag set to true
        let segment5 = Segment {
            start: Point2D { x: 0.0, y: 2.5 },
            end: Point2D { x: 1.5, y: 2.5 },
        };
        let infinite_intersection = segment1.intersects_segment(&segment5, true);
        match infinite_intersection {
            SegmentSegmentIntersection::Intersection(infinite_point) => {
                assert!(abs_diff_eq!(infinite_point.x(), 2.5));
                assert!(abs_diff_eq!(infinite_point.y(), 2.5));
            }
            _ => panic!("Expected SegmentSegmentIntersection::Intersection"),
        }
    }

    #[test]
    fn test_segment_distance_to_coincident_segment() {
        use super::Segment as _;
        use crate::kernelf64::Point2D;
        use crate::kernelf64::Segment;

        // test segments that will merely touch at one point
        //   | segment 2
        //   |
        // |   segment 1
        // |
        let segment1 = Segment {
            start: Point2D { x: 0.0, y: 0.0 },
            end: Point2D { x: 0.0, y: 1.0 },
        };
        let segment2 = Segment {
            start: Point2D { x: 1.0, y: 1.0 },
            end: Point2D { x: 1.0, y: 2.0 },
        };
        let direction = Point2D { x: -1.0, y: 0.0 };
        let distance = segment1.distance_to_segment_along_direction(&segment2, direction);
        assert_eq!(distance, None);

        // test segments that will miss each other completely
        //   | segment 2
        //   |
        //
        // |   segment 1
        // |
        let segment1 = Segment {
            start: Point2D { x: 0.0, y: 0.0 },
            end: Point2D { x: 0.0, y: 1.0 },
        };
        let segment2 = Segment {
            start: Point2D { x: 1.0, y: 2.0 },
            end: Point2D { x: 1.0, y: 3.0 },
        };
        let distance = segment1.distance_to_segment_along_direction(&segment2, direction);
        assert_eq!(distance, None);

        // test segments that are one away from each other
        //   | segment 2
        // | |
        // | |
        // |   segment 1
        let segment1 = Segment {
            start: Point2D { x: 0.0, y: 0.0 },
            end: Point2D { x: 0.0, y: 2.0 },
        };
        let segment2 = Segment {
            start: Point2D { x: 1.0, y: 1.0 },
            end: Point2D { x: 1.0, y: 3.0 },
        };
        let distance = segment1.distance_to_segment_along_direction(&segment2, direction);
        assert_eq!(distance, Some(1.0));
    }

    #[test]
    fn test_segment_intersects_polygon() {
        use super::Segment as _;
        use crate::kernelf64::{Point2D, Polygon, Segment};

        // Create a square polygon
        let square = Polygon::from(vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 0.0, y: 2.0 },
            Point2D { x: 2.0, y: 2.0 },
            Point2D { x: 2.0, y: 0.0 },
        ]);

        // Create a segment that intersects the square
        let segment = Segment {
            start: Point2D { x: -1.0, y: 1.0 },
            end: Point2D { x: 3.0, y: 1.0 },
        };

        // Get intersections
        let intersections = segment.intersects_polygon(&square);

        // Check if we have the correct number of intersections
        assert_eq!(intersections.len(), 2);

        // Check if the intersections are correct
        assert!(intersections.contains(&Point2D { x: 0.0, y: 1.0 }));
        assert!(intersections.contains(&Point2D { x: 2.0, y: 1.0 }));

        // Create a segment that doesn't intersect the square
        let non_intersecting_segment = Segment {
            start: Point2D { x: -2.0, y: -1.0 },
            end: Point2D { x: -1.0, y: -1.0 },
        };

        // Get intersections
        let no_intersections = non_intersecting_segment.intersects_polygon(&square);

        // Check that there are no intersections
        assert_eq!(no_intersections.len(), 0);
    }
}

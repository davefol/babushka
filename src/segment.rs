use crate::point::Point2D;
use approx::abs_diff_eq;
use num_traits::{Float, One, Zero};
use std::ops::Add;

pub trait Segment: Clone + Copy + Add<Self::Point, Output = Self> {
    type Point: Point2D;

    fn start(&self) -> &Self::Point;
    fn end(&self) -> &Self::Point;

    fn intersects_segment(&self, other: &Self, infinite: bool) -> Option<Self::Point> {
        let a = self.start();
        let b = self.end();
        let c = other.start();
        let d = other.end();

        let a1 = b.y() - a.y();
        let b1 = a.x() - b.x();
        let c1 = b.x() * a.y() - a.x() * b.y();
        let a2 = d.y() - c.y();
        let b2 = c.x() - d.x();
        let c2 = d.x() * c.y() - c.x() * d.y();

        let denominator = a1 * b2 - a2 * b1;

        let x = (b1 * c2 - b2 * c1) / denominator;
        let y = (a2 * c1 - a1 * c2) / denominator;

        if !x.is_finite() || !y.is_finite() {
            return None;
        };

        if !infinite {
            if (a.x() - b.x()).abs() > Self::Point::epsilon()
                && if a.x() < b.x() {
                    x < a.x() || x > b.x()
                } else {
                    x > a.x() || x < b.x()
                }
            {
                return None;
            }

            if (a.y() - b.y()).abs() > Self::Point::epsilon()
                && if a.y() < b.y() {
                    y < a.y() || y > b.y()
                } else {
                    y > a.y() || y < b.y()
                }
            {
                return None;
            }

            if (c.x() - d.x()).abs() > Self::Point::epsilon()
                && if c.x() < d.x() {
                    x < c.x() || x > d.x()
                } else {
                    x > c.x() || x < d.x()
                }
            {
                return None;
            }

            if (c.y() - d.y()).abs() > Self::Point::epsilon()
                && if c.y() < d.y() {
                    y < c.y() || y > d.y()
                } else {
                    y > c.y() || y < d.y()
                }
            {
                return None;
            }
        }

        let mut out = a.clone();
        out.set_x(x);
        out.set_y(y);
        Some(out)
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
        let mut normal = direction.clone();
        normal.set_x(direction.y());
        normal.set_y(-direction.x());

        let mut reverse = direction.clone();
        reverse.set_x(-direction.x());
        reverse.set_y(-direction.y());

        let a = self.start();
        let b = self.end();
        let e = other.start();
        let f = other.end();

        let dot_a = a.x() * normal.x() + a.y() * normal.y();
        let dot_b = b.x() * normal.x() + b.y() * normal.y();
        let dot_e = e.x() * normal.x() + e.y() * normal.y();
        let dot_f = f.x() * normal.x() + f.y() * normal.y();

        let cross_a = a.x() * direction.x() + a.y() * direction.y();
        let cross_b = b.x() * direction.x() + b.y() * direction.y();
        let cross_e = e.x() * direction.x() + e.y() * direction.y();
        let cross_f = f.x() * direction.x() + f.y() * direction.y();

        let ab_min = dot_a.min(dot_b);
        let ab_max = dot_a.max(dot_b);
        let ef_min = dot_e.min(dot_f);
        let ef_max = dot_e.max(dot_f);

        // segments that will merely touch at one point
        if abs_diff_eq!(ab_max, ef_min) || abs_diff_eq!(ab_min, ef_max) {
            return None;
        }

        // segments miss each other completely
        if ab_max < ef_min || ab_min > ef_max {
            return None;
        }

        let overlap =
            if (ab_max > ef_max && ab_min < ef_min) || (ef_max > ab_max && ef_min < ab_min) {
                <<Self as Segment>::Point as Point2D>::Value::one()
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
            <<Self as Segment>::Point as Point2D>::Value::zero()
        ) && abs_diff_eq!(
            cross_abf,
            <<Self as Segment>::Point as Point2D>::Value::zero()
        ) {
            let mut ab_norm = normal.clone();
            ab_norm.set_x(b.y() - a.y());
            ab_norm.set_y(a.x() - b.x());
            let Some(ab_norm) = ab_norm.normalized() else {
                return None;
            };

            let mut ef_norm = normal.clone();
            ef_norm.set_x(f.y() - e.y());
            ef_norm.set_y(e.x() - f.x());
            let Some(ef_norm) = ef_norm.normalized() else {
                return None;
            };

            // segment normals must point in opposite directions
            if (ab_norm.y() * ef_norm.x() - ab_norm.x() * ef_norm.y()).abs()
                < <<Self as Segment>::Point as Point2D>::Value::epsilon()
                && ab_norm.y() * ef_norm.y() + ab_norm.x() * ef_norm.x()
                    < <<Self as Segment>::Point as Point2D>::Value::zero()
            {
                // normal of AB segment must point in same direction as given direction vector
                let norm_dot = ab_norm.y() * direction.y() + ab_norm.x() * direction.x();
                if abs_diff_eq!(
                    norm_dot,
                    <<Self as Segment>::Point as Point2D>::Value::zero()
                ) {
                    return None;
                }
                if norm_dot < <<Self as Segment>::Point as Point2D>::Value::zero() {
                    return Some(<<Self as Segment>::Point as Point2D>::Value::zero());
                }
            }
            return None;
        }

        let mut distances = vec![];

        // coincident points
        if abs_diff_eq!(dot_a, dot_e) {
            distances.push(cross_a - cross_e);
        } else if abs_diff_eq!(dot_a, dot_f) {
            distances.push(cross_a - cross_f);
        } else if dot_a > ef_min && dot_a < ef_max {
            let mut d = a.distance_to_segment(other, reverse, false);
            if let Some(dhat) = d {
                if abs_diff_eq!(dhat, <<Self as Segment>::Point as Point2D>::Value::zero()) {
                    let db = b.distance_to_segment(other, reverse, true);
                    if let Some(db) = db {
                        if db < <<Self as Segment>::Point as Point2D>::Value::zero()
                            || abs_diff_eq!(
                                db * overlap,
                                <<Self as Segment>::Point as Point2D>::Value::zero()
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

        if abs_diff_eq!(dot_b, dot_e) {
            distances.push(cross_b - cross_e);
        } else if abs_diff_eq!(dot_b, dot_f) {
            distances.push(cross_b - cross_f);
        } else if dot_b > ef_min && dot_b < ef_max {
            let mut d = b.distance_to_segment(other, reverse, false);
            if let Some(dhat) = d {
                if abs_diff_eq!(dhat, <<Self as Segment>::Point as Point2D>::Value::zero()) {
                    let da = a.distance_to_segment(other, reverse, true);
                    if let Some(da) = da {
                        if da < <<Self as Segment>::Point as Point2D>::Value::zero()
                            || abs_diff_eq!(
                                da * overlap,
                                <<Self as Segment>::Point as Point2D>::Value::zero()
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
                if abs_diff_eq!(dhat, <<Self as Segment>::Point as Point2D>::Value::zero()) {
                    let df = f.distance_to_segment(self, reverse, true);
                    if let Some(df) = df {
                        if df < <<Self as Segment>::Point as Point2D>::Value::zero()
                            || abs_diff_eq!(
                                df * overlap,
                                <<Self as Segment>::Point as Point2D>::Value::zero()
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
                if abs_diff_eq!(dhat, <<Self as Segment>::Point as Point2D>::Value::zero()) {
                    let de = e.distance_to_segment(self, reverse, true);
                    if let Some(de) = de {
                        if de < <<Self as Segment>::Point as Point2D>::Value::zero()
                            || abs_diff_eq!(
                                de * overlap,
                                <<Self as Segment>::Point as Point2D>::Value::zero()
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
        assert!(intersection.is_some());
        let point = intersection.unwrap();
        assert!(abs_diff_eq!(point.x(), 2.5));
        assert!(abs_diff_eq!(point.y(), 2.5));

        // Test non-intersecting segments
        let no_intersection = segment1.intersects_segment(&segment3, false);
        assert!(no_intersection.is_none());

        // Test parallel segments
        let segment4 = Segment {
            start: Point2D { x: 1.0, y: 1.0 },
            end: Point2D { x: 6.0, y: 6.0 },
        };
        let parallel_intersection = segment1.intersects_segment(&segment4, false);
        assert!(parallel_intersection.is_none());

        // Test with infinite flag set to true
        let segment5 = Segment {
            start: Point2D { x: 0.0, y: 2.5 },
            end: Point2D { x: 1.5, y: 2.5 },
        };
        let infinite_intersection = segment1.intersects_segment(&segment5, true);
        assert!(infinite_intersection.is_some());
        let infinite_point = infinite_intersection.unwrap();
        assert!(abs_diff_eq!(infinite_point.x(), 2.5));
        assert!(abs_diff_eq!(infinite_point.y(), 2.5));
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
}

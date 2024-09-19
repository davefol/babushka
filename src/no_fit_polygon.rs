use crate::point::Point2D;
use crate::polygon::Polygon;
use crate::segment::Segment;
use approx::{abs_diff_eq, AbsDiffEq};
use itertools::Itertools;
use num_traits::{Float, Zero};

#[derive(Debug)]
enum TouchingType {
    A,
    B,
    C,
}

#[derive(Debug)]
struct Touching {
    tt: TouchingType,
    a: usize,
    b: usize,
}

#[derive(Clone, Copy, Debug)]
enum PolygonSource {
    A,
    B,
}

#[derive(Clone, Copy, Debug)]
struct Vector<P> {
    point: P,
    start: usize,
    end: usize,
    source: PolygonSource,
}
pub trait ComputeNoFitPolygon: Polygon {
    /// Return the vertex at the given index after transformations.
    fn get_vertex(&self, index: usize) -> <Self as Polygon>::Point;
    fn value_epsilon() -> <<<Self as Polygon>::Point as Point2D>::Value as AbsDiffEq>::Epsilon;

    fn no_fit_polygon(
        &self,
        other: &Self,
        inside: bool,
        search_edges: bool,
    ) -> Option<Vec<Vec<<Self as Polygon>::Point>>> {
        // we will be mucking with the offset of other so clone it
        let mut self_c = self.clone();
        self_c.set_offset(Zero::zero());

        let mut other = other.clone();
        other.set_offset(Zero::zero());

        // keep track of visited vertices
        let mut self_marked = vec![false; self_c.length()];
        let mut other_marked = vec![false; other.length()];

        let min_self_by_y = self_c
            .iter_vertices()
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
            self_c.search_start_point(&other, &self_marked, true, None)
        };
        // if cfg!(debug_assertions) {
        //     println!(
        //         "startpoint: {}, {}",
        //         &start_point.unwrap().x().to_f64().unwrap(),
        //         &start_point.unwrap().y().to_f64().unwrap()
        //     );
        // }

        let mut nfp_list = vec![];

        while let Some(current_start_point) = start_point {
            other.set_offset(current_start_point);

            // Touching Type, A index, B index
            let mut touchings: Vec<Touching>;
            let mut prev_vector = None::<Vector<<Self as Polygon>::Point>>;
            let mut nfp: Option<Vec<<Self as Polygon>::Point>> = Some(vec![other.get_vertex(0)]);

            let mut reference = other.get_vertex(0);
            let start = reference;
            let mut counter = 0;

            // Sanity check, prevent infinite loop
            // if cfg!(debug_assertions) {
            //     println!(
            //         "Reference point: {}, {}",
            //         &reference.x().to_f64().unwrap(),
            //         &reference.y().to_f64().unwrap()
            //     );
            // }
            while counter < 10 * (self_c.length() + other.length()) {
                touchings = vec![];

                // find touching vertices / edges
                // we need to carry around indices into self and other
                // to avoid dealing with lots of mutable refernces
                for ((idx_self_start, self_segment), (idx_other_start, other_segment)) in self_c
                    .iter_segments()
                    .enumerate()
                    .cartesian_product(other.iter_segments().enumerate())
                {
                    let idx_self_end = if idx_self_start == self_c.length() - 1 {
                        0
                    } else {
                        idx_self_start + 1
                    };
                    let idx_other_end = if idx_other_start == other.length() - 1 {
                        0
                    } else {
                        idx_other_start + 1
                    };
                    // if cfg!(debug_assertions) {
                    //     println!(
                    //         "A[{}]: {}, {}, B[{}]: {}, {}",
                    //         idx_self_start,
                    //         &self_segment.start().x().to_f64().unwrap(),
                    //         &self_segment.start().y().to_f64().unwrap(),
                    //         idx_other_start,
                    //         &other_segment.start().x().to_f64().unwrap(),
                    //         &other_segment.start().y().to_f64().unwrap(),
                    //     );
                    // }

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
                // if cfg!(debug_assertions) {
                //     println!("Touching:");
                // }
                for touching in touchings {
                    // if cfg!(debug_assertions) {
                    //     println!(
                    //         "\tTouching: {{ type: {}, A: {}, B: {} }}",
                    //         match touching.tt {
                    //             TouchingType::A => 0,
                    //             TouchingType::B => 1,
                    //             TouchingType::C => 2,
                    //         },
                    //         touching.a,
                    //         touching.b
                    //     )
                    // }
                    
                    let vertex_self = self_c.get_vertex(touching.a);
                    self_marked[touching.a] = true;

                    // adjacent self vertices
                    let prev_self_index = if touching.a == 0 {
                        self_c.length() - 1
                    } else {
                        touching.a - 1
                    };
                    let next_self_index = if touching.a == self_c.length() - 1 {
                        0
                    } else {
                        touching.a + 1
                    };

                    let prev_vertex_self = self_c.get_vertex(prev_self_index);
                    let next_vertex_self = self_c.get_vertex(next_self_index);

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
                                source: PolygonSource::A,
                            });
                            vectors.push(Vector {
                                point: next_vertex_self - vertex_self,
                                start: touching.a,
                                end: next_self_index,
                                source: PolygonSource::A,
                            });

                            // other's vectors need to be inverted
                            // TODO: check if we need to actually localize the other polygon
                            vectors.push(Vector {
                                point: vertex_other - prev_vertex_other, // - other.offset(),
                                start: prev_other_index,
                                end: touching.b,
                                source: PolygonSource::B,
                            });
                            vectors.push(Vector {
                                point: vertex_other - next_vertex_other, // - other.offset(),
                                start: next_other_index,
                                end: touching.b,
                                source: PolygonSource::B,
                            });
                        }
                        TouchingType::B => {
                            vectors.push(Vector {
                                point: vertex_self - vertex_other,
                                start: prev_self_index,
                                end: touching.a,
                                source: PolygonSource::A,
                            });
                            vectors.push(Vector {
                                point: prev_vertex_self - vertex_other,
                                start: touching.a,
                                end: prev_self_index,
                                source: PolygonSource::A,
                            });
                        }
                        TouchingType::C => {
                            vectors.push(Vector {
                                point: vertex_self - vertex_other,
                                start: prev_other_index,
                                end: touching.b,
                                source: PolygonSource::B,
                            });
                            vectors.push(Vector {
                                point: vertex_self - prev_vertex_other,
                                start: touching.b,
                                end: prev_other_index,
                                source: PolygonSource::B,
                            });
                        }
                    }
                }
                let mut translate = None::<Vector<<Self as Polygon>::Point>>;
                let mut max_d = <<Self as Polygon>::Point as Point2D>::Value::zero();

                // if cfg!(debug_assertions) {
                //     for vector in &vectors {
                //         println!(
                //             "Vector: {}, {}",
                //             &vector.point.x().to_f64().unwrap(),
                //             &vector.point.y().to_f64().unwrap(),
                //         );
                //     }
                // }
                for vector in vectors {
                    if vector.point.is_zero() {
                        continue;
                    }

                    // if this vector points us back to where we came from, ignore it.
                    // ie cross product = 0 and dot product < 0
                    if let Some(prev_vector) = &prev_vector {
                        if prev_vector.point.dot(&vector.point) < Zero::zero() {
                            // compare magnitude with unit vectors
                            let vector_unit = vector.point.normalized().unwrap();
                            let prev_unit = prev_vector.point.normalized().unwrap();

                            if (vector_unit.y() * prev_unit.x() - vector_unit.x() * prev_unit.y())
                                .abs()
                                < Self::Point::epsilon()
                            {
                                continue;
                            }
                        }
                    }

                    // i think this should return 0 if a slide is not possible
                    let mut d = self_c.slide_distance_on_polygon(&other, vector.point, true);
                    let vector_d2 = vector.point.dot(&vector.point);

                    if d.is_none() || d.unwrap() * d.unwrap() > vector_d2 {
                        d = Some(vector_d2.sqrt());
                    }

                    if let Some(d) = d {
                        // if cfg!(debug_assertions) {
                        //     println!(
                        //         "**** d: {}, max_d: {}, vector: {}, {}",
                        //         d.to_f64().unwrap(),
                        //         max_d.to_f64().unwrap(),
                        //         &vector.point.x().to_f64().unwrap(),
                        //         &vector.point.y().to_f64().unwrap(),
                        //     );
                        // }
                        if d > max_d {
                            max_d = d;
                            translate = Some(vector);
                        }
                    }
                }

                // if cfg!(debug_assertions) {
                //     if let Some(ref translate) = translate {
                //         println!(
                //             "translate: {}, {}",
                //             &translate.point.x().to_f64().unwrap(),
                //             &translate.point.y().to_f64().unwrap(),
                //         );
                //     }
                // }

                if translate.is_none()
                    || abs_diff_eq!(max_d, Zero::zero(), epsilon = Self::value_epsilon())
                {
                    // didn't close the loop, something went wrong here
                    nfp = None;
                    break;
                }
                if let Some(translate) = &translate {
                    match translate.source {
                        PolygonSource::A => {
                            self_marked[translate.start] = true;
                            self_marked[translate.end] = true;
                        }
                        PolygonSource::B => {
                            other_marked[translate.start] = true;
                            other_marked[translate.end] = true;
                        }
                    }
                }
                prev_vector = translate;

                // trim
                let vector_length_2 = translate.unwrap().point.dot(&translate.unwrap().point);
                if max_d * max_d < vector_length_2
                    && !abs_diff_eq!(
                        max_d * max_d,
                        vector_length_2,
                        epsilon = Self::value_epsilon()
                    )
                {
                    let scale = ((max_d * max_d) / vector_length_2).sqrt();
                    translate = translate.map(|mut translate| {
                        translate.point.set_x(translate.point.x() * scale);
                        translate.point.set_y(translate.point.y() * scale);
                        translate
                    });
                }

                reference.set_x(reference.x() + translate.unwrap().point.x());
                reference.set_y(reference.y() + translate.unwrap().point.y());

                // we've made a full loop
                if abs_diff_eq!(reference, start) {
                    break;
                }

                // if self and other start on a touching horizontal line,
                // the end point may not be the start point
                let mut looped = false;

                if let Some(nfp) = &nfp {
                    if !nfp.is_empty() {
                        for i in 0..nfp.len() - 1 {
                            if abs_diff_eq!(reference, nfp[i]) {
                                looped = true;
                            }
                        }
                    }
                }

                if looped {
                    break;
                }

                if let Some(nfp) = nfp.as_mut() {
                    nfp.push(reference);
                }

                other.set_offset(other.offset() + translate.unwrap().point);

                counter += 1;
            }

            if let Some(nfp) = nfp {
                if !nfp.is_empty() {
                    nfp_list.push(nfp);
                }
            }

            if !search_edges {
                break;
            }

            start_point =
                self_c.search_start_point(&other, &self_marked, inside, Some(nfp_list.clone()))
        }
        nfp_list
            .iter_mut()
            .for_each(|n| n.iter_mut().for_each(|v| *v = v.translate(&self.offset())));
        Some(nfp_list)
    }

    fn search_start_point(
        &self,
        other: &Self,
        self_marked: &Vec<bool>,
        inside: bool,
        nfp: Option<Vec<Vec<<Self as Polygon>::Point>>>,
    ) -> Option<<Self as Polygon>::Point> {
        // let self_clone = self.clone();
        let mut other = other.clone();
        let mut self_marked = self_marked.clone();

        // since we are iterating over every segment, the index i will be the index of
        // the starting point of that segment
        for (i, self_segment) in self.iter_segments().enumerate() {
            // if cfg!(debug_assertions) {
            //     println!("\t\touter loop: {i}");
            // }
            if !self_marked[i] {
                self_marked[i] = true;

                for j in 0..other.length() {
                    // if cfg!(debug_assertions) {
                    //     println!("\t\tinner loop: {j}");
                    // }
                    other.set_offset(*self_segment.start() - (other.get_vertex(j) - other.offset()));

                    let mut other_inside = None::<bool>;
                    // TODO: This kinda looks suspicious
                    for kp in other.iter_vertices() {
                        // if cfg!(debug_assertions) {
                        //     println!(
                        //         "\t\tkp: {}, {}",
                        //         &kp.x().to_f64().unwrap(),
                        //         &kp.y().to_f64().unwrap(),
                        //     );
                        // }
                        if let Some(in_poly) = kp.in_polygon(self) {
                            other_inside = Some(in_poly);
                            break;
                        }
                    }
                    // if cfg!(debug_assertions) {
                    //     println!(
                    //         "\t\tBinside: {}",
                    //         match other_inside {
                    //             Some(true) => "true",
                    //             Some(false) => "false",
                    //             None => "null"
                    //         }
                    //     )
                    // }

                    // A and B are the same
                    let Some(mut other_inside) = other_inside else {
                        return None;
                    };

                    let mut start_point = other.offset();
                    if ((other_inside && inside) || (!other_inside && !inside))
                        && !self.intersects_polygon(&other)
                        && !Self::in_nfp(&start_point, &nfp)
                    {
                        // if cfg!(debug_assertions) {
                        //     println!("\t\tfirst early return");
                        // }
                        return Some(start_point);
                    }

                    // Slide other along vector
                    let mut v = *self_segment.end() - *self_segment.start();
                    // if cfg!(debug_assertions) {
                    //     println!(
                    //         "\t\tv: {}, {}",
                    //         v.x().to_f64().unwrap(),
                    //         v.y().to_f64().unwrap(),
                    //     );
                    // }
                    let d1 = self.project_distance_on_polygon(&other, v);
                    let d2 = other.project_distance_on_polygon(self, -v);

                    let d = if d1.is_none() && d2.is_none() {
                        None
                    } else if d1.is_none() {
                        d2
                    } else if d2.is_none() {
                        d1
                    } else {
                        Some(d1.unwrap().min(d2.unwrap()))
                    };
                    // if cfg!(debug_assertions) {
                    //     println!(
                    //         "\t\td: {}, d1: {}, d2: {}",
                    //         match d { Some(d) => d.to_f64().unwrap(), None => std::f64::NAN},
                    //         match d1 { Some(d) => d.to_f64().unwrap(), None => std::f64::NAN},
                    //         match d2 { Some(d) => d.to_f64().unwrap(), None => std::f64::NAN},
                    //     );
                    // } 

                    // only slide until no longer negative
					// console.log(`d !== null: ${d !== null}, !almosteq: ${!_almostEqual(d, 0)}, d > 0: ${d}`);
                    // if cfg!(debug_assertions) {
                    //     println!(
                    //         "\t\td !== null: {}, !almosteq: {}, d > 0: {}",
                    //         &d.is_some(),
                    //         match d {
                    //             Some(d) => format!("{}", !abs_diff_eq!(d, Zero::zero(), epsilon = Self::value_epsilon())),
                    //             None => "null".to_string()
                    //         },
                    //         match d {
                    //             Some(d) => format!("{}", d > Zero::zero()),
                    //             None => "null".to_string()
                    //         }
                    //     );
                    // }
                    let Some(d) = d else {
                        // if cfg!(debug_assertions) {
                        //     println!("continue");
                        // }
                        continue;
                    };
                    if !(!abs_diff_eq!(d, Zero::zero(), epsilon = Self::value_epsilon())
                        && d > Zero::zero())
                    {
                        continue;
                    }

                    let vd2 = v.dot(&v);
                    if d * d < vd2 && !abs_diff_eq!(d * d, vd2, epsilon = Self::value_epsilon()) {
                        let vd = v.dot(&v).sqrt();
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
                    // if cfg!(debug_assertions) {
                    //     println!("\t\tsuspect Binside: {}", other_inside);
                    // }
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
                if abs_diff_eq!(p, point) {
                    return true;
                }
            }
        }

        false
    }
}

mod tests {
    #[test]
    fn test_no_fit_polygon_one_convex_no_holes_outside() {
        use super::ComputeNoFitPolygon;
        use crate::kernelf64::*;
        use crate::polygon::Polygon as _;
        let mut polygon1 = Polygon::from(vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 2.0, y: 4.0 },
            Point2D { x: 2.0, y: 2.0 },
            Point2D { x: 2.9, y: 1.0 },
            Point2D { x: 5.0, y: 1.0 },
            Point2D { x: 5.0, y: 0.0 },
        ]);
        polygon1.translate(5.0, 5.0);

        let mut polygon2 = Polygon::from(vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 1.0, y: 1.0 },
            Point2D { x: 1.0, y: -1.0 },
        ]);
        polygon2.translate(8.0, 8.0);

        let expected_nfp = vec![vec![
            Point2D { x: 4.0, y: 4.0 },
            Point2D { x: 9.0, y: 4.0 },
            Point2D { x: 10.0, y: 5.0 },
            Point2D { x: 10.0, y: 6.0 },
            Point2D { x: 9.0, y: 7.0 },
            Point2D { x: 7.0, y: 7.0 },
            Point2D { x: 7.0, y: 9.0 },
            Point2D { x: 6.0, y: 10.0 },
            Point2D { x: 4.0, y: 6.0 },
        ]];
        let nfp = polygon1.no_fit_polygon(&polygon2, false, false);
        assert_eq!(nfp, Some(expected_nfp));
    }

    #[test]
    fn test_no_fit_polygon_one_convex_no_holes_inside() {
        use super::ComputeNoFitPolygon;
        use crate::kernelf64::*;
        use crate::polygon::Polygon as _;

        let mut polygon1 = Polygon::from(vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 2.0, y: 4.0 },
            Point2D { x: 2.0, y: 2.0 },
            Point2D { x: 2.9, y: 1.0 },
            Point2D { x: 5.0, y: 1.0 },
            Point2D { x: 5.0, y: 0.0 },
        ]);
        polygon1.translate(5.0, 5.0);

        let mut polygon2 = Polygon::from(vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 1.0, y: 1.0 },
            Point2D { x: 1.0, y: -1.0 },
        ]);
        polygon2.translate(8.0, 8.0);

        let expected_nfp = vec![vec![
            Point2D { x: 5.5, y: 6.0 },
            Point2D { x: 6.0, y: 7.0 },
            Point2D { x: 6.0, y: 6.0 },
        ]];
        let nfp = polygon1.no_fit_polygon(&polygon2, true, false);
        assert_eq!(nfp, Some(expected_nfp));
    }

    #[test]
    fn test_no_fit_polygon_one_convex_no_holes_outside_rotated() {
        use super::ComputeNoFitPolygon;
        use crate::kernelf64::*;
        use crate::polygon::Polygon as _;
        let mut polygon1 = Polygon::from(vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 2.0, y: 4.0 },
            Point2D { x: 2.0, y: 2.0 },
            Point2D { x: 2.9, y: 1.0 },
            Point2D { x: 5.0, y: 1.0 },
            Point2D { x: 5.0, y: 0.0 },
        ]);

        let mut polygon2 = Polygon::from(vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 1.0, y: 1.0 },
            Point2D { x: 1.0, y: -1.0 },
        ]);
        polygon2.set_rotation(0.9);
        let expected_nfp = vec![vec![
            Point2D {
                x: 0.16171694135681902,
                y: -1.4049368778981477,
            },
            Point2D {
                x: 5.161716941356819,
                y: -1.4049368778981477,
            },
            Point2D {
                x: 5.161716941356819,
                y: -0.4049368778981477,
            },
            Point2D { x: 5.0, y: 1.0 },
            Point2D { x: 2.9, y: 1.0 },
            Point2D {
                x: 2.161716941356819,
                y: 1.8203145096035347,
            },
            Point2D {
                x: 2.161716941356819,
                y: 2.5950631221018523,
            },
            Point2D { x: 2.0, y: 4.0 },
            Point2D {
                x: 0.5950631221018523,
                y: 3.8382830586431806,
            },
            Point2D {
                x: -1.4049368778981477,
                y: -0.16171694135681935,
            },
        ]];
        let nfp = polygon1.no_fit_polygon(&polygon2, false, false);
        assert_eq!(nfp, Some(expected_nfp));

        polygon1.set_rotation(0.5);
        polygon2.set_rotation(0.0);
        let expected_nfp = vec![vec![
            Point2D { x: -1.0, y: -1.0 },
            Point2D {
                x: 3.387912809451864,
                y: 1.397127693021015,
            },
            Point2D {
                x: 4.387912809451864,
                y: 2.397127693021015,
            },
            Point2D {
                x: 3.908487270847661,
                y: 3.274710254911388,
            },
            Point2D {
                x: 2.908487270847661,
                y: 4.274710254911388,
            },
            Point2D {
                x: 1.0655638908778777,
                y: 3.267916623842561,
            },
            Point2D {
                x: 0.3578259797002594,
                y: 3.516663223515923,
            },
            Point2D {
                x: -0.1625370306360665,
                y: 4.469181324769897,
            },
            Point2D {
                x: -1.1625370306360665,
                y: 5.469181324769897,
            },
            Point2D {
                x: -1.1625370306360665,
                y: 3.469181324769897,
            },
        ]];
        let nfp = polygon1.no_fit_polygon(&polygon2, false, false);
        assert_eq!(nfp, Some(expected_nfp));
    }
}

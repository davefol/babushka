use crate::{no_fit_polygon::ComputeNoFitPolygon, polygon::Polygon};

#[derive(Debug, Clone)]
pub struct MultiPolygon<P: Polygon> {
    outer: P,
    holes: Vec<P>,
}

impl<P: Polygon> MultiPolygon<P> {
    pub fn new(outer: P, holes: Vec<P>) -> Self {
        MultiPolygon { outer, holes }
    }

    pub fn outer(&self) -> &P {
        &self.outer
    }

    pub fn holes(&self) -> &[P] {
        &self.holes
    }

    pub fn for_each_polygon<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut P),
    {
        f(&mut self.outer);
        for hole in &mut self.holes {
            f(hole);
        }
    }
}

impl<P: Polygon + ComputeNoFitPolygon> MultiPolygon<P> {
    pub fn no_fit_polygon(&self, other: &Self) -> Vec<Vec<P::Point>> {
        let mut nfp_list = vec![];
        nfp_list.extend(
            self.outer()
                .no_fit_polygon(other.outer(), false, false)
                .unwrap(),
        );
        for hole in self.holes() {
            nfp_list.extend(hole.no_fit_polygon(other.outer(), true, false).unwrap());
        }

        nfp_list
    }
}

mod tests {
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_no_fit_polygon() {
        use super::MultiPolygon;
        use crate::point::Point2D as _;
        use crate::polygon::Polygon as _;
        use itertools::izip;
        use crate::kernelf64::*;
        use std::f64::consts::PI;

        let n_points = 16;
        let mut outer = Polygon::from((0..n_points).map(|i| {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / n_points as f64;
            let x = 100.0 * angle.cos();
            let y = 100.0 * angle.sin();
            Point2D::from_xy(x, y)
        }));
        outer.set_offset(Point2D::from_xy(400.0, 300.0));

        let mut inner = Polygon::from((0..n_points).map(|i| {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / n_points as f64;
            let x = 50.0 * angle.cos();
            let y = 50.0 * angle.sin();
            Point2D::from_xy(x, y)
        }));
        inner.set_offset(Point2D::from_xy(400.0, 300.0));

        let piece_0 = MultiPolygon::new(outer, vec![inner]);

        let mut square = Polygon::from(vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 20.0, y: 0.0 },
            Point2D { x: 20.0, y: 20.0 },
            Point2D { x: 0.0, y: 20.0 },
        ]);
        square.set_offset(Point2D::from_xy(390.0, 290.0));
        let piece_1 = MultiPolygon::new(square, vec![]);

        let nfp = piece_0.no_fit_polygon(&piece_1);

        let expected = vec![
            vec![
                Point2D { x: 400.0, y: 180.0 },
                Point2D {
                    x: 438.268343236509,
                    y: 187.61204674887134,
                },
                Point2D {
                    x: 470.71067811865476,
                    y: 209.28932188134524,
                },
                Point2D {
                    x: 492.38795325112864,
                    y: 241.73165676349095,
                },
                Point2D { x: 500.0, y: 280.0 },
                Point2D { x: 500.0, y: 300.0 },
                Point2D {
                    x: 492.3879532511287,
                    y: 338.268343236509,
                },
                Point2D {
                    x: 470.71067811865476,
                    y: 370.71067811865476,
                },
                Point2D {
                    x: 438.268343236509,
                    y: 392.3879532511287,
                },
                Point2D { x: 400.0, y: 400.0 },
                Point2D { x: 380.0, y: 400.0 },
                Point2D {
                    x: 341.731656763491,
                    y: 392.3879532511287,
                },
                Point2D {
                    x: 309.28932188134524,
                    y: 370.71067811865476,
                },
                Point2D {
                    x: 287.6120467488713,
                    y: 338.268343236509,
                },
                Point2D { x: 280.0, y: 300.0 },
                Point2D { x: 280.0, y: 280.0 },
                Point2D {
                    x: 287.6120467488713,
                    y: 241.73165676349106,
                },
                Point2D {
                    x: 309.28932188134524,
                    y: 209.28932188134524,
                },
                Point2D {
                    x: 341.73165676349095,
                    y: 187.61204674887136,
                },
                Point2D { x: 380.0, y: 180.0 },
            ],
            vec![
                Point2D {
                    x: 428.0108763262034,
                    y: 290.0,
                },
                Point2D {
                    x: 426.19397662556435,
                    y: 280.8658283817455,
                },
                Point2D {
                    x: 415.3553390593274,
                    y: 264.6446609406726,
                },
                Point2D {
                    x: 399.1341716182545,
                    y: 253.80602337443565,
                },
                Point2D {
                    x: 390.0,
                    y: 251.9891236737966,
                },
                Point2D {
                    x: 380.8658283817455,
                    y: 253.80602337443568,
                },
                Point2D {
                    x: 364.6446609406726,
                    y: 264.6446609406726,
                },
                Point2D {
                    x: 353.80602337443565,
                    y: 280.86582838174553,
                },
                Point2D {
                    x: 351.9891236737966,
                    y: 290.0,
                },
                Point2D {
                    x: 353.80602337443565,
                    y: 299.13417161825447,
                },
                Point2D {
                    x: 364.6446609406726,
                    y: 315.3553390593274,
                },
                Point2D {
                    x: 380.86582838174553,
                    y: 326.19397662556435,
                },
                Point2D {
                    x: 390.0,
                    y: 328.0108763262034,
                },
                Point2D {
                    x: 399.13417161825447,
                    y: 326.19397662556435,
                },
                Point2D {
                    x: 415.3553390593274,
                    y: 315.3553390593274,
                },
                Point2D {
                    x: 426.19397662556435,
                    y: 299.13417161825447,
                },
            ],
        ];
        for (a, b) in izip!(nfp, expected) {
            for (i, j) in izip!(a, b) {
                assert_abs_diff_eq!(i, j);
            }
        }
    }
}

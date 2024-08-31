#[derive(Clone, Copy)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl crate::point::Point2D for Point2D {
    type Value = f64;

    fn x(&self) -> Self::Value {
        self.x
    }

    fn y(&self) -> Self::Value {
        self.y
    }

    fn set_x(&mut self, x: Self::Value) {
        self.x = x;
    }

    fn set_y(&mut self, y: Self::Value) {
        self.y = y;
    }
}

pub struct Segment {
    pub start: Point2D,
    pub end: Point2D,
}
impl crate::segment::Segment for Segment {
    type Point = Point2D;
    fn start(&self) -> &Self::Point {
        &self.start
    }
    fn end(&self) -> &Self::Point {
        &self.end
    }
}

pub struct Polygon {
    pub vertices: Vec<Point2D>,
    pub offset: Point2D,
}

impl crate::polygon::Polygon for Polygon {
    type Point = Point2D;
    type Segment = Segment;
    fn iter_vertices(&self) -> impl Iterator<Item = &<Self as crate::polygon::Polygon>::Point> {
        self.vertices.iter()
    }
    fn iter_mut_vertices(
        &mut self,
    ) -> impl Iterator<Item = &mut <Self as crate::polygon::Polygon>::Point> {
        self.vertices.iter_mut()
    }

    fn iter_segments(&self) -> impl Iterator<Item = Segment> {
        self.vertices
            .iter()
            .zip(self.vertices.iter().cycle().skip(1))
            .take(self.vertices.len())
            .map(|window| Segment {
                start: *window.0,
                end: *window.1,
            })
    }

    fn offset(&self) -> &Self::Point {
        &self.offset
    }

    fn length(&self) -> usize {
        self.vertices.len()
    }
}

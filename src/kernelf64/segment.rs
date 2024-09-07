use super::Point2D;
use std::ops::Add;

#[derive(Clone, Copy)]
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

impl From<(Point2D, Point2D)> for Segment {
    fn from((start, end): (Point2D, Point2D)) -> Self {
        Self { start, end }
    }
}

impl Add<Point2D> for Segment {
    type Output = Self;
    fn add(self, other: Point2D) -> Self::Output {
        Self {
            start: self.start + other,
            end: self.end + other,
        }
    }
}

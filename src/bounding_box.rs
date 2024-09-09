use crate::point::Point2D;
use num_traits::Float;

#[derive(Debug)]
pub struct BoundingBox<T> {
    pub min_x: T,
    pub min_y: T,
    pub max_x: T,
    pub max_y: T,
}

impl<T> BoundingBox<T>
where
    T: Float,
{
    pub fn center<P>(&self) -> P
    where
        P: Point2D<Value = T>,
    {
        P::from_xy(
            (self.min_x + self.max_x) / (T::one() + T::one()),
            (self.min_y + self.max_y) / (T::one() + T::one()),
        )
    }
}

impl<T: Copy + std::ops::Sub<Output = T>> BoundingBox<T> {
    pub fn width(&self) -> T {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> T {
        self.max_y - self.min_y
    }
}

#[derive(Debug)]
pub struct BoundingBox<T> {
    pub min_x: T,
    pub min_y: T,
    pub max_x: T,
    pub max_y: T,
}

impl<T: Copy + std::ops::Sub<Output = T>> BoundingBox<T> {
    pub fn width(&self) -> T {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> T {
        self.max_y - self.min_y
    }
}

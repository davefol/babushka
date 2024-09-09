use approx::AbsDiffEq;
use num_traits::Zero;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq)]
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

    fn from_xy(x: Self::Value, y: Self::Value) -> Self {
        Self { x, y }
    }

    fn set_x(&mut self, x: Self::Value) {
        self.x = x;
    }

    fn set_y(&mut self, y: Self::Value) {
        self.y = y;
    }

    fn epsilon() -> Self::Value {
        1e-9
    }

    fn value_epsilon() -> <Self::Value as AbsDiffEq>::Epsilon {
        1e-9
    }
}

impl From<(f64, f64)> for Point2D {
    fn from(tuple: (f64, f64)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl Add for Point2D {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point2D {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Neg for Point2D {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Zero for Point2D {
    fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
}

impl AbsDiffEq<Point2D> for Point2D {
    type Epsilon = f64;
    fn abs_diff_eq(&self, other: &Point2D, epsilon: Self::Epsilon) -> bool {
        self.x.abs_diff_eq(&other.x, epsilon) && self.y.abs_diff_eq(&other.y, epsilon)
    }

    fn default_epsilon() -> Self::Epsilon {
        1e-9
    }
}

impl Div<f64> for Point2D {
    type Output = Self;
    fn div(self, other: f64) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Mul<f64> for Point2D {
    type Output = Self;
    fn mul(self, other: f64) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

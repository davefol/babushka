use crate::polygon::Polygon;

use super::Clippable;

pub enum ClipOperation {
    Intersection,
}

pub fn clip<P>(subject: &P, clipper: &P, operation: ClipOperation) -> P 
where P: Polygon + Clippable
{
    unimplemented!()
}
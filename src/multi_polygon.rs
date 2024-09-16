use crate::polygon::Polygon;

#[derive(Debug, Clone)]
pub struct MultiPolygon<P: Polygon> {
    outer: P,
    holes: Vec<P>,
}

impl <P: Polygon> MultiPolygon<P> {
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

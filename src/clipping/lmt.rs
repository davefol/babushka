use crate::{point::Point2D, polygon::Polygon, segment::Segment};
use itertools::izip;

use super::Clippable;

struct LMTNode<P: Polygon + Clippable> {
    y: <<P as Polygon>::Point as Point2D>::Value,
    bounds: Vec<<P as Polygon>::Segment>,
}

struct LocalMinimaTable<P: Polygon + Clippable> {
    nodes: Vec<LMTNode<P>>,
}

/// Represents an edge
/// This is a linked list of nodes sorted in ascending order of y-coordinate
/// Each node points to a list of bounds that start at the y coordinate.
/// Thus each node corresponds to the y coordinate of one or more local minima.
///  The LM  is built at the time of forming the bounds, prior to clipping.
impl<P> LocalMinimaTable<P>
where
    P: Polygon + Clippable,
{
    pub fn new(polygon: &P) -> Self {
        // find all local minima
        let local_minima_indices: Vec<usize> = izip!(
            0..polygon.length(),
            (0..polygon.length()).cycle().skip(1).take(polygon.length()),
            (0..polygon.length()).cycle().skip(2).take(polygon.length()),
        )
        .filter(|(prev, current, next)| {
            polygon.get_vertex(*prev).y() >= polygon.get_vertex(*current).y() && polygon.get_vertex(*next).y() > polygon.get_vertex(*current).y()
        })
        .map(|(_, current, _)| current)
        .collect();

        let mut min_max: Vec<(usize, usize, usize)> = local_minima_indices.iter().map(|min| {
            // find the next local maxima in the forward direction
            let max_forward = izip!(
                (0..polygon.length()).cycle().skip(min + 1).take(polygon.length()),
                (0..polygon.length()).cycle().skip(min + 2).take(polygon.length()),
            )
            .filter(|(current, next)| {
                // we have stopped increasing from the local minima, 
                // so current is the local maxima
                polygon.get_vertex(*current).y() > polygon.get_vertex(*next).y()
            })
            .next()
            .map(|(current, _)| current)
            .unwrap();
            
            // find the next local maxima in the reverse direction
            let max_reverse = izip!(
                (0..polygon.length()).rev().cycle().skip(min + 1).take(polygon.length()),
                (0..polygon.length()).rev().cycle().skip(min + 2).take(polygon.length()),
            )
            .filter(|(current, prev)| {
                // we have stopped increasing from the local minima, 
                // so current is the local maxima
                polygon.get_vertex(*current).y() > polygon.get_vertex(*prev).y()
            })
            .next()
            .map(|(current, _)| current)
            .unwrap();


            (*min, max_forward, max_reverse)
        }).collect();

        min_max.sort_by(|a, b| {
            polygon.get_vertex(a.0).y().partial_cmp(&polygon.get_vertex(b.0).y()).unwrap()
        });

        let nodes = vec![];

        Self { nodes }
    }
}

mod tests {
    #[test]
    fn test_local_minima_table() {
        use super::LocalMinimaTable;
        use crate::kernelf64::*;
        Polygon::from_tuples(vec![
            (0.0, 0.0),
            (3.0, 1.0),
            (4.0, 2.0),
            (5.0, 4.0),
            (4.0, 5.0),
            (2.0, 4.0),
            (1.0, 3.0),
            (0.0, 4.0),
            (1.0, 5.0),
            (0.0, 6.0),
            (-1.0, 5.0),
            (-2.0, 3.0),
            (-1.0, 1.0),
        ])
    }
}

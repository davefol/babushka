//! 2D Polygon Clipping library based on Vatti 1992

pub struct LocalMinimaTable {

}

/// Represents an edge
/// This is a linked list of nodes sorted in ascending order of y-coordinate
/// Each node points to a list of bounds that start at the y coordinate. 
/// Thus each node corresponds to the y coordinate of one or more local minima.
///  The LM  is built at the time of forming the bounds, prior to clipping. 
impl LocalMinimaTable {
    pub fn new() -> Self {
        Self {}
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
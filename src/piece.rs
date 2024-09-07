use crate::polygon::Polygon;
use petgraph::graph::{Graph, NodeIndex};

pub struct Piece<P: Polygon> {
    graph: Graph<P, ()>,
    roots: Vec<NodeIndex>,
}

impl<P: Polygon> Piece<P> {
    pub fn new(root_polygon: P) -> Self {
        let mut graph = Graph::new();
        let root = graph.add_node(root_polygon);
        Piece {
            graph,
            roots: vec![root],
        }
    }

    pub fn for_each_polygon<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut P),
    {
        for node_index in self.graph.node_indices() {
            let polygon = self.graph.node_weight_mut(node_index).unwrap();
            f(polygon); 
        }
    }

    pub fn node_indices(&self) -> impl Iterator<Item = NodeIndex> {
        self.graph.node_indices()
    }

    pub fn from_roots(root_polygons: impl IntoIterator<Item = P>) -> Self {
        let mut graph = Graph::new();
        let roots = root_polygons
            .into_iter()
            .map(|polygon| graph.add_node(polygon))
            .collect::<Vec<_>>();
        Piece { graph, roots }
    }

    pub fn add_root(&mut self, polygon: P) -> NodeIndex {
        let root = self.graph.add_node(polygon);
        self.roots.push(root);
        root
    }

    pub fn add_child(&mut self, parent: NodeIndex, child: P) -> NodeIndex {
        let child_index = self.graph.add_node(child);
        self.graph.add_edge(parent, child_index, ());
        child_index
    }

    pub fn get_roots(&self) -> &[NodeIndex] {
        &self.roots
    }

    pub fn get_polygon(&self, index: NodeIndex) -> Option<&P> {
        self.graph.node_weight(index)
    }

    pub fn get_polygon_mut(&mut self, index: NodeIndex) -> Option<&mut P> {
        self.graph.node_weight_mut(index)
    }

    pub fn iter_children(&self, parent: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph.neighbors(parent)
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn node_depth(&self, node: NodeIndex) -> Option<usize> {
        if self
            .graph
            .neighbors_directed(node, petgraph::Direction::Incoming)
            .count()
            == 0
            && self
                .graph
                .neighbors_directed(node, petgraph::Direction::Outgoing)
                .count()
                == 0
        {
            return None;
        }

        let mut depth = 0;
        let mut current = node;

        while let Some(parent) = self
            .graph
            .neighbors_directed(current, petgraph::Direction::Incoming)
            .next()
        {
            depth += 1;
            current = parent;
        }

        Some(depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernelf64::polygon::Polygon as KernelPolygon;
    use crate::kernelf64::Point2D;

    #[test]
    fn test_rectangle_with_triangular_hole() {
        // Create a rectangle
        let rectangle = KernelPolygon::from(vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 4.0, y: 0.0 },
            Point2D { x: 4.0, y: 3.0 },
            Point2D { x: 0.0, y: 3.0 },
        ]);

        // Create a triangular hole
        let triangle = KernelPolygon::from(vec![
            Point2D { x: 1.0, y: 1.0 },
            Point2D { x: 3.0, y: 1.0 },
            Point2D { x: 2.0, y: 2.0 },
        ]);

        // Create a piece with the rectangle as the root
        let mut piece = Piece::new(rectangle);

        // Add the triangular hole as a child of the rectangle
        let rectangle_index = piece.get_roots()[0];
        let triangle_index = piece.add_child(rectangle_index, triangle);

        // Verify the structure
        assert_eq!(piece.get_roots().len(), 1);
        assert_eq!(piece.iter_children(rectangle_index).count(), 1);

        // Verify the polygons
        let root_polygon = piece.get_polygon(rectangle_index).unwrap();
        assert_eq!(root_polygon.length(), 4);

        let hole_polygon = piece.get_polygon(triangle_index).unwrap();
        assert_eq!(hole_polygon.length(), 3);

        // Verify that the root (rectangle) is at depth 0
        assert_eq!(piece.node_depth(rectangle_index).unwrap(), 0);

        // Verify that the child (triangle) is at depth 1
        assert_eq!(piece.node_depth(triangle_index).unwrap(), 1);
    }
}

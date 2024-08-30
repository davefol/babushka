use crate::polygon::Polygon;
use crate::point::Point2D;

pub fn calculate_nfp<T>(stationary: &Polygon<T>, orbiting: &Polygon<T>) -> Polygon<T>
where
    T: Clone + std::ops::Add<Output = T> + std::ops::Sub<Output = T>,
{
    let mut nfp = Polygon { vertices: Vec::new() };

    for i in 0..stationary.vertices.len() {
        let reference_vertex = &stationary.vertices[i];
        
        for j in 0..orbiting.vertices.len() {
            let orbiting_vertex = &orbiting.vertices[j];
            let nfp_vertex = Point2D {
                x: reference_vertex.x.clone() - orbiting_vertex.x.clone(),
                y: reference_vertex.y.clone() - orbiting_vertex.y.clone(),
            };
            nfp.vertices.push(nfp_vertex);
        }
    }

    // Sort and remove duplicate vertices if necessary
    // Implement convex hull algorithm if needed

    nfp
}

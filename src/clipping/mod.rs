//! 2D Polygon Clipping library based on Vatti 1992

use std::{cell::RefCell, rc::Rc};

use approx::AbsDiffEq;
use num_traits::{Float, Zero};

use crate::point::Point2D;
use crate::polygon::Polygon;

mod clip;

pub use self::clip::{clip, ClipOperation};

const ABOVE: usize = 0;
const BELOW: usize = 1;
const CLIP: usize = 0;
const SUBJ: usize = 1;
const LEFT: usize = 0;
const RIGHT: usize = 1;

pub trait Clippable: Polygon {
    fn get_vertex(&self, index: usize) -> <Self as Polygon>::Point;
}

#[derive(Debug)]
struct SbTree<P: Point2D> {
    y: P::Value,
    less: Option<Rc<RefCell<SbTree<P>>>>,
    more: Option<Rc<RefCell<SbTree<P>>>>,
}

impl<P: Point2D> SbTree<P> {
    fn new(y: P::Value) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            y,
            less: None,
            more: None,
        }))
    }
}

#[derive(Debug, Clone)]
struct EdgeNode<P: Point2D> {
    pub vertex: P,
    pub bot: P,
    pub top: P,
    pub xb: P::Value,
    pub xt: P::Value,
    pub dx: P::Value,
    pub polygon_type: PolygonType,
    pub bundle: [[bool; 2]; 2],
    pub bside: [Side; 2],
    pub bstate: [BundleState; 2],
    pub outp: [Option<Rc<RefCell<PolygonNode<P>>>>; 2],
    pub prev: Option<Rc<RefCell<EdgeNode<P>>>>,
    pub next: Option<Rc<RefCell<EdgeNode<P>>>>,
    pub pred: Option<Rc<RefCell<EdgeNode<P>>>>,
    pub succ: Option<Rc<RefCell<EdgeNode<P>>>>,
    pub next_bound: Option<Rc<RefCell<EdgeNode<P>>>>,
}

#[derive(Debug)]
struct VertexNode<P: Point2D> {
    pub point: P,
    pub next: Option<Rc<RefCell<VertexNode<P>>>>,
}

#[derive(Debug)]
struct PolygonNode<P: Point2D> {
    pub active: bool,
    pub hole: bool,
    pub vertices: [Option<Rc<RefCell<VertexNode<P>>>>; 2], // Left and right vertex lists
    pub next: Option<Rc<RefCell<PolygonNode<P>>>>,
    pub proxy: Option<Rc<RefCell<PolygonNode<P>>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VertexType {
    Nul,
    Emx,
    Eli,
    Ted,
    Eri,
    Red,
    Imm,
    Imn,
    Emn,
    Emm,
    Led,
    Ili,
    Bed,
    Iri,
    Imx,
    Ful,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HState {
    NH,
    BH,
    TH,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BundleState {
    Unbundled,
    BundleHead,
    BundleTail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PolygonType {
    Clip,
    Subj,
}

impl<P: Point2D> EdgeNode<P> {
    fn new(vertex: P, bot: P, top: P, polygon_type: PolygonType) -> Rc<RefCell<Self>> {
        let dy = top.y() - bot.y();
        let zero = P::Value::zero();

        let dx = if dy.abs_diff_eq(&zero, P::value_epsilon()) {
            zero
        } else {
            (top.x() - bot.x()) / dy
        };

        Rc::new(RefCell::new(Self {
            vertex,
            bot,
            top,
            xb: bot.x(),
            xt: top.x(),
            dx,
            polygon_type,
            bundle: [[false; 2]; 2],
            bside: [Side::Left; 2],
            bstate: [BundleState::Unbundled; 2],
            outp: [None, None],
            prev: None,
            next: None,
            pred: None,
            succ: None,
            next_bound: None,
        }))
    }
}

fn add_edge_to_aet<P: Point2D>(
    aet: &mut Option<Rc<RefCell<EdgeNode<P>>>>,
    edge: Rc<RefCell<EdgeNode<P>>>,
    prev: Option<Rc<RefCell<EdgeNode<P>>>>,
) {
    if let Some(aet_edge_rc) = aet.as_mut() {
        // Clone `aet_edge_rc` before borrowing to avoid move issues
        let aet_edge_clone = aet_edge_rc.clone();

        // Begin scope of borrow
        {
            let mut aet_edge = aet_edge_rc.borrow_mut();
            let edge_xb = edge.borrow().xb;
            let aet_xb = aet_edge.xb;

            if edge_xb < aet_xb
                || (edge_xb.abs_diff_eq(&aet_xb, P::value_epsilon())
                    && edge.borrow().dx < aet_edge.dx)
            {
                // Insert edge before current AET edge
                edge.borrow_mut().prev = prev;
                edge.borrow_mut().next = Some(aet_edge_clone.clone());
                aet_edge.prev = Some(edge.clone());
                // Mutable borrow of `aet_edge` ends here
            } else {
                // Store `next` before ending the mutable borrow
                let next = aet_edge.next.clone();
                // Mutable borrow of `aet_edge` ends here
                drop(aet_edge); // Explicitly drop the mutable borrow

                // Now it's safe to clone `aet_edge_clone`
                add_edge_to_aet(&mut next.clone(), edge.clone(), Some(aet_edge_clone));
                // Update `aet_edge_rc.next` after recursive call
                aet_edge_rc.borrow_mut().next = next;
                return; // Return early to avoid reassigning `*aet`
            }
        }
        // Now it's safe to assign to `*aet`
        *aet = Some(edge);
    } else {
        // Append edge to AET
        edge.borrow_mut().prev = prev;
        edge.borrow_mut().next = None;
        *aet = Some(edge);
    }
}

fn calculate_intersection<P: Point2D>(
    e0: &EdgeNode<P>,
    e1: &EdgeNode<P>,
    yb: P::Value,
    dy: P::Value,
) -> (P::Value, P::Value) {
    let x0 = e0.xb;
    let dx0 = e0.dx;
    let x1 = e1.xb;
    let dx1 = e1.dx;

    let zero = P::Value::zero();

    let abs_dx0 = dx0.abs();
    let abs_dx1 = dx1.abs();

    let xi = if dx0.abs_diff_eq(&dx1, P::value_epsilon()) {
        x0
    } else {
        let num = (dx0 * (x1 - x0)) + (dx1 * (x0 - x1));
        let den = dx1 - dx0;
        if den.abs_diff_eq(&zero, P::value_epsilon()) {
            zero
        } else {
            x0 + (num / den)
        }
    };

    let yi = yb;

    (xi, yi)
}

fn build_sbt<P: Point2D>(
    entries: &mut usize,
    sbt: &mut Vec<P::Value>,
    sbtree: Option<Rc<RefCell<SbTree<P>>>>,
) {
    if let Some(node_rc) = sbtree {
        let node = node_rc.borrow();
        build_sbt(entries, sbt, node.less.clone());
        sbt.push(node.y);
        *entries += 1;
        build_sbt(entries, sbt, node.more.clone());
    }
}

fn build_lmt<P: Point2D>(
    lmt: &mut Option<Rc<RefCell<LmtNode<P>>>>,
    sbtree: &mut Option<Rc<RefCell<SbTree<P>>>>,
    sbt_entries: &mut usize,
    polygon: &GpcPolygon<P>,
    polygon_type: PolygonType,
) -> Vec<Rc<RefCell<EdgeNode<P>>>> {
    let mut edge_table: Vec<Rc<RefCell<EdgeNode<P>>>> = Vec::new();

    for contour in &polygon.contours {
        let num_vertices = contour.vertices.len();
        if num_vertices == 0 {
            continue;
        }

        // Clean up the contour to remove degenerate edges
        let mut vertices: Vec<P> = Vec::new();
        let mut i = 0;
        while i < num_vertices {
            let current = contour.vertices[i];
            let next = contour.vertices[(i + 1) % num_vertices];
            if !current.x().abs_diff_eq(&next.x(), P::value_epsilon())
                || !current.y().abs_diff_eq(&next.y(), P::value_epsilon())
            {
                vertices.push(current);
            }
            i += 1;
        }

        let num_vertices = vertices.len();
        if num_vertices < 3 {
            continue; // Not enough vertices to form a polygon
        }

        // Process the edges
        for i in 0..num_vertices {
            let bot = vertices[i];
            let top = vertices[(i + 1) % num_vertices];

            let edge;
            if bot.y() < top.y() {
                edge = EdgeNode::new(bot, bot, top, polygon_type);
            } else if bot.y() > top.y() {
                edge = EdgeNode::new(top, top, bot, polygon_type);
            } else {
                // Horizontal edges are ignored in LMT construction
                continue;
            }

            // Add the edge to the edge table
            edge_table.push(edge.clone());

            // Insert the edge into the LMT
            let y = edge.borrow().bot.y();
            let lmt_node = insert_lmt_node(lmt, y);
            insert_bound(&mut lmt_node.borrow_mut().first_bound, edge.clone());

            // Add the y-coordinates to the SBT
            add_to_sbtree(sbt_entries, sbtree, edge.borrow().bot.y());
            add_to_sbtree(sbt_entries, sbtree, edge.borrow().top.y());
        }
    }

    edge_table
}

fn insert_lmt_node<P: Point2D>(
    lmt: &mut Option<Rc<RefCell<LmtNode<P>>>>,
    y: P::Value,
) -> Rc<RefCell<LmtNode<P>>> {
    if let Some(lmt_node) = lmt.as_mut() {
        let mut current = lmt_node.clone();
        let mut prev: Option<Rc<RefCell<LmtNode<P>>>> = None;
        loop {
            {
                let node = current.borrow();
                if y < node.y {
                    // Insert before current node
                    let new_node = LmtNode::new(y);
                    new_node.borrow_mut().next = Some(current.clone());
                    if let Some(prev_node) = prev {
                        prev_node.borrow_mut().next = Some(new_node.clone());
                    } else {
                        *lmt = Some(new_node.clone());
                    }
                    return new_node;
                } else if y > node.y {
                    // Move to next node
                    prev = Some(current.clone());
                    if let Some(next_node) = node.next.clone() {
                        // The borrow of `node` ends here
                        drop(node);
                        current = next_node;
                    } else {
                        // Insert at the end
                        let new_node = LmtNode::new(y);
                        current.borrow_mut().next = Some(new_node.clone());
                        return new_node;
                    }
                } else {
                    // Node with the same y already exists
                    return current.clone();
                }
            }
            // The borrow of `current` has ended, and we can safely assign to it now.
        }
    } else {
        // LMT is empty, create the first node
        let new_node = LmtNode::new(y);
        *lmt = Some(new_node.clone());
        return new_node;
    }
}

#[derive(Debug, Clone)]
struct LmtNode<P: Point2D> {
    pub y: P::Value,
    pub first_bound: Option<Rc<RefCell<EdgeNode<P>>>>,
    pub next: Option<Rc<RefCell<LmtNode<P>>>>,
}

impl<P: Point2D> LmtNode<P> {
    /// Creates a new LmtNode with the given y-coordinate.
    fn new(y: P::Value) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            y,
            first_bound: None,
            next: None,
        }))
    }
}

fn remove_edges_ending_at_yb<P: Point2D>(aet: &mut Option<Rc<RefCell<EdgeNode<P>>>>, yb: P::Value) {
    let mut edge = aet.clone();
    while let Some(edge_rc) = edge.clone() {
        let mut edge_borrowed = edge_rc.borrow_mut();
        if edge_borrowed.top.y().abs_diff_eq(&yb, P::value_epsilon()) {
            // Remove edge from AET
            let prev = edge_borrowed.prev.clone();
            let next = edge_borrowed.next.clone();

            if let Some(ref prev_rc) = prev {
                prev_rc.borrow_mut().next = next.clone();
            } else {
                *aet = next.clone();
            }

            if let Some(ref next_rc) = next {
                next_rc.borrow_mut().prev = prev.clone();
            }
        }
        edge = edge_borrowed.next.clone();
    }
}

fn process_intersections<P: Point2D>(
    aet: &mut Option<Rc<RefCell<EdgeNode<P>>>>,
    it: &mut Option<Rc<RefCell<ItNode<P>>>>,
    yb: P::Value,
    yt: P::Value,
) {
    // Iterate over the edges in the AET
    let mut edge = aet.clone();
    while let Some(edge_rc) = edge.clone() {
        let next_edge = edge_rc.borrow().next.clone();
        if let Some(next_edge_rc) = next_edge.clone() {
            // Check for intersection between edge_rc and next_edge_rc
            let edge_xb = edge_rc.borrow().xb;
            let next_edge_xb = next_edge_rc.borrow().xb;

            if edge_xb > next_edge_xb {
                // Edges intersect, add to IT
                let intersection_point =
                    calculate_intersection(&edge_rc.borrow(), &next_edge_rc.borrow(), yb, yt - yb);

                let intersection_node = ItNode::new(
                    edge_rc.clone(),
                    next_edge_rc.clone(),
                    P::from_xy(intersection_point.0, intersection_point.1),
                );

                // Add intersection to the IT
                add_intersection(it, intersection_node);

                // Swap edges in AET
                swap_edges_in_aet(aet, edge_rc.clone(), next_edge_rc.clone());
            }
        }

        edge = edge_rc.borrow().next.clone();
    }
}

fn swap_edges_in_aet<P: Point2D>(
    aet: &mut Option<Rc<RefCell<EdgeNode<P>>>>,
    edge1: Rc<RefCell<EdgeNode<P>>>,
    edge2: Rc<RefCell<EdgeNode<P>>>,
) {
    // Swap edge1 and edge2 in the AET
    let prev1 = edge1.borrow().prev.clone();
    let next2 = edge2.borrow().next.clone();

    if let Some(ref prev1_rc) = prev1 {
        prev1_rc.borrow_mut().next = Some(edge2.clone());
    } else {
        *aet = Some(edge2.clone());
    }

    edge2.borrow_mut().prev = prev1;
    edge2.borrow_mut().next = Some(edge1.clone());

    edge1.borrow_mut().prev = Some(edge2.clone());
    edge1.borrow_mut().next = next2.clone();

    if let Some(next2_rc) = next2 {
        next2_rc.borrow_mut().prev = Some(edge1.clone());
    }
}

fn add_local_min<P: Point2D>(
    out_poly: &mut Option<Rc<RefCell<PolygonNode<P>>>>,
    edge: Rc<RefCell<EdgeNode<P>>>,
    x: P::Value,
    y: P::Value,
) -> Rc<RefCell<PolygonNode<P>>> {
    let new_poly = PolygonNode::new();
    new_poly.borrow_mut().active = true;
    new_poly.borrow_mut().vertices[Side::Left as usize] = Some(VertexNode::new(P::from_xy(x, y)));

    // Add new_poly to out_poly list
    if let Some(out_poly_rc) = out_poly {
        new_poly.borrow_mut().next = Some(out_poly_rc.clone());
    }
    *out_poly = Some(new_poly.clone());

    // Assign output pointer to the edge
    edge.borrow_mut().outp[ABOVE] = Some(new_poly.clone());

    new_poly
}

fn add_left<P: Point2D>(polygon: Rc<RefCell<PolygonNode<P>>>, x: P::Value, y: P::Value) {
    let vertex = VertexNode::new(P::from_xy(x, y));
    let left_vertices = &mut polygon.borrow_mut().vertices[Side::Left as usize];

    if let Some(left_vertex_rc) = left_vertices {
        vertex.borrow_mut().next = Some(left_vertex_rc.clone());
    }
    *left_vertices = Some(vertex);
}

fn add_right<P: Point2D>(polygon: Rc<RefCell<PolygonNode<P>>>, x: P::Value, y: P::Value) {
    let vertex = VertexNode::new(P::from_xy(x, y));
    let right_vertices = &mut polygon.borrow_mut().vertices[Side::Right as usize];

    if let Some(mut last_vertex_rc) = right_vertices.clone() {
        loop {
            let next_vertex_option = {
                let last_vertex = last_vertex_rc.borrow();
                last_vertex.next.clone()
            };
            if let Some(next_vertex_rc) = next_vertex_option {
                last_vertex_rc = next_vertex_rc;
            } else {
                break;
            }
        }
        last_vertex_rc.borrow_mut().next = Some(vertex);
    } else {
        *right_vertices = Some(vertex);
    }

}

fn extract_polygons<P: Point2D>(
    out_poly: Option<Rc<RefCell<PolygonNode<P>>>>,
    result: &mut GpcPolygon<P>,
) {
    let mut polygon_node = out_poly;
    while let Some(poly_rc) = polygon_node {
        let poly = poly_rc.borrow();

        // Collect vertices from left and right sides
        let mut vertices: Vec<P> = Vec::new();

        // Left vertices
        let mut vertex_node = poly.vertices[Side::Left as usize].clone();
        while let Some(vertex_rc) = vertex_node {
            vertices.push(vertex_rc.borrow().point);
            vertex_node = vertex_rc.borrow().next.clone();
        }

        // Right vertices (in reverse order)
        let mut right_vertices: Vec<P> = Vec::new();
        vertex_node = poly.vertices[Side::Right as usize].clone();
        while let Some(vertex_rc) = vertex_node {
            right_vertices.push(vertex_rc.borrow().point);
            vertex_node = vertex_rc.borrow().next.clone();
        }
        right_vertices.reverse();
        vertices.extend(right_vertices);

        // Create a new contour
        let contour = GpcContour {
            hole: poly.hole,
            vertices,
        };

        // Add the contour to the result
        result.contours.push(contour);

        // Move to the next polygon
        polygon_node = poly.next.clone();
    }
}

pub fn polygon_clip<P: Point2D>(
    op: GpcOp,
    subj: &GpcPolygon<P>,
    clip: &GpcPolygon<P>,
    result: &mut GpcPolygon<P>,
) {
    // Initialize data structures
    let mut lmt: Option<Rc<RefCell<LmtNode<P>>>> = None;
    let mut sbtree: Option<Rc<RefCell<SbTree<P>>>> = None;
    let mut sbt_entries = 0;
    let mut aet: Option<Rc<RefCell<EdgeNode<P>>>> = None;
    let mut it: Option<Rc<RefCell<ItNode<P>>>> = None;
    let mut out_poly: Option<Rc<RefCell<PolygonNode<P>>>> = None;

    // Build LMT and SBT for subject and clipping polygons
    let subj_edge_table = build_lmt(
        &mut lmt,
        &mut sbtree,
        &mut sbt_entries,
        subj,
        PolygonType::Subj,
    );
    let clip_edge_table = build_lmt(
        &mut lmt,
        &mut sbtree,
        &mut sbt_entries,
        clip,
        PolygonType::Clip,
    );

    // Build the Scanbeam Table from the Scanbeam Tree
    let mut sbt: Vec<P::Value> = Vec::with_capacity(sbt_entries);
    build_sbt(&mut sbt_entries, &mut sbt, sbtree.clone());

    // Sort the scanbeam values
    sbt.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Process the scanbeams
    let mut scanbeam = 0;
    while scanbeam < sbt_entries {
        let yb = sbt[scanbeam];
        scanbeam += 1;
        let yt = if scanbeam < sbt_entries {
            sbt[scanbeam]
        } else {
            yb
        };

        // === Step 1: Update AET for the current scanbeam ===

        // Add edges starting at yb
        let mut lmt_node = lmt.clone();
        while let Some(lmt_node_rc) = lmt_node.clone() {
            if !lmt_node_rc.borrow().y.abs_diff_eq(&yb, P::value_epsilon()) {
                break;
            }
            let mut edge = lmt_node_rc.borrow_mut().first_bound.clone();
            while let Some(edge_rc) = edge.clone() {
                // Add edge to AET
                add_edge_to_aet(&mut aet, edge_rc.clone(), None);

                // Set the bundle fields
                let polygon_type_index = match edge_rc.borrow().polygon_type {
                    PolygonType::Clip => CLIP,
                    PolygonType::Subj => SUBJ,
                };
                edge_rc.borrow_mut().bundle[ABOVE][polygon_type_index] = true;

                // Advance to next bound
                edge = edge_rc.borrow().next_bound.clone();
            }
            // Move to next LMT node
            lmt_node = lmt_node_rc.borrow().next.clone();
        }

        // Remove edges ending at yb from the AET
        remove_edges_ending_at_yb(&mut aet, yb);

        // === Step 2: Process intersections ===

        // Build the intersection table
        process_intersections(&mut aet, &mut it, yb, yt);

        // === Step 3: Construct output polygons ===

        // Handle the AET and generate output polygons
        let mut edge = aet.clone();
        while let Some(edge_rc) = edge.clone() {
            // Implement logic for constructing output polygons based on the operation
            // This part is complex and requires handling the Boolean operations
            // For now, we can leave it as a placeholder or implement a simple case
            edge = edge_rc.borrow().next.clone();
        }

        // Reset the intersection table
        reset_it(&mut it);

        // Advance edges in AET
        advance_aet_edges(&mut aet, yt);
    }

    // Extract polygons from out_poly and store them in result
    extract_polygons(out_poly, result);
}


fn reset_it<P: Point2D>(it: &mut Option<Rc<RefCell<ItNode<P>>>>) {
    *it = None;
}

fn advance_aet_edges<P: Point2D>(aet: &mut Option<Rc<RefCell<EdgeNode<P>>>>, yt: P::Value) {
    let mut edge = aet.clone();
    while let Some(edge_rc) = edge.clone() {
        // Update xb for the edge
        let dx = edge_rc.borrow().dx;
        edge_rc.borrow_mut().xb += dx * (yt - edge_rc.borrow().bot.y());

        // Move to the next edge in the AET
        edge = edge_rc.borrow().next.clone();
    }
}



#[derive(Debug, Clone)]
struct ItNode<P: Point2D> {
    ie: [Rc<RefCell<EdgeNode<P>>>; 2],    // The intersecting edges
    point: P,                             // The intersection point
    next: Option<Rc<RefCell<ItNode<P>>>>, // Next node in the IT
}

impl<P: Point2D> ItNode<P> {
    /// Creates a new ItNode with the given edges and intersection point.
    fn new(
        edge0: Rc<RefCell<EdgeNode<P>>>,
        edge1: Rc<RefCell<EdgeNode<P>>>,
        point: P,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            ie: [edge0, edge1],
            point,
            next: None,
        }))
    }
}
fn add_intersection<P: Point2D>(
    it: &mut Option<Rc<RefCell<ItNode<P>>>>,
    new_node: Rc<RefCell<ItNode<P>>>,
) {
    if let Some(it_node_rc) = it {
        let it_node_point_y = it_node_rc.borrow().point.y();
        let new_node_point_y = new_node.borrow().point.y();

        if new_node_point_y < it_node_point_y
            || (new_node_point_y.abs_diff_eq(&it_node_point_y, P::value_epsilon())
                && new_node.borrow().point.x() < it_node_rc.borrow().point.x())
        {
            // Insert new_node before it_node_rc
            new_node.borrow_mut().next = Some(it_node_rc.clone());
            *it = Some(new_node);
        } else {
            // Recurse into next
            let next = &mut it_node_rc.borrow_mut().next;
            add_intersection(next, new_node);
        }
    } else {
        // Insert as the first node
        *it = Some(new_node);
    }
}

#[derive(Debug, Clone)]
pub struct GpcPolygon<P: Point2D> {
    pub contours: Vec<GpcContour<P>>,
}

#[derive(Debug, Clone)]
pub struct GpcContour<P: Point2D> {
    pub hole: bool,
    pub vertices: Vec<P>,
}

#[derive(Debug, Clone, Copy)]
pub enum GpcOp {
    Difference,
    Intersection,
    ExclusiveOr,
    Union,
}

fn insert_bound<P: Point2D>(
    first_bound: &mut Option<Rc<RefCell<EdgeNode<P>>>>,
    edge: Rc<RefCell<EdgeNode<P>>>,
) {
if let Some(bound_edge) = first_bound {
    let mut current = bound_edge.clone();
    let mut prev: Option<Rc<RefCell<EdgeNode<P>>>> = None;

    loop {
        {
            let current_borrow = current.borrow();
            if edge.borrow().bot.x() < current_borrow.bot.x() {
                // Insert before current edge
                edge.borrow_mut().next_bound = Some(current.clone());
                if let Some(prev_edge) = prev {
                    prev_edge.borrow_mut().next_bound = Some(edge.clone());
                } else {
                    *first_bound = Some(edge.clone());
                }
                return;
            } else {
                prev = Some(current.clone());
                if let Some(next_edge) = current_borrow.next_bound.clone() {
                    // Borrow ends here
                    drop(current_borrow);
                    current = next_edge;
                } else {
                    // Insert at the end
                    current.borrow_mut().next_bound = Some(edge.clone());
                    return;
                }
            }
        }
    }
} else {
    // No bounds yet, insert as the first bound
    *first_bound = Some(edge);
}

}

fn add_to_sbtree<P: Point2D>(
    entries: &mut usize,
    sbtree: &mut Option<Rc<RefCell<SbTree<P>>>>,
    y: P::Value,
) {
    if let Some(node) = sbtree {
        let mut node_borrowed = node.borrow_mut();
        if y < node_borrowed.y {
            add_to_sbtree(entries, &mut node_borrowed.less, y);
        } else if y > node_borrowed.y {
            add_to_sbtree(entries, &mut node_borrowed.more, y);
        }
        // If y == node.y, do nothing (already in tree)
    } else {
        // Create new node
        *sbtree = Some(SbTree::new(y));
        *entries += 1;
    }
}

impl<P: Point2D> PolygonNode<P> {
    fn new() -> Rc<RefCell<Self>> {
        let node = Rc::new(RefCell::new(Self {
            active: true,
            hole: false,
            vertices: [None, None],
            next: None,
            proxy: None, // Initialize as None
        }));
        // Set proxy to point to self
        node.borrow_mut().proxy = Some(node.clone());
        node
    }

    fn default_placeholder() -> Self {
        Self {
            active: true,
            hole: false,
            vertices: [None, None],
            next: None,
            proxy: None, // Initialize as None
        }
    }
}

impl<P: Point2D> VertexNode<P> {
    fn new(point: P) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { point, next: None }))
    }
}

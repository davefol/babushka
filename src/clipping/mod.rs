//! 2D Polygon Clipping library based on Vatti 1992

use crate::point::Point2D;
use crate::polygon::Polygon;

mod lmt;
mod clip;

pub use self::clip::{clip, ClipOperation};

pub trait Clippable: Polygon {
    fn get_vertex(&self, index: usize) -> <Self as Polygon>::Point;
}

#[derive(Debug, Clone)]
struct EdgeNode<P: Point2D> {
    pub vertex: P,
    pub bot: P,
    pub top: P,
    pub xb: f64,
    pub xt: f64,
    pub dx: f64,
    pub polygon_type: PolygonType, // CLIP or SUBJ
    pub bundle: [[bool; 2]; 2],
    pub bside: [Side; 2],
    pub bstate: [BundleState; 2],
    pub outp: [Option<PolygonNode<P>>; 2],
    pub prev: Option<Box<EdgeNode<P>>>,
    pub next: Option<Box<EdgeNode<P>>>,
    pub pred: Option<Box<EdgeNode<P>>>,
    pub succ: Option<Box<EdgeNode<P>>>,
    pub next_bound: Option<Box<EdgeNode<P>>>,
}

#[derive(Debug, Clone)]
struct VertexNode<P: Point2D> {
    pub point: P,
    pub next: Option<Box<VertexNode<P>>>,
}

#[derive(Debug, Clone)]
struct PolygonNode<P: Point2D> {
    pub active: bool,
    pub hole: bool,
    pub vertices: [Option<Box<VertexNode<P>>>; 2], // Left and right vertex lists
    pub next: Option<Box<PolygonNode<P>>>,
    pub proxy: Option<Box<PolygonNode<P>>>,
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

fn add_edge_to_aet<P: Point2D>(
    aet: &mut Option<Box<EdgeNode<P>>>,
    mut edge: Box<EdgeNode<P>>,
    prev: Option<Box<EdgeNode<P>>>,
) {
    if aet.is_none() {
        // Append edge to the AET
        *aet = Some(edge);
        if let Some(aet_edge) = aet.as_mut() {
            aet_edge.prev = prev;
            aet_edge.next = None;
        }
    } else {
        // Primary sort on xb
        let aet_edge = aet.as_mut().unwrap();
        if edge.xb < aet_edge.xb {
            // Insert edge before current AET edge
            edge.prev = prev;
            edge.next = Some(aet_edge.clone());
            aet_edge.prev = Some(edge.clone());
            *aet = Some(edge);
        } else {
            // Recursively add to AET
            add_edge_to_aet(&mut aet_edge.clone().next, edge, Some(aet_edge.clone()));
        }
    }
}

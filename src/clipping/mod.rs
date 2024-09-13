//! 2D Polygon Clipping library based on Vatti 1992

use crate::point::Point2D;
use crate::polygon::Polygon;

mod lmt;
mod clip;

pub use self::clip::{clip, ClipOperation};

pub trait Clippable: Polygon {
    fn get_vertex(&self, index: usize) -> <Self as Polygon>::Point;
}

#[derive(Debug)]
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

#[derive(Debug)]
struct VertexNode<P: Point2D> {
    pub point: P,
    pub next: Option<Box<VertexNode<P>>>,
}

#[derive(Debug)]
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



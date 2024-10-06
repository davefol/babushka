//! Genetic algorithm for irregular bin packing
use num_traits::ToPrimitive;
use std::collections::HashMap;
use std::hash::Hash;

use super::problem::{
    IrregularBinPackingPlacement, IrregularBinPackingProblem, IrregularBinPackingSolution,
};
use crate::no_fit_polygon::ComputeNoFitPolygon;
use crate::{point::Point2D, polygon::Polygon};
use anyhow::{anyhow, Result};
use itertools::izip;
use num_traits::Zero;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
enum NFPCacheIndex {
    Indivudual(usize),
    Bin,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct NFPCacheKey<P: Polygon> {
    a: NFPCacheIndex,
    b: NFPCacheIndex,
    a_rotation: <P::Point as Point2D>::Value,
    b_rotation: <P::Point as Point2D>::Value,
    inside: bool,
}

impl<P: Polygon> Hash for NFPCacheKey<P> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.a.hash(state);
        self.b.hash(state);
        self.a_rotation.to_f64().unwrap().to_bits().hash(state);
        self.b_rotation.to_f64().unwrap().to_bits().hash(state);
        self.inside.hash(state);
    }
}

pub struct GeneticIrregularBinPacker<P: Polygon + ComputeNoFitPolygon> {
    problem: IrregularBinPackingProblem<P>,
    population_size: usize,
    mutation_rate: f64,
    population: Vec<Individual<P>>,
    rng: ChaCha8Rng,
    nfp_cache: HashMap<NFPCacheKey<P>, Vec<Vec<P::Point>>>,
}

impl<P: Polygon + ComputeNoFitPolygon> GeneticIrregularBinPacker<P> {
    pub fn new(
        problem: IrregularBinPackingProblem<P>,
        population_size: usize,
        mutation_rate: f64,
        seed: u64,
    ) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        let mut order = vec![];
        let mut rotations = vec![];
        let mut indices: Vec<usize> = (0..problem.piece_descriptions().len()).collect();

        // heuristic: place bigger elements first
        indices.sort_by_key(|i| {
            problem.piece_descriptions()[*i].piece.area();
        });
        indices.reverse();

        for i in indices {
            let piece_description = &problem.piece_descriptions()[i];
            for _ in 0..piece_description.instances {
                order.push(i);
                if let Some(rotation) = piece_description.allowed_rotations.choose(&mut rng) {
                    rotations.push(rotation.clone());
                } else {
                    rotations.push(Zero::zero());
                }
            }
        }
        let adam = Individual::new(order, rotations);
        let population = vec![adam];
        let mut packer = Self {
            problem,
            population_size,
            mutation_rate,
            population,
            rng,
            nfp_cache: HashMap::new(),
        };
        while packer.population.len() < packer.population_size {
            let clone = packer.mutate(&packer.population[0].clone());
            packer.population.push(clone);
        }
        packer
    }

    fn mutate(&mut self, individual: &Individual<P>) -> Individual<P> {
        let mut clone = individual.clone();
        for i in 0..clone.order.len() {
            let r: f64 = self.rng.gen();
            if r < 0.01 * self.mutation_rate {
                let j = i + 1;
                if j < clone.order.len() {
                    let temp = clone.order[i];
                    clone.order[i] = clone.order[j];
                    clone.order[j] = temp;
                }
            }

            let r: f64 = self.rng.gen();
            if r < 0.01 * self.mutation_rate {
                clone.rotations[i] = self.problem.piece_descriptions()[clone.order[i]]
                    .allowed_rotations
                    .choose(&mut self.rng)
                    .unwrap_or(&Zero::zero())
                    .clone();
            }
        }
        clone
    }

    fn place(&self, individual: Individual<P>) -> IrregularBinPackingSolution<P> {
        let mut bin_id = 0;
        let mut placed: usize = 0;
        let mut first_in_bin: bool = true;
        let pieces_to_place = individual.order.len();
        //let mut bins = vec![];
        let mut bin = self.problem.bin();
        let mut placements: Vec<IrregularBinPackingPlacement<P>> = vec![];
        while placed < pieces_to_place {
            for (piece_id, rotation) in izip!(individual.order.iter(), individual.rotations.iter())
            {
                // grab the piece and rotate it
                let piece_description = &self.problem.piece_descriptions()[*piece_id];
                let mut piece = piece_description.piece.clone();
                piece.for_each_polygon(|p| p.set_rotation(*rotation));

                // for the first placement, put the piece on the left
                if first_in_bin {
                    let nf = bin.no_fit_polygon(&piece, false, true);
                    let left_most = nf
                        .iter()
                        .flatten()
                        .min_by(|a, b| a.x().partial_cmp(&b.x()).unwrap())
                        .unwrap();
                    // TODO: add test for when polygon doesn't fit.
                    placements.push(IrregularBinPackingPlacement::new(
                        bin_id,
                        *piece_id,
                        left_most.clone(),
                        rotation.clone(),
                    ));
                }
            }
        }
        unimplemented!()
    }

    pub fn builder() -> GeneticIrregularBinPackerBuilder<P> {
        GeneticIrregularBinPackerBuilder::new()
    }
}

/// Represents order and rotation for each polygon
#[derive(Debug, Clone)]
struct Individual<P: Polygon> {
    order: Vec<usize>,
    rotations: Vec<<P::Point as Point2D>::Value>,
}

impl<P: Polygon> Individual<P> {
    pub fn new(order: Vec<usize>, rotations: Vec<<P::Point as Point2D>::Value>) -> Self {
        Self { order, rotations }
    }
}

pub struct GeneticIrregularBinPackerBuilder<P: Polygon + ComputeNoFitPolygon> {
    problem: Option<IrregularBinPackingProblem<P>>,
    population_size: usize,
    mutation_rate: f64,
    seed: u64,
}

impl<P: Polygon + ComputeNoFitPolygon> GeneticIrregularBinPackerBuilder<P> {
    pub fn new() -> Self {
        Self {
            problem: None,
            population_size: 10,
            mutation_rate: 10.0,
            seed: 0,
        }
    }

    pub fn problem(mut self, problem: IrregularBinPackingProblem<P>) -> Self {
        self.problem = Some(problem);
        self
    }

    pub fn population_size(mut self, population_size: usize) -> Self {
        self.population_size = population_size;
        self
    }

    pub fn mutation_rate(mut self, mutation_rate: f64) -> Self {
        self.mutation_rate = mutation_rate;
        self
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn build(self) -> Result<GeneticIrregularBinPacker<P>> {
        Ok(GeneticIrregularBinPacker::new(
            self.problem.ok_or(anyhow!("No problem provided"))?,
            self.population_size,
            self.mutation_rate,
            self.seed,
        ))
    }
}

//! Genetic algorithm for irregular bin packing
use super::problem::{
    IrregularBinPackingPlacement, IrregularBinPackingProblem, IrregularBinPackingSolution,
};
use crate::{point::Point2D, polygon::Polygon};
use anyhow::{anyhow, Result};
use num_traits::Zero;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub struct GeneticIrregularBinPacker<P: Polygon> {
    problem: IrregularBinPackingProblem<P>,
    population_size: usize,
    mutation_rate: f64,
    population: Vec<Individual<P>>,
    rng: ChaCha8Rng,
}

impl<P: Polygon> GeneticIrregularBinPacker<P> {
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

pub struct GeneticIrregularBinPackerBuilder<P: Polygon> {
    problem: Option<IrregularBinPackingProblem<P>>,
    population_size: usize,
    mutation_rate: f64,
    seed: u64,
}

impl<P: Polygon> GeneticIrregularBinPackerBuilder<P> {
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

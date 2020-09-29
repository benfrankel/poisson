use glam::Vec2;
use rand::distributions::Uniform;
use rand::Rng;
use sphere::sphere_volume;

use crate::algorithm::{Algorithm, Creator};
use crate::utils::*;
use crate::Builder;

/// Generates uniform maximal Poisson disk samplings with O(n2<sup>d</sup>) time and O(n2<sup>d</sup>) space complexity relative to the number of samples generated and the dimensionality of the sampling volume.
/// Based on Ebeida, Mohamed S., et al. "A Simple Algorithm for Maximal Poisson‐Disk Sampling in High Dimensions." Computer Graphics Forum. Vol. 31. No. 2pt4. Blackwell Publishing Ltd, 2012.
#[derive(Debug, Clone, Copy)]
pub struct Ebeida;

impl Creator for Ebeida {
    type Algo = Algo;

    fn create(poisson: &Builder) -> Self::Algo {
        let grid = Grid::new(poisson.radius, poisson.poisson_type);
        let mut indices = Vec::with_capacity(grid.cells() * 2);
        let choices = (0..grid.side()).map(|i| i as f32).collect::<Vec<_>>();
        indices.extend(each_combination(&choices));
        let a = 0.3;
        Algo {
            a,
            grid,
            throws: (a * indices.len() as f64).ceil() as usize,
            range: Uniform::new(0, indices.len()),
            indices,
            level: 0,
            success: 0,
            outside: vec![],
            mantissa_digits: f32::MANTISSA_DIGITS as usize,
        }
    }
}

/// Implementation for the Ebeida algorithm
pub struct Algo {
    grid: Grid,
    indices: Vec<Vec2>,
    level: usize,
    range: Uniform<usize>,
    throws: usize,
    success: usize,
    outside: Vec<Vec2>,
    mantissa_digits: usize,
    a: f64,
}

impl Algorithm for Algo {
    fn next<R>(&mut self, poisson: &mut Builder, rng: &mut R) -> Option<mint::Vector2<f32>>
    where
        R: Rng,
    {
        if self.indices.is_empty() {
            return None;
        }
        while self.level < self.mantissa_digits {
            while self.throws > 0 {
                self.throws -= 1;
                let index = rng.sample(self.range);
                let cur = self.indices[index].clone();
                let parent = get_parent(cur.clone(), self.level);
                if !self
                    .grid
                    .get(parent.clone())
                    .expect("Indexing base grid by valid parent failed.")
                    .is_empty()
                {
                    self.indices.swap_remove(index);
                    if self.indices.is_empty() {
                        return None;
                    }
                    self.range = Uniform::new(0, self.indices.len());
                } else {
                    let sample = choose_random_sample(rng, &self.grid, cur.clone(), self.level);
                    if is_disk_free(
                        &self.grid,
                        poisson,
                        cur.clone(),
                        self.level,
                        sample.clone(),
                        &self.outside,
                    ) {
                        self.grid
                            .get_mut(parent)
                            .expect("Indexing base grid by already indexed valid parent failed.")
                            .push(sample.clone());
                        self.indices.swap_remove(index);
                        if !self.indices.is_empty() {
                            self.range = Uniform::new(0, self.indices.len());
                        }
                        self.success += 1;
                        return Some(sample.into());
                    }
                }
            }
            self.subdivide(&poisson);
            if self.indices.is_empty() {
                return None;
            }
            self.range = Uniform::new(0, self.indices.len());
            self.throws = (self.a * self.indices.len() as f64).ceil() as usize;
            self.level += 1;
        }
        let index = rng.sample(self.range);
        let cur = self.indices.swap_remove(index);
        let side = 2usize.pow(self.level as u32);
        let sample = index_to_sample(&cur, side);
        if is_disk_free(
            &self.grid,
            poisson,
            cur.clone(),
            self.level,
            sample.clone(),
            &self.outside,
        ) {
            Some(sample.into())
        } else {
            None
        }
    }

    fn size_hint(&self, poisson: &Builder) -> (usize, Option<usize>) {
        // Calculating lower bound should work because we calculate how much volume is left to be filled at worst case and
        // how much sphere can fill it at best case and just figure out how many fills are still needed.
        let side = 2usize.pow(self.level as u32);
        let spacing = self.grid.cell() / (side as f32);
        let grid_volume = (self.indices.len() as f32) * spacing.powi(2);
        let sphere_volume = sphere_volume(2.0 * poisson.radius, 2);
        let lower = grid_volume / sphere_volume;
        let mut lower = lower.floor() as usize;
        if lower > 0 {
            lower -= 1;
        }
        // Calculating upper bound should work because there is this many places left in the grid and no more can fit into it.
        let upper = self.grid.cells() - self.success;
        (lower, Some(upper))
    }

    fn restrict(&mut self, sample: mint::Vector2<f32>) {
        let sample: Vec2 = sample.into();
        self.success += 1;
        let index = sample_to_index(&sample, self.grid.side());
        if let Some(g) = self.grid.get_mut(index) {
            g.push(sample);
        } else {
            self.outside.push(sample);
        }
    }

    fn stays_legal(&self, poisson: &Builder, sample: mint::Vector2<f32>) -> bool {
        let sample: Vec2 = sample.into();
        let index = sample_to_index(&sample, self.grid.side());
        is_disk_free(&self.grid, poisson, index, 0, sample.clone(), &self.outside)
    }
}

impl Algo {
    fn subdivide(&mut self, poisson: &Builder) {
        let choices = &[0.0, 1.0];
        let (grid, outside, level) = (&self.grid, &self.outside, self.level);
        self.indices.flat_map_inplace(|i| {
            each_combination(choices)
                .map(move |n: Vec2| n + i.clone() * 2.0)
                .filter(|c| !covered(grid, poisson, outside, c.clone(), level + 1))
        });
    }
}

fn covered(
    grid: &Grid,
    poisson: &Builder,
    outside: &[Vec2],
    index: Vec2,
    level: usize,
) -> bool {
    // TODO: This does 4^d checking of points even though it could be done 3^d
    let side = 2usize.pow(level as u32);
    let spacing = grid.cell() / (side as f32);
    let sqradius = (2.0 * poisson.radius).powi(2);
    let parent = get_parent(index.clone(), level);
    each_combination(&[0.0, 1.0])
        .map(|t| (index.clone() + t) * spacing)
        .all(|t| {
            each_combination(&[-2.0, -1.0, 0.0, 1.0, 2.0])
                .filter_map(|t| grid.get(parent.clone() + t))
                .flat_map(|t| t)
                .any(|v| sqdist(v.clone(), t.clone(), poisson.poisson_type) < sqradius)
                || !is_valid(poisson, &outside, t)
        })
}

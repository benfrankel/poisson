use glam::Vec2;
use rand::distributions::Uniform;
use rand::Rng;
use rand_distr::StandardNormal;
use sphere::sphere_volume;

use crate::algorithm::{Algorithm, Creator};
use crate::utils::*;
use crate::Builder;

/// Generates approximately uniform non-maximal Poisson disk samplings with O(n) time and O(n) space complexity relative to the number of samples generated.
/// Based on Bridson, Robert. "Fast Poisson disk sampling in arbitrary dimensions." SIGGRAPH Sketches. 2007.
#[derive(Debug, Clone, Copy)]
pub struct Bridson;

impl Creator for Bridson {
    type Algo = Algo;

    fn create(poisson: &Builder) -> Self::Algo {
        Algo {
            grid: Grid::new(poisson.radius, poisson.poisson_type),
            active_samples: vec![],
            outside: vec![],
            success: 0,
        }
    }
}

/// Implementation for the Bridson algorithm
pub struct Algo {
    grid: Grid,
    active_samples: Vec<Vec2>,
    outside: Vec<Vec2>,
    success: usize,
}

impl Algorithm for Algo {
    fn next<R>(&mut self, poisson: &mut Builder, rng: &mut R) -> Option<mint::Vector2<f32>>
    where
        R: Rng,
    {
        while !self.active_samples.is_empty() {
            let index = rng.sample(Uniform::new(0, self.active_samples.len()));
            let cur = self.active_samples[index].clone();
            for _ in 0..30 {
                let min = 2.0 * poisson.radius;
                let max = 4.0 * poisson.radius;
                let sample = cur.clone() + random_point_annulus(rng, min, max).into();
                if (0..2)
                    .map(|n| sample[n])
                    .all(|c| 0.0 <= c && c < 1.0)
                {
                    let index = sample_to_index(&sample, self.grid.side());
                    if self.insert_if_valid(poisson, index, sample.clone()) {
                        return Some(sample.into());
                    }
                }
            }
            self.active_samples.swap_remove(index);
        }
        while self.success == 0 {
            let cell = rng.sample(Uniform::new(0, self.grid.cells()));
            let index: Vec2 = decode(cell, self.grid.side()).expect(
                "Because we are decoding random index within grid \
                 this should work.",
            );
            let sample = choose_random_sample(rng, &self.grid, index.clone(), 0);
            if self.insert_if_valid(poisson, index, sample.clone()) {
                return Some(sample.into());
            }
        }
        None
    }

    fn size_hint(&self, poisson: &Builder) -> (usize, Option<usize>) {
        // Calculating upper bound should work because there is this many places left in the grid and no more can fit into it.
        let upper = if self.grid.cells() > self.success {
            self.grid.cells() - self.success
        } else {
            0
        };
        // Calculating lower bound should work because we calculate how much volume is left to be filled at worst case and
        // how much sphere can fill it at best case and just figure out how many fills are still needed.
        let spacing = self.grid.cell();
        let grid_volume = (upper as f32) * spacing.powi(2);
        let sphere_volume = sphere_volume(2.0 * poisson.radius, 2);
        let lower: f32 = grid_volume / sphere_volume;
        let mut lower = lower.floor() as usize;
        if lower > 0 {
            lower -= 1;
        }
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
    fn insert_if_valid(&mut self, poisson: &mut Builder, index: Vec2, sample: Vec2) -> bool {
        if is_disk_free(
            &self.grid,
            poisson,
            index.clone(),
            0,
            sample.clone(),
            &self.outside,
        ) {
            self.active_samples.push(sample.clone());
            self.grid
                .get_mut(index)
                .expect("Because the sample is [0, 1) indexing it should work.")
                .push(sample);
            self.success += 1;
            true
        } else {
            false
        }
    }
}

fn random_point_annulus<R>(rand: &mut R, min: f32, max: f32) -> Vec2
where
    R: Rng,
{
    loop {
        let mut result = Vec2::zero();
        for n in 0..2 {
            result[n] = rand.sample(StandardNormal);
        }
        let result = result.normalize() * rand.gen::<f32>() * max;
        if result.length() >= min {
            return result;
        }
    }
}

#[test]
fn random_point_annulus_does_not_generate_outside_annulus() {
    use rand::{rngs::SmallRng, SeedableRng};
    let mut rng = SmallRng::seed_from_u64(42);
    for _ in 0..10000 {
        let result = random_point_annulus(&mut rng, 1., 2.);
        assert!(result.length() >= 1.);
        assert!(result.length() <= 2.);
    }
}

#[test]
fn random_point_annulus_generates_all_quadrants() {
    use rand::{rngs::SmallRng, SeedableRng};
    let mut rng = SmallRng::seed_from_u64(42);
    let (mut top_left, mut top_right, mut bottom_left, mut bottom_right) =
        (false, false, false, false);
    for _ in 0..10000 {
        let result = random_point_annulus(&mut rng, 1., 2.);
        if result.y() < 0. {
            if result.x() < 0. {
                bottom_left = true;
            } else {
                bottom_right = true;
            }
        } else {
            if result.x() < 0. {
                top_left = true;
            } else {
                top_right = true;
            }
        }
    }
    assert!(top_left);
    assert!(top_right);
    assert!(bottom_left);
    assert!(bottom_right);
}

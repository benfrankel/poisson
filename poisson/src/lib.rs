//! # Poisson disk sampling
//!
//! Generates a sampling of points in [0, 1)<sup>2</sup> where:
//!
//! * Sample points fill the space uniformly.
//! * Sample points stay a given minimum distance apart.
//!
//! This is equivalent to uniformly filling a unit square with non-overlapping
//! disks of equal radius, where the radius is half the minimum distance.
//!
//! Due to their blue noise properties, Poisson disk samplings can be used for
//! object placement in procedural texture/world generation, digital stippling,
//! sampling in rendering, or (re)meshing.
//!
//! # Examples
//!
//! Generate a non-tiling Poisson disk sampling in [0, 1)<sup>2</sup> with disk radius 0.1
//! using a slower but more accurate algorithm.
//!
//! ````rust
//! use poisson::{Builder, Type, algorithm};
//! use rand::SeedableRng;
//! use rand::rngs::SmallRng;
//!
//! fn main() {
//!     let poisson =
//!         Builder::with_radius(0.1, Type::Normal)
//!             .build(SmallRng::from_entropy(), algorithm::Ebeida);
//!     let samples = poisson.generate();
//!     println!("{:?}", samples);
//! }
//! ````
//!
//! Generate a tiling Poisson disk sampling in [0, 1)<sup>2</sup> with approximately 100 samples
//! and relative disk radius 0.9 using a faster but less accurate algorithm.
//!
//! ````rust
//! # use poisson::{Builder, Type, algorithm};
//! # use rand::SeedableRng;
//! # use rand::rngs::SmallRng;
//!
//! fn main() {
//!     let poisson =
//!         Builder::with_samples(100, 0.9, Type::Periodic)
//!             .build(SmallRng::from_entropy(), algorithm::Bridson);
//!     for sample in poisson {
//!         println!("{:?}", sample)
//!     }
//! }
//! ````

use std::marker::PhantomData;

use rand::Rng;

use crate::algorithm::{Algorithm, Creator};
use crate::utils::math::calc_radius;

pub mod algorithm;
mod utils;

/// Enum for determining the type of Poisson disk sampling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    /// Acts like there is void all around the space placing no restrictions to sides.
    Normal,
    /// Makes the space wrap around on edges allowing tiling of the generated Poisson disk sampling.
    Periodic,
}

impl Default for Type {
    fn default() -> Type {
        Type::Normal
    }
}

/// Builder for the generator.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Builder {
    radius: f32,
    poisson_type: Type,
}

impl Builder {
    /// New Builder with type of distribution and radius specified.
    /// The radius should be in (0, √2 / 2]
    pub fn with_radius(radius: f32, poisson_type: Type) -> Self {
        assert!(0.0 < radius && radius <= 2f32.sqrt() / 2.0);
        Builder {
            radius,
            poisson_type,
        }
    }

    /// New Builder with type of distribution and relative radius specified.
    /// The relative radius should be in (0, 1]
    pub fn with_relative_radius(relative: f32, poisson_type: Type) -> Self {
        assert!(0.0 < relative && relative <= 1.0);
        Builder {
            radius: relative * 2f32.sqrt() / 2.0,
            poisson_type,
        }
    }

    /// New Builder with type of distribution, approximate amount of samples and relative radius specified.
    /// The amount of samples should be larger than 0.
    /// The relative radius should be in (0, 1].
    pub fn with_samples(samples: usize, relative: f32, poisson_type: Type) -> Self {
        Builder {
            radius: calc_radius(samples, relative, poisson_type),
            poisson_type,
        }
    }

    /// Returns the radius of the generator.
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Returns the type of the generator.
    pub fn poisson_type(&self) -> Type {
        self.poisson_type
    }

    /// Builds generator with random number generator and algorithm specified.
    pub fn build<R, A>(self, rng: R, _algo: A) -> Generator<R, A>
    where
        R: Rng,
        A: Creator,
    {
        Generator::new(self, rng)
    }
}

/// Generates a Poisson disk sampling in a [0, 1]<sup>d</sup> area.
#[derive(Clone, Debug)]
pub struct Generator<R, A>
where
    R: Rng,
    A: Creator,
{
    poisson: Builder,
    rng: R,
    _algo: PhantomData<A>,
}

impl<R, A> Generator<R, A>
where
    R: Rng,
    A: Creator,
{
    fn new(poisson: Builder, rng: R) -> Self {
        Generator {
            rng,
            poisson,
            _algo: PhantomData,
        }
    }

    /// Sets the radius of the generator.
    pub fn set_radius(&mut self, radius: f32) {
        assert!(0.0 < radius && radius <= 2f32.sqrt() / 2.0);
        self.poisson.radius = radius;
    }

    /// Returns the radius of the generator.
    pub fn radius(&self) -> f32 {
        self.poisson.radius
    }

    /// Returns the type of the generator.
    pub fn poisson_type(&self) -> Type {
        self.poisson.poisson_type
    }
}

impl<R, A> Generator<R, A>
where
    R: Rng + Clone,
    A: Creator,
{
    /// Generates a Poisson disk sampling.
    pub fn generate(&self) -> Vec<mint::Vector2<f32>> {
        self.clone().into_iter().collect()
    }
}

impl<R, A> IntoIterator for Generator<R, A>
where
    R: Rng,
    A: Creator,
{
    type Item = mint::Vector2<f32>;
    type IntoIter = PoissonIter<R, A::Algo>;

    fn into_iter(self) -> Self::IntoIter {
        PoissonIter {
            rng: self.rng,
            algo: A::create(&self.poisson),
            poisson: self.poisson,
        }
    }
}

/// Iterator for generating a Poisson disk sampling.
#[derive(Clone)]
pub struct PoissonIter<R, A>
where
    R: Rng,
    A: Algorithm,
{
    poisson: Builder,
    rng: R,
    algo: A,
}

impl<R, A> Iterator for PoissonIter<R, A>
where
    R: Rng,
    A: Algorithm,
{
    type Item = mint::Vector2<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        self.algo.next(&mut self.poisson, &mut self.rng)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.algo.size_hint(&self.poisson)
    }
}

impl<R, A> PoissonIter<R, A>
where
    R: Rng,
    A: Algorithm,
{
    /// Returns the radius of the generator.
    pub fn radius(&self) -> f32 {
        self.poisson.radius
    }

    /// Returns the type of the generator.
    pub fn poisson_type(&self) -> Type {
        self.poisson.poisson_type
    }

    /// Restricts the poisson algorithm with arbitrary sample.
    pub fn restrict(&mut self, value: mint::Vector2<f32>) {
        self.algo.restrict(value);
    }

    /// Checks legality of sample for current distribution.
    pub fn stays_legal(&self, value: mint::Vector2<f32>) -> bool {
        self.algo.stays_legal(&self.poisson, value)
    }
}

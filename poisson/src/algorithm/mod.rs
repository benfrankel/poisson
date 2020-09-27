//! Module that contains traits that describe Poisson disk sampling generating algorithms.

use std::fmt::Debug;

use rand::Rng;

use crate::Builder;
pub use self::bridson::Bridson;
pub use self::ebeida::Ebeida;

mod bridson;
mod ebeida;

/// Constructs new instance of the algorithm.
pub trait Creator: Copy + Debug {
    /// Algorithm instance associated with the trait
    type Algo: Algorithm;

    /// Creates new and empty algorithm instance.
    fn create(_: &Builder) -> Self::Algo;
}

/// Trait that describes a Poisson disk sampling generating algorithm.
pub trait Algorithm {
    /// Generates new sample advancing the algorithm.
    fn next<R>(&mut self, _: &mut Builder, _: &mut R) -> Option<mint::Vector2<f32>>
    where
        R: Rng;

    /// Returns lower and upper bound of the amount of samples remaining for the algorithm to generate.
    fn size_hint(&self, _: &Builder) -> (usize, Option<usize>);

    /// Restricts the algorithm with an arbitrary sample.
    fn restrict(&mut self, _: mint::Vector2<f32>);

    /// Checks if a sample is valid for the Poisson disk sampling generated thus far by the algorithm.
    fn stays_legal(&self, _: &Builder, _: mint::Vector2<f32>) -> bool;
}

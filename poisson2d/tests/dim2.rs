use poisson2d::Type::*;
use poisson2d::{algorithm, Builder};
use rand::{rngs::SmallRng, SeedableRng};

use crate::helper::test_with_samples;

mod helper;

#[test]
fn test_one_sample_works() {
    let rand = SmallRng::from_seed([
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    ]);
    let builder = Builder::with_samples(1, 0.8, Normal);
    let builder = builder.build(rand, algorithm::Ebeida);
    builder.into_iter().for_each(drop);

    let rand = SmallRng::from_seed([
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    ]);
    let builder = Builder::with_samples(1, 0.8, Normal);
    let builder = builder.build(rand, algorithm::Bridson);
    builder.into_iter().for_each(drop);
}

#[test]
fn test_2d_1_80_normal() {
    test_with_samples(1, 0.8, 1600, Normal);
}

#[test]
fn test_2d_1_80_periodic() {
    test_with_samples(1, 0.8, 800, Periodic);
}

#[test]
fn test_2d_10_80_normal() {
    test_with_samples(10, 0.8, 800, Normal);
}

#[test]
fn test_2d_10_80_periodic() {
    test_with_samples(10, 0.8, 400, Periodic);
}

#[test]
fn test_2d_100_80_normal() {
    test_with_samples(100, 0.8, 400, Normal);
}

#[test]
fn test_2d_100_80_periodic() {
    test_with_samples(100, 0.8, 200, Periodic);
}

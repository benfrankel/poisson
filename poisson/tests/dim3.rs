extern crate nalgebra as na;

use poisson::Type::*;

use crate::helper::test_with_samples;

mod helper;

pub type Vect = na::Vector3<f64>;

#[test]
fn test_3d_1_80_normal() {
    test_with_samples::<Vect>(1, 0.8, 1600, Normal);
}

#[test]
fn test_3d_1_80_periodic() {
    test_with_samples::<Vect>(1, 0.8, 800, Periodic);
}

#[test]
fn test_3d_10_80_normal() {
    test_with_samples::<Vect>(10, 0.8, 400, Normal);
}

#[test]
fn test_3d_10_80_periodic() {
    test_with_samples::<Vect>(10, 0.8, 200, Periodic);
}

#[test]
fn test_3d_100_80_normal() {
    test_with_samples::<Vect>(100, 0.8, 100, Normal);
}

#[test]
fn test_3d_100_80_periodic() {
    test_with_samples::<Vect>(100, 0.8, 50, Periodic);
}

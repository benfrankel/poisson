extern crate nalgebra as na;

use poisson::{algorithm, Builder, Type};
use rand::{rngs::SmallRng, SeedableRng};

#[test]
fn reproduce_issue_29() {
    let seed = [160, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let rng = SmallRng::from_seed(seed);
    Builder::with_radius(0.004, Type::Normal)
        .build(rng, algorithm::Bridson)
        .generate();
}

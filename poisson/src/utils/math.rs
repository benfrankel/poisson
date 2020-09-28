use std::f32::consts::PI;

use crate::Type;

fn newton(samples: usize) -> usize {
    const ALPHA: f64 = 1.0997;
    const BETA: f64 = -0.4999;

    let mut n = 1f64;
    for _ in 0..5 {
        n = n
            - (n + ALPHA * n.powf(BETA + 1.0) - samples as f64)
                / (1.0 + ALPHA * (BETA + 1.0) * n.powf(BETA));
        if n < 1.0 {
            return 1;
        }
    }
    n as usize
}

/// Calculates radius from approximate samples and relative radius.
/// The amount of samples should be larger than 0.
/// The relative radius should be in (0, 1].
/// Based on Gamito, Manuel N., and Steve C. Maddock. "Accurate multidimensional Poisson-disk sampling." ACM Transactions on Graphics (TOG) 29.1 (2009): 8.
pub fn calc_radius(samples: usize, relative: f32, poisson_type: Type) -> f32 {
    use crate::Type::*;

    const GAMMA: f32 = 1.0;
    // TODO: Replace 1.7320508 with 3f32.sqrt() once sqrt is const
    //       (see https://github.com/rust-lang/rust/issues/57241)
    const MAX_PACKING_DENSITY: f32 = 1.0 / 6.0 * PI * 1.7320508;
    const MAX_RADIUS: f32 = MAX_PACKING_DENSITY * GAMMA / PI;

    assert!(samples > 0);
    assert!(0.0 < relative && relative <= 1.0);

    let samples = match poisson_type {
        Periodic => samples,
        Normal => newton(samples),
    };
    (MAX_RADIUS / (samples as f32)).sqrt() * relative
}

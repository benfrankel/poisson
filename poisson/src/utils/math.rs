use crate::Type;

// const TAU: f64  = 6.283185307179586476925286766559005768394338798750211641949;
const HALF_TAU: f64 = 3.141592653589793238462643383279502884197169399375105820974;

lazy_static! {
    static ref MAX_PACKING_DENSITIES: [f64; 7] = [
        1. / 6. * HALF_TAU * 3f64.sqrt(),
        1. / 6. * HALF_TAU * 2f64.sqrt(),
        1. / 16. * HALF_TAU.powi(2),
        1. / 30. * HALF_TAU.powi(2) * 2f64.sqrt(),
        1. / 144. * HALF_TAU.powi(3) * 3f64.sqrt(),
        1. / 105. * HALF_TAU.powi(3),
        1. / 384. * HALF_TAU.powi(4),
        ];
    // gamma((index + 2) / 2 + 1)
    static ref GAMMA: [f64; 7] = [
        1.,
        (3. * HALF_TAU.sqrt()) / 4.,
        2.,
        (15. * HALF_TAU.sqrt()) / 8.,
        6.,
        (105. * HALF_TAU.sqrt()) / 16.,
        24.,
        ];
    static ref MAX_RADII: [f64; 7] = [
            precalc(2),
            precalc(3),
            precalc(4),
            precalc(5),
            precalc(6),
            precalc(7),
            precalc(8),
        ];
    //TODO: Paper provides needed constants only for 2, 3 and 4 dimensions.
    static ref ALPHA: [f64; 3] = [
            1.0997,
            2.2119,
            4.1114,
        ];
    static ref BETA: [f64; 3] = [
            -0.4999,
            -0.3538,
            -0.3056,
        ];
}

fn precalc(dim: usize) -> f64 {
    let index = dim - 2;
    (MAX_PACKING_DENSITIES[index] * GAMMA[index]) / HALF_TAU.powf(dim as f64 / 2.)
}

fn newton(samples: usize, dim: usize) -> usize {
    let mut n = 1f64;
    let alpha = ALPHA[dim - 2];
    let beta = BETA[dim - 2];
    for _ in 0..5 {
        n = n
            - (n + alpha * n.powf(beta + 1.) - samples as f64)
                / (1. + alpha * (beta + 1.) * n.powf(beta));
        if n < 1. {
            return 1;
        }
    }
    n as usize
}

/// Calculates radius from approximate samples and relative radius.
/// The amount of samples should be larger than 0.
/// The relative radius should be [0, 1].
/// For non-periodic this is supported only for 2, 3 and 4 dimensional generation.
/// For periodic this is supported up to 8 dimensions.
/// Based on Gamito, Manuel N., and Steve C. Maddock. "Accurate multidimensional Poisson-disk sampling." ACM Transactions on Graphics (TOG) 29.1 (2009): 8.
pub fn calc_radius(samples: usize, relative: f32, poisson_type: Type) -> f32 {
    use crate::Type::*;
    assert!(samples > 0);
    assert!(0.0 < relative && relative <= 1.0);
    let samples = match poisson_type {
        Periodic => samples,
        Normal => newton(samples, 2),
    };
    let max_radii = MAX_RADII[0] as f32;
    (max_radii / (samples as f32)).powf(0.5) * relative
}

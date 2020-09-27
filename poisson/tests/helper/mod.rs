#![allow(unused)]

use std::fmt::Debug;

use glam::Vec2;
use poisson::{algorithm, Builder, Type};
use rand::distributions::{Distribution, Standard};
use rand::{rngs::SmallRng, SeedableRng};

pub fn print_v(v: Vec2) -> String {
    let mut result = "(".to_owned();
    for i in 0..2 {
        result.push_str(&format!("{}, ", v[i] as f64));
    }
    result.push(')');
    result
}

#[derive(Clone, Copy)]
pub enum When {
    Always,
    Sometimes,
    Never,
}

pub fn test_with_samples(samples: usize, relative_radius: f32, seeds: u32, ptype: Type) {
    test_with_samples_prefilled(
        samples,
        relative_radius,
        seeds,
        ptype,
        |_| |_| None,
        When::Always,
    );
}

pub fn test_with_samples_prefilled<F, I>(
    samples: usize,
    relative_radius: f32,
    seeds: u32,
    ptype: Type,
    mut prefiller: F,
    valid: When,
) where
    F: FnMut(f32) -> I,
    I: FnMut(Option<Vec2>) -> Option<Vec2>,
{
    test_algo(
        samples,
        relative_radius,
        seeds,
        ptype,
        &mut prefiller,
        valid,
        algorithm::Ebeida,
    );
    test_algo(
        samples,
        relative_radius,
        seeds,
        ptype,
        &mut prefiller,
        valid,
        algorithm::Bridson,
    );
}

fn test_algo<F, I, A>(
    samples: usize,
    relative_radius: f32,
    seeds: u32,
    ptype: Type,
    prefiller: &mut F,
    valid: When,
    algo: A,
) where
    F: FnMut(f32) -> I,
    I: FnMut(Option<Vec2>) -> Option<Vec2>,
    A: algorithm::Creator,
{
    use self::When::*;
    for i in 0..seeds {
        let mut prefilled = vec![];
        let rand = SmallRng::from_seed([
            (i * 3 + 2741) as u8,
            (i * 7 + 2729) as u8,
            (i * 13 + 2713) as u8,
            (i * 19 + 2707) as u8,
            (i * 29 + 2693) as u8,
            (i * 37 + 2687) as u8,
            (i * 43 + 2677) as u8,
            (i * 53 + 2663) as u8,
            (i * 61 + 2657) as u8,
            (i * 71 + 2633) as u8,
            (i * 79 + 2609) as u8,
            (i * 89 + 2591) as u8,
            (i * 101 + 2557) as u8,
            (i * 107 + 2549) as u8,
            (i * 113 + 2539) as u8,
            (i * 131 + 2521) as u8,
        ]);
        let mut poisson_iter = Builder::with_samples(samples, relative_radius, ptype)
            .build(rand, algo)
            .into_iter();
        let mut poisson = vec![];
        let mut prefill = (prefiller)(poisson_iter.radius());
        let mut last = None;
        let mut does_prefill = false;
        loop {
            while let Some(p) = (prefill)(last) {
                does_prefill = true;
                match valid {
                    Always => assert!(
                        poisson_iter.stays_legal(p.into()),
                        "All prefilled should be accepted by the '{:?}' algorithm. \
                         {} was rejected.",
                        algo,
                        print_v(p)
                    ),
                    Never => assert!(
                        !poisson_iter.stays_legal(p.into()),
                        "All prefilled should be rejected by the '{:?}' algorithm. \
                         {} was allowed even though {:?} was last to be generated.",
                        algo,
                        print_v(p),
                        last.map(print_v)
                    ),
                    _ => {}
                }
                prefilled.push(p);
                poisson_iter.restrict(p.into());
            }
            if let Some(pp) = poisson_iter.next() {
                last = Some(pp.into());
                poisson.push(pp.into());
            } else {
                break;
            }
        }
        let radius = poisson_iter.radius();
        let poisson_type = poisson_iter.poisson_type();
        let poisson = poisson.into_iter().chain(
            if let Always = valid {
                prefilled
            } else {
                vec![]
            }
            .into_iter(),
        );
        test_poisson(poisson, radius, poisson_type, algo, does_prefill);
    }
}

pub fn test_poisson<I, A>(poisson: I, radius: f32, poisson_type: Type, algo: A, does_prefill: bool)
where
    I: Iterator<Item = Vec2>,
    A: algorithm::Creator,
{
    use poisson::Type::*;
    let mut vecs = vec![];
    let mut hints = vec![];
    {
        let mut iter = poisson.into_iter();
        while let Some(v) = iter.next() {
            if let (low, Some(high)) = iter.size_hint() {
                hints.push((low, high));
            } else {
                panic!(
                    "There wasn't hint for {}th iteration for the '{:?}' algorithm.",
                    hints.len(),
                    algo
                );
            }
            vecs.push(v);
        }
    }
    let len = hints.len();
    for (n, (l, h)) in hints.into_iter().enumerate() {
        let remaining = len - (n + 1);
        assert!(l <= remaining, "For the '{:?}' algorithm the lower bound of hint should be smaller than or equal to actual: {} <= {}", algo, l, remaining);
        assert!(h >= remaining, "For the '{:?}' algorithm the upper bound of hint should be larger than or equal to actual: {} >= {}", algo, h, remaining);
    }

    if !does_prefill {
        for v in &vecs {
            for n in 0..2 {
                assert!(v[n] >= 0.0);
                assert!(v[n] < 1.0);
            }
        }
    }

    let vecs = match poisson_type {
        Periodic => {
            let mut vecs2 = vec![];
            for n in 0..9i64 {
                let mut t = Vec2::zero();
                let mut div = n;
                for i in 0..2 {
                    let rem = div % 3;
                    div /= 3;
                    t[i] = (rem - 1) as f32;
                }
                for v in &vecs {
                    vecs2.push(*v + t);
                }
            }
            vecs2
        }
        Normal => vecs,
    };

    //TODO: Figure out how to check if distribution is maximal.
    assert_legal_poisson(&vecs, radius, algo);
}

pub fn assert_legal_poisson<A>(vecs: &Vec<Vec2>, radius: f32, algo: A)
where
    A: algorithm::Creator,
{
    for &v1 in vecs {
        for &v2 in vecs {
            if v1 == v2 {
                continue;
            }
            let dist = (v1 - v2).length();
            assert!(dist > radius * 2.0,
                    "Poisson-disk distribution requirement not met while generating using the '{:?}' algorithm: There exists 2 vectors with \
                     distance to each other of {} which is smaller than smallest allowed one {}. \
                     The samples: [{:?}, {:?}]",
                    algo,
                    dist as f64,
                    (radius as f64) * 2.0,
                    v1,
                    v2);
        }
    }
}

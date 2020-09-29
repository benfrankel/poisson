use std::iter::repeat;

use glam::Vec2;
use poisson2d::{algorithm, Builder, Type};
use rand::{rngs::SmallRng, SeedableRng};

use crate::helper::When::*;

mod helper;

#[test]
fn adding_valid_start_works() {
    let samples = 100;
    let relative_radius = 0.8;
    let rand = SmallRng::from_seed([
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    ]);
    let prefiller = |_| {
        let mut pre =
            Builder::with_samples(samples, relative_radius, Type::Normal)
                .build(rand.clone(), algorithm::Ebeida)
                .into_iter()
                .take(25);
        move |_| pre.next().map(|v| v.into())
    };
    helper::test_with_samples_prefilled(
        samples,
        relative_radius,
        100,
        Type::Normal,
        prefiller,
        Always,
    );
}

#[test]
fn adding_valid_middle_works() {
    let samples = 100;
    let relative_radius = 0.8;
    let rand = SmallRng::from_seed([
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    ]);
    let prefiller = |_| {
        let prefiller =
            Builder::with_samples(samples, relative_radius, Type::Normal)
                .build(rand.clone(), algorithm::Ebeida);
        let mut pre = repeat(None)
            .take(25)
            .chain(prefiller.into_iter().take(25).map(Some));
        move |_| pre.next().and_then(|s| s).map(|v| v.into())
    };

    // TODO: At 40 the test suddenly takes forever and takes all of the memory resulting into getting killed by oom killer
    helper::test_with_samples_prefilled(
        samples,
        relative_radius,
        30,
        Type::Normal,
        prefiller,
        Sometimes,
    );
}

#[test]
fn adding_to_edges_start_works() {
    let samples = 100;
    let relative_radius = 0.8;
    let prefiller = [
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 0.5),
        Vec2::new(0.0, 1.0),
        Vec2::new(0.5, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.5, 1.0),
        Vec2::new(1.0, 0.5),
        Vec2::new(1.0, 1.0),
    ];
    let prefiller = |_| {
        let mut pre = prefiller.iter().cloned().map(Some as fn(_) -> _);
        move |_| pre.next().and_then(|s| s)
    };
    helper::test_with_samples_prefilled(
        samples,
        relative_radius,
        100,
        Type::Normal,
        prefiller,
        Always,
    );
}

#[test]
fn adding_to_outside_of_edges_start_works() {
    let samples = 100;
    let relative_radius = 0.8;
    let prefiller = [
        Vec2::new(-0.1, -0.1),
        Vec2::new(-0.1, 0.5),
        Vec2::new(-0.1, 1.1),
        Vec2::new(0.5, -0.1),
        Vec2::new(1.1, -0.1),
        Vec2::new(0.5, 1.1),
        Vec2::new(1.1, 0.5),
        Vec2::new(1.1, 1.1),
    ];
    let prefiller = |_| {
        let mut pre = prefiller.iter().cloned().map(Some as fn(_) -> _);
        move |_| pre.next().and_then(|s| s)
    };
    helper::test_with_samples_prefilled(
        samples,
        relative_radius,
        100,
        Type::Normal,
        prefiller,
        Always,
    );
}

#[test]
fn completely_filled_works() {
    let samples = 100;
    let relative_radius = 0.8;
    let rand = SmallRng::from_seed([
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    ]);
    let prefiller = |_| {
        let mut pre =
            Builder::with_samples(samples, relative_radius, Type::Normal)
                .build(rand.clone(), algorithm::Ebeida)
                .into_iter();
        move |_| pre.next().map(|v| v.into())
    };
    helper::test_with_samples_prefilled(
        samples,
        relative_radius,
        100,
        Type::Normal,
        prefiller,
        Always,
    );
}

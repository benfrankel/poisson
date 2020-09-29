//! Helper functions that poisson2d uses.

use glam::Vec2;
use modulo::Mod;
use rand::Rng;

use crate::{Builder, Type};

pub mod math;

#[derive(Clone)]
pub struct Grid {
    data: Vec<Vec<Vec2>>,
    side: usize,
    cell: f32,
    poisson_type: Type,
}

impl Grid {
    pub fn new(radius: f32, poisson_type: Type) -> Grid {
        let cell = radius * 2.0 / 2f32.sqrt();
        let side = (1.0 / cell) as usize;
        Grid {
            cell,
            side,
            data: vec![vec![]; side.pow(2)],
            poisson_type,
        }
    }

    pub fn get(&self, index: Vec2) -> Option<&Vec<Vec2>> {
        encode(&index, self.side, self.poisson_type).map(|t| &self.data[t])
    }

    pub fn get_mut(&mut self, index: Vec2) -> Option<&mut Vec<Vec2>> {
        encode(&index, self.side, self.poisson_type).map(move |t| &mut self.data[t])
    }

    pub fn cells(&self) -> usize {
        self.data.len()
    }

    pub fn side(&self) -> usize {
        self.side
    }

    pub fn cell(&self) -> f32 {
        self.cell
    }
}

pub fn encode(v: &Vec2, side: usize, poisson_type: Type) -> Option<usize> {
    use crate::Type::*;
    let mut index = 0;
    for n in 0..2 {
        let n = v[n];
        let cur = match poisson_type {
            Periodic => (n as isize)
                .modulo(side as isize) as usize,
            Normal => {
                if n < 0.0 || n >= side as f32 {
                    return None;
                }
                n as usize
            }
        };
        index = (index + cur) * side;
    }
    Some(index / side)
}

pub fn decode(index: usize, side: usize) -> Option<Vec2> {
    if index >= side.pow(2) {
        return None;
    }
    let mut result = Vec2::zero();
    let mut last = index;
    for n in (0..2).rev() {
        let cur = last / side;
        result[n] = (last - cur * side) as f32;
        last = cur;
    }
    Some(result)
}

#[test]
fn encoding_decoding_works() {
    let n = Vec2::new(10.0, 7.0);
    assert_eq!(
        n,
        decode(encode(&n, 15, Type::Normal).unwrap(), 15).unwrap(),
    );
}

#[test]
fn encoding_decoding_at_edge_works() {
    let n = Vec2::new(14.0, 14.0);
    assert_eq!(
        n,
        decode(encode(&n, 15, Type::Normal).unwrap(), 15).unwrap()
    );
}

#[test]
fn encoding_outside_of_area_fails() {
    let n = Vec2::new(9.0, 7.0);
    assert_eq!(None, encode(&n, 9, Type::Normal));
    let n = Vec2::new(7.0, 9.0);
    assert_eq!(None, encode(&n, 9, Type::Normal));
}

#[test]
fn decoding_outside_of_area_fails() {
    assert_eq!(None, decode(100, 10));
}

pub fn choose_random_sample<R>(rng: &mut R, grid: &Grid, index: Vec2, level: usize) -> Vec2
where
    R: Rng,
{
    let side = 2usize.pow(level as u32);
    let spacing = grid.cell / (side as f32);
    (index + rng.gen()) * spacing
}

#[test]
fn random_point_is_between_right_values_top_lvl() {
    use rand::{rngs::SmallRng, SeedableRng};
    let mut rand = SmallRng::from_seed([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let radius = 0.2;
    let grid = Grid::new(radius, Type::Normal);
    for _ in 0..1000 {
        let result = choose_random_sample(&mut rand, &grid, Vec2::zero(), 0);
        assert!(result.x() >= 0.0);
        assert!(result.x() < grid.cell);
        assert!(result.y() >= 0.0);
        assert!(result.y() < grid.cell);
    }
}

pub fn sample_to_index(value: &Vec2, side: usize) -> Vec2 {
    let mut cur = value.clone();
    for n in 0..2 {
        cur[n] = (cur[n] * (side as f32)).floor();
    }
    cur
}

pub fn index_to_sample(value: &Vec2, side: usize) -> Vec2 {
    let mut cur = value.clone();
    for n in 0..2 {
        cur[n] = cur[n] / (side as f32);
    }
    cur
}

pub fn is_disk_free(
    grid: &Grid,
    poisson: &Builder,
    index: Vec2,
    level: usize,
    sample: Vec2,
    outside: &[Vec2],
) -> bool {
    let parent = get_parent(index, level);
    let sqradius = (2.0 * poisson.radius).powi(2);
    // NOTE: This does unnecessary checks for corners, but it doesn't affect much in higher dimensions: 5^d vs 5^d - 2d
    each_combination(&[-2.0, -1.0, 0.0, 1.0, 2.0])
        .filter_map(|t| grid.get(parent.clone() + t))
        .flat_map(|t| t)
        .all(|v| sqdist(v.clone(), sample.clone(), poisson.poisson_type) >= sqradius)
        && is_valid(poisson, outside, sample)
}

pub fn is_valid(poisson: &Builder, samples: &[Vec2], sample: Vec2) -> bool {
    let sqradius = (2.0 * poisson.radius).powi(2);
    samples
        .iter()
        .all(|t| sqdist(t.clone(), sample.clone(), poisson.poisson_type) >= sqradius)
}

pub fn sqdist(v1: Vec2, v2: Vec2, poisson_type: Type) -> f32 {
    use crate::Type::*;
    let diff = v2 - v1;
    match poisson_type {
        Periodic => each_combination(&[-1.0, 0.0, 1.0])
            .map(|v| (diff.clone() + v).length_squared())
            .fold(std::f32::MAX, |a, b| a.min(b)),
        Normal => diff.length_squared(),
    }
}

pub fn get_parent(mut index: Vec2, level: usize) -> Vec2 {
    let split = 2usize.pow(level as u32);
    for n in 0..2 {
        index[n] = (index[n] / (split as f32)).floor();
    }
    index
}

#[test]
fn getting_parent_works() {
    let divides = 4;
    let cells_per_cell = 2usize.pow(divides as u32);
    let testee = Vec2::new(1.0, 2.0);
    assert_eq!(
        testee,
        get_parent(
            (testee * cells_per_cell as f32) + Vec2::new(0.0, 15.0),
            divides
        )
    );
}

pub struct CombiIter<'a> {
    cur: usize,
    choices: &'a [f32],
}

impl<'a> Iterator for CombiIter<'a> {
    type Item = Vec2;
    fn next(&mut self) -> Option<Self::Item> {
        let len = self.choices.len();
        if self.cur >= len.pow(2) {
            None
        } else {
            let mut result = Vec2::zero();
            let mut div = self.cur;
            self.cur += 1;
            for n in 0..2 {
                let rem = div % len;
                div /= len;
                let choice = self.choices[rem as usize].clone();
                result[n] = choice as f32;
            }
            Some(result)
        }
    }
}

/// Iterates through all combinations of vectors with allowed values as scalars.
pub fn each_combination(choices: &[f32]) -> CombiIter {
    CombiIter {
        cur: 0,
        choices,
    }
}

/// Trait that allows flat mapping inplace.
pub trait Inplace<T> {
    /// Does flat map inplace without maintaining order of elements.
    fn flat_map_inplace<F, I>(&mut self, f: F)
    where
        I: IntoIterator<Item = T>,
        F: FnMut(T) -> I;
}

impl<T> Inplace<T> for Vec<T> {
    fn flat_map_inplace<F, I>(&mut self, mut f: F)
    where
        I: IntoIterator<Item = T>,
        F: FnMut(T) -> I,
    {
        for i in (0..self.len()).rev() {
            for t in f(self.swap_remove(i)) {
                self.push(t);
            }
        }
    }
}

#[test]
fn mapping_inplace_works() {
    let vec = vec![1, 2, 3, 4, 5, 6];
    let mut result = vec.clone();
    let func = |t| {
        match t % 3 {
            0 => (0..0),
            1 => (0..1),
            _ => (0..2),
        }
        .map(move |n| t + n)
    };
    result.flat_map_inplace(&func);
    let mut expected = vec.into_iter().flat_map(func).collect::<Vec<_>>();
    assert_eq!(expected.sort(), result.sort());
}

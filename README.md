# poisson

[![Documentation][di]][dl] [![Crates.io][ri]][rl] [![License: MIT][li]][ll] [![Build Status][ti]][tl] [![Coverage Status][ci]][cl]   

[di]: https://docs.rs/poisson/badge.svg
[dl]: https://docs.rs/poisson

[ri]: https://img.shields.io/crates/v/poisson.svg
[rl]: https://crates.io/crates/poisson/

[li]: https://img.shields.io/badge/License-MIT-blue.svg
[ll]: https://opensource.org/licenses/MIT

[ti]: https://travis-ci.org/WaDelma/poisson.svg?branch=master
[tl]: https://travis-ci.org/WaDelma/poisson

[ci]: https://coveralls.io/repos/github/WaDelma/poisson/badge.svg?branch=master
[cl]: https://coveralls.io/github/WaDelma/poisson?branch=master

This is a library for generating 2-dimensional [Poisson disk samplings](http://mollyrocket.com/casey/stream_0014.html).    

Specifically, it can generate a sampling of points in [0, 1)<sup>2</sup> where:

 * Sample points fill the space uniformly.
 * Sample points stay a given minimum distance apart.

This is equivalent to uniformly filling a unit square with non-overlapping
disks of equal radius, where the radius is half the minimum distance.

Due to their blue noise properties, Poisson disk samplings can be used for
object placement in procedural texture/world generation, digital stippling,
sampling in rendering, or (re)meshing.

# Usage

Works with mint 0.5 and rand 0.7.

```rust
use poisson::{Builder, Type, algorithm};
use rand::FromEntropy;
use rand::rngs::SmallRng;

fn main() {
    let poisson =
        Builder::with_radius(0.1, Type::Normal)
            .build(SmallRng::from_entropy(), algorithm::Ebeida);
    println!("{:?}", poisson.generate());
}
```

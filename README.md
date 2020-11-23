# Bumpalo-herd

[![Actions Status](https://github.com/vorner/bumpalo-herd/workflows/test/badge.svg)](https://github.com/vorner/bumpalo-herd/actions)
[![codecov](https://codecov.io/gh/vorner/bumpalo-herd/branch/main/graph/badge.svg?token=3KA3R2D9fV)](https://codecov.io/gh/vorner/bumpalo-herd)
[![docs](https://docs.rs/bumpalo-herd/badge.svg)](https://docs.rs/bumpalo-herd)

The [bumpalo](https://crates.io/crates/bumpalo) offers a good speedup for
certain application by providing a bump allocator. But it is not well suited for
some multi threaded scenarios.

This provides a wrapper on top of `bumpalo` to make it possible to use in such
scenarios (like inside [rayon](https://crates.io/crates/rayon) or with scoped
threads).

For further details, see the [documentation](https://docs.rs/bumpalo-herd).

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms
or conditions.

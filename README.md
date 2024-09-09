# Babushka &emsp; [![Build Status]][actions] [![Latest Version]][crates.io] [![documentation](https://docs.rs/num-traits/badge.svg)](https://docs.rs/num-traits)
[Build Status]: https://img.shields.io/github/actions/workflow/status/davefol/babushka/rust.yml?branch=main
[actions]: https://github.com/davefol/babushka/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/babushka.svg
[crates.io]: https://crates.io/crates/babushka
Babushka is a Rust library for 2D geometry algorithms, with a focus on efficient bin packing and nesting.

## Features
- 2D geometry primitives (points, segments, polygons)
- Hierarchical primitives (piece)
- No Fit Polygon implementation inspired by SVGNest
- Rasterizing into `Vec<u32>` using the "raster" feature eg. drawing shapes with holes

![nfp_0](./assets/nfp_0.gif)

## Installation

Add this to your `Cargo.toml`:

[dependencies]
babushka = "0.1.7"

## Documentation

For detailed documentation and advanced usage examples, please visit [docs.rs/babushka](https://docs.rs/babushka).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

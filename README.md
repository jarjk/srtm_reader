# SRTM/HGT reader

A performant DTM[(srtm)](https://www.earthdata.nasa.gov/sensors/srtm) reader for `.hgt` files in [Rust](https://rust-lang.org).

## Supported resolutions

-   0.5 angle second (SRTM05) <- *not sure that's how it's called*
-   1 angle second (SRTM1)
-   3 angle second (SRTM3)

-   _feel free to open an issue if you need more_

## Example

```rust
use srtm_reader::*;

let coord = Coord::new(13.3255424, 56.92856);
// we get the filename, that shall include the elevation data for this `coord`
let filename = coord.get_filename();
// load the srtm, .hgt file
// NOTE: to be able to load it, you'll need the actual file
let tile = srtm_reader::Tile::from_file(filename).unwrap();
// and finally, retrieve our elevation data
let elevation = tile.get(coord);
```

also, see [cli example](./examples/cli.rs) for a real-life one

> [!NOTE]
> a great source of DEM data, `.hgt` files is [Sonny's collection](https://sonny.4lima.de/)

## Dependents

-   [fit2gpx-rs](https://github.com/jarjk/fit2gpx-rs)
-   *file an issue if yours could be listed as well*

## Disclaimer

this crate is a forked version of the [srtm crate](https://github.com/grtlr/srtm) which hasn't been updated in 6 years, and the PR hasn't been merged either in a long time.
I've needed 0.5 angle support and also some more convenience methods for [fit2gpx-rs](https://github.com/jarjk/fit2gpx-rs), and here we are.

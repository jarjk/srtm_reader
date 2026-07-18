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

let coord = Coord::new(44.32554, 15.92856);
// we get the filename, that shall include the elevation data for this `coord`
let filename = coord.get_filename();
// in this case, the filename will be:
assert_eq!(filename, "N44E015.hgt");
// load the srtm, .hgt file
// NOTE: this file is included in the repo
let tile = srtm_reader::Tile::from_file(filename).unwrap();
// and finally, retrieve our elevation data
let elevation = tile.get(coord);
println!("elevation for coordinates ({coord:?}): {elevation:?}m");
```

also, see [cli example](./examples/cli.rs) for a real-life one

> [!NOTE]
> great source of DEM data, `.hgt` files is
> - the high quality [collection of Sonny](https://sonny.4lima.de/) for Europe
> - and [the SRTM tile downloader](https://dwtkns.com/srtm30m/) or [AWS terrain tiles](https://registry.opendata.aws/terrain-tiles/) otherwise

## Dependents

-   [fit2gpx-rs](https://github.com/jarjk/fit2gpx-rs)
-   *file an issue if yours could be listed as well*

## Disclaimer, Acknowledgement

This crate is a forked version of the [srtm crate](https://github.com/grtlr/srtm) which hasn't been updated since 2018.
I've needed 0.5 angle support and also some further convenience methods for [fit2gpx-rs](https://github.com/jarjk/fit2gpx-rs),
but my PR for the improvements hasn't been merged for a long-long time, so here we are.

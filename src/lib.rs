//! A performant [srtm](https://www.earthdata.nasa.gov/sensors/srtm) reader for `.hgt` files.
//!
//! # Usage
//!
//! ```rust
//! use srtm_reader::{Tile, Coord};
//! use std::path::PathBuf;
//!
//! // the actual elevation of Veli Brig, 263m
//! const TRUE_ELEV: i16 = 263;
//! // the coordinates of Veli Brig, actual elevation: 263m
//! let coord = Coord::new(44.4480403, 15.0733053);
//! // we get the filename, that shall include the elevation data for this `coord`
//! let filename = coord.get_filename();
//! // in this case, the filename will be:
//! assert_eq!(filename, "N44E015.hgt");
//! // load the srtm tile: .hgt file
//! let tile = Tile::from_file(filename).unwrap();
//! // and finally, retrieve our elevation for Veli Brig
//! let elevation = tile.get(coord).unwrap();
//! // test with a ± 5m accuracy
//! assert!((TRUE_ELEV - 5..TRUE_ELEV + 5).contains(&elevation));
//! println!("Veli Brig:\n\t- coordinates: {coord:?}\n\t- elevation\n\t\t- actual: {TRUE_ELEV}m\n\t\t- calculated: {elevation}m");
//! ```

pub use coords::Coord;
pub use resolutions::Resolution;
pub use tiles::Tile;

pub mod coords;
pub mod resolutions;
#[cfg(test)]
mod tests;
pub mod tiles;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NotFound,
    ParseLatLong,
    Filesize,
    Read,
}

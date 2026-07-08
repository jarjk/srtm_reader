#![doc = include_str!("../README.md")]

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

#![doc = include_str!("../README.md")]
#![deny(clippy::unwrap_used)]
#![deny(unsafe_code)]

pub use coords::Coord;
pub use resolutions::Resolution;
pub use tiles::Tile;

pub mod coords;
pub mod resolutions;
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
pub mod tiles;

macro_rules! new_err {
    (of: $err:expr) => {
        std::io::Error::other($err)
    };
    ($kind:ident) => {
        std::io::Error::from(std::io::ErrorKind::$kind)
    };
    ($kind:ident, $err:expr) => {
        std::io::Error::new(std::io::ErrorKind::$kind, $err)
    };
}

pub(crate) use new_err;

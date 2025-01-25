//! A performant [srtm](https://www.earthdata.nasa.gov/sensors/srtm) reader for `.hgt` files.
//!
//! # Usage
//!
//! ```rust
//! use srtm_reader::Tile;
//! use std::path::PathBuf;
//!
//! // the actual elevation of Veli Brig, 263m
//! const TRUE_ELEV: i16 = 263;
//! // the coordinates of Veli Brig, actual elevation: 263m
//! let coord = (44.4480403, 15.0733053);
//! // we get the filename, that shall include the elevation data for this `coord`
//! let filename = srtm_reader::get_filename(coord);
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
use resolutions::Resolution;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub mod coords;
mod resolutions;

/// the SRTM tile, which contains the actual elevation data
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tile {
    /// north-south position of the [`Tile`]
    /// angle, ranges from −90° (south pole) to 90° (north pole), 0° is the Equator
    pub latitude: i8,
    /// east-west position of the [`Tile`]
    /// angle, ranges from -180° to 180°
    pub longitude: i16,
    pub resolution: Resolution,
    pub data: Vec<i16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NotFound,
    ParseLatLong,
    Filesize,
    Read,
}

impl Tile {
    fn empty(lat: i8, lon: i16, res: Resolution) -> Tile {
        Tile {
            latitude: lat,
            longitude: lon,
            resolution: res,
            data: Vec::with_capacity(res.total_len()),
        }
    }

    /// read an srtm: `.hgt` file, and create a [`Tile`] if possible
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Tile, Error> {
        let file = File::open(&path).map_err(|_| Error::NotFound)?;
        // eprintln!("file: {file:?}");

        let f_len = file.metadata().map_err(|_| Error::Filesize)?.len();
        let res = Resolution::try_from(f_len).map_err(|_| Error::Filesize)?;
        // eprintln!("resolution: {res:?}");

        let (lat, lon) = get_lat_long(&path)?;
        let mut tile = Tile::empty(lat, lon, res);

        tile.data = parse_hgt(file, res).map_err(|_| Error::Read)?;

        Ok(tile)
    }

    /// the maximum height that this [`Tile`] contains
    pub fn max_height(&self) -> i16 {
        *self.data.iter().max().unwrap_or(&0)
    }
    pub fn min_height(&self) -> i16 {
        *self.data.iter().min().unwrap_or(&0)
    }
    /// get lower-left corner's latitude and longitude
    /// it's needed for [`Tile::get_offset()`]
    fn get_origin(&self, coord: Coord) -> Coord {
        let lat = coord.lat.trunc() + 1.; // The latitude of the lower-left corner of the tile
        let lon = coord.lon.trunc(); // The longitude of the lower-left corner of the tile
        Coord { lat, lon }
    }
    /// calculate where this `coord` is located in this [`Tile`]
    fn get_offset(&self, coord: Coord) -> (usize, usize) {
        let origin = self.get_origin(coord);
        // eprintln!("origin: ({}, {})", origin.0, origin.1);
        let extent = self.resolution.extent() as f64;

        let row = ((origin.lat - coord.lat) * extent) as usize;
        let col = ((coord.lon - origin.lon) * extent) as usize;
        (row, col)
    }

    /// get the elevation of this `coord` from this [`Tile`]
    ///
    /// # Panics
    /// If this [`Tile`] doesn't contain `coord`'s elevation
    /// *NOTE*: shouldn't happen if [`get_filename()`] was used
    pub fn get(&self, coord: impl Into<Coord>) -> Option<&i16> {
        let coord: Coord = coord.into();
        let offset = self.get_offset(coord);
        let lat = coord.lat.trunc() as i8;
        let lon = coord.lon.trunc() as i16;
        assert!(
            self.latitude <= lat,
            "hgt lat: {}, coord lat: {lat}",
            self.latitude
        );
        assert!(
            self.longitude <= lon,
            "hgt lon: {}, coord lon: {lon}",
            self.longitude
        );
        let elev = self.get_at_offset(offset.1, offset.0);
        if elev.is_some_and(|e| *e == -9999 || *e == i16::MIN) {
            eprintln!(
                "WARNING: in file {:?} {coord:?} doesn't contain a valid elevation: {elev:?}",
                get_filename((self.latitude, self.longitude))
            );
            None
        } else {
            elev
        }
    }

    fn get_at_offset(&self, x: usize, y: usize) -> Option<&i16> {
        self.data.get(self.idx(x, y))
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        assert!(
            x < self.resolution.extent() && y < self.resolution.extent(),
            "extent: {}, x: {x}, y: {y}",
            self.resolution.extent()
        );
        y * self.resolution.extent() + x
    }
}

fn parse_hgt(mut reader: impl Read, res: Resolution) -> io::Result<Vec<i16>> {
    let mut buffer = vec![0; res.total_len() * 2];
    reader.read_exact(&mut buffer)?;
    let mut elevations = Vec::with_capacity(res.total_len());
    for chunk in buffer.chunks_exact(2) {
        let value = i16::from_be_bytes([chunk[0], chunk[1]]);
        elevations.push(value);
    }
    Ok(elevations)
}

// FIXME: Better error handling.
fn get_lat_long<P: AsRef<Path>>(path: P) -> Result<(i8, i16), Error> {
    let stem = path.as_ref().file_stem().ok_or(Error::ParseLatLong)?;
    let desc = stem.to_str().ok_or(Error::ParseLatLong)?;
    if desc.len() != 7 {
        return Err(Error::ParseLatLong);
    }

    let get_char = |n| desc.chars().nth(n).ok_or(Error::ParseLatLong);
    let lat_sign = if get_char(0)? == 'N' { 1 } else { -1 };
    let lat: i8 = desc[1..3].parse().map_err(|_| Error::ParseLatLong)?;

    let lon_sign = if get_char(3)? == 'E' { 1 } else { -1 };
    let lon: i16 = desc[4..7].parse().map_err(|_| Error::ParseLatLong)?;
    Ok((lat_sign * lat, lon_sign * lon))
}
/// get the name of the file, which shall include this `coord`s elevation
///
/// # Usage
///
/// ```rust
/// // the `coord`inate, whe want the elevation for
/// let coord = (87.235, 10.4234423);
/// // this convenient function gives us the filename for
/// // any `coord`inate, that is `impl Into<srtm_reader::Coord>`
/// // which is true for this tuple
/// let filename = srtm_reader::get_filename(coord);
/// assert_eq!(filename, "N87E010.hgt");
/// ```
pub fn get_filename<C: Into<Coord>>(coord: C) -> String {
    let coord: Coord = coord.into();
    let lat_ch = if coord.lat >= 0. { 'N' } else { 'S' };
    let lon_ch = if coord.lon >= 0. { 'E' } else { 'W' };
    let lat = (coord.lat.trunc() as i32).abs();
    let lon = (coord.lon.trunc() as i32).abs();
    format!(
        "{lat_ch}{}{lat}{lon_ch}{}{lon}.hgt",
        if lat < 10 { "0" } else { "" },
        if lon < 10 {
            "00"
        } else if lon < 100 {
            "0"
        } else {
            ""
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parse_latitute_and_longitude() {
        let ne = Path::new("N35E138.hgt");
        assert_eq!(get_lat_long(ne).unwrap(), (35, 138));

        let nw = Path::new("N35W138.hgt");
        assert_eq!(get_lat_long(nw).unwrap(), (35, -138));

        let se = Path::new("S35E138.hgt");
        assert_eq!(get_lat_long(se).unwrap(), (-35, 138));

        let sw = Path::new("S35W138.hgt");
        assert_eq!(get_lat_long(sw).unwrap(), (-35, -138));
    }
    #[test]
    fn total_file_sizes() {
        assert_eq!(103_708_802 / 2, Resolution::SRTM05.total_len());
        assert_eq!(25_934_402 / 2, Resolution::SRTM1.total_len());
        assert_eq!(2_884_802 / 2, Resolution::SRTM3.total_len());
    }
    #[test]
    fn extents() {
        assert_eq!(7201, Resolution::SRTM05.extent());
        assert_eq!(3601, Resolution::SRTM1.extent());
        assert_eq!(1201, Resolution::SRTM3.extent());
    }

    #[test]
    #[should_panic]
    fn wrong_coord_0() {
        let _ = Coord::new(-190, 42.4);
    }
    #[test]
    #[should_panic]
    fn wrong_coord_1() {
        let _ = Coord::new(180, -42.4);
    }
    #[test]
    #[should_panic]
    fn wrong_coord_2() {
        let _ = Coord::new(-90., 181.);
    }
    #[test]
    #[should_panic]
    fn wrong_coord_3() {
        let _ = Coord::new(90., -180.00001);
    }
    #[test]
    fn correct_coord_0() {
        let _ = Coord::new(-90, 180);
    }
    #[test]
    fn correct_coord_1() {
        let _ = Coord::new(90, -180);
    }
    #[test]
    fn correct_coord_2() {
        let c = Coord::new(90, -180).with_lon(-85.7);
        assert_eq!(Coord::new(90, -85.7), c);
    }
    #[test]
    fn correct_coord_3() {
        let c = Coord::new(90, -180).with_lat(0.3);
        assert_eq!(Coord::new(0.3, -180), c);
    }
    #[test]
    fn correct_coord_4() {
        let c = Coord::new(90, -180).with_lat(0.3).with_lon(83.3);
        assert_eq!(Coord::new(0.3, 83.3), c);
    }
    #[test]
    fn correct_coord_5() {
        let c: Coord = (90, -180).into();
        let c = c.with_lat(0.3).with_lon(83.3);
        assert_eq!(Coord::new(0.3, 83.3), c);
    }
    #[test]
    fn correct_coord_6() {
        let c: Coord = (90, -180).into();
        let c = c.with_lat(0.3).with_lon(83.3);
        assert_eq!(Coord::new(0.3, 83.3), c);
    }
    #[test]
    fn correct_coord_7() {
        let c: Coord = (-90, 180).into();
        let c = c.add_to_lat(0.3252).add_to_lon(-3.2);
        assert_eq!(Coord::new(-89.6748, 176.8), c);
    }
    fn coords() -> [Coord; 3] {
        [(45, 1.4).into(), (-2.3, 87).into(), (35, -7).into()]
    }
    #[test]
    fn file_names() {
        let fnames = coords()
            .iter()
            .map(|c| get_filename(*c))
            .collect::<Vec<_>>();
        assert_eq!(fnames[0], "N45E001.hgt");
        assert_eq!(fnames[1], "S02E087.hgt");
        assert_eq!(fnames[2], "N35W007.hgt");
    }
    #[test]
    fn read() {
        let coord = Coord::new(44.4480403, 15.0733053);
        let fname = get_filename(coord);
        let tile = Tile::from_file(fname).unwrap();
        assert_eq!(tile.latitude, 44);
        assert_eq!(tile.longitude, 15);
        assert_eq!(tile.resolution, Resolution::SRTM1);
        assert_eq!(tile.data.len(), Resolution::SRTM1.total_len());

        let elev = tile.get(coord);
        assert_eq!(elev, Some(&258));
    }
}

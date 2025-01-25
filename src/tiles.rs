use super::{Coord, Error};
use crate::resolutions::Resolution;
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

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

// impl for pub fn-s
impl Tile {
    pub fn new(lat: i8, lon: i16, res: Resolution, data: Vec<i16>) -> Tile {
        Tile {
            latitude: lat,
            longitude: lon,
            resolution: res,
            data,
        }
    }

    /// read an srtm: `.hgt` file, and create a [`Tile`] if possible
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Tile, Error> {
        let file = File::open(&path).map_err(|_| Error::NotFound)?;
        // eprintln!("file: {file:?}");

        let f_len = file.metadata().map_err(|_| Error::Filesize)?.len();
        let res = Resolution::try_from(f_len).map_err(|_| Error::Filesize)?;
        // eprintln!("resolution: {res:?}");

        let (lat, lon) = Tile::get_lat_lon(&path)?;

        let elevation_data = Self::parse_hgt(file, res).map_err(|_| Error::Read)?;

        Ok(Tile::new(lat, lon, res, elevation_data))
    }

    /// the maximum height that this [`Tile`] contains
    pub fn max_height(&self) -> i16 {
        *self.data.iter().max().unwrap_or(&0)
    }
    /// the minimum height that this [`Tile`] contains
    pub fn min_height(&self) -> i16 {
        *self.data.iter().min().unwrap_or(&0)
    }

    /// get the elevation of this `coord` from this [`Tile`]
    ///
    /// # Panics
    /// If this [`Tile`] doesn't contain `coord`'s elevation
    /// *NOTE*: shouldn't happen if [`get_filename()`] was used
    pub fn get(&self, coord: impl Into<Coord>) -> Option<&i16> {
        let coord: Coord = coord.into();
        let offset = self.get_offset(coord);
        let (lat, lon) = coord.trunc();
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
                Coord::new(self.latitude, self.longitude).get_filename()
            );
            None
        } else {
            elev
        }
    }

    /// extract the heights from the `hgt` content
    pub fn parse_hgt(mut reader: impl Read, res: Resolution) -> io::Result<Vec<i16>> {
        let mut buffer = vec![0; res.total_len() * 2];
        reader.read_exact(&mut buffer)?;
        let mut elevations = Vec::with_capacity(res.total_len());
        for chunk in buffer.chunks_exact(2) {
            let value = i16::from_be_bytes([chunk[0], chunk[1]]);
            elevations.push(value);
        }
        Ok(elevations)
    }

    /// extract the latitude and longitude from a filepath
    /// let ne = Path::new("N35E138.hgt");
    /// assert_eq!(Tile::get_lat_lon(ne).unwrap(), (35, 138));
    pub fn get_lat_lon(path: impl AsRef<Path>) -> Result<(i8, i16), Error> {
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
}
// impl for non-pub fn-s
impl Tile {
    /// index `self` as if it was a matrix
    fn get_at_offset(&self, x: usize, y: usize) -> Option<&i16> {
        self.data.get(self.idx(x, y))
    }

    /// convert an `x` `y` coordinate to an idx of `self`
    /// # panics
    /// if `self` doesn't contain the requested coordinate
    fn idx(&self, x: usize, y: usize) -> usize {
        assert!(
            x < self.resolution.extent() && y < self.resolution.extent(),
            "extent: {}, x: {x}, y: {y}",
            self.resolution.extent()
        );
        y * self.resolution.extent() + x
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
}

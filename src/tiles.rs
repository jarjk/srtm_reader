use super::{Coord, new_err};
use crate::resolutions::Resolution;
use std::{fs::File, io, path::Path};

/// the SRTM tile, which contains the actual elevation data
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tile {
    /// South edge latitude (−90° to 90°, 0° is Equator)
    pub latitude: i8,
    /// West edge longitude (-180° to 180°)
    pub longitude: i16,
    /// [`Resolution`]
    pub resolution: Resolution,
    /// each elevation record the tile contains
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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Tile, io::Error> {
        let file = File::open(&path)?;
        // eprintln!("file: {file:?}");

        let f_len = file.metadata()?.len();
        let res = Resolution::try_from(f_len)?;
        // eprintln!("resolution: {res:?}");

        let (lat, lon) = Tile::get_lat_lon(&path)?;

        let elevation_data = Self::parse_hgt(file, res)?;

        Ok(Tile::new(lat, lon, res, elevation_data))
    }

    /// the maximum height that this [`Tile`] contains, falls back to 0
    pub fn max_height(&self) -> i16 {
        *self.data.iter().max().unwrap_or(&0)
    }
    /// the minimum height that this [`Tile`] contains, falls back to 0
    pub fn min_height(&self) -> i16 {
        *self.data.iter().min().unwrap_or(&0)
    }

    /// get the elevation of this `coord` from this [`Tile`]
    ///
    /// Returns `None` if the coord is outside this tile's bounds
    /// or the elevation is an invalid value.
    pub fn get(&self, coord: impl Into<Coord>) -> Option<&i16> {
        let coord: Coord = coord.into();
        let (lat, lon) = coord.floor();
        if !(self.latitude..=self.latitude + 1).contains(&lat)
            || !(self.longitude..=self.longitude + 1).contains(&lon)
        {
            return None;
        }
        let offset = self.get_offset(coord);
        let elev = self.get_at_offset(offset.1, offset.0);
        if elev.is_some_and(|e| *e == -9999 || *e == i16::MIN || *e == i16::MAX) {
            // TODO: WARN the end-user somehow
            // 1. should we make this an Err?
            // 2. should we use `log`?
            None
        } else {
            elev
        }
    }

    /// extract the heights from the `hgt` content
    pub fn parse_hgt(mut reader: impl io::Read, res: Resolution) -> io::Result<Vec<i16>> {
        let mut buffer = vec![0; res.total_len() * Resolution::BYTES_PER_ELEVATION];
        reader.read_exact(&mut buffer)?;
        let mut elevations = Vec::with_capacity(res.total_len());
        for chunk in buffer.chunks_exact(2) {
            let value = i16::from_be_bytes([chunk[0], chunk[1]]);
            elevations.push(value);
        }
        Ok(elevations)
    }

    /// extract the latitude and longitude from a filepath
    /// ```rust
    /// let north_east = std::path::Path::new("N35E138.hgt");
    /// assert_eq!(srtm_reader::Tile::get_lat_lon(north_east).unwrap(), (35, 138));
    /// ```
    pub fn get_lat_lon(path: impl AsRef<Path>) -> Result<(i8, i16), io::Error> {
        let path = path.as_ref();
        let stem = path.file_stem().ok_or(new_err!(of: "no file stem"))?;
        let desc = stem
            .to_str()
            .ok_or(new_err!(InvalidData, "invalid UTF-8"))?;
        if desc.len() != 7 {
            return Err(new_err!(InvalidData, "length isn't 7"));
        }

        let get_char = |n| desc.chars().nth(n).ok_or(new_err!(InvalidData));
        let lat_sign = if get_char(0)? == 'N' { 1 } else { -1 };
        let lat: i8 = desc[1..3].parse().map_err(|e| new_err!(of: e))?;

        let lon_sign = if get_char(3)? == 'E' { 1 } else { -1 };
        let lon: i16 = desc[4..7].parse().map_err(|e| new_err!(of: e))?;
        Ok((lat_sign * lat, lon_sign * lon))
    }
}
// impl for non-pub fn-s
impl Tile {
    /// index `self` as if it was a matrix
    fn get_at_offset(&self, x: usize, y: usize) -> Option<&i16> {
        self.data.get(self.idx(x, y)?)
    }

    /// convert an `x` `y` coordinate to an idx of `self`
    fn idx(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.resolution.extent() || y >= self.resolution.extent() {
            None
        } else {
            Some(y * self.resolution.extent() + x)
        }
    }
    /// get upper-left corner's latitude and longitude
    /// it's needed for [`Tile::get_offset()`]
    /// The upper left corner is the value at (0, 0) in the
    /// data vector.
    fn get_data_origin(&self) -> Coord {
        let lat = f64::from(self.latitude) + 1.;
        let lon = f64::from(self.longitude);
        Coord { lat, lon }
    }
    /// calculate where this `coord` is located in this [`Tile`]
    ///
    /// Matches GDAL's geo-transform with half-pixel offset (pixel-as-point convention):
    /// <https://github.com/OSGeo/gdal/blob/master/frmts/srtmhgt/gdal-srtmhgtdataset.cpp>
    fn get_offset(&self, coord: Coord) -> (usize, usize) {
        let origin = self.get_data_origin();

        // `extent` samples span exactly 1 degree, so there are `extent - 1` intervals between them
        let intervals = (self.resolution.extent() - 1) as f64;
        let half_pixel = 0.5 / intervals;

        let row = ((origin.lat + half_pixel - coord.lat) * intervals) as usize;
        let col = ((coord.lon - origin.lon + half_pixel) * intervals) as usize;
        (row, col)
    }
}

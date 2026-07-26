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
    pub fn get(&self, coord: impl Into<Coord>) -> Option<i16> {
        let coord: Coord = coord.into();
        let (lat, lon) = coord.floor();
        if !(self.latitude..=self.latitude + 1).contains(&lat)
            || !(self.longitude..=self.longitude + 1).contains(&lon)
        {
            return None;
        }
        let (row, col) = self.get_offset(coord)?;
        let elev = self.get_at_offset(col, row);
        if elev.is_some_and(|e| *e == -9999 || *e == i16::MIN || *e == i16::MAX) {
            // TODO: WARN the end-user somehow
            // 1. should we make this an Err?
            // 2. should we use `log`?
            None
        } else {
            elev.copied()
        }
    }

    /// extract the heights from the `hgt` content
    pub fn parse_hgt(mut reader: impl io::Read, res: Resolution) -> io::Result<Vec<i16>> {
        let num_elev_items = res.total_len();
        let byte_len = num_elev_items * Resolution::BYTES_PER_ELEVATION;
        let mut elevations = vec![0i16; num_elev_items];
        // SAFETY: `elevations` is a Vec<i16> so the pointer is aligned for i16,
        // which implies alignment for u8. `byte_len` equals the Vec's byte size.
        // `buf` is consumed (moved) by `read_exact`, so after it returns there
        // is no outstanding &mut to the memory and `elevations` may be used again.
        let buf =
            unsafe { std::slice::from_raw_parts_mut(elevations.as_mut_ptr() as *mut u8, byte_len) };
        reader.read_exact(buf)?;
        for e in elevations.iter_mut() {
            *e = i16::from_be(*e);
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

        if !desc.is_ascii() || desc.len() != 7 {
            return Err(new_err!(InvalidData, "filename must be 7 ASCII characters"));
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
    /// Returns `None` if the coordinate maps outside the tile's data grid.
    ///
    /// Matches GDAL's geo-transform with half-pixel offset (pixel-as-point convention):
    /// <https://github.com/OSGeo/gdal/blob/master/frmts/srtmhgt/gdal-srtmhgtdataset.cpp>
    fn get_offset(&self, coord: Coord) -> Option<(usize, usize)> {
        let origin = self.get_data_origin();

        // `extent` samples span exactly 1 degree, so there are `extent - 1` intervals between them
        let intervals = (self.resolution.extent() - 1) as f64;
        let half_pixel = 0.5 / intervals;

        let row_f = (origin.lat + half_pixel - coord.lat) * intervals;
        let col_f = (coord.lon - origin.lon + half_pixel) * intervals;

        let extent = self.resolution.extent();
        if row_f < 0.0 || col_f < 0.0 || row_f >= extent as f64 || col_f >= extent as f64 {
            return None;
        }

        Some((row_f as usize, col_f as usize))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod private_tests {
    use super::*;
    use std::{io::Write, path::PathBuf, sync::LazyLock};

    const EXT: usize = Resolution::SRTM3.extent();

    static TEST_TILE: LazyLock<Tile> = LazyLock::new(make_tile);

    // TODO: should we promote this to `Tile::write_to()`?
    fn write_tmp(t: Tile) -> PathBuf {
        let mut buffer =
            Vec::with_capacity(t.resolution.total_len() * Resolution::BYTES_PER_ELEVATION);
        let fname = Coord::new(t.latitude, t.longitude).get_filename();

        eprintln!("writing data to buffer...");
        for elev in t.data {
            buffer.write_all(&elev.to_be_bytes()).unwrap();
        }
        eprintln!("done, written {} bytes to buffer", buffer.len());

        let tmp_path = std::env::temp_dir().join(fname);
        eprintln!("writing data to path: {tmp_path:?}");
        std::fs::write(&tmp_path, buffer).unwrap();
        eprintln!("done.");
        tmp_path
    }

    /// Every cell holds `(y * extent + x) as i16` so positions are uniquely identifiable.
    fn make_tile() -> Tile {
        let res = Resolution::SRTM3;
        // WARN: items will quickly start to overflow => wrap around, that doesn't matter though
        let data: Vec<i16> = (0..EXT * EXT).map(|v| v as i16).collect();
        Tile::new(44, 15, res, data)
    }

    #[test]
    fn roundtrip() {
        let tile = TEST_TILE.clone();
        eprintln!("created test tile: {tile:?}");
        let tmp_path = write_tmp(tile.clone());
        let new_tile = Tile::from_file(tmp_path).unwrap();
        assert_eq!(tile.max_height(), new_tile.max_height());
        assert_eq!(tile.min_height(), new_tile.min_height());
        assert_eq!(tile.get_data_origin(), new_tile.get_data_origin());
        assert_eq!(tile, new_tile);
    }

    #[test]
    fn corners() {
        let tile = &TEST_TILE;

        let coord = Coord::new(44.4480, 15.0733);
        let fname = coord.get_filename();
        assert_eq!(fname, "N44E015.hgt");
        assert_eq!(tile.latitude, 44);
        assert_eq!(tile.longitude, 15);
        assert_eq!(tile.resolution, Resolution::SRTM3);
        assert_eq!(tile.data.len(), Resolution::SRTM3.total_len());

        let elev = tile.get(coord);
        assert_eq!(elev, Some(8718)); // Validated with QGis/GDAL (half-pixel offset)

        // top left, origin
        let c = Coord::new(45.0, 15.0);
        assert_eq!(c, tile.get_data_origin());
        assert_eq!(tile.idx(0, 0), Some(0));
        assert_eq!(tile.get_offset(c), Some((0, 0)));
        assert_eq!(tile.get_at_offset(0, 0), Some(&0));
        assert_eq!(tile.get(c), Some(0));

        // top right
        let e = EXT - 1;
        let c = Coord::new(45.0, 16.0);
        assert_eq!(tile.idx(e, 0), Some(e));
        assert_eq!(tile.get_offset(c), Some((0, e)));
        assert_eq!(tile.get(c), Some(e as i16));

        // bottom left
        let c = Coord::new(44.0, 15.0);
        assert_eq!(Coord::new(tile.latitude, tile.longitude), c);
        assert_eq!(tile.idx(0, e), Some(e * EXT));
        assert_eq!(tile.get_offset(c), Some((e, 0)));
        assert_eq!(tile.get(c), Some((e * EXT) as i16));

        // bottom right
        let c = Coord::new(44.0, 16.0);
        assert_eq!(tile.idx(e, e), Some(EXT * EXT - 1));
        assert_eq!(tile.get_offset(c), Some((e, e)));
        assert_eq!(tile.get(c), Some((EXT * EXT - 1) as i16));
    }

    #[test]
    fn arbitrary_offset() {
        let tile = &TEST_TILE;
        // idx(3,2) = 2*1201+3 = 2405
        assert_eq!(tile.idx(3, 2), Some(2405));
        assert_eq!(tile.get_at_offset(3, 2), Some(&2405));
    }

    #[test]
    fn out_of_bounds() {
        let tile = &TEST_TILE;
        assert_eq!(tile.get(Coord::new(43.9, 15.5)), None); // lat low
        assert_eq!(tile.get(Coord::new(46.0, 15.5)), None); // lat high
        assert_eq!(tile.get(Coord::new(44.5, 14.9)), None); // lon low
        assert_eq!(tile.get(Coord::new(44.5, 16.1)), None); // lon high
    }

    #[test]
    fn oob_idx() {
        let tile = &TEST_TILE;
        assert_eq!(tile.idx(EXT, 0), None);
        assert_eq!(tile.idx(0, EXT), None);
        assert_eq!(tile.idx(EXT, EXT), None);
        assert_eq!(tile.get_at_offset(EXT, 0), None);
        assert_eq!(tile.get_at_offset(0, EXT), None);
    }

    fn tile_with_hole(val: i16) -> Tile {
        let res = Resolution::SRTM3;
        let ext = res.extent();
        let mut data: Vec<i16> = (0..ext * ext).map(|v| v as i16).collect();
        data[0] = val;
        Tile::new(44, 15, res, data)
    }

    #[test]
    fn tile_has_a_hole() {
        let c = Coord::new(45.0, 15.0);
        assert_eq!(tile_with_hole(-9999).get(c), None);
        assert_eq!(tile_with_hole(i16::MIN).get(c), None);
        assert_eq!(tile_with_hole(i16::MAX).get(c), None);
    }
}

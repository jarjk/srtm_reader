/// coordinates
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Coord {
    /// latitude: north-south
    pub lat: f64,
    /// longitude: east-west
    pub lon: f64,
}

impl Coord {
    pub fn new(lat: impl Into<f64>, lon: impl Into<f64>) -> Self {
        let lat = lat.into();
        let lon = lon.into();
        assert!((-90. ..=90.).contains(&lat));
        assert!((-180. ..=180.).contains(&lon));
        Self { lat, lon }
    }
    pub fn with_lat(self, lat: impl Into<f64>) -> Self {
        Self::new(lat, self.lon)
    }
    pub fn with_lon(self, lon: impl Into<f64>) -> Self {
        Self::new(self.lat, lon)
    }
    pub fn add_to_lat(self, lat: impl Into<f64>) -> Self {
        self.with_lat(self.lat + lat.into())
    }
    pub fn add_to_lon(self, lon: impl Into<f64>) -> Self {
        self.with_lon(self.lon + lon.into())
    }

    /// truncate both latitude and longitude
    pub fn trunc(&self) -> (i8, i16) {
        (self.lat.trunc() as i8, self.lon.trunc() as i16)
    }
    /// get the name of the file, which shall include this `coord`s elevation
    ///
    /// # Usage
    ///
    /// ```rust
    /// // the `coord`inate, whe want the elevation for
    /// let coord = srtm_reader::Coord::new(87.235, 10.4234423);
    /// let filename = coord.get_filename();
    /// assert_eq!(filename, "N87E010.hgt");
    /// ```
    pub fn get_filename(self) -> String {
        let lat_ch = if self.lat >= 0. { 'N' } else { 'S' };
        let lon_ch = if self.lon >= 0. { 'E' } else { 'W' };
        let lat = (self.lat.trunc() as i32).abs();
        let lon = (self.lon.trunc() as i32).abs();
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
}

impl<F1: Into<f64>, F2: Into<f64>> From<(F1, F2)> for Coord {
    fn from(value: (F1, F2)) -> Self {
        let (lat, lon) = (value.0.into(), value.1.into());
        Coord { lat, lon }
    }
}

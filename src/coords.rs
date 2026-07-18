/// earth coordinates
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Coord {
    /// latitude: north-south
    /// between -90 and 90, 0 is equator
    pub lat: f64,
    /// longitude: east-west
    /// between -180 and 180
    pub lon: f64,
}

impl Coord {
    /// return `None` on invalid earth coordinates
    pub fn opt_new(lat: impl Into<f64>, lon: impl Into<f64>) -> Option<Self> {
        let (lat, lon) = (lat.into(), lon.into());
        if (-90. ..=90.).contains(&lat) && (-180. ..=180.).contains(&lon) {
            Some(Self { lat, lon })
        } else {
            None
        }
    }
    /// # panics
    /// invalid earth coordinate
    pub fn new(lat: impl Into<f64>, lon: impl Into<f64>) -> Self {
        Self::opt_new(lat, lon).expect("latitude must be between -90 and 90 degrees, longitude must be between -180 and 180 degrees")
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

    /// floor of both latitude and longitude
    pub fn floor(&self) -> (i8, i16) {
        (self.lat.floor() as i8, self.lon.floor() as i16)
    }
    /// get the name of the file, which shall include this `coord`s elevation
    ///
    /// # Usage
    ///
    /// ```rust
    /// // the `coord`inate, we want the elevation for
    /// let coord = srtm_reader::Coord::new(87.235, 10.42344);
    /// let filename = coord.get_filename();
    /// assert_eq!(filename, "N87E010.hgt");
    /// ```
    ///
    /// # Note
    ///
    /// Due to the nature of hgt files we return "N45E016.hgt" for `Coord { lat: 45.0, lon: 16.0 }`
    /// *although "N44E015.hgt" also contains it.* This is a so-called padding; weird but intended behaviour.
    pub fn get_filename(self) -> String {
        let lat_ch = if self.lat >= 0. { 'N' } else { 'S' };
        let lon_ch = if self.lon >= 0. { 'E' } else { 'W' };
        let (lat, lon) = self.floor();
        let (lat, lon) = (lat.abs(), lon.abs());
        format!("{lat_ch}{lat:02}{lon_ch}{lon:03}.hgt")
    }
}

impl<F1: Into<f64>, F2: Into<f64>> From<(F1, F2)> for Coord {
    fn from(value: (F1, F2)) -> Self {
        let (lat, lon) = (value.0.into(), value.1.into());
        Coord { lat, lon }
    }
}

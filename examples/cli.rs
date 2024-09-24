use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Coord {
    pub lat: f64,
    pub lon: f64,
}
impl From<Coord> for (f64, f64) {
    fn from(coord: Coord) -> Self {
        (coord.lat, coord.lon)
    }
}
impl From<Coord> for srtm_reader::Coord {
    fn from(val: Coord) -> Self {
        srtm_reader::Coord::new(val.lat, val.lon)
    }
}
impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.lat, self.lon)?;
        Ok(())
    }
}
impl Coord {
    fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }

    /// can parse format: "<LAT>,<LON>", eg: "14.43534214,32.328791"
    fn parse(str: &str) -> Self {
        let mut coord = str.split(',');
        let lat: f64 = coord
            .next()
            .unwrap_or_else(|| quit_help("coord parsing"))
            .parse()
            .unwrap_or_else(|_| quit_help("coord parsing"));
        let lon: f64 = coord
            .next()
            .unwrap_or_else(|| quit_help("coord parsing"))
            .parse()
            .unwrap_or_else(|_| quit_help("coord parsing"));

        Self::new(lat, lon)
    }
}

/// quit, showing help
fn quit_help(cx: &str) -> ! {
    eprintln!(
        "error: {}

Get elevation data for a coordinate from SRTM data (.hgt files).

USAGE: elev_data [OPTIONS] <ARGS>

ARGS:  <LATITUDE_FLOAT,LONGITUDE_FLOAT> 

OPTIONS:
       --elev_data_dir: <ELEVATION_DATA_DIR> or $elev_data_dir set",
        if cx.is_empty() { "unknown" } else { cx }
    );
    std::process::exit(1);
}

fn get_arg(args: &[String], arg: &str) -> Option<String> {
    args.iter()
        .enumerate()
        .find(|(_, item)| item == &arg)
        .map(|(i, _)| args[i + 1].clone())
}

fn main() -> io::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();

    let coord = Coord::parse(&args[1]);
    // eprintln!("coord: ({};{})", coord.0, coord.1);
    let elev_data_dir = if let Some(arg_data_dir) = get_arg(&args, "--elev_data_dir") {
        arg_data_dir
    } else if let Some(env_data_dir) = option_env!("ELEV_DATA_DIR") {
        env_data_dir.into()
    } else {
        quit_help("no elev_data_dir got");
    };
    let elev_data_dir = PathBuf::from(elev_data_dir);
    // eprintln!("is tiff: {is_tiff}");
    // eprintln!("elev_data_dir: {}", elev_data_dir.display());
    let file_name = srtm_reader::get_filename(coord);
    // eprintln!("file_name: {file_path}");
    let file_path = elev_data_dir.join(file_name);
    // eprintln!("path to .hgt file: {}", file_path.display());

    let data = srtm_reader::Tile::from_file(file_path).unwrap();
    // eprintln!("resolution: {:?}", data.resolution);
    let elevation = data.get(coord);

    // eprintln!("offset: row: {row}, col: {col}");
    // let elevation = coord.get_elevation(&data);
    // coord.get_elevation(&data)

    println!("Elevation at {coord} is {elevation} meters");

    Ok(())
}

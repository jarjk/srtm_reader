use super::*;
use std::path::Path;

#[test]
fn parse_latidute_and_longitude() {
    let ne = Path::new("/tmp/N35E138.hgt");
    assert_eq!(Tile::get_lat_lon(ne).unwrap(), (35, 138));

    let nw = Path::new("/home/juhuu/DTM of me self/N35W138.hgt");
    assert_eq!(Tile::get_lat_lon(nw).unwrap(), (35, -138));

    let se = Path::new("S35E138.hgt");
    assert_eq!(Tile::get_lat_lon(se).unwrap(), (-35, 138));

    let sw = Path::new("S35W138.hgt");
    assert_eq!(Tile::get_lat_lon(sw).unwrap(), (-35, -138));
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
fn wrong_coords() {
    let coord_new_panics = |lat: f64, lon: f64| assert!(Coord::opt_new(lat, lon).is_none());
    coord_new_panics(-190., 42.4);
    coord_new_panics(180., -42.4);
    coord_new_panics(-90., 181.);
    coord_new_panics(90., -180.00001);
}
#[test]
fn correct_coords() {
    let coord_new = |lat: f64, lon: f64| assert!(Coord::opt_new(lat, lon).is_some());
    coord_new(-90., 180.);
    coord_new(90., -180.);

    let c = Coord::new(90, -180).with_lon(-85.7);
    assert_eq!(Coord::new(90, -85.7), c);

    let c = Coord::new(90, -180).with_lat(0.3);
    assert_eq!(Coord::new(0.3, -180), c);

    let c = Coord::new(90, -180).with_lat(0.3).with_lon(83.3);
    assert_eq!(Coord::new(0.3, 83.3), c);

    let c: Coord = (90, -180).into();
    let c = c.with_lat(0.3).with_lon(83.3);
    assert_eq!(Coord::new(0.3, 83.3), c);

    let c: Coord = (90, -180).into();
    let c = c.with_lat(0.3).with_lon(83.3);
    assert_eq!(Coord::new(0.3, 83.3), c);

    let c: Coord = (-90, 180).into();
    let c = c.add_to_lat(0.3252).add_to_lon(-3.2);
    assert_eq!(Coord::new(-89.6748, 176.8), c);
}

#[test]
fn file_names() {
    const TEST_VALUES: &[((f64, f64), &str)] = &[
        ((45., 1.4), "N45E001.hgt"),  // NE
        ((-2.3, 87.), "S02E087.hgt"), // SE
        ((35., -7.), "N35W007.hgt"),  // NW
        ((-5., -7.), "S05W007.hgt"),  // SW
    ];

    for (coord, filename) in TEST_VALUES {
        assert_eq!(&Coord::from(*coord).get_filename(), filename);
    }
}
#[test]
fn read_north_east() {
    let coord = Coord::new(44.4480403, 15.0733053);
    let fname = coord.get_filename();
    let tile = Tile::from_file(fname).unwrap();
    assert_eq!(tile.latitude, 44);
    assert_eq!(tile.longitude, 15);
    assert_eq!(tile.resolution, Resolution::SRTM1);
    assert_eq!(tile.data.len(), Resolution::SRTM1.total_len());

    let elev = tile.get(coord);
    assert_eq!(elev, Some(&258)); // Validated manually with QGis
}

/// Test at exactly (44,15)
#[test]
fn read_integer_coords_north_east_origin() {
    let coord = Coord::new(44.0, 15.0);
    let fname = coord.get_filename();
    let tile = Tile::from_file(fname).unwrap();
    assert_eq!(tile.latitude, 44);
    assert_eq!(tile.longitude, 15);
    assert_eq!(tile.resolution, Resolution::SRTM1);
    assert_eq!(tile.data.len(), Resolution::SRTM1.total_len());

    // Check Bottom Left corner
    let elev = tile.get(coord);
    assert_eq!(elev, Some(&0)); // Inside water. Validated manually with QGis

    // Check Top left corner
    let coord = Coord::new(45.0, 15.0);
    let elev = tile.get(coord);
    assert_eq!(elev, Some(&858));

    // Check Top right corner
    let coord = Coord::new(45.0, 16.0);
    let elev = tile.get(coord);
    assert_eq!(elev, Some(&498));

    // Check Bottom right corner
    let coord = Coord::new(44.0, 16.0);
    let elev = tile.get(coord);
    assert_eq!(elev, Some(&239));
}

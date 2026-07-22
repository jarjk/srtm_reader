use super::*;

#[test]
fn parse_latidute_and_longitude() {
    const TEST_VALUES: [(&str, (i8, i16)); 4] = [
        ("/tmp/N35E138.hgt", (35, 138)),                        // ne
        ("/home/juhuu/DTM of me self/N35W138.hgt", (35, -138)), // nw
        ("S35E138.hgt", (-35, 138)),                            // se
        ("S35W138.hgt", (-35, -138)),                           // sw
    ];

    for (path, coord) in TEST_VALUES {
        assert_eq!(Tile::get_lat_lon(path).unwrap(), coord);
    }
}
#[test]
fn total_file_sizes() {
    let bpe = Resolution::BYTES_PER_ELEVATION;
    assert_eq!(103_708_802 / bpe, Resolution::SRTM05.total_len());
    assert_eq!(25_934_402 / bpe, Resolution::SRTM1.total_len());
    assert_eq!(2_884_802 / bpe, Resolution::SRTM3.total_len());
}
#[test]
fn extents() {
    assert_eq!(7201, Resolution::SRTM05.extent());
    assert_eq!(3601, Resolution::SRTM1.extent());
    assert_eq!(1201, Resolution::SRTM3.extent());
}

#[test]
fn wrong_coords() {
    let coord_new_none = |lat: f64, lon: f64| assert!(Coord::opt_new(lat, lon).is_none());
    coord_new_none(-190., 42.4);
    coord_new_none(180., -42.4);
    coord_new_none(-90., 181.);
    coord_new_none(90., -180.00001);
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
        ((45., 1.4), "N45E001.hgt"),   // NE
        ((-2.3, 87.), "S03E087.hgt"),  // SE
        ((35., -7.), "N35W007.hgt"),   // NW
        ((-5., -7.), "S05W007.hgt"),   // SW
        ((-2.3, -7.5), "S03W008.hgt"), // SW with non-integer both
    ];

    for (coord, filename) in TEST_VALUES {
        assert_eq!(&Coord::from(*coord).get_filename(), filename);
    }
}

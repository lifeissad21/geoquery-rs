mod common;

use common::{assert_shape, geo_object_type};
use geoquery::{parseGEO, GEOObject, Meta, Table};
use std::path::PathBuf;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path)
}

#[test]
fn generic_gds_parsing_works_as_expected() {
    let gds_path = fixture("extdata/GDS507.soft.gz");
    let parsed = parseGEO(&gds_path, None, None, false, true)
        .unwrap_or_else(|err| panic!("GDS fixture {} should parse: {err}", gds_path.display()));
    let GEOObject::GDS(gds) = parsed else {
        panic!(
            "GDS fixture {} parsed to {}, expected GDS",
            gds_path.display(),
            geo_object_type(&parsed)
        );
    };

    let table = Table::Table(&gds);
    assert_shape("GDS507 data table", table.nrow(), table.ncol(), 22_645, 19);
    assert_eq!(
        Meta::Meta(&gds).len(),
        23,
        "GDS507 metadata count changed: expected 23 entries, got {}. Metadata keys: {:?}",
        Meta::Meta(&gds).len(),
        Meta::Meta(&gds).keys().collect::<Vec<_>>()
    );
}

#[test]
fn empty_files_produce_an_error() {
    let broken_path = fixture("extdata/GPLbroken.soft.gz");
    let result = parseGEO(&broken_path, None, None, false, true);
    assert!(
        result.is_err(),
        "Broken fixture {} unexpectedly parsed successfully as {}",
        broken_path.display(),
        result
            .as_ref()
            .map(geo_object_type)
            .unwrap_or("unreachable error case")
    );
}

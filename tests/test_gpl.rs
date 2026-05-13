mod common;

use common::{assert_shape, geo_object_type};
use geoquery::{getGEO, parseGEO, GEOObject, GetGeoOptions, Meta, Table};
use std::io::Write;

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn generic_gpl_parsing_works_as_expected() {
    let accession = "GPL96";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GPL: {err}"));
    let GEOObject::GPL(gpl) = result else {
        panic!(
            "{accession} parsed to {}, expected GPL",
            geo_object_type(&result)
        );
    };
    let table = Table::Table(&gpl);
    assert_shape(
        "GPL96 platform table",
        table.nrow(),
        table.ncol(),
        22_283,
        16,
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn gpl_with_no_data_table_works() {
    let accession = "GPL5082";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GPL: {err}"));
    let GEOObject::GPL(gpl) = result else {
        panic!(
            "{accession} parsed to {}, expected GPL",
            geo_object_type(&result)
        );
    };
    assert_eq!(
        Table::Table(&gpl).nrow(),
        0,
        "{accession} should have no data table rows, got {} rows x {} columns",
        Table::Table(&gpl).nrow(),
        Table::Table(&gpl).ncol()
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn quoted_gpl_works() {
    let accession = "GPL4133";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GPL: {err}"));
    let GEOObject::GPL(gpl) = result else {
        panic!(
            "{accession} parsed to {}, expected GPL",
            geo_object_type(&result)
        );
    };
    assert_eq!(
        Table::Table(&gpl).nrow(),
        45_220,
        "{accession} expected 45220 platform rows, got {} rows x {} columns",
        Table::Table(&gpl).nrow(),
        Table::Table(&gpl).ncol()
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn short_gpl_works() {
    let accession = "GPL15505";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GPL: {err}"));
    let GEOObject::GPL(gpl) = result else {
        panic!(
            "{accession} parsed to {}, expected GPL",
            geo_object_type(&result)
        );
    };
    assert_eq!(
        Table::Table(&gpl).nrow(),
        52,
        "{accession} expected 52 platform rows, got {} rows x {} columns",
        Table::Table(&gpl).nrow(),
        Table::Table(&gpl).ncol()
    );
}

#[test]
fn local_gpl_fixture_parses_without_network() {
    let dir = tempfile::tempdir().expect("tempdir should be available");
    let path = dir.path().join("GPLTEST.soft.gz");
    let soft = b"^PLATFORM = GPLTEST\n!Platform_geo_accession = GPLTEST\n!Platform_title = Local GPL fixture\n#ID = Probe identifier\n#VALUE = Numeric value\n!platform_table_begin\nID\tVALUE\nprobe1\t1.25\nprobe2\t2.50\n!platform_table_end\n";
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(soft)
        .expect("local GPL fixture should gzip");
    std::fs::write(
        &path,
        encoder
            .finish()
            .expect("local GPL fixture gzip should finish"),
    )
    .expect("local GPL fixture should be written");

    let parsed = parseGEO(&path, None, None, false, true)
        .unwrap_or_else(|err| panic!("local GPL fixture should parse: {err}"));
    let GEOObject::GPL(gpl) = parsed else {
        panic!(
            "local GPL fixture parsed to {}, expected GPL",
            geo_object_type(&parsed)
        );
    };
    assert_eq!(
        Meta::Meta(&gpl)
            .get("geo_accession")
            .and_then(|values| values.first())
            .map(String::as_str),
        Some("GPLTEST")
    );
    assert_shape(
        "local GPL fixture",
        Table::Table(&gpl).nrow(),
        Table::Table(&gpl).ncol(),
        2,
        2,
    );
    assert_eq!(
        Table::Table(&gpl)
            .column_metadata
            .get("ID")
            .map(String::as_str),
        Some("Probe identifier"),
        "local GPL fixture should populate column metadata"
    );
}

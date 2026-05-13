mod common;

use common::{assert_shape, geo_object_type};
use flate2::write::GzEncoder;
use flate2::Compression;
use geoquery::{getGEO, parseGEO, Accession, GEOObject, GetGeoOptions, Meta, Table};
use std::io::Write;

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn basic_gsm_works() {
    let accession = "GSM11805";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GSM: {err}"));
    let GEOObject::GSM(gsm) = result else {
        panic!(
            "{accession} parsed to {}, expected GSM",
            geo_object_type(&result)
        );
    };
    assert_eq!(
        Accession::Accession(&gsm),
        Some(accession),
        "{accession} accession accessor returned {:?}; metadata keys: {:?}",
        Accession::Accession(&gsm),
        Meta::Meta(&gsm).keys().collect::<Vec<_>>()
    );
    let table = Table::Table(&gsm);
    assert_shape(
        "GSM11805 sample table",
        table.nrow(),
        table.ncol(),
        22_283,
        3,
    );
    assert_eq!(
        Meta::Meta(&gsm).len(),
        28,
        "{accession} metadata count changed: expected 28 entries, got {}. Metadata keys: {:?}",
        Meta::Meta(&gsm).len(),
        Meta::Meta(&gsm).keys().collect::<Vec<_>>()
    );
}

#[test]
fn gzipped_gsm_content_with_soft_extension_parses() {
    let dir = tempfile::tempdir().expect("tempdir should be created for gzipped GSM fixture");
    let path = dir.path().join("GSMTEST.soft");
    let soft = b"^SAMPLE = GSMTEST\n!Sample_geo_accession = GSMTEST\n#ID_REF = Probe identifier\n#VALUE = Signal\n!sample_table_begin\nID_REF\tVALUE\nprobe1\t7.5\n!sample_table_end\n";

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(soft)
        .expect("fixture bytes should gzip successfully");
    let compressed = encoder.finish().expect("gzip fixture should finish");
    std::fs::write(&path, compressed).expect("gzipped fixture should be written");

    let parsed = parseGEO(&path, None, None, false, true).unwrap_or_else(|err| {
        panic!(
            "gzipped GSM content in {} should parse: {err}",
            path.display()
        )
    });
    let GEOObject::GSM(gsm) = parsed else {
        panic!(
            "gzipped GSM fixture {} parsed to {}, expected GSM",
            path.display(),
            geo_object_type(&parsed)
        );
    };
    assert_eq!(Accession::Accession(&gsm), Some("GSMTEST"));
    assert_shape(
        "gzipped .soft GSM fixture",
        Table::Table(&gsm).nrow(),
        Table::Table(&gsm).ncol(),
        1,
        2,
    );
}

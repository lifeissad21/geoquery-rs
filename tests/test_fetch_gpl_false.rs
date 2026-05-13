mod common;

use common::{assert_shape, geo_object_type};
use geoquery::{getGEO, GDS2eSet, GEOObject, GetGeoOptions};

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn gds_without_gpl_works() {
    let accession = "GDS10";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        getGPL: false,
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse without GPL: {err}"));
    let GEOObject::GDS(gds) = result else {
        panic!(
            "{accession} parsed to {}, expected GDS",
            geo_object_type(&result)
        );
    };

    let eset = GDS2eSet(&gds, false);
    assert_shape(
        "GDS10 featureData without GPL",
        eset.featureData.nrow(),
        eset.featureData.ncol(),
        39_114,
        0,
    );
}

#[test]
#[ignore = "requires full GSE matrix ExpressionSet parity and live NCBI GEO network access"]
fn gse_without_gpl_works() {
    let accession = "GSE2553";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        getGPL: false,
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse without GPL: {err}"));
    let GEOObject::GSEMatrix(esets) = result else {
        panic!(
            "{accession} parsed to {}, expected GSEMatrix/ExpressionSet",
            geo_object_type(&result)
        );
    };
    let eset = esets
        .first()
        .unwrap_or_else(|| panic!("{accession} returned an empty GSEMatrix list"));
    assert_shape(
        "GSE2553 featureData without GPL",
        eset.featureData.nrow(),
        eset.featureData.ncol(),
        12_600,
        0,
    );
}

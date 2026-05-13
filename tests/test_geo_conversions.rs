mod common;

use common::{assert_shape, geo_object_type};
use geoquery::{parseGEO, GDS2eSet, GEOObject, Meta, GDS2MA};
use std::path::PathBuf;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path)
}

fn gds507() -> geoquery::GDS {
    let path = fixture("extdata/GDS507.soft.gz");
    let parsed = parseGEO(&path, None, None, false, true)
        .unwrap_or_else(|err| panic!("GDS fixture {} should parse: {err}", path.display()));
    let GEOObject::GDS(gds) = parsed else {
        panic!(
            "GDS fixture {} parsed to {}, expected GDS",
            path.display(),
            geo_object_type(&parsed)
        );
    };
    gds
}

#[test]
fn gds2eset_works() {
    let gds = gds507();
    let eset = GDS2eSet(&gds, false);

    assert_shape(
        "GDS2eSet expression matrix",
        eset.nrow(),
        eset.ncol(),
        22_645,
        17,
    );
    assert_eq!(
        eset.experimentData
            .get("pubmed_id")
            .and_then(|values| values.first())
            .map(String::as_str),
        Some("14641932"),
        "GDS2eSet should preserve pubmed_id in experimentData. Keys: {:?}",
        eset.experimentData.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        eset.featureData.nrow(),
        22_645,
        "GDS2eSet featureData should preserve one row per feature even without GPL annotation"
    );
    assert_eq!(
        eset.featureData.ncol(),
        0,
        "GDS2eSet featureData should have zero annotation columns when GPL annotation is not attached"
    );
    assert_eq!(
        eset.annotation.as_deref(),
        Meta::Meta(&gds)
            .get("platform")
            .and_then(|values| values.first())
            .map(String::as_str),
        "GDS2eSet annotation should match the GDS platform metadata"
    );
}

#[test]
fn gds2eset_populates_annotated_data_frame_metadata() {
    let gds = gds507();
    let eset = GDS2eSet(&gds, false);

    assert_eq!(
        eset.pheno_data.nrow(),
        eset.phenoData.nrow(),
        "Annotated pheno_data should mirror compatibility phenoData row count"
    );
    assert!(
        eset.pheno_data.var_metadata.nrow() > 0,
        "Annotated pheno_data should carry variable metadata derived from GDS column descriptions"
    );
    assert!(
        eset.pheno_data
            .var_metadata
            .get(0, "labelDescription")
            .is_some(),
        "pheno_data variable metadata should include labelDescription entries"
    );
    assert_eq!(
        eset.feature_data.nrow(),
        22_645,
        "Annotated feature_data should preserve one row per feature"
    );
}

#[test]
fn gds2ma_works() {
    let gds = gds507();
    let malist = GDS2MA(&gds, false);

    assert_shape("GDS2MA M matrix", malist.nrow(), malist.ncol(), 22_645, 17);
    assert_eq!(
        malist.row_names.len(),
        22_645,
        "GDS2MA should preserve one row name per feature"
    );
    assert_eq!(
        malist.column_names.len(),
        17,
        "GDS2MA should preserve one column name per GSM sample"
    );
    assert_eq!(
        malist
            .notes
            .get("pubmed_id")
            .and_then(|values| values.first())
            .map(String::as_str),
        Some("14641932"),
        "GDS2MA notes should preserve GDS metadata"
    );
}

#[test]
fn gds2ma_numeric_matrix_has_typed_accessors() {
    let gds = gds507();
    let malist = GDS2MA(&gds, false);

    assert!(
        malist.M.get(0, 0).is_some(),
        "GDS2MA numeric matrix should expose typed f64 values"
    );
    assert_eq!(
        malist.M.row_name(0),
        malist.row_names.first().map(String::as_str),
        "NumericMatrix row_name helper should match MAList row_names"
    );
    assert_eq!(
        malist.M.column_name(0),
        malist.column_names.first().map(String::as_str),
        "NumericMatrix column_name helper should match MAList column_names"
    );
}

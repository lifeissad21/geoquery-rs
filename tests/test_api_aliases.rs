use geoquery::{
    browse_geo_accession, browse_rnaseq_search, gds_to_expression_set, gds_to_ma, get_geo_file,
    get_geo_supp_file_url, parse_geo, url_for_accession, GEOObject, GeoError,
};
use std::path::PathBuf;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path)
}

#[test]
fn idiomatic_aliases_match_r_compatible_names() {
    let accession = "GSE262484";
    assert_eq!(
        url_for_accession(accession),
        geoquery::urlForAccession(accession),
        "snake_case URL alias should match R-compatible urlForAccession"
    );
    assert_eq!(
        browse_geo_accession(accession),
        geoquery::browseGEOAccession(accession),
        "snake_case browse alias should match R-compatible browseGEOAccession"
    );
    assert_eq!(
        browse_rnaseq_search(),
        geoquery::browseWebsiteRNASeqSearch(),
        "snake_case RNA-seq browse alias should match R-compatible browseWebsiteRNASeqSearch"
    );
    assert_eq!(
        get_geo_supp_file_url("GSE161228").unwrap(),
        geoquery::getGEOSuppFileURL("GSE161228").unwrap(),
        "snake_case supplemental URL alias should match R-compatible getGEOSuppFileURL"
    );
}

#[test]
fn core_snake_case_aliases_match_r_compatible_functions() {
    let gds_path = fixture("extdata/GDS507.soft.gz");
    let parsed = parse_geo(&gds_path, None, None, false, true)
        .unwrap_or_else(|err| panic!("parse_geo alias should parse fixture: {err}"));
    let GEOObject::GDS(gds) = parsed else {
        panic!("parse_geo alias should return GDS for GDS507 fixture");
    };
    let eset = gds_to_expression_set(&gds, false);
    let malist = gds_to_ma(&gds, false);
    assert_eq!(eset.nrow(), 22_645);
    assert_eq!(malist.ncol(), 17);
    assert_eq!(
        gds.meta()
            .get("pubmed_id")
            .and_then(|values| values.first()),
        geoquery::Meta::Meta(&gds)
            .get("pubmed_id")
            .and_then(|values| values.first()),
        "snake_case GDS::meta accessor should match R-compatible Meta trait"
    );
    assert_eq!(
        gds.table().nrow(),
        geoquery::Table::Table(&gds).nrow(),
        "snake_case GDS::table accessor should match R-compatible Table trait"
    );
    assert_eq!(
        gds.columns().nrow(),
        geoquery::Columns::Columns(&gds).nrow(),
        "snake_case GDS::columns accessor should match R-compatible Columns trait"
    );
}

#[test]
fn get_geo_file_alias_preserves_structured_errors() {
    let tempdir = tempfile::tempdir().expect("tempdir should be available");
    let err = get_geo_file("BAD123", tempdir.path(), false, "full")
        .expect_err("unsupported accession should return an error");
    assert!(
        matches!(err, GeoError::UnsupportedAccession(_)),
        "Expected UnsupportedAccession, got {err:?}"
    );
}

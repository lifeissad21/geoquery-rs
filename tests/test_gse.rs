mod common;

use common::geo_object_type;
use geoquery::{getGEO, parseGEO, GEOObject, GPLList, GSMList, GetGeoOptions, Table};

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn generic_soft_format_gse_handled_correctly() {
    let accession = "GSE1563";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        GSEMatrix: false,
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as SOFT GSE: {err}"));
    let GEOObject::GSE(gse) = result else {
        panic!(
            "{accession} parsed to {}, expected GSE",
            geo_object_type(&result)
        );
    };
    assert_eq!(
        GSMList::GSMList(&gse).len(),
        62,
        "{accession} expected 62 GSM records, got {}. First GSM keys: {:?}",
        GSMList::GSMList(&gse).len(),
        GSMList::GSMList(&gse).keys().take(5).collect::<Vec<_>>()
    );
    assert_eq!(
        GPLList::GPLList(&gse).len(),
        1,
        "{accession} expected 1 GPL record, got {}. GPL keys: {:?}",
        GPLList::GPLList(&gse).len(),
        GPLList::GPLList(&gse).keys().collect::<Vec<_>>()
    );
    assert_eq!(
        GPLList::GPLList(&gse)
            .values()
            .next()
            .map(|gpl| Table::Table(gpl).nrow()),
        Some(12_625),
        "{accession} first GPL table should have 12625 rows, got {:?}",
        GPLList::GPLList(&gse)
            .values()
            .next()
            .map(|gpl| (Table::Table(gpl).nrow(), Table::Table(gpl).ncol()))
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn empty_gse_is_handled_correctly() {
    let accession = "GSE11413";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GSEMatrix: {err}"));
    let GEOObject::GSEMatrix(esets) = result else {
        panic!(
            "{accession} parsed to {}, expected GSEMatrix",
            geo_object_type(&result)
        );
    };
    let eset = esets
        .first()
        .unwrap_or_else(|| panic!("{accession} returned no ExpressionSet entries"));
    assert_eq!(
        eset.phenoData.nrow(),
        12,
        "{accession} expected 12 phenotype rows, got {}",
        eset.phenoData.nrow()
    );
    assert_eq!(
        eset.featureData.ncol(),
        0,
        "{accession} expected empty feature annotation columns, got {}",
        eset.featureData.ncol()
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn case_mismatched_ids_in_gses_handled_correctly() {
    let accession = "GSE35683";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GSEMatrix: {err}"));
    let GEOObject::GSEMatrix(esets) = result else {
        panic!(
            "{accession} parsed to {}, expected GSEMatrix",
            geo_object_type(&result)
        );
    };
    let eset = esets
        .first()
        .unwrap_or_else(|| panic!("{accession} returned no ExpressionSet entries"));
    assert_eq!(
        eset.nrow(),
        54_675,
        "{accession} expected 54675 expression rows, got {} rows x {} columns",
        eset.nrow(),
        eset.ncol()
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn single_sample_gse_handled_correctly() {
    let accession = "GSE11595";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GSEMatrix: {err}"));
    let GEOObject::GSEMatrix(esets) = result else {
        panic!(
            "{accession} parsed to {}, expected GSEMatrix",
            geo_object_type(&result)
        );
    };
    let eset = esets
        .first()
        .unwrap_or_else(|| panic!("{accession} returned no ExpressionSet entries"));
    assert_eq!(
        eset.ncol(),
        1,
        "{accession} expected 1 sample column, got {}",
        eset.ncol()
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn short_gse_handled_correctly() {
    let accession = "GSE34145";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GSEMatrix: {err}"));
    let GEOObject::GSEMatrix(esets) = result else {
        panic!(
            "{accession} parsed to {}, expected GSEMatrix",
            geo_object_type(&result)
        );
    };
    let eset = esets
        .first()
        .unwrap_or_else(|| panic!("{accession} returned no ExpressionSet entries"));
    assert_eq!(
        eset.nrow(),
        15,
        "{accession} expected 15 expression rows, got {}",
        eset.nrow()
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn gse_with_more_than_one_value_per_characteristic_handled() {
    let accession = "GSE71989";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GSEMatrix: {err}"));
    let GEOObject::GSEMatrix(esets) = result else {
        panic!(
            "{accession} parsed to {}, expected GSEMatrix",
            geo_object_type(&result)
        );
    };
    let eset = esets
        .first()
        .unwrap_or_else(|| panic!("{accession} returned no ExpressionSet entries"));
    assert_eq!(
        (eset.nrow(), eset.ncol()),
        (54_675, 22),
        "{accession} expected expression shape 54675 x 22, got {} x {}",
        eset.nrow(),
        eset.ncol()
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn gse_has_populated_experiment_data() {
    let accession = "GSE53986";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GSEMatrix: {err}"));
    let GEOObject::GSEMatrix(esets) = result else {
        panic!(
            "{accession} parsed to {}, expected GSEMatrix",
            geo_object_type(&result)
        );
    };
    let eset = esets
        .first()
        .unwrap_or_else(|| panic!("{accession} returned no ExpressionSet entries"));
    assert_eq!(
        eset.experimentData
            .get("pubmed_id")
            .and_then(|values| values.first())
            .map(String::as_str),
        Some("24739962"),
        "{accession} should preserve PubMed ID in experimentData"
    );
    assert_eq!(
        eset.experimentData
            .get("contact_name")
            .and_then(|values| values.first())
            .map(String::as_str),
        Some("Jason,A,Hackney"),
        "{accession} should preserve contact_name"
    );
    assert_eq!(
        eset.experimentData
            .get("contact_email")
            .and_then(|values| values.first())
            .map(String::as_str),
        Some("hackney.jason@gene.com"),
        "{accession} should preserve contact_email"
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn gse_populates_experiment_data_as_much_as_possible() {
    for (accession, pubmed, contact, email, title) in [
        (
            "GSE99709",
            "",
            "John,,Mariani",
            "john_mariani@urmc.rochester.edu",
            "RNA-Sequencing of Stat3 silenced oligodendrocyte progenitor cells.",
        ),
        (
            "GSE27712",
            "22253802",
            "Joachim,L,Schultze",
            "j.schultze@uni-bonn.de",
            "GC424 tumor cells and gastric tumors",
        ),
    ] {
        let result = getGEO(GetGeoOptions {
            GEO: Some(accession),
            ..Default::default()
        })
        .unwrap_or_else(|err| panic!("{accession} should download and parse as GSEMatrix: {err}"));
        let GEOObject::GSEMatrix(esets) = result else {
            panic!(
                "{accession} parsed to {}, expected GSEMatrix",
                geo_object_type(&result)
            );
        };
        let eset = esets
            .first()
            .unwrap_or_else(|| panic!("{accession} returned no ExpressionSet entries"));
        assert_eq!(
            eset.experimentData
                .get("pubmed_id")
                .and_then(|values| values.first())
                .map(String::as_str)
                .unwrap_or(""),
            pubmed,
            "{accession} pubmed_id mismatch"
        );
        assert_eq!(
            eset.experimentData
                .get("contact_name")
                .and_then(|values| values.first())
                .map(String::as_str),
            Some(contact),
            "{accession} contact_name mismatch"
        );
        assert_eq!(
            eset.experimentData
                .get("contact_email")
                .and_then(|values| values.first())
                .map(String::as_str),
            Some(email),
            "{accession} contact_email mismatch"
        );
        assert_eq!(
            eset.experimentData
                .get("title")
                .and_then(|values| values.first())
                .map(String::as_str),
            Some(title),
            "{accession} title mismatch"
        );
    }
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn gse_gpl_with_large_integer_columns_handled_correctly() {
    let accession = "GSE7864";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GSEMatrix: {err}"));
    let GEOObject::GSEMatrix(esets) = result else {
        panic!(
            "{accession} parsed to {}, expected GSEMatrix",
            geo_object_type(&result)
        );
    };
    let eset = esets
        .first()
        .unwrap_or_else(|| panic!("{accession} returned no ExpressionSet entries"));
    assert!(
        eset.featureData
            .row_names
            .iter()
            .all(|name| !name.is_empty()),
        "{accession} featureData row names should be non-empty strings"
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn issue_144_regression() {
    let accession = "GSE225759";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GSEMatrix: {err}"));
    let GEOObject::GSEMatrix(esets) = result else {
        panic!(
            "{accession} parsed to {}, expected GSEMatrix",
            geo_object_type(&result)
        );
    };
    let eset = esets
        .first()
        .unwrap_or_else(|| panic!("{accession} returned no ExpressionSet entries"));
    assert_eq!(
        (eset.nrow(), eset.ncol()),
        (442, 272),
        "{accession} expected expression shape 442 x 272, got {} x {}",
        eset.nrow(),
        eset.ncol()
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn gse425_parsing_with_malformed_sample_lines() {
    let accession = "GSE425";
    let result = getGEO(GetGeoOptions {
        GEO: Some(accession),
        ..Default::default()
    })
    .unwrap_or_else(|err| panic!("{accession} should download and parse as GSEMatrix: {err}"));
    let GEOObject::GSEMatrix(esets) = result else {
        panic!(
            "{accession} parsed to {}, expected GSEMatrix",
            geo_object_type(&result)
        );
    };
    let eset = esets
        .first()
        .unwrap_or_else(|| panic!("{accession} returned no ExpressionSet entries"));
    assert!(
        eset.ncol() > 0,
        "{accession} should have at least one expression column"
    );
    assert!(
        eset.phenoData.nrow() > 0,
        "{accession} should have non-empty phenotype data"
    );
}

#[test]
fn local_gse_matrix_expands_characteristics() {
    let dir = tempfile::tempdir().expect("tempdir should be available");
    let path = dir.path().join("GSELOCAL_series_matrix.txt");
    let matrix = [
        "!Series_title\t\"Local series\"",
        "!Series_geo_accession\t\"GSELOCAL\"",
        "!Sample_geo_accession\t\"GSMLOCAL1\"\t\"GSMLOCAL2\"",
        "!Sample_platform_id\t\"GPLLOCAL\"\t\"GPLLOCAL\"",
        "!Sample_characteristics_ch1\t\"tissue: liver\"\t\"tissue: brain\"",
        "!Sample_characteristics_ch1\t\"age: 10\"\t\"age: 11\"",
        "!Sample_characteristics_ch1\t\"age: 12\"\t\"age: 13\"",
        "!series_matrix_table_begin",
        "ID_REF\tGSMLOCAL1\tGSMLOCAL2",
        "probe1\t1.0\t2.0",
        "probe2\t3.0\t4.0",
        "!series_matrix_table_end",
    ]
    .join("\n");
    std::fs::write(&path, matrix).expect("local GSE matrix fixture should be written");

    let parsed = parseGEO(&path, None, None, false, false)
        .unwrap_or_else(|err| panic!("local GSE matrix fixture should parse: {err}"));
    let GEOObject::GSEMatrix(esets) = parsed else {
        panic!(
            "local GSE matrix fixture parsed to {}, expected GSEMatrix",
            geo_object_type(&parsed)
        );
    };
    let eset = esets.first().expect("local matrix should produce one eset");
    assert_eq!((eset.nrow(), eset.ncol()), (2, 2));
    assert_eq!(eset.phenoData.get(0, "tissue:ch1"), Some("liver"));
    assert_eq!(eset.phenoData.get(1, "tissue:ch1"), Some("brain"));
    assert_eq!(
        eset.phenoData.get(0, "age:ch1"),
        Some("10;12"),
        "Repeated characteristics should collapse per sample with semicolons"
    );
    assert_eq!(eset.exprs.row_name(0), Some("probe1"));
    assert_eq!(eset.exprs.column_name(1), Some("GSMLOCAL2"));
}

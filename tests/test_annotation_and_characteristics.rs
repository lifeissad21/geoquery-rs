use geoquery::{
    join_gpl_annotations, join_gpl_annotations_by_key, parse_characteristics, AnnotatedDataFrame,
    AnnotationKey, DataFrame, ExperimentData, ExpressionSet, GEODataTable, GeoError, NumericMatrix,
    TypedColumn, GPL,
};
use std::path::PathBuf;

fn expression_set(feature_names: Vec<&str>) -> ExpressionSet {
    let sample_names = vec!["GSM1".to_string(), "GSM2".to_string()];
    let exprs = feature_names
        .iter()
        .enumerate()
        .map(|(idx, _)| vec![Some(idx as f64), Some((idx + 1) as f64)])
        .collect::<Vec<_>>();
    let feature_names = feature_names
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let feature_data = DataFrame::empty_with_row_names(feature_names.clone());
    ExpressionSet {
        exprs: NumericMatrix::new(exprs, feature_names.clone(), sample_names.clone()),
        feature_names,
        sample_names,
        phenoData: DataFrame::empty(),
        featureData: feature_data.clone(),
        pheno_data: AnnotatedDataFrame::new(DataFrame::empty()),
        feature_data: AnnotatedDataFrame::new(feature_data),
        annotation: Some("GPLTEST".into()),
        experimentData: Default::default(),
        experiment_data: ExperimentData::default(),
    }
}

fn gpl(columns: Vec<&str>, rows: Vec<Vec<Option<&str>>>) -> GPL {
    GPL {
        header: Default::default(),
        dataTable: GEODataTable {
            columns: DataFrame::empty(),
            table: DataFrame::new(
                columns.into_iter().map(str::to_string).collect(),
                rows.into_iter()
                    .map(|row| {
                        row.into_iter()
                            .map(|value| value.map(str::to_string))
                            .collect()
                    })
                    .collect(),
            ),
        },
    }
}

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path)
}

#[test]
fn gpl_annotation_join_preserves_duplicates_and_missing_values() {
    let expression = expression_set(vec!["1007_s_at", "missing_at", "1007_s_at", "117_at"]);
    let gpl = gpl(
        vec![
            "ID",
            "Gene Symbol",
            "ENTREZ_GENE_ID",
            "Gene Title",
            "RefSeq",
        ],
        vec![
            vec![
                Some("1007_s_at"),
                Some("DDR1"),
                Some("780"),
                Some("discoidin domain receptor"),
                Some("NM_001954"),
            ],
            vec![
                Some("117_at"),
                Some("HSPA6"),
                Some("3310"),
                Some("heat shock protein"),
                None,
            ],
        ],
    );

    let joined = join_gpl_annotations(&expression, &gpl).unwrap();

    assert_eq!(
        expression.featureData.ncol(),
        0,
        "input must not be mutated"
    );
    assert_eq!(joined.featureData.nrow(), 4);
    assert_eq!(joined.featureData.row_names, expression.feature_names);
    assert_eq!(
        joined.featureData.get(0, "Gene Symbol").as_deref(),
        Some("DDR1")
    );
    assert_eq!(joined.featureData.get(1, "Gene Symbol"), None);
    assert_eq!(
        joined.featureData.get(2, "Gene Symbol").as_deref(),
        Some("DDR1")
    );
    assert_eq!(joined.featureData.get(3, "RefSeq"), None);
}

#[test]
fn gpl_annotation_join_supports_custom_keys_and_large_tables() {
    let expression = expression_set(vec!["DDR1", "HSPA6", "NOT_PRESENT"]);
    let mut rows = (0..5_000)
        .map(|idx| {
            vec![
                Some(format!("probe_{idx}")),
                Some(format!("GENE{idx}")),
                Some(format!("{idx}")),
            ]
        })
        .collect::<Vec<_>>();
    rows.push(vec![
        Some("1007_s_at".to_string()),
        Some("DDR1".to_string()),
        Some("780".to_string()),
    ]);
    rows.push(vec![
        Some("117_at".to_string()),
        Some("HSPA6".to_string()),
        Some("3310".to_string()),
    ]);
    let gpl = GPL {
        header: Default::default(),
        dataTable: GEODataTable {
            columns: DataFrame::empty(),
            table: DataFrame::new(
                vec!["ID".into(), "Gene Symbol".into(), "ENTREZ_GENE_ID".into()],
                rows,
            ),
        },
    };

    let joined = join_gpl_annotations_by_key(&expression, &gpl, AnnotationKey::GeneSymbol).unwrap();

    assert_eq!(
        joined.featureData.get(0, "ID").as_deref(),
        Some("1007_s_at")
    );
    assert_eq!(
        joined.featureData.get(1, "ENTREZ_GENE_ID").as_deref(),
        Some("3310")
    );
    assert_eq!(joined.featureData.get(2, "ID"), None);
}

#[test]
fn malformed_gpl_tables_return_structured_errors() {
    let expression = expression_set(vec!["1007_s_at"]);
    let gpl = gpl(vec!["Gene Symbol"], vec![vec![Some("DDR1")]]);

    let err = join_gpl_annotations(&expression, &gpl).unwrap_err();
    assert!(
        matches!(err, GeoError::AnnotationMissing { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn characteristic_parser_recovers_mixed_and_malformed_values() {
    let parsed = parse_characteristics("disease:control;sex=male | age: 72");
    assert_eq!(parsed.get("disease").map(String::as_str), Some("control"));
    assert_eq!(parsed.get("sex").map(String::as_str), Some("male"));
    assert_eq!(parsed.get("age").map(String::as_str), Some("72"));

    let parsed = parse_characteristics("male");
    assert_eq!(
        parsed.get("characteristic_1").map(String::as_str),
        Some("male")
    );

    let parsed = parse_characteristics("disease-control");
    assert_eq!(
        parsed.get("characteristic_1").map(String::as_str),
        Some("disease-control")
    );
}

#[test]
fn gse_matrix_expands_repeated_collapsed_and_unknown_characteristics() {
    let eset = geoquery::parse_geo::parseGSEMatrix(fixture(
        "matrix/repeated_characteristics_series_matrix.txt",
    ))
    .unwrap();

    assert_eq!(
        eset.phenoData.get(0, "disease:ch1").as_deref(),
        Some("control")
    );
    assert_eq!(eset.phenoData.get(0, "sex:ch1").as_deref(), Some("male"));
    assert_eq!(eset.phenoData.get(1, "age:ch1").as_deref(), Some("72"));
    assert_eq!(
        eset.phenoData.get(0, "characteristic_1:ch1").as_deref(),
        Some("unstructured note")
    );
    assert_eq!(
        eset.phenoData.get(1, "characteristic_1:ch1").as_deref(),
        Some("disease-control")
    );
}

#[test]
fn compressed_gse_matrix_fixture_parses_without_network() {
    let eset = geoquery::parse_geo::parseGSEMatrix(fixture(
        "matrix/repeated_characteristics_series_matrix.txt.gz",
    ))
    .unwrap();

    assert_eq!((eset.nrow(), eset.ncol()), (3, 2));
    assert_eq!(
        eset.phenoData.get(0, "disease:ch1").as_deref(),
        Some("control")
    );
}

#[test]
fn dataframe_infers_schema_aware_columns_without_breaking_accessors() {
    let df = DataFrame::new(
        vec!["int".into(), "float".into(), "text".into()],
        vec![
            vec![Some("1".into()), Some("1.5".into()), Some("A".into())],
            vec![Some("2".into()), Some("NaN".into()), Some("1".into())],
            vec![None, Some("3.25".into()), Some("B".into())],
        ],
    );

    assert!(matches!(
        df.typed_column("int"),
        Some(TypedColumn::Integer(_))
    ));
    assert!(matches!(
        df.typed_column("float"),
        Some(TypedColumn::Float(_))
    ));
    assert!(matches!(
        df.typed_column("text"),
        Some(TypedColumn::Text(_))
    ));
    assert_eq!(df.get(0, "text").as_deref(), Some("A"));
}

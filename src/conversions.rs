use crate::classes::{AnnotatedDataFrame, Columns, ExperimentData, Meta, NumericMatrix, Table};
use crate::classes::{DataFrame, ExpressionSet, MAList, GDS, GPL};
use crate::error::{GeoError, Result};
use std::collections::BTreeMap;

/// Feature key used when joining GPL platform annotations to an expression set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnnotationKey {
    /// Probe identifier columns such as `ID` or `ID_REF`.
    ProbeId,
    /// Gene symbol columns such as `Gene Symbol`.
    GeneSymbol,
    /// Entrez Gene identifier columns such as `ENTREZ_GENE_ID`.
    Entrez,
    /// Caller-provided GPL table column.
    Custom(String),
}

impl AnnotationKey {
    fn candidates(&self) -> Vec<String> {
        match self {
            Self::ProbeId => vec![
                "ID".into(),
                "ID_REF".into(),
                "ProbeID".into(),
                "probe_id".into(),
            ],
            Self::GeneSymbol => vec![
                "Gene Symbol".into(),
                "Gene symbol".into(),
                "gene_symbol".into(),
                "GENE_SYMBOL".into(),
                "Symbol".into(),
            ],
            Self::Entrez => vec![
                "ENTREZ_GENE_ID".into(),
                "Entrez Gene".into(),
                "ENTREZID".into(),
                "Gene ID".into(),
            ],
            Self::Custom(column) => vec![column.clone()],
        }
    }
}

/// Join GPL platform annotations to an [`ExpressionSet`] using probe IDs.
///
/// The expression matrix and input expression set are never mutated. Row order
/// and duplicate expression feature IDs are preserved; unmatched features have
/// missing annotation values.
///
/// ```
/// use geoquery::{join_gpl_annotations, DataFrame, ExpressionSet, GPL};
/// # use geoquery::{AnnotatedDataFrame, ExperimentData, GEODataTable, NumericMatrix};
/// # let expression = ExpressionSet {
/// #     exprs: NumericMatrix::new(vec![vec![Some(1.0)]], vec!["1007_s_at".into()], vec!["GSM1".into()]),
/// #     feature_names: vec!["1007_s_at".into()],
/// #     sample_names: vec!["GSM1".into()],
/// #     phenoData: DataFrame::empty(),
/// #     featureData: DataFrame::empty_with_row_names(vec!["1007_s_at".into()]),
/// #     pheno_data: AnnotatedDataFrame::new(DataFrame::empty()),
/// #     feature_data: AnnotatedDataFrame::new(DataFrame::empty_with_row_names(vec!["1007_s_at".into()])),
/// #     annotation: Some("GPL570".into()),
/// #     experimentData: Default::default(),
/// #     experiment_data: ExperimentData::default(),
/// # };
/// # let gpl = GPL {
/// #     header: Default::default(),
/// #     dataTable: GEODataTable {
/// #         columns: DataFrame::empty(),
/// #         table: DataFrame::new(
/// #             vec!["ID".into(), "Gene Symbol".into()],
/// #             vec![vec![Some("1007_s_at".into()), Some("DDR1".into())]],
/// #         ),
/// #     },
/// # };
/// let joined = join_gpl_annotations(&expression, &gpl)?;
/// assert_eq!(joined.featureData.get(0, "Gene Symbol").as_deref(), Some("DDR1"));
/// # Ok::<(), geoquery::GeoError>(())
/// ```
pub fn join_gpl_annotations(expression: &ExpressionSet, gpl: &GPL) -> Result<ExpressionSet> {
    join_gpl_annotations_by_key(expression, gpl, AnnotationKey::ProbeId)
}

/// Join GPL platform annotations to an [`ExpressionSet`] using a caller-selected key.
pub fn join_gpl_annotations_by_key(
    expression: &ExpressionSet,
    gpl: &GPL,
    key: AnnotationKey,
) -> Result<ExpressionSet> {
    let gpl_table = gpl.table();
    if gpl_table.ncol() == 0 {
        return Err(GeoError::GPLJoinFailure(
            "GPL table has no annotation columns".into(),
        ));
    }
    let key_column =
        resolve_annotation_key(gpl_table, &key).ok_or_else(|| GeoError::AnnotationMissing {
            column: key.candidates().join("|"),
            context: "GPL annotation table".into(),
        })?;
    let key_idx =
        gpl_table
            .column_index(key_column)
            .ok_or_else(|| GeoError::AnnotationMissing {
                column: key_column.to_string(),
                context: "GPL annotation table".into(),
            })?;

    let mut lookup: BTreeMap<String, usize> = BTreeMap::new();
    for row_idx in 0..gpl_table.nrow() {
        let Some(raw_key) = gpl_table.get_by_index(row_idx, key_idx) else {
            continue;
        };
        let normalized = normalize_annotation_key(&raw_key);
        if !normalized.is_empty() {
            lookup.entry(normalized).or_insert(row_idx);
        }
    }

    let mut feature_data = DataFrame::empty_with_row_names(expression.feature_names.clone());
    for column in gpl_table.column_names() {
        let Some(gpl_col_idx) = gpl_table.column_index(column) else {
            continue;
        };
        let values = expression
            .feature_names
            .iter()
            .map(|feature_id| {
                let row_idx = lookup.get(&normalize_annotation_key(feature_id))?;
                gpl_table.get_by_index(*row_idx, gpl_col_idx)
            })
            .collect::<Vec<_>>();
        feature_data.push_column(column.clone(), values);
    }
    feature_data.column_metadata = gpl_table.column_metadata.clone();

    let mut joined = expression.clone();
    joined.featureData = feature_data.clone();
    joined.feature_data = AnnotatedDataFrame {
        var_metadata: variable_metadata_from_columns(&feature_data),
        data: feature_data,
    };
    if joined.annotation.is_none() {
        joined.annotation = gpl.accession().map(str::to_string);
    }
    Ok(joined)
}

fn resolve_annotation_key<'a>(table: &'a DataFrame, key: &AnnotationKey) -> Option<&'a str> {
    let lower_columns = table
        .column_names()
        .iter()
        .map(|column| (column.to_ascii_lowercase(), column.as_str()))
        .collect::<BTreeMap<_, _>>();
    for candidate in key.candidates() {
        if let Some(column) = table
            .column_names()
            .iter()
            .find(|column| column.as_str() == candidate.as_str())
        {
            return Some(column);
        }
        if let Some(column) = lower_columns.get(&candidate.to_ascii_lowercase()) {
            return Some(column);
        }
    }
    None
}

fn normalize_annotation_key(value: &str) -> String {
    value.trim().trim_matches('"').to_ascii_lowercase()
}

/// Convert a GEO dataset into a native limma-like [`MAList`].
///
/// When `do_log2` is `true`, expression values are transformed with `log2`.
///
/// ```no_run
/// use geoquery::{parseGEO, GEOObject, GDS2MA};
///
/// let parsed = parseGEO("GDS507.soft.gz", None, None, false, true)?;
/// if let GEOObject::GDS(gds) = parsed {
///     let ma = GDS2MA(&gds, false);
///     assert_eq!(ma.nrow(), gds.table().nrow());
/// }
/// # Ok::<(), geoquery::GeoError>(())
/// ```
pub fn GDS2MA(gds: &GDS, do_log2: bool) -> MAList {
    let (row_names, column_names) = expression_names(gds.Table());
    let matrix = gsm_numeric_matrix(gds.Table(), do_log2);
    MAList {
        M: NumericMatrix::new(matrix, row_names.clone(), column_names.clone()),
        row_names,
        column_names,
        A: None,
        targets: gds.Columns().clone(),
        genes: None,
        notes: gds.Meta().clone(),
    }
}

/// Convert a GEO dataset into a native [`ExpressionSet`].
///
/// When `do_log2` is `true`, expression values are transformed with `log2`.
///
/// ```no_run
/// use geoquery::{parseGEO, GEOObject, GDS2eSet};
///
/// let parsed = parseGEO("GDS507.soft.gz", None, None, false, true)?;
/// if let GEOObject::GDS(gds) = parsed {
///     let eset = GDS2eSet(&gds, false);
///     assert_eq!(eset.nrow(), gds.table().nrow());
/// }
/// # Ok::<(), geoquery::GeoError>(())
/// ```
pub fn GDS2eSet(gds: &GDS, do_log2: bool) -> ExpressionSet {
    let (feature_names, sample_names) = expression_names(gds.Table());
    let exprs = NumericMatrix::new(
        gsm_numeric_matrix(gds.Table(), do_log2),
        feature_names.clone(),
        sample_names.clone(),
    );
    let phenoData = gds.Columns().clone();
    let featureData = DataFrame::empty_with_row_names(feature_names.clone());
    let pheno_var_metadata = variable_metadata_from_columns(&phenoData);
    let feature_var_metadata = variable_metadata_for_names(&feature_names);
    ExpressionSet {
        exprs,
        feature_names: feature_names.clone(),
        sample_names,
        pheno_data: AnnotatedDataFrame {
            data: phenoData.clone(),
            var_metadata: pheno_var_metadata,
        },
        feature_data: AnnotatedDataFrame {
            data: featureData.clone(),
            var_metadata: feature_var_metadata,
        },
        phenoData,
        featureData,
        annotation: gds
            .Meta()
            .get("platform")
            .and_then(|values| values.first())
            .cloned(),
        experimentData: gds.Meta().clone(),
        experiment_data: ExperimentData::from_header(gds.Meta()),
    }
}

fn variable_metadata_from_columns(df: &DataFrame) -> DataFrame {
    let rows = df
        .column_names()
        .iter()
        .map(|column| {
            vec![
                Some(column.clone()),
                Some(
                    df.column_metadata
                        .get(column)
                        .cloned()
                        .unwrap_or_else(|| column.clone()),
                ),
            ]
        })
        .collect::<Vec<_>>();
    DataFrame::with_row_names(
        vec!["Column".to_string(), "labelDescription".to_string()],
        rows,
        df.column_names().to_vec(),
    )
}

fn variable_metadata_for_names(names: &[String]) -> DataFrame {
    DataFrame::with_row_names(
        vec!["Column".to_string(), "labelDescription".to_string()],
        Vec::new(),
        names.to_vec(),
    )
}

fn expression_names(table: &DataFrame) -> (Vec<String>, Vec<String>) {
    let sample_names = table
        .column_names()
        .iter()
        .filter(|name| name.starts_with("GSM"))
        .cloned()
        .collect::<Vec<_>>();
    let feature_names = table
        .column_values("ID_REF")
        .or_else(|| table.column_values("ID"))
        .map(|values| {
            values
                .into_iter()
                .map(|value| value.unwrap_or_default())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| (0..table.nrow()).map(|idx| idx.to_string()).collect());
    (feature_names, sample_names)
}

fn gsm_numeric_matrix(table: &DataFrame, do_log2: bool) -> Vec<Vec<Option<f64>>> {
    let gsm_indices = table
        .column_names()
        .iter()
        .enumerate()
        .filter_map(|(idx, name)| name.starts_with("GSM").then_some(idx))
        .collect::<Vec<_>>();

    (0..table.nrow())
        .map(|row_idx| {
            gsm_indices
                .iter()
                .map(|idx| {
                    let value = table.get_by_index(row_idx, *idx)?.parse::<f64>().ok()?;
                    Some(if do_log2 { value.log2() } else { value })
                })
                .collect()
        })
        .collect()
}

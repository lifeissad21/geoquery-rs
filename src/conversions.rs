use crate::classes::{AnnotatedDataFrame, Columns, ExperimentData, Meta, NumericMatrix, Table};
use crate::classes::{DataFrame, ExpressionSet, MAList, GDS};

/// Convert a GEO dataset into a native limma-like [`MAList`].
///
/// When `do_log2` is `true`, expression values are transformed with `log2`.
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
        .columns
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
        df.columns.clone(),
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
        .columns
        .iter()
        .filter(|name| name.starts_with("GSM"))
        .cloned()
        .collect::<Vec<_>>();
    let feature_names = table
        .column_index("ID_REF")
        .or_else(|| table.column_index("ID"))
        .map(|idx| {
            table
                .rows
                .iter()
                .map(|row| {
                    row.get(idx)
                        .and_then(|value| value.clone())
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| (0..table.nrow()).map(|idx| idx.to_string()).collect());
    (feature_names, sample_names)
}

fn gsm_numeric_matrix(table: &DataFrame, do_log2: bool) -> Vec<Vec<Option<f64>>> {
    let gsm_indices = table
        .columns
        .iter()
        .enumerate()
        .filter_map(|(idx, name)| name.starts_with("GSM").then_some(idx))
        .collect::<Vec<_>>();

    table
        .rows
        .iter()
        .map(|row| {
            gsm_indices
                .iter()
                .map(|idx| {
                    let value = row.get(*idx)?.as_deref()?.parse::<f64>().ok()?;
                    Some(if do_log2 { value.log2() } else { value })
                })
                .collect()
        })
        .collect()
}

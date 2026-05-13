use geoquery::GEOObject;

pub fn geo_object_type(value: &GEOObject) -> &'static str {
    match value {
        GEOObject::GDS(_) => "GDS",
        GEOObject::GPL(_) => "GPL",
        GEOObject::GSM(_) => "GSM",
        GEOObject::GSE(_) => "GSE",
        GEOObject::GSEMatrix(_) => "GSEMatrix",
    }
}

#[allow(dead_code)]
pub fn assert_shape(
    context: &str,
    actual_rows: usize,
    actual_cols: usize,
    expected_rows: usize,
    expected_cols: usize,
) {
    assert_eq!(
        (actual_rows, actual_cols),
        (expected_rows, expected_cols),
        "{context}: expected table shape {expected_rows} rows x {expected_cols} columns, got {actual_rows} rows x {actual_cols} columns"
    );
}

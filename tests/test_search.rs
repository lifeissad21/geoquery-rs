use geoquery::searchFieldsGEO;

#[test]
#[ignore = "requires live NCBI E-utilities network access"]
fn search_fields_returns_expected_columns() {
    let result = searchFieldsGEO().unwrap_or_else(|err| {
        panic!("GEO search fields should be available from NCBI E-utilities: {err}")
    });
    for expected in ["Name", "Description", "FullName"] {
        assert!(
            result.column_names().contains(&expected.to_string()),
            "searchFieldsGEO result missing expected column `{expected}`. Columns returned: {:?}",
            result.column_names()
        );
    }
}

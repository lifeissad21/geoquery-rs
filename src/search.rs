use crate::classes::DataFrame;
use crate::error::{GeoError, Result};
use regex::Regex;
use url::form_urlencoded;

/// Search NCBI GEO DataSets with an Entrez query.
///
/// `step` is used as the E-utilities `retmax` value. The current return value
/// preserves the raw JSON response in a one-column [`DataFrame`].
pub fn searchGEO(query: &str, step: usize) -> Result<DataFrame> {
    let encoded = form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
    let url = format!(
        "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=gds&term={encoded}&retmax={step}&retmode=json"
    );
    let body = reqwest::blocking::get(url)?.error_for_status()?.text()?;
    Ok(DataFrame::new(
        vec!["raw_json".into()],
        vec![vec![Some(body)]],
    ))
}

/// Return searchable field metadata for the GEO DataSets Entrez database.
pub fn searchFieldsGEO() -> Result<DataFrame> {
    let url = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/einfo.fcgi?db=gds&retmode=xml";
    let body = reqwest::blocking::get(url)?.error_for_status()?.text()?;
    parse_einfo_fields(&body)
}

fn parse_einfo_fields(xml: &str) -> Result<DataFrame> {
    let field_re = Regex::new(r"(?s)<Field>(.*?)</Field>").expect("valid field regex");
    let tag_re = Regex::new(r"(?s)<([^/>]+)>(.*?)</[^>]+>").expect("valid tag regex");
    let columns = vec![
        "Name".to_string(),
        "FullName".to_string(),
        "Description".to_string(),
        "TermCount".to_string(),
        "IsDate".to_string(),
        "IsNumerical".to_string(),
        "SingleToken".to_string(),
        "Hierarchy".to_string(),
        "IsHidden".to_string(),
    ];
    let mut rows = Vec::new();
    for cap in field_re.captures_iter(xml) {
        let Some(field_xml) = cap.get(1).map(|m| m.as_str()) else {
            continue;
        };
        let mut values = std::collections::BTreeMap::new();
        for tag in tag_re.captures_iter(field_xml) {
            values.insert(tag[1].to_string(), tag[2].trim().to_string());
        }
        rows.push(
            columns
                .iter()
                .map(|col| values.get(col).cloned())
                .collect::<Vec<_>>(),
        );
    }
    if rows.is_empty() {
        return Err(GeoError::Parse("no GEO search fields found".into()));
    }
    Ok(DataFrame::new(columns, rows))
}

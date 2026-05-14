use crate::classes::{
    AnnotatedDataFrame, DataFrame, ExperimentData, ExpressionSet, GEODataTable, GEOObject, Header,
    NumericMatrix, GDS, GPL, GSE, GSM,
};
use crate::error::{GeoError, Result};
use flate2::read::GzDecoder;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::Path;

const NA_STRINGS: &[&str] = &["NA", "null", "NULL", "Null"];

#[derive(Clone, Debug, PartialEq, Eq)]
enum Entity {
    Dataset,
    Sample,
    Series,
    Platform,
    Matrix,
}

/// Parse a local GEO SOFT or GSE series matrix file and infer its entity type.
pub fn parseGEO<P: AsRef<Path>>(
    fname: P,
    GSElimits: Option<(usize, usize)>,
    _destdir: Option<&Path>,
    _AnnotGPL: bool,
    _getGPL: bool,
) -> Result<GEOObject> {
    let path = fname.as_ref();
    let lines = read_lines(path)?;
    match find_first_entity(&lines) {
        Some(Entity::Sample) => Ok(GEOObject::GSM(parseGSM_lines(&lines)?)),
        Some(Entity::Series) => Ok(GEOObject::GSE(parseGSE_lines(&lines, GSElimits)?)),
        Some(Entity::Dataset) => Ok(GEOObject::GDS(parseGDS_lines(&lines)?)),
        Some(Entity::Platform) => Ok(GEOObject::GPL(parseGPL_lines(&lines)?)),
        Some(Entity::Matrix) => Ok(GEOObject::GSEMatrix(vec![parseGSEMatrix_lines(&lines)?])),
        None => Err(GeoError::UnknownEntity(path.display().to_string())),
    }
}

/// Parse a local GEO dataset SOFT file.
pub fn parseGDS<P: AsRef<Path>>(fname: P) -> Result<GDS> {
    parseGDS_lines(&read_lines(fname)?)
}

/// Parse a local GEO platform SOFT or annotation file.
pub fn parseGPL<P: AsRef<Path>>(fname: P) -> Result<GPL> {
    parseGPL_lines(&read_lines(fname)?)
}

/// Parse a local GEO sample SOFT file.
pub fn parseGSM<P: AsRef<Path>>(fname: P) -> Result<GSM> {
    parseGSM_lines(&read_lines(fname)?)
}

/// Parse a local full GEO series SOFT file.
pub fn parseGSE<P: AsRef<Path>>(fname: P, GSElimits: Option<(usize, usize)>) -> Result<GSE> {
    parseGSE_lines(&read_lines(fname)?, GSElimits)
}

/// Parse a local GSE series matrix file into an [`ExpressionSet`].
pub fn parseGSEMatrix<P: AsRef<Path>>(fname: P) -> Result<ExpressionSet> {
    parseGSEMatrix_lines(&read_lines(fname)?)
}

pub(crate) fn read_lines<P: AsRef<Path>>(fname: P) -> Result<Vec<String>> {
    let path = fname.as_ref();
    if !path.exists() {
        return Err(GeoError::MissingFile(path.display().to_string()));
    }
    let bytes = fs::read(path)?;
    let text_bytes = if is_gzip_bytes(&bytes) {
        let mut decoder = GzDecoder::new(bytes.as_slice());
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded)?;
        decoded
    } else {
        bytes
    };
    let text = String::from_utf8_lossy(&text_bytes);
    Ok(text
        .lines()
        .map(|line| line.trim_end_matches('\r').to_string())
        .filter(|line| !line.is_empty())
        .collect())
}

fn is_gzip_bytes(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[0] == 0x1f && bytes[1] == 0x8b
}

fn find_first_entity(lines: &[String]) -> Option<Entity> {
    for line in lines {
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("^dataset") {
            return Some(Entity::Dataset);
        }
        if lower.starts_with("^sample") {
            return Some(Entity::Sample);
        }
        if lower.starts_with("^series") {
            return Some(Entity::Series);
        }
        if lower.starts_with("^platform") || lower.starts_with("^annotation") {
            return Some(Entity::Platform);
        }
        if lower.starts_with("!series_title") && !line.contains(" = ") {
            return Some(Entity::Matrix);
        }
    }
    None
}

pub(crate) fn parseGeoMeta(txt: &[String]) -> Header {
    let re = Regex::new(r"^!\w*?_([^=]+?) = (.*)$").expect("valid metadata regex");
    let mut header = Header::new();
    for line in txt {
        if let Some(caps) = re.captures(line) {
            let key = caps.get(1).unwrap().as_str().trim().to_string();
            let value = caps.get(2).unwrap().as_str().to_string();
            if !key.is_empty() {
                header.entry(key).or_default().push(value);
            }
        }
    }
    header
}

fn split_on_first<'a>(x: &'a str, pat: &str) -> Option<(&'a str, &'a str)> {
    let idx = x.find(pat)?;
    Some((&x[..idx], &x[(idx + pat.len())..]))
}

/// Parse a GEO sample characteristic string into key/value pairs.
///
/// GEO submitters use several characteristic forms, including repeated
/// `key: value` fields, semicolon-collapsed values, and mixed `:` / `=`
/// delimiters. This parser recovers every recognizable pair and stores
/// unkeyed tokens as `characteristic_N`.
///
/// ```
/// let parsed = geoquery::parse_characteristics("disease:control;sex=male");
/// assert_eq!(parsed.get("disease").map(String::as_str), Some("control"));
/// assert_eq!(parsed.get("sex").map(String::as_str), Some("male"));
///
/// let parsed = geoquery::parse_characteristics("male");
/// assert_eq!(parsed.get("characteristic_1").map(String::as_str), Some("male"));
/// ```
pub fn parse_characteristics(value: &str) -> BTreeMap<String, String> {
    let mut parsed: BTreeMap<String, String> = BTreeMap::new();
    let mut unknown_count = 0usize;
    let tokens = value.split([';', '|']).map(str::trim).collect::<Vec<_>>();
    let tokens = if tokens.is_empty() {
        vec![value]
    } else {
        tokens
    };

    for token in tokens {
        let token = token.trim().trim_matches('"').trim();
        let pair = split_on_first(token, ":")
            .or_else(|| split_on_first(token, "="))
            .and_then(|(key, value)| {
                let key = key.trim();
                if key.is_empty() {
                    None
                } else {
                    Some((key, value.trim()))
                }
            });

        if let Some((key, value)) = pair {
            parsed
                .entry(key.to_string())
                .and_modify(|existing| {
                    if !existing.is_empty() && !value.is_empty() {
                        existing.push(';');
                    }
                    existing.push_str(value);
                })
                .or_insert_with(|| value.to_string());
        } else {
            unknown_count += 1;
            parsed.insert(format!("characteristic_{unknown_count}"), token.to_string());
        }
    }

    if parsed.is_empty() {
        parsed.insert("characteristic_1".to_string(), String::new());
    }
    parsed
}

pub(crate) fn parseGeoColumns(txt: &[String]) -> DataFrame {
    let mut rows = Vec::new();
    for line in txt.iter().filter(|line| line.starts_with('#')) {
        if let Some((left, right)) = split_on_first(line, " = ") {
            rows.push(vec![
                Some(left.trim_start_matches('#').to_string()),
                Some(right.to_string()),
            ]);
        }
    }
    DataFrame::new(vec!["Column".into(), "Description".into()], rows)
}

fn apply_column_descriptions(mut table: DataFrame, descriptions: &DataFrame) -> DataFrame {
    for row_idx in 0..descriptions.nrow() {
        let Some(column) = descriptions.get(row_idx, "Column") else {
            continue;
        };
        let Some(description) = descriptions.get(row_idx, "Description") else {
            continue;
        };
        table.set_column_metadata(column, description);
    }
    table
}

fn parseGDSSubsets(txt: &[String]) -> DataFrame {
    let mut descriptions = Vec::<(String, String)>::new();
    for line in txt.iter().filter(|line| line.starts_with("#GSM")) {
        if let Some((sample, description)) = split_on_first(line, " = ") {
            descriptions.push((
                sample.trim_start_matches('#').to_string(),
                description.to_string(),
            ));
        }
    }

    let mut subset_lut: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let subset_types = values_for(txt, "!subset_type");
    let subset_descriptions = values_for(txt, "!subset_description");
    let subset_samples = values_for(txt, "!subset_sample_id");
    for ((kind, description), samples) in subset_types
        .iter()
        .zip(subset_descriptions.iter())
        .zip(subset_samples.iter())
    {
        let key = kind.replace(' ', ".");
        for sample in samples.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            subset_lut
                .entry(key.clone())
                .or_default()
                .insert(sample.to_string(), description.clone());
        }
    }

    let mut columns = vec!["sample".to_string()];
    columns.extend(subset_lut.keys().cloned());
    columns.push("description".to_string());

    let rows = descriptions
        .into_iter()
        .map(|(sample, description)| {
            let mut row = vec![Some(sample.clone())];
            for key in subset_lut.keys() {
                row.push(subset_lut.get(key).and_then(|m| m.get(&sample)).cloned());
            }
            row.push(Some(description));
            row
        })
        .collect();
    DataFrame::new(columns, rows)
}

fn values_for(txt: &[String], prefix: &str) -> Vec<String> {
    txt.iter()
        .filter_map(|line| {
            if line.starts_with(prefix) {
                split_on_first(line, " = ").map(|(_, value)| value.to_string())
            } else {
                None
            }
        })
        .collect()
}

struct GenericTable {
    meta_text: Vec<String>,
    data_frame: DataFrame,
}

fn generic_table_parser(txt: &[String]) -> Result<GenericTable> {
    let begin = txt.iter().position(|line| is_table_begin(line));
    let end = txt.iter().position(|line| is_table_end(line));
    match begin {
        Some(begin_idx) => {
            let end_idx = end.unwrap_or(txt.len());
            let table_lines = if begin_idx + 1 <= end_idx {
                &txt[(begin_idx + 1)..end_idx]
            } else {
                &[]
            };
            Ok(GenericTable {
                meta_text: txt[..begin_idx].to_vec(),
                data_frame: parse_tsv_lines(table_lines)?,
            })
        }
        None => Ok(GenericTable {
            meta_text: txt.to_vec(),
            data_frame: DataFrame::empty(),
        }),
    }
}

fn is_table_begin(line: &str) -> bool {
    line.starts_with('!') && line.contains("_table_begin")
}

fn is_table_end(line: &str) -> bool {
    line.starts_with('!') && line.contains("_table_end")
}

pub(crate) fn parse_tsv_lines(lines: &[String]) -> Result<DataFrame> {
    if lines.is_empty() {
        return Ok(DataFrame::empty());
    }
    let input = lines.join("\n");
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .flexible(true)
        .from_reader(input.as_bytes());
    let headers = reader
        .headers()?
        .iter()
        .map(String::from)
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record?;
        let mut row = record
            .iter()
            .map(|value| {
                if value.trim().is_empty() || NA_STRINGS.contains(&value) {
                    None
                } else {
                    Some(value.to_string())
                }
            })
            .collect::<Vec<_>>();
        row.resize(headers.len(), None);
        rows.push(row);
    }
    Ok(DataFrame::new(headers, rows))
}

fn parseGDS_lines(lines: &[String]) -> Result<GDS> {
    let parsed = generic_table_parser(lines)?;
    let columns = parseGDSSubsets(&parsed.meta_text);
    Ok(GDS {
        header: parseGeoMeta(&parsed.meta_text),
        gpl: None,
        dataTable: GEODataTable {
            columns: columns.clone(),
            table: apply_column_descriptions(parsed.data_frame, &columns),
        },
    })
}

fn parseGPL_lines(lines: &[String]) -> Result<GPL> {
    let parsed = generic_table_parser(lines)?;
    let columns = parseGeoColumns(&parsed.meta_text);
    Ok(GPL {
        header: parseGeoMeta(&parsed.meta_text),
        dataTable: GEODataTable {
            columns: columns.clone(),
            table: apply_column_descriptions(parsed.data_frame, &columns),
        },
    })
}

fn parseGSM_lines(lines: &[String]) -> Result<GSM> {
    let parsed = generic_table_parser(lines)?;
    let columns = parseGeoColumns(&parsed.meta_text);
    Ok(GSM {
        header: parseGeoMeta(&parsed.meta_text),
        dataTable: GEODataTable {
            columns: columns.clone(),
            table: apply_column_descriptions(parsed.data_frame, &columns),
        },
    })
}

fn parseGSE_lines(lines: &[String], GSElimits: Option<(usize, usize)>) -> Result<GSE> {
    let entity_rows = lines
        .iter()
        .enumerate()
        .filter_map(|(idx, line)| {
            if line.starts_with("^SAMPLE") || line.starts_with("^PLATFORM") {
                Some(idx)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if entity_rows.is_empty() {
        return Err(GeoError::Parse(
            "GSE file had no SAMPLE or PLATFORM entities".into(),
        ));
    }

    let header = parseGeoMeta(&lines[..entity_rows[0]]);
    let mut gsms = BTreeMap::new();
    let mut gpls = BTreeMap::new();
    let mut sample_seen = 0usize;

    for (entity_idx, start) in entity_rows.iter().copied().enumerate() {
        let end = entity_rows
            .get(entity_idx + 1)
            .copied()
            .unwrap_or(lines.len());
        let entity_line = &lines[start];
        let accession = split_on_first(entity_line, " = ")
            .map(|(_, value)| value.to_string())
            .unwrap_or_default();
        let segment = &lines[start..end];
        if entity_line.starts_with("^SAMPLE") {
            sample_seen += 1;
            if let Some((lo, hi)) = GSElimits {
                if sample_seen < lo || sample_seen > hi {
                    continue;
                }
            }
            gsms.insert(accession, parseGSM_lines(segment)?);
        } else if entity_line.starts_with("^PLATFORM") {
            gpls.insert(accession, parseGPL_lines(segment)?);
        }
    }

    Ok(GSE { header, gsms, gpls })
}

pub(crate) fn parseGSEMatrix_lines(lines: &[String]) -> Result<ExpressionSet> {
    let table_begin = lines
        .iter()
        .position(|line| line.eq_ignore_ascii_case("!series_matrix_table_begin"))
        .ok_or_else(|| GeoError::Parse("missing !series_matrix_table_begin".into()))?;
    let table_end = lines
        .iter()
        .position(|line| line.eq_ignore_ascii_case("!series_matrix_table_end"))
        .ok_or_else(|| GeoError::Parse("missing !series_matrix_table_end".into()))?;

    let series_lines = lines
        .iter()
        .filter(|line| line.starts_with("!Series_"))
        .cloned()
        .collect::<Vec<_>>();
    let sample_lines = lines
        .iter()
        .filter(|line| line.starts_with("!Sample_"))
        .cloned()
        .collect::<Vec<_>>();
    let experimentData = parse_matrix_header(&series_lines, "!Series_")?;
    let phenoData = parse_matrix_wide_metadata(&sample_lines, "!Sample_")?;
    let data = parse_tsv_lines(&lines[(table_begin + 1)..table_end])?;

    let mut exprs = Vec::new();
    for row_idx in 0..data.nrow() {
        let values = data
            .row_values(row_idx)
            .into_iter()
            .skip(1)
            .map(|v| v.as_deref().and_then(|txt| txt.parse::<f64>().ok()))
            .collect::<Vec<_>>();
        exprs.push(values);
    }

    let annotation = phenoData
        .column_index("platform_id")
        .and_then(|idx| phenoData.get_by_index(0, idx));

    let feature_names = data
        .column_values("ID_REF")
        .or_else(|| data.column_values("ID"))
        .unwrap_or_default()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let sample_names = data
        .column_names()
        .iter()
        .skip(1)
        .cloned()
        .collect::<Vec<_>>();
    let featureData = DataFrame::empty_with_row_names(feature_names.clone());
    Ok(ExpressionSet {
        exprs: NumericMatrix::new(exprs, feature_names.clone(), sample_names.clone()),
        feature_names: feature_names.clone(),
        sample_names,
        pheno_data: AnnotatedDataFrame::new(phenoData.clone()),
        feature_data: AnnotatedDataFrame::new(featureData.clone()),
        phenoData,
        featureData,
        annotation,
        experiment_data: ExperimentData::from_header(&experimentData),
        experimentData,
    })
}

fn parse_matrix_header(lines: &[String], prefix: &str) -> Result<Header> {
    let mut header = Header::new();
    for line in lines {
        let mut parts = line.split('\t');
        let key = parts
            .next()
            .unwrap_or_default()
            .trim_start_matches(prefix)
            .to_string();
        let values = parts
            .map(|value| value.trim_matches('"').to_string())
            .collect::<Vec<_>>();
        header.insert(key, values);
    }
    Ok(header)
}

fn parse_matrix_wide_metadata(lines: &[String], prefix: &str) -> Result<DataFrame> {
    if lines.is_empty() {
        return Ok(DataFrame::empty());
    }
    let mut columns = Vec::new();
    let mut rows: Vec<Vec<Option<String>>> = Vec::new();
    for line in lines {
        let mut parts = line.split('\t');
        let key = parts
            .next()
            .unwrap_or_default()
            .trim_start_matches(prefix)
            .to_string();
        let values = parts
            .map(|value| Some(value.trim_matches('"').to_string()))
            .collect::<Vec<_>>();
        if rows.is_empty() {
            rows.resize(values.len(), Vec::new());
        }
        columns.push(key);
        for (idx, value) in values.into_iter().enumerate() {
            if let Some(row) = rows.get_mut(idx) {
                row.push(value);
            }
        }
    }
    expand_characteristics(DataFrame::new(make_unique(columns), rows))
}

fn make_unique(columns: Vec<String>) -> Vec<String> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    columns
        .into_iter()
        .map(|column| {
            let count = seen.entry(column.clone()).or_insert(0);
            let out = if *count == 0 {
                column.clone()
            } else {
                format!("{column}.{count}")
            };
            *count += 1;
            out
        })
        .collect()
}

fn expand_characteristics(mut df: DataFrame) -> Result<DataFrame> {
    let column_names = df.column_names().to_vec();
    let characteristic_indices = column_names
        .iter()
        .enumerate()
        .filter_map(|(idx, column)| column.starts_with("characteristics_ch").then_some(idx))
        .collect::<Vec<_>>();
    if characteristic_indices.is_empty() {
        return Ok(df);
    }

    let mut expanded: BTreeMap<String, Vec<Option<String>>> = BTreeMap::new();
    for idx in characteristic_indices {
        let channel = if column_names[idx].contains("_ch2") {
            "ch2"
        } else {
            "ch1"
        };
        for row_idx in 0..df.nrow() {
            let Some(value) = df.get_by_index(row_idx, idx) else {
                continue;
            };
            for (key, parsed_value) in parse_characteristics(&value) {
                let column = format!("{key}:{channel}");
                let values = expanded
                    .entry(column)
                    .or_insert_with(|| vec![None; df.nrow()]);
                values[row_idx] = match &values[row_idx] {
                    Some(existing) if !existing.is_empty() && !parsed_value.is_empty() => {
                        Some(format!("{existing};{parsed_value}"))
                    }
                    Some(existing) if !existing.is_empty() => Some(existing.clone()),
                    _ => Some(parsed_value),
                };
            }
        }
    }

    for (column, values) in expanded {
        df.push_column(column, values);
    }

    Ok(df)
}

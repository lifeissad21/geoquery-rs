use crate::classes::{DataFrame, GenomeInfo, NumericMatrix, SummarizedExperiment};
use crate::error::{GeoError, Result};
use crate::parse_geo::parse_tsv_lines;
use flate2::read::GzDecoder;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

/// RNA-seq quantification matrix and gene annotation downloaded from GEO.
#[derive(Clone, Debug, PartialEq)]
pub struct RNASeqQuantResults {
    /// Raw count matrix.
    pub quants: NumericMatrix,
    /// Gene annotation table.
    pub annotation: DataFrame,
}

/// Return links from the GEO download page for a GSE accession.
pub fn getGSEDownloadPageURLs(gse: &str) -> Result<Vec<String>> {
    let url = format!("https://ncbi.nlm.nih.gov/geo/download/?acc={gse}");
    let body = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?
        .get(url)
        .send()?
        .error_for_status()?
        .text()?;
    let re = Regex::new(r#"href\s*=\s*["']([^"']+)["']"#).expect("valid href regex");
    Ok(re
        .captures_iter(&body)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .map(|href| decode_html_href(&href))
        .map(|href| {
            href.strip_prefix("/geo/")
                .map(|rest| format!("https://www.ncbi.nlm.nih.gov/geo/{rest}"))
                .unwrap_or(href)
        })
        .map(|href| href.replacen("ftp://", "https://", 1))
        .collect())
}

/// Select the raw-counts download URL from a list of GEO download links.
pub fn getRNAQuantRawCountsURL(links: &[String]) -> Option<String> {
    links
        .iter()
        .find(|link| link.contains("raw_counts"))
        .cloned()
}

/// Select the gene-annotation download URL from a list of GEO download links.
pub fn getRNAQuantAnnotationURL(links: &[String]) -> Option<String> {
    links
        .iter()
        .find(|link| link.contains("annot.tsv.gz"))
        .cloned()
}

/// Extract the `file` query parameter from a GEO download URL.
pub fn extractFilenameFromDownloadURL(url: &str) -> Result<Option<String>> {
    let decoded_url = decode_html_href(url);
    let parsed = Url::parse(&decoded_url)?;
    Ok(parsed
        .query_pairs()
        .find(|(key, _)| key == "file")
        .map(|(_, value)| value.into_owned()))
}

/// Extract genome build, species, and filename metadata from a GEO download URL.
pub fn urlExtractRNASeqQuantGenomeInfo(url: &str) -> Result<Option<(String, String, String)>> {
    let Some(fname) = extractFilenameFromDownloadURL(url)? else {
        return Ok(None);
    };
    Ok(genome_info_from_filename(&fname).map(|(build, species)| (build, species, fname)))
}

/// Return genome build, species, and filename metadata for a GSE accession.
pub fn getRNASeqQuantGenomeInfo(gse: &str) -> Result<Option<(String, String, String)>> {
    let links = getGSEDownloadPageURLs(gse)?;
    if let Some(annotation_link) = getRNAQuantAnnotationURL(&links) {
        return urlExtractRNASeqQuantGenomeInfo(&annotation_link);
    }
    if let Some(raw_counts_link) = getRNAQuantRawCountsURL(&links) {
        return urlExtractRNASeqQuantGenomeInfo(&raw_counts_link);
    }
    Ok(None)
}

/// Read a GEO RNA-seq annotation TSV or TSV.GZ URL/path into a [`DataFrame`].
pub fn readRNAQuantAnnotation(link: &str) -> Result<DataFrame> {
    let text = fetch_text_maybe_gzip(link)?;
    let lines = text.lines().map(String::from).collect::<Vec<_>>();
    parse_tsv_lines(&lines)
}

/// Read a GEO RNA-seq raw-counts TSV or TSV.GZ URL/path into a [`NumericMatrix`].
pub fn readRNAQuantRawCounts(link: &str) -> Result<NumericMatrix> {
    let text = fetch_text_maybe_gzip(link)?;
    let lines = text.lines().map(String::from).collect::<Vec<_>>();
    let table = parse_tsv_lines(&lines)?;
    let row_names = table
        .column_values(
            table
                .column_names()
                .first()
                .map(String::as_str)
                .unwrap_or_default(),
        )
        .unwrap_or_default()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let column_names = table
        .column_names()
        .iter()
        .skip(1)
        .cloned()
        .collect::<Vec<_>>();
    let values = (0..table.nrow())
        .map(|row_idx| {
            (1..table.ncol())
                .map(|col_idx| {
                    table
                        .get_by_index(row_idx, col_idx)
                        .and_then(|v| v.parse::<f64>().ok())
                })
                .collect()
        })
        .collect();
    Ok(NumericMatrix::new(values, row_names, column_names))
}

fn fetch_text_maybe_gzip(link: &str) -> Result<String> {
    let bytes = if let Some(path) = local_path_from_link(link) {
        fs::read(path)?
    } else {
        reqwest::blocking::get(link)?
            .error_for_status()?
            .bytes()?
            .to_vec()
    };
    let raw = bytes.as_slice();
    let decoded = if raw.len() >= 2 && raw[0] == 0x1f && raw[1] == 0x8b {
        let mut decoder = GzDecoder::new(raw);
        let mut out = Vec::new();
        decoder.read_to_end(&mut out)?;
        out
    } else {
        raw.to_vec()
    };
    Ok(String::from_utf8_lossy(&decoded).into_owned())
}

fn local_path_from_link(link: &str) -> Option<std::path::PathBuf> {
    let path = Path::new(link);
    if path.exists() {
        return Some(path.to_path_buf());
    }
    let parsed = Url::parse(link).ok()?;
    if parsed.scheme() == "file" {
        return parsed.to_file_path().ok();
    }
    None
}

/// Download/read GEO RNA-seq quantifications and annotation for a GSE accession.
pub fn getRNASeqQuantResults(gse: &str) -> Result<RNASeqQuantResults> {
    let links = getGSEDownloadPageURLs(gse)?;
    let raw_counts_link = getRNAQuantRawCountsURL(&links).ok_or_else(|| {
        GeoError::Parse(format!(
            "No raw counts file found for {gse}; check https://ncbi.nlm.nih.gov/geo/download/?acc={gse}"
        ))
    })?;
    let annotation_link = getRNAQuantAnnotationURL(&links)
        .ok_or_else(|| GeoError::Parse(format!("No annotation file found for {gse}")))?;
    Ok(RNASeqQuantResults {
        quants: readRNAQuantRawCounts(&raw_counts_link)?,
        annotation: readRNAQuantAnnotation(&annotation_link)?,
    })
}

/// Build a native [`SummarizedExperiment`] from GEO RNA-seq count resources.
pub fn getRNASeqData(gse: &str) -> Result<SummarizedExperiment> {
    let quantifications = getRNASeqQuantResults(gse)?;
    let mut assays = BTreeMap::new();
    assays.insert("counts".to_string(), quantifications.quants.clone());
    let genome_info =
        getRNASeqQuantGenomeInfo(gse)?.map(|(genome_build, species, fname)| GenomeInfo {
            genome_build,
            species,
            fname,
        });
    let mut metadata = BTreeMap::new();
    metadata.insert("geo_accession".to_string(), vec![gse.to_string()]);
    if let Some(info) = &genome_info {
        metadata.insert("genome_build".to_string(), vec![info.genome_build.clone()]);
        metadata.insert("species".to_string(), vec![info.species.clone()]);
        metadata.insert("genome_info_file".to_string(), vec![info.fname.clone()]);
    }
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs().to_string());
    Ok(SummarizedExperiment {
        assays,
        rowData: quantifications.annotation,
        colData: DataFrame::empty_with_row_names(quantifications.quants.column_names.clone()),
        metadata,
        genome_info,
        created_at,
    })
}

/// Return whether a GSE accession exposes GEO RNA-seq raw-count files.
pub fn hasRNASeqQuantifications(accession: &str) -> Result<bool> {
    Ok(getRNAQuantRawCountsURL(&getGSEDownloadPageURLs(accession)?).is_some())
}

fn decode_html_href(href: &str) -> String {
    href.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn genome_info_from_filename(fname: &str) -> Option<(String, String)> {
    let annot_re = Regex::new(r"^(?P<species>[^.]+)\.(?P<build>[^.]+\.[^.]+)\.annot\.tsv\.gz$")
        .expect("valid annotation filename regex");
    if let Some(caps) = annot_re.captures(fname) {
        return Some((
            caps.name("build")?.as_str().to_string(),
            caps.name("species")?.as_str().to_string(),
        ));
    }

    let raw_re = Regex::new(r"_raw_counts_(?P<build>[^_]+)_NCBI\.tsv\.gz$")
        .expect("valid raw-counts filename regex");
    let caps = raw_re.captures(fname)?;
    let build = caps.name("build")?.as_str().to_string();
    let species = species_from_genome_build(&build)?.to_string();
    Some((build, species))
}

fn species_from_genome_build(build: &str) -> Option<&'static str> {
    if build.starts_with("GRCh") || build.starts_with("hg") {
        Some("Human")
    } else if build.starts_with("GRCm") || build.starts_with("mm") {
        Some("Mouse")
    } else {
        None
    }
}

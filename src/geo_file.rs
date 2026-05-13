use crate::classes::GEOObject;
use crate::error::{GeoError, Result};
use crate::parse_geo;
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Options for [`getGEO`].
///
/// The field names intentionally mirror the R GEOquery API. Use
/// [`Default::default`] and override the fields needed for a specific request.
#[derive(Clone, Debug)]
pub struct GetGeoOptions<'a> {
    /// GEO accession to download, such as `GDS507`, `GPL96`, `GSM11805`, or `GSE2553`.
    pub GEO: Option<&'a str>,
    /// Existing local SOFT or series matrix file to parse instead of downloading.
    pub filename: Option<&'a Path>,
    /// Directory used for downloaded files.
    pub destdir: PathBuf,
    /// Optional inclusive sample bounds for parsing large full GSE SOFT files.
    pub GSElimits: Option<(usize, usize)>,
    /// For GSE accessions, download and parse series matrix files when `true`.
    pub GSEMatrix: bool,
    /// Prefer GPL annotation files when downloading platform data.
    pub AnnotGPL: bool,
    /// Fetch platform records referenced by a GSE when supported.
    pub getGPL: bool,
    /// Parse GSE matrix sample characteristics into expanded columns.
    pub parseCharacteristics: bool,
}

impl<'a> Default for GetGeoOptions<'a> {
    fn default() -> Self {
        Self {
            GEO: None,
            filename: None,
            destdir: std::env::temp_dir(),
            GSElimits: None,
            GSEMatrix: true,
            AnnotGPL: false,
            getGPL: true,
            parseCharacteristics: true,
        }
    }
}

/// Download and parse a GEO accession or parse a local GEO file.
///
/// This is the high-level R-compatible entry point. For GSE accessions,
/// `GSEMatrix` defaults to `true`, returning [`GEOObject::GSEMatrix`] when
/// matrix files are available.
pub fn getGEO(options: GetGeoOptions<'_>) -> Result<GEOObject> {
    let filename = match options.filename {
        Some(filename) => filename.to_path_buf(),
        None => {
            let geo = options
                .GEO
                .ok_or_else(|| GeoError::Parse("supply either GEO or filename".into()))?;
            if options.GSEMatrix && geo.to_ascii_uppercase().starts_with("GSE") {
                return getAndParseGSEMatrices(geo, &options.destdir);
            }
            getGEOfile(geo, &options.destdir, options.AnnotGPL, "full")?
        }
    };
    parse_geo::parseGEO(
        filename,
        options.GSElimits,
        Some(&options.destdir),
        options.AnnotGPL,
        options.getGPL,
    )
}

/// Download all series matrix files for a GSE accession and parse them.
pub fn getAndParseGSEMatrices(GEO: &str, destdir: &Path) -> Result<GEOObject> {
    let geo = GEO.to_ascii_uppercase();
    let stub = accession_stub(&geo);
    fs::create_dir_all(destdir)?;
    let base_url = format!("https://ftp.ncbi.nlm.nih.gov/geo/series/{stub}/{geo}/matrix/");
    let files = get_matrix_listing(&base_url)?;
    let mut esets = Vec::new();
    for file in files {
        let destfile = destdir.join(&file);
        if !destfile.exists() {
            let url = format!("{base_url}{file}");
            downloadFile(&url, &destfile, true)?;
        }
        esets.push(parse_geo::parseGSEMatrix(&destfile)?);
    }
    Ok(GEOObject::GSEMatrix(esets))
}

fn get_matrix_listing(base_url: &str) -> Result<Vec<String>> {
    let body = reqwest::blocking::get(base_url)?
        .error_for_status()?
        .text()?;
    let re = Regex::new(r#"href\s*=\s*["']([^"']+series_matrix[^"']*\.txt\.gz)["']"#)
        .expect("valid GSE matrix href regex");
    let files = re
        .captures_iter(&body)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .filter_map(|href| href.rsplit('/').next().map(str::to_string))
        .collect::<Vec<_>>();
    if files.is_empty() {
        return Err(GeoError::NoMatrixFiles(base_url.to_string()));
    }
    Ok(files)
}

/// Parse a local GEO SOFT or GSE matrix file.
pub fn parseGEO<P: AsRef<Path>>(
    fname: P,
    GSElimits: Option<(usize, usize)>,
    destdir: Option<&Path>,
    AnnotGPL: bool,
    getGPL: bool,
) -> Result<GEOObject> {
    parse_geo::parseGEO(fname, GSElimits, destdir, AnnotGPL, getGPL)
}

/// Download a GEO SOFT or annotation file and return the local path.
///
/// `amount` corresponds to GEO's text view amount. The common value is
/// `"full"`.
pub fn getGEOfile(GEO: &str, destdir: &Path, AnnotGPL: bool, amount: &str) -> Result<PathBuf> {
    let geo = GEO.to_ascii_uppercase();
    let geotype = geo.get(0..3).unwrap_or_default();
    let stub = accession_stub(&geo);
    fs::create_dir_all(destdir)?;

    let (url, destfile, mode_binary) = match geotype {
        "GDS" => (
            format!("https://ftp.ncbi.nlm.nih.gov/geo/datasets/{stub}/{geo}/soft/{geo}.soft.gz"),
            destdir.join(format!("{geo}.soft.gz")),
            true,
        ),
        "GSE" if amount == "full" => (
            format!("https://ftp.ncbi.nlm.nih.gov/geo/series/{stub}/{geo}/soft/{geo}_family.soft.gz"),
            destdir.join(format!("{geo}.soft.gz")),
            true,
        ),
        "GSE" => (
            format!("https://www.ncbi.nlm.nih.gov/geo/query/acc.cgi?targ=self&acc={geo}&form=text&view={amount}"),
            destdir.join(format!("{geo}.soft")),
            false,
        ),
        "GPL" => {
            if AnnotGPL {
                let annot_url = format!(
                    "https://ftp.ncbi.nlm.nih.gov/geo/platforms/{stub}/{geo}/annot/{geo}.annot.gz"
                );
                let annot_dest = destdir.join(format!("{geo}.annot.gz"));
                if !annot_dest.exists() && downloadFile(&annot_url, &annot_dest, true).is_ok() {
                    return Ok(annot_dest);
                }
            }
            (
                format!("https://www.ncbi.nlm.nih.gov/geo/query/acc.cgi?targ=self&acc={geo}&form=text&view={amount}"),
                destdir.join(format!("{geo}.soft.gz")),
                false,
            )
        }
        "GSM" => (
            format!("https://www.ncbi.nlm.nih.gov/geo/query/acc.cgi?targ=self&acc={geo}&form=text&view={amount}"),
            destdir.join(format!("{geo}.soft")),
            false,
        ),
        _ => return Err(GeoError::UnsupportedAccession(GEO.into())),
    };

    if !destfile.exists() {
        downloadFile(&url, &destfile, mode_binary)?;
    }
    Ok(destfile)
}

pub(crate) fn accession_stub(geo: &str) -> String {
    let split = geo
        .char_indices()
        .find(|(_, c)| c.is_ascii_digit())
        .map(|(idx, _)| idx)
        .unwrap_or(geo.len());
    let (prefix, digits) = geo.split_at(split);
    if digits.len() <= 3 {
        format!("{prefix}nnn")
    } else {
        let keep = &digits[..digits.len() - 3];
        format!("{prefix}{keep}nnn")
    }
}

/// Download a URL to a local destination path.
pub fn downloadFile(url: &str, destfile: &Path, _mode_binary: bool) -> Result<()> {
    let response = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?
        .get(url)
        .header(reqwest::header::ACCEPT_ENCODING, "gzip")
        .send()?
        .error_for_status()?;
    let bytes = response.bytes()?;
    let mut file = fs::File::create(destfile)?;
    file.write_all(&bytes)?;
    Ok(())
}

use crate::error::{GeoError, Result};
use crate::geo_file::{accession_stub, downloadFile};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

/// Metadata for a GEO supplemental file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuppFile {
    /// File name as listed by the GEO supplemental FTP directory.
    pub fname: String,
    /// Fully qualified download URL.
    pub url: String,
    /// Destination directory when files were fetched.
    pub destdir: Option<PathBuf>,
    /// Full local file path when files were fetched.
    pub filepath: Option<PathBuf>,
    /// GEO accession that owns this supplemental file.
    pub GEO: String,
}

/// Return the GEO supplemental-file base URL for a GSM, GSE, or GPL accession.
pub fn getGEOSuppFileURL(GEO: &str) -> Result<String> {
    let geo = GEO.to_ascii_uppercase();
    let stub = accession_stub(&geo);
    match geo.get(0..3).unwrap_or_default() {
        "GSM" => Ok(format!(
            "https://ftp.ncbi.nlm.nih.gov/geo/samples/{stub}/{geo}/suppl/"
        )),
        "GSE" => Ok(format!(
            "https://ftp.ncbi.nlm.nih.gov/geo/series/{stub}/{geo}/suppl/"
        )),
        "GPL" => Ok(format!(
            "https://ftp.ncbi.nlm.nih.gov/geo/platforms/{stub}/{geo}/suppl/"
        )),
        _ => Err(GeoError::UnsupportedAccession(GEO.into())),
    }
}

/// Return file names from a GEO supplemental directory listing URL.
pub fn getDirListing(url: &str) -> Result<Vec<String>> {
    let body = reqwest::blocking::get(url)?.error_for_status()?.text()?;
    let re = Regex::new(r#"href="([^"]+)""#).expect("valid href regex");
    let files = re
        .captures_iter(&body)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .filter(|name| name.starts_with('G'))
        .collect::<Vec<_>>();
    if files.is_empty() {
        return Err(GeoError::NoSupplementalFiles(url.to_string()));
    }
    Ok(files)
}

/// List or download supplemental files for a GEO accession.
///
/// When `fetch_files` is `false`, returned [`SuppFile`] values contain only
/// remote metadata. When `fetch_files` is `true`, files are downloaded into
/// `baseDir` or `baseDir/GEO` depending on `makeDirectory`.
pub fn getGEOSuppFiles(
    GEO: &str,
    makeDirectory: bool,
    baseDir: &Path,
    fetch_files: bool,
    filter_regex: Option<&str>,
) -> Result<Vec<SuppFile>> {
    let geo = GEO.to_ascii_uppercase();
    let url = getGEOSuppFileURL(&geo)?;
    let mut fnames = getDirListing(&url)?;
    if let Some(pattern) = filter_regex {
        let re = Regex::new(pattern).map_err(|err| GeoError::Parse(err.to_string()))?;
        fnames.retain(|name| re.is_match(name));
    }

    let storedir = if makeDirectory {
        baseDir.join(&geo)
    } else {
        baseDir.to_path_buf()
    };

    if fetch_files {
        fs::create_dir_all(&storedir)?;
    }

    let mut files = Vec::new();
    for fname in fnames {
        let file_url = format!("{url}{fname}");
        let filepath = storedir.join(&fname);
        if fetch_files && !filepath.exists() {
            downloadFile(&file_url, &filepath, true)?;
        }
        files.push(SuppFile {
            fname,
            url: file_url,
            destdir: fetch_files.then(|| storedir.clone()),
            filepath: fetch_files.then_some(filepath),
            GEO: geo.clone(),
        });
    }
    Ok(files)
}

/// Return the tab-delimited GEO series supplemental file listing.
pub fn getGEOSeriesFileListing(GSE: &str) -> Result<Vec<Vec<String>>> {
    let url = format!("{}filelist.txt", getGEOSuppFileURL(GSE)?);
    let text = reqwest::blocking::get(url)?.error_for_status()?.text()?;
    Ok(text
        .lines()
        .map(|line| line.split('\t').map(String::from).collect())
        .collect())
}

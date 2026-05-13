use geoquery::{getGEOSuppFileURL, getGEOSuppFiles};

#[test]
fn supplemental_file_urls_match_geo_layout() {
    let gse = "GSE161228";
    assert_eq!(
        getGEOSuppFileURL(gse)
            .unwrap_or_else(|err| panic!("Could not build supplemental URL for {gse}: {err}")),
        "https://ftp.ncbi.nlm.nih.gov/geo/series/GSE161nnn/GSE161228/suppl/",
        "{gse} supplemental URL should use the GEO series FTP layout"
    );
    let gsm = "GSM15789";
    assert_eq!(
        getGEOSuppFileURL(gsm)
            .unwrap_or_else(|err| panic!("Could not build supplemental URL for {gsm}: {err}")),
        "https://ftp.ncbi.nlm.nih.gov/geo/samples/GSM15nnn/GSM15789/suppl/",
        "{gsm} supplemental URL should use the GEO sample FTP layout"
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access and downloads supplemental files"]
fn gse_supplemental_files_downloading_works() {
    let accession = "GSE1000";
    let tempdir = tempfile::tempdir().expect("tempdir should be available");
    let result = getGEOSuppFiles(accession, true, tempdir.path(), true, None)
        .unwrap_or_else(|err| panic!("{accession} supplemental files should download: {err}"));
    assert_eq!(
        result.len(),
        1,
        "{accession} expected one supplemental file, got {}: {:?}",
        result.len(),
        result
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access and downloads supplemental files"]
fn gsm_supplemental_files_downloading_works() {
    let accession = "GSM15789";
    let tempdir = tempfile::tempdir().expect("tempdir should be available");
    let result = getGEOSuppFiles(accession, true, tempdir.path(), true, None)
        .unwrap_or_else(|err| panic!("{accession} supplemental files should download: {err}"));
    assert_eq!(
        result.len(),
        1,
        "{accession} expected one supplemental file, got {}: {:?}",
        result.len(),
        result
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn gse_supplemental_file_no_download_works() {
    let accession = "GSE63137";
    let result = getGEOSuppFiles(accession, true, std::path::Path::new("."), false, None)
        .unwrap_or_else(|err| panic!("{accession} supplemental listing should load: {err}"));
    assert_eq!(
        result.len(),
        12,
        "{accession} expected 12 supplemental listings, got {}: {:?}",
        result.len(),
        result
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn gse_supplemental_file_filtering_works() {
    let accession = "GSE63137";
    let result = getGEOSuppFiles(
        accession,
        true,
        std::path::Path::new("."),
        false,
        Some("txt.gz"),
    )
    .unwrap_or_else(|err| panic!("{accession} filtered supplemental listing should load: {err}"));
    assert_eq!(
        result.len(),
        4,
        "{accession} expected 4 txt.gz supplemental listings, got {}: {:?}",
        result.len(),
        result
    );
}

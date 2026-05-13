use geoquery::{
    extractFilenameFromDownloadURL, getGSEDownloadPageURLs, getRNAQuantAnnotationURL,
    getRNAQuantRawCountsURL, getRNASeqData, getRNASeqQuantGenomeInfo, getRNASeqQuantResults,
    hasRNASeqQuantifications, readRNAQuantAnnotation, readRNAQuantRawCounts,
    urlExtractRNASeqQuantGenomeInfo,
};
use std::io::Write;

#[test]
fn rnaseq_download_url_metadata_parses() {
    let url = "https://www.ncbi.nlm.nih.gov/geo/download/?format=file&type=rnaseq_counts&file=Human.GRCh38.p13.annot.tsv.gz";
    assert_eq!(
        extractFilenameFromDownloadURL(url)
            .unwrap_or_else(|err| panic!("Could not parse annotation download URL `{url}`: {err}"))
            .as_deref(),
        Some("Human.GRCh38.p13.annot.tsv.gz"),
        "Expected `file` query parameter to be extracted from annotation URL `{url}`"
    );
    let (genome_build, species, fname) = urlExtractRNASeqQuantGenomeInfo(url)
        .unwrap_or_else(|err| {
            panic!("Could not parse genome info from annotation URL `{url}`: {err}")
        })
        .unwrap_or_else(|| panic!("Annotation URL `{url}` did not produce genome info"));
    assert_eq!(species, "Human", "Wrong species parsed from `{fname}`");
    assert_eq!(
        genome_build, "GRCh38.p13",
        "Wrong genome build parsed from `{fname}`"
    );
    assert_eq!(fname, "Human.GRCh38.p13.annot.tsv.gz");

    let raw_url = "https://www.ncbi.nlm.nih.gov/geo/download/?acc=GSE83322&format=file&type=rnaseq_counts&file=GSE83322_raw_counts_GRCh38.p13_NCBI.tsv.gz";
    let (raw_genome_build, raw_species, raw_fname) = urlExtractRNASeqQuantGenomeInfo(raw_url)
        .unwrap_or_else(|err| {
            panic!("Could not parse genome info from raw-counts URL `{raw_url}`: {err}")
        })
        .unwrap_or_else(|| panic!("Raw-counts URL `{raw_url}` did not produce genome info"));
    assert_eq!(
        raw_species, "Human",
        "Wrong species parsed from `{raw_fname}`"
    );
    assert_eq!(
        raw_genome_build, "GRCh38.p13",
        "Wrong genome build parsed from `{raw_fname}`"
    );
    assert_eq!(raw_fname, "GSE83322_raw_counts_GRCh38.p13_NCBI.tsv.gz");

    let escaped_url = "https://www.ncbi.nlm.nih.gov/geo/download/?acc=GSE83322&amp;format=file&amp;type=rnaseq_counts&amp;file=Human.GRCh38.p13.annot.tsv.gz";
    let (escaped_genome_build, escaped_species, escaped_fname) =
        urlExtractRNASeqQuantGenomeInfo(escaped_url)
            .unwrap_or_else(|err| {
                panic!("Could not parse genome info from escaped URL `{escaped_url}`: {err}")
            })
            .unwrap_or_else(|| panic!("Escaped URL `{escaped_url}` did not produce genome info"));
    assert_eq!(
        escaped_species, "Human",
        "Wrong species parsed from escaped filename `{escaped_fname}`"
    );
    assert_eq!(
        escaped_genome_build, "GRCh38.p13",
        "Wrong genome build parsed from escaped filename `{escaped_fname}`"
    );
}

#[test]
fn rnaseq_link_filters_work() {
    let links = vec![
        "https://example.test/other".to_string(),
        "https://example.test/download?file=Human.GRCh38.p13.annot.tsv.gz".to_string(),
        "https://example.test/download?file=GSE_raw_counts.tsv.gz".to_string(),
    ];
    let annotation = getRNAQuantAnnotationURL(&links)
        .unwrap_or_else(|| panic!("Expected annotation URL in links: {links:?}"));
    assert!(
        annotation.contains("annot.tsv.gz"),
        "Annotation URL filter returned `{annotation}`, expected it to contain `annot.tsv.gz`"
    );
    let raw_counts = getRNAQuantRawCountsURL(&links)
        .unwrap_or_else(|| panic!("Expected raw-counts URL in links: {links:?}"));
    assert!(
        raw_counts.contains("raw_counts"),
        "Raw-counts URL filter returned `{raw_counts}`, expected it to contain `raw_counts`"
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn gse_download_page_urls_return_links() {
    let accession = "GSE83322";
    let links = getGSEDownloadPageURLs(accession)
        .unwrap_or_else(|err| panic!("{accession} download page should be readable: {err}"));
    assert!(
        !links.is_empty(),
        "{accession} download page returned zero links"
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn has_rnaseq_quantifications_matches_known_records() {
    assert!(
        hasRNASeqQuantifications("GSE83322").unwrap_or_else(|err| panic!(
            "Could not check RNA-seq quantifications for GSE83322: {err}"
        )),
        "GSE83322 should have an RNA-seq raw-counts link"
    );
    assert!(
        !hasRNASeqQuantifications("GSE2553").unwrap_or_else(|err| panic!(
            "Could not check RNA-seq quantifications for GSE2553: {err}"
        )),
        "GSE2553 should not have an RNA-seq raw-counts link"
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn genome_info_returns_correct_data() {
    let accession = "GSE83322";
    let result = getRNASeqQuantGenomeInfo(accession)
        .unwrap_or_else(|err| panic!("{accession} genome-info request/parsing failed: {err}"));

    let (genome_build, species, fname) =
        result.unwrap_or_else(|| panic!("No genome info found for {accession}. The download page may no longer expose an annotation/raw-counts URL with a parsable `file=` query parameter."));

    assert_eq!(
        species, "Human",
        "{accession} expected species `Human`, got `{species}` from filename `{fname}`"
    );
    assert!(
        genome_build.starts_with("GR"),
        "{accession} expected genome build to start with `GR`, got `{genome_build}` from filename `{fname}`"
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn read_rna_quant_raw_counts_returns_matrix() {
    let accession = "GSE83322";
    let links = getGSEDownloadPageURLs(accession)
        .unwrap_or_else(|err| panic!("{accession} download page should be readable: {err}"));
    let link = getRNAQuantRawCountsURL(&links)
        .unwrap_or_else(|| panic!("{accession} should expose a raw-counts URL. Links: {links:?}"));
    let result = readRNAQuantRawCounts(&link)
        .unwrap_or_else(|err| panic!("Could not read raw counts from `{link}`: {err}"));
    assert!(
        result.nrow() > 0,
        "Raw-count matrix from `{link}` should have at least one row"
    );
    assert!(
        result.ncol() > 0,
        "Raw-count matrix from `{link}` should have at least one column"
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn read_rna_quant_annotation_returns_data_frame() {
    let accession = "GSE83322";
    let links = getGSEDownloadPageURLs(accession)
        .unwrap_or_else(|err| panic!("{accession} download page should be readable: {err}"));
    let link = getRNAQuantAnnotationURL(&links)
        .unwrap_or_else(|| panic!("{accession} should expose an annotation URL. Links: {links:?}"));
    let result = readRNAQuantAnnotation(&link)
        .unwrap_or_else(|err| panic!("Could not read annotation from `{link}`: {err}"));
    assert!(
        result.nrow() > 0,
        "Annotation from `{link}` should have rows"
    );
    assert!(
        result.ncol() > 0,
        "Annotation from `{link}` should have columns"
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn get_rnaseq_quant_results_returns_quantifications_and_annotation() {
    let accession = "GSE83322";
    let result = getRNASeqQuantResults(accession)
        .unwrap_or_else(|err| panic!("{accession} RNA-seq quant results should load: {err}"));
    assert!(
        result.quants.nrow() > 0,
        "{accession} quant matrix should have rows"
    );
    assert!(
        result.quants.ncol() > 0,
        "{accession} quant matrix should have columns"
    );
    assert!(
        result.annotation.nrow() > 0,
        "{accession} annotation should have rows"
    );
}

#[test]
#[ignore = "requires live NCBI GEO network access"]
fn get_rnaseq_data_returns_summarized_experiment() {
    let accession = "GSE83322";
    let result = getRNASeqData(accession)
        .unwrap_or_else(|err| panic!("{accession} SummarizedExperiment should load: {err}"));
    assert!(
        result.nrow() > 0,
        "{accession} SummarizedExperiment should have rows"
    );
    assert!(
        result.ncol() > 0,
        "{accession} SummarizedExperiment should have columns"
    );
    assert!(
        result.assays.contains_key("counts"),
        "{accession} SummarizedExperiment should contain a `counts` assay"
    );
}

#[test]
fn local_gzipped_rnaseq_files_parse_without_network() {
    let dir = tempfile::tempdir().expect("tempdir should be available");
    let counts_path = dir.path().join("counts.tsv.gz");
    let annot_path = dir.path().join("annot.tsv.gz");
    write_gzip_tsv(
        &counts_path,
        "GeneID\tGSM1\tGSM2\ngene1\t10\t20\ngene2\t30\t40\n",
    );
    write_gzip_tsv(
        &annot_path,
        "GeneID\tSymbol\tDescription\ngene1\tA\tAlpha\ngene2\tB\tBeta\n",
    );

    let counts = readRNAQuantRawCounts(
        counts_path
            .to_str()
            .expect("local counts path should be UTF-8"),
    )
    .unwrap_or_else(|err| panic!("local gzipped counts should parse: {err}"));
    assert_eq!((counts.nrow(), counts.ncol()), (2, 2));
    assert_eq!(counts.row_name(1), Some("gene2"));
    assert_eq!(counts.column_name(0), Some("GSM1"));
    assert_eq!(counts.get(0, 1), Some(20.0));

    let annotation = readRNAQuantAnnotation(
        annot_path
            .to_str()
            .expect("local annotation path should be UTF-8"),
    )
    .unwrap_or_else(|err| panic!("local gzipped annotation should parse: {err}"));
    assert_eq!((annotation.nrow(), annotation.ncol()), (2, 3));
    assert_eq!(annotation.get(0, "Symbol"), Some("A"));
}

fn write_gzip_tsv(path: &std::path::Path, body: &str) {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(body.as_bytes())
        .expect("fixture body should gzip");
    std::fs::write(path, encoder.finish().expect("gzip fixture should finish"))
        .expect("gzip fixture should be written");
}

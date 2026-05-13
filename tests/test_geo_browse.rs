use geoquery::{browseGEOAccession, browseWebsiteRNASeqSearch, urlForAccession};

#[test]
fn accession_urls_match_r_package_shape() {
    let accession = "GSE262484";
    let expected = "https://www.ncbi.nlm.nih.gov/geo/query/acc.cgi?acc=GSE262484";
    assert_eq!(
        urlForAccession(accession),
        expected,
        "urlForAccession({accession}) should match the R package URL format"
    );
    assert_eq!(
        browseGEOAccession(accession),
        urlForAccession(accession),
        "browseGEOAccession({accession}) should return the same URL in the native Rust package"
    );
    assert!(
        browseWebsiteRNASeqSearch().contains("rnaseq%20counts"),
        "RNA-seq browse URL should include the encoded `rnaseq counts` filter, got `{}`",
        browseWebsiteRNASeqSearch()
    );
}

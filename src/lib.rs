#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![warn(missing_docs)]
//! Native Rust access to NCBI Gene Expression Omnibus (GEO) data.
//!
//! This crate is a Rust rewrite of the R/Bioconductor GEOquery package. It
//! keeps compatibility-oriented names such as [`getGEO`], [`GDS2MA`], and
//! [`getGEOSuppFiles`], and also exposes idiomatic snake_case aliases for
//! common entry points.
//!
//! # Example
//!
//! ```no_run
//! use geoquery::{get_geo, GEOObject, GetGeoOptions};
//!
//! let object = get_geo(GetGeoOptions {
//!     GEO: Some("GDS507"),
//!     GSEMatrix: false,
//!     ..Default::default()
//! })?;
//!
//! if let GEOObject::GDS(gds) = object {
//!     println!("{} rows", gds.table().nrow());
//! }
//! # Ok::<(), geoquery::GeoError>(())
//! ```
//!
//! # Cookbook
//!
//! Parse a local series matrix without network access:
//!
//! ```no_run
//! let eset = geoquery::parse_gse_matrix("GSE2553_series_matrix.txt.gz")?;
//! println!("{} features x {} samples", eset.nrow(), eset.ncol());
//! # Ok::<(), geoquery::GeoError>(())
//! ```
//!
//! Join GPL annotations into `featureData` while preserving expression row
//! order and duplicate probe IDs:
//!
//! ```
//! use geoquery::{
//!     join_gpl_annotations, AnnotatedDataFrame, DataFrame, ExperimentData,
//!     ExpressionSet, GEODataTable, GPL, NumericMatrix,
//! };
//!
//! let expression = ExpressionSet {
//!     exprs: NumericMatrix::new(
//!         vec![vec![Some(1.0)], vec![Some(2.0)]],
//!         vec!["1007_s_at".into(), "117_at".into()],
//!         vec!["GSM1".into()],
//!     ),
//!     feature_names: vec!["1007_s_at".into(), "117_at".into()],
//!     sample_names: vec!["GSM1".into()],
//!     phenoData: DataFrame::empty(),
//!     featureData: DataFrame::empty_with_row_names(vec!["1007_s_at".into(), "117_at".into()]),
//!     pheno_data: AnnotatedDataFrame::new(DataFrame::empty()),
//!     feature_data: AnnotatedDataFrame::new(DataFrame::empty_with_row_names(vec![
//!         "1007_s_at".into(),
//!         "117_at".into(),
//!     ])),
//!     annotation: Some("GPL570".into()),
//!     experimentData: Default::default(),
//!     experiment_data: ExperimentData::default(),
//! };
//! let gpl = GPL {
//!     header: Default::default(),
//!     dataTable: GEODataTable {
//!         columns: DataFrame::empty(),
//!         table: DataFrame::new(
//!             vec!["ID".into(), "Gene Symbol".into()],
//!             vec![
//!                 vec![Some("1007_s_at".into()), Some("DDR1".into())],
//!                 vec![Some("117_at".into()), Some("HSPA6".into())],
//!             ],
//!         ),
//!     },
//! };
//!
//! let joined = join_gpl_annotations(&expression, &gpl)?;
//! assert_eq!(joined.featureData.get(0, "Gene Symbol").as_deref(), Some("DDR1"));
//! # Ok::<(), geoquery::GeoError>(())
//! ```
//!
//! Work with schema-aware `DataFrame` storage:
//!
//! ```
//! use geoquery::{DataFrame, TypedColumn};
//!
//! let df = DataFrame::new(
//!     vec!["count".into(), "symbol".into()],
//!     vec![
//!         vec![Some("1".into()), Some("DDR1".into())],
//!         vec![Some("2".into()), Some("HSPA6".into())],
//!     ],
//! );
//! assert!(matches!(df.typed_column("count"), Some(TypedColumn::Integer(_))));
//! assert_eq!(df.column_names(), &["count".to_string(), "symbol".to_string()]);
//! ```

/// Native replacements for GEOquery and Bioconductor data structures.
pub mod classes;
/// Conversion helpers from GEO datasets into analysis-oriented structures.
pub mod conversions;
/// Error and result types.
pub mod error;
/// High-level download and parse entry points.
pub mod geo_file;
/// Parsers for local GEO SOFT and series matrix files.
pub mod parse_geo;
/// Helpers for GEO RNA-seq quantification resources.
pub mod rnaseq;
/// NCBI GEO search helpers.
pub mod search;
/// GEO supplemental-file listing and download helpers.
pub mod supp_files;

pub use classes::{
    dataTable, Accession, AnnotatedDataFrame, ColumnData, Columns, DataFrame, ExperimentData,
    ExpressionSet, GEOData, GEODataTable, GEOMatrix, GEOObject, GPLList, GSMList, GenomeInfo,
    MAList, Meta, NumericMatrix, SummarizedExperiment, Table, TypedColumn, Value, GDS, GPL, GSE,
    GSM,
};
pub use conversions::{
    join_gpl_annotations, join_gpl_annotations_by_key, AnnotationKey, GDS2eSet, GDS2MA,
};
pub use error::{GeoError, Result};
pub use geo_file::{
    downloadFile, getAndParseGSEMatrices, getGEO, getGEOfile, parseGEO, GetGeoOptions,
};
pub use parse_geo::parse_characteristics;
pub use rnaseq::{
    extractFilenameFromDownloadURL, getGSEDownloadPageURLs, getRNAQuantAnnotationURL,
    getRNAQuantRawCountsURL, getRNASeqData, getRNASeqQuantGenomeInfo, getRNASeqQuantResults,
    hasRNASeqQuantifications, readRNAQuantAnnotation, readRNAQuantRawCounts,
    urlExtractRNASeqQuantGenomeInfo,
};
pub use search::{searchFieldsGEO, searchGEO};
pub use supp_files::{
    getDirListing, getGEOSeriesFileListing, getGEOSuppFileURL, getGEOSuppFiles, SuppFile,
};

/// Return the NCBI GEO browser URL for an accession.
///
/// ```
/// assert_eq!(
///     geoquery::url_for_accession("GSE262484"),
///     "https://www.ncbi.nlm.nih.gov/geo/query/acc.cgi?acc=GSE262484"
/// );
/// ```
pub fn url_for_accession(geo: &str) -> String {
    urlForAccession(geo)
}

/// R-compatible alias for [`url_for_accession`].
pub fn urlForAccession(geo: &str) -> String {
    format!("https://www.ncbi.nlm.nih.gov/geo/query/acc.cgi?acc={geo}")
}

/// Return the GEO browser URL for an accession.
///
/// In the R package this opens a browser. The native Rust package returns the
/// URL so callers can choose how to open it.
///
/// ```
/// assert!(geoquery::browse_geo_accession("GDS10").contains("GDS10"));
/// ```
pub fn browse_geo_accession(geo: &str) -> String {
    browseGEOAccession(geo)
}

/// R-compatible alias for [`browse_geo_accession`].
pub fn browseGEOAccession(geo: &str) -> String {
    urlForAccession(geo)
}

/// Return the GEO website URL for browsing RNA-seq count datasets.
///
/// ```
/// assert!(geoquery::browse_rnaseq_search().contains("rnaseq%20counts"));
/// ```
pub fn browse_rnaseq_search() -> &'static str {
    browseWebsiteRNASeqSearch()
}

/// R-compatible alias for [`browse_rnaseq_search`].
pub fn browseWebsiteRNASeqSearch() -> &'static str {
    "https://ncbi.nlm.nih.gov/gds?term=%22rnaseq%20counts%22%5BFilter%5D"
}

/// Return the GEO supplemental-file FTP URL for an accession.
///
/// ```
/// assert_eq!(
///     geoquery::get_geo_supp_file_url("GSE161228").unwrap(),
///     "https://ftp.ncbi.nlm.nih.gov/geo/series/GSE161nnn/GSE161228/suppl/"
/// );
/// ```
pub fn get_geo_supp_file_url(geo: &str) -> Result<String> {
    getGEOSuppFileURL(geo)
}

/// Native snake_case alias for [`getGEO`].
pub fn get_geo(options: GetGeoOptions<'_>) -> Result<GEOObject> {
    getGEO(options)
}

/// Native snake_case alias for [`parseGEO`].
pub fn parse_geo<P: AsRef<std::path::Path>>(
    fname: P,
    gse_limits: Option<(usize, usize)>,
    destdir: Option<&std::path::Path>,
    annot_gpl: bool,
    get_gpl: bool,
) -> Result<GEOObject> {
    parseGEO(fname, gse_limits, destdir, annot_gpl, get_gpl)
}

/// Native snake_case alias for [`parse_geo::parseGSE`].
///
/// ```no_run
/// let gse = geoquery::parse_gse("GSE1563_family.soft.gz", None)?;
/// assert!(!gse.gsm_list().is_empty());
/// # Ok::<(), geoquery::GeoError>(())
/// ```
pub fn parse_gse<P: AsRef<std::path::Path>>(
    fname: P,
    gse_limits: Option<(usize, usize)>,
) -> Result<GSE> {
    parse_geo::parseGSE(fname, gse_limits)
}

/// Native snake_case alias for [`parse_geo::parseGSEMatrix`].
///
/// ```no_run
/// let eset = geoquery::parse_gse_matrix("GSE2553_series_matrix.txt.gz")?;
/// assert!(eset.ncol() > 0);
/// # Ok::<(), geoquery::GeoError>(())
/// ```
pub fn parse_gse_matrix<P: AsRef<std::path::Path>>(fname: P) -> Result<ExpressionSet> {
    parse_geo::parseGSEMatrix(fname)
}

/// Native snake_case alias for [`getGEOfile`].
pub fn get_geo_file(
    geo: &str,
    destdir: &std::path::Path,
    annot_gpl: bool,
    amount: &str,
) -> Result<std::path::PathBuf> {
    getGEOfile(geo, destdir, annot_gpl, amount)
}

/// Native snake_case alias for [`getRNASeqData`].
///
/// ```no_run
/// let se = geoquery::get_rna_seq_data("GSE161228")?;
/// assert!(se.assay("counts").is_some());
/// # Ok::<(), geoquery::GeoError>(())
/// ```
pub fn get_rna_seq_data(gse: &str) -> Result<SummarizedExperiment> {
    getRNASeqData(gse)
}

/// Native snake_case alias for [`GDS2MA`].
pub fn gds_to_ma(gds: &GDS, do_log2: bool) -> MAList {
    GDS2MA(gds, do_log2)
}

/// Native snake_case alias for [`GDS2eSet`].
pub fn gds_to_expression_set(gds: &GDS, do_log2: bool) -> ExpressionSet {
    GDS2eSet(gds, do_log2)
}

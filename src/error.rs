use thiserror::Error;

/// Crate-wide result type returned by GEO parsing, download, and conversion APIs.
pub type Result<T> = std::result::Result<T, GeoError>;

/// Errors that can occur while downloading, parsing, or converting GEO data.
#[derive(Debug, Error)]
pub enum GeoError {
    /// The accession prefix is not one of the supported GEO entity types.
    #[error("unsupported GEO accession type for `{0}`")]
    UnsupportedAccession(String),
    /// The accession looked like a GEO entity but failed validation.
    #[error("invalid GEO accession `{0}`: {1}")]
    InvalidAccession(String, String),
    /// A requested local SOFT, matrix, or tabular file does not exist.
    #[error("file does not exist: {0}")]
    MissingFile(String),
    /// The parser could not identify a GEO entity marker in the file.
    #[error("could not identify GEO entity in {0}")]
    UnknownEntity(String),
    /// A parser expected one GEO entity type but found another.
    #[error("expected GEO entity `{expected}` but found `{found}`")]
    EntityMismatch {
        /// Expected GEO entity type.
        expected: &'static str,
        /// Entity type detected in the input.
        found: &'static str,
    },
    /// A GSE matrix directory did not contain series matrix files.
    #[error("no GEO matrix files found at {0}")]
    NoMatrixFiles(String),
    /// A GSE matrix was required but missing or unusable.
    #[error("missing GEO matrix: {0}")]
    MissingMatrix(String),
    /// A supplemental-file directory did not contain matching files.
    #[error("no GEO supplemental files found at {0}")]
    NoSupplementalFiles(String),
    /// A required supplemental file was absent.
    #[error("missing GEO supplemental resource: {0}")]
    MissingSupplemental(String),
    /// Required structured GEO metadata was absent.
    #[error("missing required GEO field `{field}` in {context}")]
    MissingField {
        /// Name of the required field.
        field: &'static str,
        /// Human-readable context describing where the field was required.
        context: String,
    },
    /// A sample characteristic could not be parsed strictly.
    #[error("invalid GEO sample characteristic `{value}` in {context}")]
    InvalidCharacteristic {
        /// Raw characteristic value.
        value: String,
        /// Human-readable context.
        context: String,
    },
    /// Platform annotation could not be joined to an expression set.
    #[error("GPL annotation join failed: {0}")]
    GPLJoinFailure(String),
    /// A requested annotation column was absent.
    #[error("missing annotation column `{column}` in {context}")]
    AnnotationMissing {
        /// Column name that was required.
        column: String,
        /// Human-readable context.
        context: String,
    },
    /// Duplicate feature identifiers were encountered where uniqueness was required.
    #[error("duplicate feature identifier `{0}`")]
    DuplicateFeature(String),
    /// A SOFT file was structurally invalid.
    #[error("invalid SOFT structure: {message}")]
    InvalidSOFTStructure {
        /// Human-readable parse context.
        message: String,
        /// Optional lower-level source error.
        #[source]
        source: Option<Box<GeoError>>,
    },
    /// A GSE series matrix file was structurally invalid.
    #[error("invalid GSE matrix: {message}")]
    InvalidGSEMatrix {
        /// Human-readable parse context.
        message: String,
        /// Optional lower-level source error.
        #[source]
        source: Option<Box<GeoError>>,
    },
    /// RNA-seq counts data was structurally invalid.
    #[error("invalid RNA-seq counts: {message}")]
    InvalidRNASeqCounts {
        /// Human-readable parse context.
        message: String,
        /// Optional lower-level source error.
        #[source]
        source: Option<Box<GeoError>>,
    },
    /// Network failure with additional GEO context.
    #[error("network failure while {context}: {source}")]
    NetworkFailure {
        /// Human-readable network context.
        context: String,
        /// Lower-level reqwest source error.
        #[source]
        source: reqwest::Error,
    },
    /// Parser failure with source context preserved.
    #[error("parse failure while {context}: {source}")]
    ParseFailure {
        /// Human-readable parse context.
        context: String,
        /// Lower-level source error.
        #[source]
        source: Box<GeoError>,
    },
    /// A GEO text or table parser failed.
    #[error("parse error: {0}")]
    Parse(String),
    /// HTTP/network failure from `reqwest`.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    /// Local filesystem I/O failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// TSV/CSV parsing failure.
    #[error("CSV/TSV error: {0}")]
    Csv(#[from] csv::Error),
    /// URL parsing failure.
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),
}

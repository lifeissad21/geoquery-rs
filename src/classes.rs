use std::collections::BTreeMap;

/// GEO metadata header map.
///
/// Keys are normalized GEO field names such as `geo_accession`, `title`, or
/// `platform`. Values remain vectors because many GEO metadata fields can occur
/// more than once.
pub type Header = BTreeMap<String, Vec<String>>;

/// Native replacement for the most commonly used MIAME/experiment metadata.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExperimentData {
    /// Contact or submitter name.
    pub name: String,
    /// Laboratory name when available.
    pub lab: String,
    /// Contact email when available.
    pub contact: String,
    /// GEO title.
    pub title: String,
    /// GEO or submitter URL.
    pub url: String,
    /// PubMed identifiers joined as a string.
    pub pubmed_ids: String,
    /// Summary or description text.
    pub abstract_text: String,
    /// Full raw GEO header map for fields not promoted above.
    pub other: Header,
}

impl ExperimentData {
    /// Build experiment metadata from a parsed GEO header.
    pub fn from_header(header: &Header) -> Self {
        let accession = first_header_value(header, "geo_accession").unwrap_or_default();
        let url = first_header_value(header, "web_link")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| {
                if accession.is_empty() {
                    String::new()
                } else {
                    format!("https://www.ncbi.nlm.nih.gov/geo/query/acc.cgi?acc={accession}")
                }
            });
        Self {
            name: first_header_value(header, "contact_name").unwrap_or_default(),
            lab: String::new(),
            contact: first_header_value(header, "contact_email").unwrap_or_default(),
            title: first_header_value(header, "title").unwrap_or_default(),
            url,
            pubmed_ids: first_header_value(header, "pubmed_id").unwrap_or_default(),
            abstract_text: first_header_value(header, "summary")
                .or_else(|| first_header_value(header, "description"))
                .unwrap_or_default(),
            other: header.clone(),
        }
    }

    /// Return the compatibility `expinfo` tuple used by older GEOquery flows.
    pub fn expinfo(&self) -> [&str; 5] {
        [&self.name, &self.lab, &self.contact, &self.title, &self.url]
    }
}

fn first_header_value(header: &Header, key: &str) -> Option<String> {
    header.get(key).and_then(|values| values.first()).cloned()
}

/// Simple string-backed tabular data frame used throughout the crate.
///
/// GEO files often mix text and numeric columns, and some downstream callers
/// need exact text preservation. Use [`DataFrame::typed_column`] when a numeric
/// view is needed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataFrame {
    /// Column names in display/storage order.
    pub columns: Vec<String>,
    /// Row values. `None` represents missing values such as `NA` or `NULL`.
    pub rows: Vec<Vec<Option<String>>>,
    /// Optional row names.
    pub row_names: Vec<String>,
    /// Per-column descriptions parsed from GEO `#COLUMN = description` lines.
    pub column_metadata: BTreeMap<String, String>,
}

impl DataFrame {
    /// Create a data frame from column names and rows.
    pub fn new(columns: Vec<String>, rows: Vec<Vec<Option<String>>>) -> Self {
        Self {
            columns,
            rows,
            row_names: Vec::new(),
            column_metadata: BTreeMap::new(),
        }
    }

    /// Create a data frame with explicit row names.
    pub fn with_row_names(
        columns: Vec<String>,
        rows: Vec<Vec<Option<String>>>,
        row_names: Vec<String>,
    ) -> Self {
        Self {
            columns,
            rows,
            row_names,
            column_metadata: BTreeMap::new(),
        }
    }

    /// Create an empty-column data frame with one row per row name.
    pub fn empty_with_row_names(row_names: Vec<String>) -> Self {
        Self {
            columns: Vec::new(),
            rows: vec![Vec::new(); row_names.len()],
            row_names,
            column_metadata: BTreeMap::new(),
        }
    }

    /// Create an empty data frame.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Number of rows.
    pub fn nrow(&self) -> usize {
        self.rows.len()
    }

    /// Number of columns.
    pub fn ncol(&self) -> usize {
        self.columns.len()
    }

    /// Return the zero-based index for a column name.
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|col| col == name)
    }

    /// Return a cell by row index and column name.
    pub fn get(&self, row: usize, column: &str) -> Option<&str> {
        let idx = self.column_index(column)?;
        self.rows.get(row)?.get(idx)?.as_deref()
    }

    /// Return a row name by zero-based row index.
    pub fn row_name(&self, row: usize) -> Option<&str> {
        self.row_names.get(row).map(String::as_str)
    }

    /// Return a typed view of a column when all non-missing values parse.
    pub fn typed_column(&self, column: &str) -> Option<TypedColumn> {
        let idx = self.column_index(column)?;
        let values = self
            .rows
            .iter()
            .map(|row| row.get(idx).and_then(|value| value.clone()))
            .collect::<Vec<_>>();
        Some(TypedColumn::from_strings(values))
    }

    /// Store a human-readable description for a column.
    pub fn set_column_metadata(
        &mut self,
        column: impl Into<String>,
        description: impl Into<String>,
    ) {
        self.column_metadata
            .insert(column.into(), description.into());
    }
}

/// Typed view over a [`DataFrame`] column.
#[derive(Clone, Debug, PartialEq)]
pub enum TypedColumn {
    /// Column values parsed as floating-point numbers.
    Float(Vec<Option<f64>>),
    /// Column values parsed as signed integers.
    Integer(Vec<Option<i64>>),
    /// Column values kept as text.
    Text(Vec<Option<String>>),
}

impl TypedColumn {
    fn from_strings(values: Vec<Option<String>>) -> Self {
        if values
            .iter()
            .flatten()
            .all(|value| value.parse::<i64>().is_ok())
        {
            return Self::Integer(
                values
                    .into_iter()
                    .map(|value| value.and_then(|v| v.parse::<i64>().ok()))
                    .collect(),
            );
        }
        if values
            .iter()
            .flatten()
            .all(|value| value.parse::<f64>().is_ok())
        {
            return Self::Float(
                values
                    .into_iter()
                    .map(|value| value.and_then(|v| v.parse::<f64>().ok()))
                    .collect(),
            );
        }
        Self::Text(values)
    }
}

/// GEO table plus the table's column descriptions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GEODataTable {
    /// Column metadata table parsed from GEO comment lines.
    pub columns: DataFrame,
    /// Main data table.
    pub table: DataFrame,
}

/// Native replacement for Bioconductor `AnnotatedDataFrame`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AnnotatedDataFrame {
    /// Observation data.
    pub data: DataFrame,
    /// Variable metadata describing the data columns.
    pub var_metadata: DataFrame,
}

impl AnnotatedDataFrame {
    /// Create an annotated data frame with empty variable metadata.
    pub fn new(data: DataFrame) -> Self {
        Self {
            data,
            var_metadata: DataFrame::empty(),
        }
    }

    /// Number of rows in the data frame.
    pub fn nrow(&self) -> usize {
        self.data.nrow()
    }

    /// Number of columns in the data frame.
    pub fn ncol(&self) -> usize {
        self.data.ncol()
    }
}

/// Numeric matrix with optional values and optional row/column names.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NumericMatrix {
    /// Row-major numeric matrix values.
    pub values: Vec<Vec<Option<f64>>>,
    /// Row names.
    pub row_names: Vec<String>,
    /// Column names.
    pub column_names: Vec<String>,
}

impl NumericMatrix {
    /// Create a numeric matrix from row-major values and names.
    pub fn new(
        values: Vec<Vec<Option<f64>>>,
        row_names: Vec<String>,
        column_names: Vec<String>,
    ) -> Self {
        Self {
            values,
            row_names,
            column_names,
        }
    }

    /// Number of rows.
    pub fn nrow(&self) -> usize {
        self.values.len()
    }

    /// Number of columns.
    pub fn ncol(&self) -> usize {
        self.values.first().map_or(0, Vec::len)
    }

    /// Return a numeric value by zero-based row and column index.
    pub fn get(&self, row: usize, column: usize) -> Option<f64> {
        self.values.get(row)?.get(column).copied().flatten()
    }

    /// Return a row name by zero-based index.
    pub fn row_name(&self, row: usize) -> Option<&str> {
        self.row_names.get(row).map(String::as_str)
    }

    /// Return a column name by zero-based index.
    pub fn column_name(&self, column: usize) -> Option<&str> {
        self.column_names.get(column).map(String::as_str)
    }
}

/// Base GEO record shape shared by accession-specific records.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GEOData {
    /// Parsed GEO metadata.
    pub header: Header,
    /// Parsed GEO table data.
    pub dataTable: GEODataTable,
}

/// GEO platform record.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GPL {
    /// Parsed platform metadata.
    pub header: Header,
    /// Parsed platform table data.
    pub dataTable: GEODataTable,
}

/// GEO sample record.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GSM {
    /// Parsed sample metadata.
    pub header: Header,
    /// Parsed sample table data.
    pub dataTable: GEODataTable,
}

/// GEO dataset record.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GDS {
    /// Parsed dataset metadata.
    pub header: Header,
    /// Optional platform annotation attached to the dataset.
    pub gpl: Option<GPL>,
    /// Parsed dataset table data.
    pub dataTable: GEODataTable,
}

/// GEO series record containing nested samples and platforms.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GSE {
    /// Parsed series metadata.
    pub header: Header,
    /// Samples keyed by GSM accession.
    pub gsms: BTreeMap<String, GSM>,
    /// Platforms keyed by GPL accession.
    pub gpls: BTreeMap<String, GPL>,
}

impl GEODataTable {
    /// Return column-description metadata.
    pub fn columns(&self) -> &DataFrame {
        &self.columns
    }

    /// Return the main data table.
    pub fn table(&self) -> &DataFrame {
        &self.table
    }
}

macro_rules! impl_geo_record_methods {
    ($type_name:ty) => {
        impl $type_name {
            /// Return parsed GEO metadata.
            pub fn meta(&self) -> &Header {
                &self.header
            }

            /// Return the combined GEO data table object.
            pub fn data_table(&self) -> &GEODataTable {
                &self.dataTable
            }

            /// Return the main parsed table.
            pub fn table(&self) -> &DataFrame {
                &self.dataTable.table
            }

            /// Return column descriptions for the main table.
            pub fn columns(&self) -> &DataFrame {
                &self.dataTable.columns
            }

            /// Return the accession from `geo_accession` metadata when present.
            pub fn accession(&self) -> Option<&str> {
                self.header
                    .get("geo_accession")
                    .and_then(|vals| vals.first())
                    .map(String::as_str)
            }
        }
    };
}

impl_geo_record_methods!(GEOData);
impl_geo_record_methods!(GPL);
impl_geo_record_methods!(GSM);
impl_geo_record_methods!(GDS);

impl GSE {
    /// Return parsed series metadata.
    pub fn meta(&self) -> &Header {
        &self.header
    }

    /// Return samples keyed by GSM accession.
    pub fn gsm_list(&self) -> &BTreeMap<String, GSM> {
        &self.gsms
    }

    /// Return platforms keyed by GPL accession.
    pub fn gpl_list(&self) -> &BTreeMap<String, GPL> {
        &self.gpls
    }
}

/// Native replacement for Bioconductor `ExpressionSet`.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpressionSet {
    /// Numeric expression matrix.
    pub exprs: NumericMatrix,
    /// Feature row names.
    pub feature_names: Vec<String>,
    /// Sample column names.
    pub sample_names: Vec<String>,
    /// R-compatible phenotype data field.
    pub phenoData: DataFrame,
    /// R-compatible feature data field.
    pub featureData: DataFrame,
    /// Idiomatic annotated phenotype data.
    pub pheno_data: AnnotatedDataFrame,
    /// Idiomatic annotated feature data.
    pub feature_data: AnnotatedDataFrame,
    /// Platform annotation accession when available.
    pub annotation: Option<String>,
    /// R-compatible raw experiment metadata.
    pub experimentData: Header,
    /// Structured experiment metadata.
    pub experiment_data: ExperimentData,
}

impl ExpressionSet {
    /// Number of features.
    pub fn nrow(&self) -> usize {
        self.exprs.nrow()
    }

    /// Number of samples.
    pub fn ncol(&self) -> usize {
        self.exprs.ncol()
    }
}

/// Native replacement for limma `MAList`.
#[derive(Clone, Debug, PartialEq)]
pub struct MAList {
    /// M matrix.
    pub M: NumericMatrix,
    /// Feature row names.
    pub row_names: Vec<String>,
    /// Sample column names.
    pub column_names: Vec<String>,
    /// Optional A matrix.
    pub A: Option<NumericMatrix>,
    /// Target/sample metadata.
    pub targets: DataFrame,
    /// Optional feature annotation.
    pub genes: Option<DataFrame>,
    /// Source GEO metadata.
    pub notes: Header,
}

impl MAList {
    /// Number of feature rows.
    pub fn nrow(&self) -> usize {
        self.M.nrow()
    }

    /// Number of sample columns.
    pub fn ncol(&self) -> usize {
        self.M.ncol()
    }
}

/// Genome build metadata for GEO RNA-seq quantification files.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GenomeInfo {
    /// Genome build identifier, for example `GRCh38`.
    pub genome_build: String,
    /// Species label inferred from the GEO file name.
    pub species: String,
    /// Source annotation or counts filename.
    pub fname: String,
}

/// Native replacement for Bioconductor `SummarizedExperiment`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SummarizedExperiment {
    /// Named assays such as `counts`.
    pub assays: BTreeMap<String, NumericMatrix>,
    /// Feature/gene metadata.
    pub rowData: DataFrame,
    /// Sample metadata.
    pub colData: DataFrame,
    /// Experiment-level metadata.
    pub metadata: Header,
    /// Parsed genome metadata when available.
    pub genome_info: Option<GenomeInfo>,
    /// Unix timestamp string for construction time when available.
    pub created_at: Option<String>,
}

impl SummarizedExperiment {
    /// Return an assay by name.
    pub fn assay(&self, name: &str) -> Option<&NumericMatrix> {
        self.assays.get(name)
    }

    /// Number of feature rows in the first assay.
    pub fn nrow(&self) -> usize {
        self.assays.values().next().map_or(0, NumericMatrix::nrow)
    }

    /// Number of sample columns in the first assay.
    pub fn ncol(&self) -> usize {
        self.assays.values().next().map_or(0, NumericMatrix::ncol)
    }
}

/// Parsed GEO object returned by high-level parsers and fetchers.
#[derive(Clone, Debug, PartialEq)]
pub enum GEOObject {
    /// GEO dataset.
    GDS(GDS),
    /// GEO platform.
    GPL(GPL),
    /// GEO sample.
    GSM(GSM),
    /// GEO series with full SOFT records.
    GSE(GSE),
    /// One or more GSE series matrix files parsed as expression sets.
    GSEMatrix(Vec<ExpressionSet>),
}

/// R-compatible metadata accessor trait.
pub trait Meta {
    /// Return parsed GEO metadata.
    fn Meta(&self) -> &Header;
}

/// R-compatible accession accessor trait.
pub trait Accession {
    /// Return the accession from `geo_accession` metadata when present.
    fn Accession(&self) -> Option<&str>;
}

/// R-compatible data table accessor trait.
pub trait dataTable {
    /// Return the combined GEO data table object.
    fn dataTable(&self) -> &GEODataTable;
}

/// R-compatible column metadata accessor trait.
pub trait Columns {
    /// Return column-description metadata.
    fn Columns(&self) -> &DataFrame;
}

/// R-compatible main table accessor trait.
pub trait Table {
    /// Return the main parsed table.
    fn Table(&self) -> &DataFrame;
}

/// R-compatible platform-list accessor trait for GSE objects.
pub trait GPLList {
    /// Return platforms keyed by GPL accession.
    fn GPLList(&self) -> &BTreeMap<String, GPL>;
}

/// R-compatible sample-list accessor trait for GSE objects.
pub trait GSMList {
    /// Return samples keyed by GSM accession.
    fn GSMList(&self) -> &BTreeMap<String, GSM>;
}

impl Meta for GEOData {
    fn Meta(&self) -> &Header {
        &self.header
    }
}

impl Meta for GPL {
    fn Meta(&self) -> &Header {
        &self.header
    }
}

impl Meta for GSM {
    fn Meta(&self) -> &Header {
        &self.header
    }
}

impl Meta for GDS {
    fn Meta(&self) -> &Header {
        &self.header
    }
}

impl Meta for GSE {
    fn Meta(&self) -> &Header {
        &self.header
    }
}

macro_rules! impl_geodata_accessors {
    ($type_name:ty) => {
        impl Accession for $type_name {
            fn Accession(&self) -> Option<&str> {
                self.header
                    .get("geo_accession")
                    .and_then(|vals| vals.first())
                    .map(String::as_str)
            }
        }

        impl dataTable for $type_name {
            fn dataTable(&self) -> &GEODataTable {
                &self.dataTable
            }
        }

        impl Columns for $type_name {
            fn Columns(&self) -> &DataFrame {
                &self.dataTable.columns
            }
        }

        impl Table for $type_name {
            fn Table(&self) -> &DataFrame {
                &self.dataTable.table
            }
        }
    };
}

impl_geodata_accessors!(GEOData);
impl_geodata_accessors!(GPL);
impl_geodata_accessors!(GSM);
impl_geodata_accessors!(GDS);

impl Columns for GEODataTable {
    fn Columns(&self) -> &DataFrame {
        &self.columns
    }
}

impl Table for GEODataTable {
    fn Table(&self) -> &DataFrame {
        &self.table
    }
}

impl GPLList for GSE {
    fn GPLList(&self) -> &BTreeMap<String, GPL> {
        &self.gpls
    }
}

impl GSMList for GSE {
    fn GSMList(&self) -> &BTreeMap<String, GSM> {
        &self.gsms
    }
}

/// Compatibility alias for a GSE matrix expression set.
pub type GEOMatrix = ExpressionSet;

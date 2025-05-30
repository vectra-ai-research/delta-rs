//! Exceptions for the deltalake crate
use chrono::{DateTime, Utc};
use object_store::Error as ObjectStoreError;

use crate::kernel::transaction::{CommitBuilderError, TransactionError};

/// A result returned by delta-rs
pub type DeltaResult<T, E = DeltaTableError> = Result<T, E>;

/// Delta Table specific error
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug)]
pub enum DeltaTableError {
    #[error("Kernel error: {0}")]
    KernelError(#[from] delta_kernel::error::Error),

    /// Error returned when reading the delta log object failed.
    #[error("Failed to read delta log object: {}", .source)]
    ObjectStore {
        /// Storage error details when reading the delta log object failed.
        #[from]
        source: ObjectStoreError,
    },

    /// Error returned when parsing checkpoint parquet.
    #[error("Failed to parse parquet: {}", .source)]
    Parquet {
        /// Parquet error details returned when reading the checkpoint failed.
        #[from]
        source: parquet::errors::ParquetError,
    },

    /// Error returned when converting the schema in Arrow format failed.
    #[error("Failed to convert into Arrow schema: {}", .source)]
    Arrow {
        /// Arrow error details returned when converting the schema in Arrow format failed
        #[from]
        source: arrow::error::ArrowError,
    },

    /// Error returned when the log record has an invalid JSON.
    #[error("Invalid JSON in log record, version={}, line=`{}`, err=`{}`", .version, .line, .json_err)]
    InvalidJsonLog {
        /// JSON error details returned when parsing the record JSON.
        json_err: serde_json::error::Error,
        /// invalid log entry content.
        line: String,
        /// corresponding table version for the log file.
        version: i64,
    },

    /// Error returned when the log contains invalid stats JSON.
    #[error("Invalid JSON in file stats: {}", .json_err)]
    InvalidStatsJson {
        /// JSON error details returned when parsing the stats JSON.
        json_err: serde_json::error::Error,
    },

    /// Error returned when the log contains invalid stats JSON.
    #[error("Invalid JSON in invariant expression, line=`{line}`, err=`{json_err}`")]
    InvalidInvariantJson {
        /// JSON error details returned when parsing the invariant expression JSON.
        json_err: serde_json::error::Error,
        /// Invariant expression.
        line: String,
    },

    /// Error returned when the DeltaTable has an invalid version.
    #[error("Invalid table version: {0}")]
    InvalidVersion(i64),

    /// Error returned when the DeltaTable has no data files.
    #[error("Corrupted table, cannot read data file {}: {}", .path, .source)]
    MissingDataFile {
        /// Source error details returned when the DeltaTable has no data files.
        source: std::io::Error,
        /// The Path used of the DeltaTable
        path: String,
    },

    /// Error returned when the datetime string is invalid for a conversion.
    #[error("Invalid datetime string: {}", .source)]
    InvalidDateTimeString {
        /// Parse error details returned of the datetime string parse error.
        #[from]
        source: chrono::ParseError,
    },

    /// Error returned when attempting to write bad data to the table
    #[error("Attempted to write invalid data to the table: {:#?}", violations)]
    InvalidData {
        /// Action error details returned of the invalid action.
        violations: Vec<String>,
    },

    /// Error returned when it is not a DeltaTable.
    #[error("Not a Delta table: {0}")]
    NotATable(String),

    /// Error returned when no metadata was found in the DeltaTable.
    #[error("No metadata found, please make sure table is loaded.")]
    NoMetadata,

    /// Error returned when no schema was found in the DeltaTable.
    #[error("No schema found, please make sure table is loaded.")]
    NoSchema,

    /// Error returned when no partition was found in the DeltaTable.
    #[error("No partitions found, please make sure table is partitioned.")]
    LoadPartitions,

    /// Error returned when writes are attempted with data that doesn't match the schema of the
    /// table
    #[error("Data does not match the schema or partitions of the table: {}", msg)]
    SchemaMismatch {
        /// Information about the mismatch
        msg: String,
    },

    /// Error returned when a partition is not formatted as a Hive Partition.
    #[error("This partition is not formatted with key=value: {}", .partition)]
    PartitionError {
        /// The malformed partition used.
        partition: String,
    },

    /// Error returned when a invalid partition filter was found.
    #[error("Invalid partition filter found: {}.", .partition_filter)]
    InvalidPartitionFilter {
        /// The invalid partition filter used.
        partition_filter: String,
    },

    /// Error returned when a partition filter uses a nonpartitioned column.
    #[error("Tried to filter partitions on non-partitioned columns: {:#?}", .nonpartitioned_columns)]
    ColumnsNotPartitioned {
        /// The columns used in the partition filter that is not partitioned
        nonpartitioned_columns: Vec<String>,
    },

    /// Error returned when a line from log record is invalid.
    #[error("Failed to read line from log record")]
    Io {
        /// Source error details returned while reading the log record.
        #[from]
        source: std::io::Error,
    },

    /// Error raised while preparing a commit
    #[error("Commit actions are unsound: {source}")]
    CommitValidation {
        /// The source error
        source: CommitBuilderError,
    },

    /// Error raised while commititng transaction
    #[error("Transaction failed: {source}")]
    Transaction {
        /// The source error
        source: TransactionError,
    },

    /// Error returned when transaction is failed to be committed because given version already exists.
    #[error("Delta transaction failed, version {0} already exists.")]
    VersionAlreadyExists(i64),

    /// Error returned when user attempts to commit actions that don't belong to the next version.
    #[error("Delta transaction failed, version {0} does not follow {1}")]
    VersionMismatch(i64, i64),

    /// A Feature is missing to perform operation
    #[error("Delta-rs must be build with feature '{feature}' to support loading from: {url}.")]
    MissingFeature {
        /// Name of the missing feature
        feature: &'static str,
        /// Storage location url
        url: String,
    },

    /// A Feature is missing to perform operation
    #[error("Cannot infer storage location from: {0}")]
    InvalidTableLocation(String),

    /// Generic Delta Table error
    #[error("Log JSON serialization error: {json_err}")]
    SerializeLogJson {
        /// JSON serialization error
        json_err: serde_json::error::Error,
    },

    /// Generic Delta Table error
    #[error("Schema JSON serialization error: {json_err}")]
    SerializeSchemaJson {
        /// JSON serialization error
        json_err: serde_json::error::Error,
    },

    /// Generic Delta Table error
    #[error("Generic DeltaTable error: {0}")]
    Generic(String),

    /// Generic Delta Table error
    #[error("Generic error: {source}")]
    GenericError {
        /// Source error
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },

    #[error("Kernel: {source}")]
    Kernel {
        #[from]
        source: crate::kernel::Error,
    },

    #[error("Table metadata is invalid: {0}")]
    MetadataError(String),

    #[error("Table has not yet been initialized")]
    NotInitialized,

    #[error("Table has not yet been initialized with files, therefore {0} is not supported")]
    NotInitializedWithFiles(String),

    #[error("Change Data not enabled for version: {version}, Start: {start}, End: {end}")]
    ChangeDataNotRecorded { version: i64, start: i64, end: i64 },

    #[error("Reading a table version: {version} that does not have change data enabled")]
    ChangeDataNotEnabled { version: i64 },

    #[error("Invalid version. Start version {start} is greater than end version {end}")]
    ChangeDataInvalidVersionRange { start: i64, end: i64 },

    #[error("End timestamp {ending_timestamp} is greater than latest commit timestamp")]
    ChangeDataTimestampGreaterThanCommit { ending_timestamp: DateTime<Utc> },

    #[error("No starting version or timestamp provided for CDC")]
    NoStartingVersionOrTimestamp,
}

impl From<object_store::path::Error> for DeltaTableError {
    fn from(err: object_store::path::Error) -> Self {
        Self::GenericError {
            source: Box::new(err),
        }
    }
}

impl From<serde_json::Error> for DeltaTableError {
    fn from(value: serde_json::Error) -> Self {
        DeltaTableError::InvalidStatsJson { json_err: value }
    }
}

impl DeltaTableError {
    /// Crate a NotATable Error with message for given path.
    pub fn not_a_table(path: impl AsRef<str>) -> Self {
        let msg = format!(
            "No snapshot or version 0 found, perhaps {} is an empty dir?",
            path.as_ref()
        );
        Self::NotATable(msg)
    }

    /// Create a [Generic](DeltaTableError::Generic) error with the given message.
    pub fn generic(msg: impl ToString) -> Self {
        Self::Generic(msg.to_string())
    }
}

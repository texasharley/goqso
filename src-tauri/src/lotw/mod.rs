pub mod client;  // LoTW API client for downloading data
pub mod sync;
pub mod tqsl;
pub mod adif;

// Re-export commonly used types
pub use client::{LotwClient, LotwQueryOptions, LotwReportResult, LotwError};

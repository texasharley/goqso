// GoQSO Library
// Re-export modules for use in main.rs

pub mod adif;      // ADIF file parsing and writing
pub mod commands;
pub mod db;
pub mod udp;
pub mod lotw;
pub mod reference;  // Authoritative DXCC/prefix data (replaces cty module)
pub mod awards;
pub mod qso_tracker; // State machine for auto-logging FT8/FT4 QSOs

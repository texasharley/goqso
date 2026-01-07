// ADIF (Amateur Data Interchange Format) Parser and Writer
// Reference: https://adif.org/

pub mod parser;
pub mod writer;
pub mod modes;
pub mod bands;

pub use parser::{parse_adif, AdifRecord, AdifFile};
pub use writer::write_adif;
pub use modes::{normalize_mode, get_mode_group, ModeGroup};
pub use bands::{freq_to_band, freq_hz_to_band, get_ft8_freq};

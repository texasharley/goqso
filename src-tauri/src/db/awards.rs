// Award progress calculations

use std::collections::{HashMap, HashSet};

/// Calculate DXCC progress from QSO data
pub fn calculate_dxcc_progress(
    qso_dxcc_entities: &[i32],
    confirmed_dxcc_entities: &[i32],
) -> (i32, i32) {
    let worked: HashSet<_> = qso_dxcc_entities.iter().collect();
    let confirmed: HashSet<_> = confirmed_dxcc_entities.iter().collect();
    
    (worked.len() as i32, confirmed.len() as i32)
}

/// Calculate WAS progress from QSO data
pub fn calculate_was_progress(
    qso_states: &[&str],
    confirmed_states: &[&str],
) -> (i32, i32) {
    let worked: HashSet<_> = qso_states.iter().collect();
    let confirmed: HashSet<_> = confirmed_states.iter().collect();
    
    (worked.len() as i32, confirmed.len() as i32)
}

/// Calculate VUCC progress (unique grid squares on VHF+)
pub fn calculate_vucc_progress(
    grids: &[&str],
    confirmed_grids: &[&str],
) -> (i32, i32) {
    // VUCC uses 4-character grid squares
    let worked: HashSet<_> = grids.iter()
        .map(|g| &g[..4.min(g.len())])
        .collect();
    let confirmed: HashSet<_> = confirmed_grids.iter()
        .map(|g| &g[..4.min(g.len())])
        .collect();
    
    (worked.len() as i32, confirmed.len() as i32)
}

/// US States list
pub const US_STATES: &[(&str, &str)] = &[
    ("AL", "Alabama"), ("AK", "Alaska"), ("AZ", "Arizona"), ("AR", "Arkansas"),
    ("CA", "California"), ("CO", "Colorado"), ("CT", "Connecticut"), ("DE", "Delaware"),
    ("FL", "Florida"), ("GA", "Georgia"), ("HI", "Hawaii"), ("ID", "Idaho"),
    ("IL", "Illinois"), ("IN", "Indiana"), ("IA", "Iowa"), ("KS", "Kansas"),
    ("KY", "Kentucky"), ("LA", "Louisiana"), ("ME", "Maine"), ("MD", "Maryland"),
    ("MA", "Massachusetts"), ("MI", "Michigan"), ("MN", "Minnesota"), ("MS", "Mississippi"),
    ("MO", "Missouri"), ("MT", "Montana"), ("NE", "Nebraska"), ("NV", "Nevada"),
    ("NH", "New Hampshire"), ("NJ", "New Jersey"), ("NM", "New Mexico"), ("NY", "New York"),
    ("NC", "North Carolina"), ("ND", "North Dakota"), ("OH", "Ohio"), ("OK", "Oklahoma"),
    ("OR", "Oregon"), ("PA", "Pennsylvania"), ("RI", "Rhode Island"), ("SC", "South Carolina"),
    ("SD", "South Dakota"), ("TN", "Tennessee"), ("TX", "Texas"), ("UT", "Utah"),
    ("VT", "Vermont"), ("VA", "Virginia"), ("WA", "Washington"), ("WV", "West Virginia"),
    ("WI", "Wisconsin"), ("WY", "Wyoming"),
];

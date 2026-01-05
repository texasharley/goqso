// LoTW API Client
// Implements GET endpoints for downloading data from LoTW
// See: https://lotw.arrl.org/lotw-help/developer-information/
//
// IMPORTANT: This module ONLY implements read operations (GET).
// Upload functionality is NOT implemented to prevent accidental submissions.

use reqwest::Client;
use std::collections::HashMap;

/// LoTW API endpoints
const LOTW_REPORT_URL: &str = "https://lotw.arrl.org/lotwuser/lotwreport.adi";
const LOTW_DXCC_CREDITS_URL: &str = "https://lotw.arrl.org/lotwuser/logbook/qslcards.php";
const LOTW_USER_ACTIVITY_URL: &str = "https://lotw.arrl.org/lotw-user-activity.csv";

/// LoTW credentials for API access
#[derive(Debug, Clone)]
pub struct LotwCredentials {
    pub username: String,
    pub password: String,
}

/// Options for querying QSO/QSL records
#[derive(Debug, Clone, Default)]
pub struct LotwQueryOptions {
    /// "yes" for QSL records (confirmations), "no" for QSO records (accepted uploads)
    pub qso_qsl: Option<String>,
    /// Return QSL records received on/after this date (YYYY-MM-DD)
    pub qso_qslsince: Option<String>,
    /// Return QSO records received on/after this date (YYYY-MM-DD)
    pub qso_qsorxsince: Option<String>,
    /// Filter by own callsign
    pub qso_owncall: Option<String>,
    /// Filter by worked callsign
    pub qso_callsign: Option<String>,
    /// Filter by mode
    pub qso_mode: Option<String>,
    /// Filter by band
    pub qso_band: Option<String>,
    /// Filter by DXCC entity number
    pub qso_dxcc: Option<i32>,
    /// Start date filter (YYYY-MM-DD)
    pub qso_startdate: Option<String>,
    /// End date filter (YYYY-MM-DD)
    pub qso_enddate: Option<String>,
    /// Include logging station location details
    pub qso_mydetail: bool,
    /// Include QSL station location details  
    pub qso_qsldetail: bool,
    /// Include own callsign in each record
    pub qso_withown: bool,
}

/// Result from LoTW report query
#[derive(Debug)]
pub struct LotwReportResult {
    /// Raw ADIF content
    pub adif_content: String,
    /// Number of records reported in header (APP_LoTW_NUMREC)
    pub num_records: Option<i32>,
    /// Last QSL timestamp from header (APP_LoTW_LASTQSL)
    pub last_qsl: Option<String>,
    /// Last QSO RX timestamp from header (APP_LoTW_LASTQSORX)
    pub last_qsorx: Option<String>,
}

/// Error types for LoTW operations
#[derive(Debug)]
pub enum LotwError {
    /// HTTP request failed
    NetworkError(String),
    /// LoTW returned an error (HTML instead of ADIF)
    ApiError(String),
    /// Failed to parse response
    ParseError(String),
    /// Missing or invalid credentials
    AuthError(String),
}

impl std::fmt::Display for LotwError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LotwError::NetworkError(e) => write!(f, "Network error: {}", e),
            LotwError::ApiError(e) => write!(f, "LoTW API error: {}", e),
            LotwError::ParseError(e) => write!(f, "Parse error: {}", e),
            LotwError::AuthError(e) => write!(f, "Authentication error: {}", e),
        }
    }
}

impl std::error::Error for LotwError {}

/// LoTW API client for downloading data
pub struct LotwClient {
    http: Client,
    credentials: LotwCredentials,
}

impl LotwClient {
    /// Create a new LoTW client with credentials
    pub fn new(username: String, password: String) -> Self {
        Self {
            http: Client::builder()
                .user_agent("GoQSO/0.1.0")
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("Failed to create HTTP client"),
            credentials: LotwCredentials { username, password },
        }
    }

    /// Download QSL confirmations from LoTW
    /// 
    /// This fetches records where QSL_RCVD="Y" - confirmed QSOs.
    /// Use `qso_qslsince` to get only new confirmations since a date.
    /// 
    /// IMPORTANT: If qso_qslsince is None, LoTW defaults to the last query date
    /// stored on their server! To get ALL confirmations, pass a very old date.
    pub async fn download_confirmations(
        &self,
        options: &LotwQueryOptions,
    ) -> Result<LotwReportResult, LotwError> {
        let mut opts = options.clone();
        opts.qso_qsl = Some("yes".to_string());
        opts.qso_qsldetail = true;  // Get location details for confirmed QSOs
        opts.qso_withown = true;    // Include our callsign
        
        // If no since date specified, use 1900-01-01 to get ALL confirmations
        // (LoTW defaults to last query date on their server if omitted!)
        if opts.qso_qslsince.is_none() {
            opts.qso_qslsince = Some("1900-01-01".to_string());
            log::info!("No since date specified, using 1900-01-01 to get all confirmations");
        }
        
        self.query_report(&opts).await
    }

    /// Download accepted (but not necessarily confirmed) QSOs from LoTW
    /// 
    /// This fetches records that LoTW has accepted from our uploads.
    /// Use `qso_qsorxsince` to get only new uploads since a date.
    pub async fn download_accepted_qsos(
        &self,
        options: &LotwQueryOptions,
    ) -> Result<LotwReportResult, LotwError> {
        let mut opts = options.clone();
        opts.qso_qsl = Some("no".to_string());
        opts.qso_withown = true;
        
        self.query_report(&opts).await
    }

    /// Query the LoTW report endpoint
    async fn query_report(
        &self,
        options: &LotwQueryOptions,
    ) -> Result<LotwReportResult, LotwError> {
        // Build query parameters
        let mut params: HashMap<&str, String> = HashMap::new();
        params.insert("login", self.credentials.username.clone());
        params.insert("password", self.credentials.password.clone());
        params.insert("qso_query", "1".to_string());
        
        if let Some(ref v) = options.qso_qsl {
            params.insert("qso_qsl", v.clone());
        }
        if let Some(ref v) = options.qso_qslsince {
            params.insert("qso_qslsince", v.clone());
        }
        if let Some(ref v) = options.qso_qsorxsince {
            params.insert("qso_qsorxsince", v.clone());
        }
        if let Some(ref v) = options.qso_owncall {
            params.insert("qso_owncall", v.clone());
        }
        if let Some(ref v) = options.qso_callsign {
            params.insert("qso_callsign", v.clone());
        }
        if let Some(ref v) = options.qso_mode {
            params.insert("qso_mode", v.clone());
        }
        if let Some(ref v) = options.qso_band {
            params.insert("qso_band", v.clone());
        }
        if let Some(d) = options.qso_dxcc {
            params.insert("qso_dxcc", d.to_string());
        }
        if let Some(ref v) = options.qso_startdate {
            params.insert("qso_startdate", v.clone());
        }
        if let Some(ref v) = options.qso_enddate {
            params.insert("qso_enddate", v.clone());
        }
        if options.qso_mydetail {
            params.insert("qso_mydetail", "yes".to_string());
        }
        if options.qso_qsldetail {
            params.insert("qso_qsldetail", "yes".to_string());
        }
        if options.qso_withown {
            params.insert("qso_withown", "yes".to_string());
        }

        log::info!("Querying LoTW report with {} parameters: qso_qsl={:?}, qso_qslsince={:?}", 
            params.len(),
            options.qso_qsl,
            options.qso_qslsince
        );
        
        let response = self.http
            .get(LOTW_REPORT_URL)
            .query(&params)
            .send()
            .await
            .map_err(|e| LotwError::NetworkError(e.to_string()))?;

        let status = response.status();
        log::info!("LoTW response status: {}", status);
        
        let body = response.text().await
            .map_err(|e| LotwError::NetworkError(e.to_string()))?;
        
        log::info!("LoTW response size: {} bytes", body.len());

        // Check for HTML error response (no <EOH> means error)
        if !body.contains("<EOH>") && !body.contains("<eoh>") {
            // Extract error message from HTML if possible
            if body.contains("password") || body.contains("login") {
                return Err(LotwError::AuthError("Invalid username or password".to_string()));
            }
            return Err(LotwError::ApiError(format!(
                "LoTW returned error (HTTP {}): {}",
                status,
                truncate_string(&body, 500)
            )));
        }

        // Parse header values
        let num_records = extract_header_value(&body, "APP_LoTW_NUMREC")
            .and_then(|s| s.parse().ok());
        let last_qsl = extract_header_value(&body, "APP_LoTW_LASTQSL");
        let last_qsorx = extract_header_value(&body, "APP_LoTW_LASTQSORX");

        log::info!(
            "LoTW report: {} records, last_qsl={:?}, last_qsorx={:?}",
            num_records.unwrap_or(0),
            last_qsl,
            last_qsorx
        );

        Ok(LotwReportResult {
            adif_content: body,
            num_records,
            last_qsl,
            last_qsorx,
        })
    }

    /// Download DXCC credits from LoTW
    /// 
    /// Returns ADIF with all DXCC credits (LoTW + QSL card confirmations).
    /// Requires a linked DXCC record.
    pub async fn download_dxcc_credits(
        &self,
        entity: Option<i32>,
    ) -> Result<String, LotwError> {
        let mut params: HashMap<&str, String> = HashMap::new();
        params.insert("login", self.credentials.username.clone());
        params.insert("password", self.credentials.password.clone());
        
        if let Some(e) = entity {
            params.insert("entity", e.to_string());
        }

        log::info!("Querying LoTW DXCC credits");
        
        let response = self.http
            .get(LOTW_DXCC_CREDITS_URL)
            .query(&params)
            .send()
            .await
            .map_err(|e| LotwError::NetworkError(e.to_string()))?;

        let body = response.text().await
            .map_err(|e| LotwError::NetworkError(e.to_string()))?;

        // Check for HTML error response
        if !body.contains("<EOH>") && !body.contains("<eoh>") {
            if body.contains("password") || body.contains("login") {
                return Err(LotwError::AuthError("Invalid username or password".to_string()));
            }
            return Err(LotwError::ApiError(truncate_string(&body, 500)));
        }

        Ok(body)
    }

    /// Download the LoTW user activity list
    /// 
    /// Returns CSV of all LoTW users with their last upload date.
    /// This is a public endpoint (no auth required).
    pub async fn download_user_activity() -> Result<String, LotwError> {
        let client = Client::builder()
            .user_agent("GoQSO/0.1.0")
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| LotwError::NetworkError(e.to_string()))?;

        log::info!("Downloading LoTW user activity list");
        
        let response = client
            .get(LOTW_USER_ACTIVITY_URL)
            .send()
            .await
            .map_err(|e| LotwError::NetworkError(e.to_string()))?;

        let body = response.text().await
            .map_err(|e| LotwError::NetworkError(e.to_string()))?;

        Ok(body)
    }

    /// Check if a callsign is a LoTW user
    /// 
    /// Searches the cached user activity list for the callsign.
    pub fn is_lotw_user(user_activity: &str, callsign: &str) -> Option<String> {
        let call_upper = callsign.to_uppercase();
        for line in user_activity.lines() {
            if line.starts_with(&call_upper) {
                // Format: "CALL,YYYY-MM-DD,HH:MM:SS"
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 && parts[0] == call_upper {
                    return Some(parts[1].to_string());
                }
            }
        }
        None
    }
}

/// Extract a header field value from ADIF content
fn extract_header_value(adif: &str, field_name: &str) -> Option<String> {
    let upper_field = field_name.to_uppercase();
    let lower = adif.to_lowercase();
    
    // Find the field in the header (before <EOH>)
    let eoh_pos = lower.find("<eoh>").unwrap_or(adif.len());
    let header = &adif[..eoh_pos];
    
    // Look for <FIELD:length>value pattern
    let field_pattern = format!("<{}:", field_name);
    let field_pattern_lower = field_pattern.to_lowercase();
    
    if let Some(start) = header.to_lowercase().find(&field_pattern_lower) {
        let rest = &header[start..];
        // Find the colon and parse length
        if let Some(colon_pos) = rest.find(':') {
            let after_colon = &rest[colon_pos + 1..];
            if let Some(gt_pos) = after_colon.find('>') {
                let len_str = &after_colon[..gt_pos];
                if let Ok(len) = len_str.parse::<usize>() {
                    let value_start = colon_pos + 1 + gt_pos + 1;
                    if rest.len() >= value_start + len {
                        return Some(rest[value_start..value_start + len].to_string());
                    }
                }
            }
        }
    }
    
    None
}

/// Truncate a string for error messages
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_header_value() {
        let adif = r#"Generated by LoTW
<PROGRAMID:4>LoTW
<APP_LoTW_LASTQSL:19>2026-01-04 12:30:45
<APP_LoTW_NUMREC:2>53
<EOH>
"#;
        assert_eq!(
            extract_header_value(adif, "APP_LoTW_LASTQSL"),
            Some("2026-01-04 12:30:45".to_string())
        );
        assert_eq!(
            extract_header_value(adif, "APP_LoTW_NUMREC"),
            Some("53".to_string())
        );
        assert_eq!(
            extract_header_value(adif, "PROGRAMID"),
            Some("LoTW".to_string())
        );
        assert_eq!(
            extract_header_value(adif, "NONEXISTENT"),
            None
        );
    }

    #[test]
    fn test_is_lotw_user() {
        let csv = "W1AW,2026-01-01,10:00:00\nK5ABC,2025-12-15,08:30:00\nJA1ABC,2025-11-20,14:45:00";
        
        assert_eq!(
            LotwClient::is_lotw_user(csv, "W1AW"),
            Some("2026-01-01".to_string())
        );
        assert_eq!(
            LotwClient::is_lotw_user(csv, "k5abc"),
            Some("2025-12-15".to_string())
        );
        assert_eq!(
            LotwClient::is_lotw_user(csv, "XXXX"),
            None
        );
    }
}

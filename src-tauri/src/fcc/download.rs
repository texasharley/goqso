// FCC Database Download Module
//
// Downloads the FCC ULS amateur license database (l_amat.zip)
// and extracts it to a temporary location for parsing.

use std::path::PathBuf;
use std::io::{Read, Cursor};
use reqwest::Client;
use zip::ZipArchive;
use tokio::fs;

/// FCC Amateur License Database URL
const FCC_AMAT_URL: &str = "https://data.fcc.gov/download/pub/uls/complete/l_amat.zip";

/// Download the FCC amateur license database
/// 
/// Returns the path to the extracted EN.dat file (contains license entity data)
pub async fn download_fcc_database(data_dir: &PathBuf) -> Result<PathBuf, String> {
    log::info!("Starting FCC database download from {}", FCC_AMAT_URL);
    
    // Create FCC cache directory
    let fcc_dir = data_dir.join("fcc_cache");
    fs::create_dir_all(&fcc_dir).await
        .map_err(|e| format!("Failed to create FCC cache directory: {}", e))?;
    
    // Download the zip file
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout for large file
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    log::info!("Downloading FCC database (~25MB)...");
    let response = client.get(FCC_AMAT_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to download FCC database: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("FCC download failed with status: {}", response.status()));
    }
    
    let bytes = response.bytes()
        .await
        .map_err(|e| format!("Failed to read FCC database response: {}", e))?;
    
    log::info!("Downloaded {} bytes, extracting...", bytes.len());
    
    // Extract synchronously using spawn_blocking since zip types aren't Send
    let fcc_dir_clone = fcc_dir.clone();
    let bytes_vec = bytes.to_vec();
    
    let extraction_result = tokio::task::spawn_blocking(move || {
        extract_fcc_files(&bytes_vec, &fcc_dir_clone)
    })
    .await
    .map_err(|e| format!("Failed to spawn blocking task: {}", e))?;
    
    extraction_result?;
    
    let en_path = fcc_dir.join("EN.dat");
    Ok(en_path)
}

/// Synchronous extraction of FCC files from zip archive
fn extract_fcc_files(bytes: &[u8], fcc_dir: &PathBuf) -> Result<(), String> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to open FCC zip archive: {}", e))?;
    
    // Extract EN.dat (entity data with callsign, name, address, state)
    let en_path = fcc_dir.join("EN.dat");
    
    // Find and extract EN.dat
    let mut found_en = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;
        
        let name = file.name().to_uppercase();
        if name == "EN.DAT" {
            log::info!("Extracting EN.dat ({} bytes compressed)", file.compressed_size());
            
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| format!("Failed to read EN.dat: {}", e))?;
            
            std::fs::write(&en_path, &contents)
                .map_err(|e| format!("Failed to write EN.dat: {}", e))?;
            
            found_en = true;
            log::info!("Extracted EN.dat: {} bytes", contents.len());
        }
        
        // Also extract HD.dat for license class info
        if name == "HD.DAT" {
            log::info!("Extracting HD.dat ({} bytes compressed)", file.compressed_size());
            
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| format!("Failed to read HD.dat: {}", e))?;
            
            let hd_path = fcc_dir.join("HD.dat");
            std::fs::write(&hd_path, &contents)
                .map_err(|e| format!("Failed to write HD.dat: {}", e))?;
            
            log::info!("Extracted HD.dat: {} bytes", contents.len());
        }
    }
    
    if !found_en {
        return Err("EN.dat not found in FCC archive".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_url_format() {
        assert!(super::FCC_AMAT_URL.starts_with("https://"));
        assert!(super::FCC_AMAT_URL.ends_with(".zip"));
    }
}

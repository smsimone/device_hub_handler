use axum::body::Bytes;

use axum::extract::multipart::Field;
use tracing::error;

pub struct ExtractedFile {
    pub name: String,
    pub bytes: Bytes,
}

// Extract a file from a Multipart field
pub async fn extract_file(field: Field<'_>) -> Option<ExtractedFile> {
    let filename = match field.file_name() {
        Some(name) => name.to_string(),
        None => {
            error!("Failed to extract name from multipart");
            return None;
        }
    };
    let bytes = match field.bytes().await {
        Ok(data) => data,
        Err(err) => {
            error!("Failed to get bytes from multipart: {}", err.to_string());
            return None;
        }
    };
    Some(ExtractedFile {
        name: filename,
        bytes,
    })
}

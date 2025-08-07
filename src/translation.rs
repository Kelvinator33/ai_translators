use crate::{TranslationRequest, TranslationResponse, database::Database};
use crate::ai_backends::AIBackendClient;

pub async fn translate(
    request: &TranslationRequest,
    db: &Database,
) -> Result<TranslationResponse, Box<dyn std::error::Error>> {
    let backend = db.get_backend(&request.backend_id).await?
        .ok_or("Backend not found")?;
    
    if !backend.enabled {
        return Err("Backend is disabled".into());
    }

    let client = AIBackendClient::new();
    client.translate_with_backend(&backend, request).await
}
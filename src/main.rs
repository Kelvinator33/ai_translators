use actix_web::{web, App, HttpServer, Result, HttpResponse, HttpRequest, middleware::Logger};
use actix_multipart::Multipart;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod ai_backends;
mod database;
mod translation;
mod ocr;
mod auth;
mod payments;
mod image_export;

use database::Database;

#[derive(Debug, Serialize, Deserialize)]
struct AIBackend {
    id: String,
    name: String,
    backend_type: String,
    config: HashMap<String, String>,
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct TranslationRequest {
    text: String,
    source_lang: Option<String>,
    target_lang: String,
    backend_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TranslationResponse {
    translated_text: String,
    source_lang: String,
    target_lang: String,
    backend_used: String,
}

async fn get_backends(db: web::Data<Database>) -> Result<HttpResponse> {
    match db.get_all_backends().await {
        Ok(backends) => Ok(HttpResponse::Ok().json(backends)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get backends")),
    }
}

async fn save_backend(
    db: web::Data<Database>,
    backend: web::Json<AIBackend>,
) -> Result<HttpResponse> {
    match db.save_backend(&backend.into_inner()).await {
        Ok(_) => Ok(HttpResponse::Ok().json("Backend saved")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to save backend")),
    }
}

async fn translate_text(
    db: web::Data<Database>,
    req: web::Json<TranslationRequest>,
) -> Result<HttpResponse> {
    let request = req.into_inner();
    
    match translation::translate(&request, &db).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(format!("Translation failed: {}", err))),
    }
}

async fn upload_and_translate(
    db: web::Data<Database>,
    req: HttpRequest,
    payload: Multipart,
) -> Result<HttpResponse> {
    // Check authentication and usage limits
    if let Some(token) = auth::extract_token_from_header(&req) {
        if let Ok(claims) = auth::verify_token(&token) {
            if let Ok(user) = db.get_user_by_id(&claims.sub).await {
                if !user.can_translate() {
                    return Ok(HttpResponse::PaymentRequired().json("Translation limit exceeded. Please upgrade your plan."));
                }
                
                // Increment usage counter
                let _ = db.increment_user_translations(&claims.sub).await;
            }
        }
    }

    match ocr::process_multipart_upload(payload, &db).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(format!("Upload failed: {}", err))),
    }
}

async fn export_translated_image(
    db: web::Data<Database>,
    req: HttpRequest,
    mut payload: Multipart,
) -> Result<HttpResponse> {
    use futures_util::TryStreamExt;
    use crate::image_export::ImageExporter;

    // For demo purposes, skip authentication for now
    // In production, add proper authentication here
    
    let mut original_image: Option<Vec<u8>> = None;
    let mut translated_text = String::new();
    let mut export_type = "overlay".to_string();

    // Parse multipart form data
    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();
        let name = content_disposition.get_name().unwrap_or("");

        match name {
            "original_image" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.try_next().await? {
                    data.extend_from_slice(&chunk);
                }
                original_image = Some(data);
            }
            "translated_text" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.try_next().await? {
                    data.extend_from_slice(&chunk);
                }
                translated_text = String::from_utf8(data).unwrap_or_default();
            }
            "export_type" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.try_next().await? {
                    data.extend_from_slice(&chunk);
                }
                export_type = String::from_utf8(data).unwrap_or("overlay".to_string());
            }
            _ => {}
        }
    }

    let image_data = original_image.ok_or_else(|| {
        actix_web::error::ErrorBadRequest("No image provided")
    })?;
    
    if translated_text.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json("No translated text provided"));
    }

    // Create the exported image
    let exporter = ImageExporter::new();
    
    let result_image = match export_type.as_str() {
        "sidebyside" => exporter.create_side_by_side_comparison(
            &image_data,
            "Original text", // In a real implementation, this would be the extracted text
            &translated_text
        ),
        _ => exporter.create_translated_image(
            &image_data,
            "Original text", // In a real implementation, this would be the extracted text
            &translated_text
        )
    };

    match result_image {
        Ok(exported_image) => {
            Ok(HttpResponse::Ok()
                .content_type("image/png")
                .append_header(("Content-Disposition", "attachment; filename=\"translated_image.png\""))
                .body(exported_image))
        }
        Err(e) => {
            eprintln!("Image export failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(format!("Export failed: {}", e)))
        }
    }
}

async fn update_subscription(
    db: web::Data<Database>,
    req: HttpRequest,
    subscription_data: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    // Check authentication
    let token = match auth::extract_token_from_header(&req) {
        Some(t) => t,
        None => return Ok(HttpResponse::Unauthorized().json("Authentication required")),
    };

    let claims = match auth::verify_token(&token) {
        Ok(c) => c,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("Invalid token")),
    };

    let data = subscription_data.into_inner();
    let new_plan = data["plan"].as_str().unwrap_or("free");

    // Update user subscription in database
    match db.update_user_subscription(&claims.sub, new_plan).await {
        Ok(_) => {
            // Get updated user data
            match db.get_user_by_id(&claims.sub).await {
                Ok(user) => Ok(HttpResponse::Ok().json(user.to_profile())),
                Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get updated user")),
            }
        },
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to update subscription")),
    }
}

async fn serve_static() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../static/index.html")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();
    env_logger::init();
    
    let db = Database::new("translation_service.db").expect("Failed to initialize database");
    
    // Initialize default backends
    db.init_default_backends().await.expect("Failed to initialize default backends");
    
    println!("Starting Translation Service on http://localhost:3000");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .wrap(Logger::default())
            .route("/", web::get().to(serve_static))
            // AI Backend routes
            .route("/api/backends", web::get().to(get_backends))
            .route("/api/backends", web::post().to(save_backend))
            .route("/api/translate", web::post().to(translate_text))
            .route("/api/upload", web::post().to(upload_and_translate))
            .route("/api/export", web::post().to(export_translated_image))
            // Authentication routes
            .route("/api/auth/register", web::post().to(auth::register))
            .route("/api/auth/login", web::post().to(auth::login))
            .route("/api/auth/profile", web::get().to(auth::profile))
            // Payment routes
            .route("/api/plans", web::get().to(payments::get_subscription_plans))
            .route("/api/payment/intent", web::post().to(payments::create_payment_intent))
            .route("/api/payment/webhook", web::post().to(payments::handle_webhook))
            .route("/api/subscription/cancel", web::post().to(payments::cancel_subscription))
            .route("/api/subscription/update", web::post().to(update_subscription))
            .service(
                actix_web::web::resource("/static/{filename:.*}")
                    .route(web::get().to(|path: web::Path<String>| async move {
                        let filename = path.into_inner();
                        match filename.as_str() {
                            "app.js" => Ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok()
                                .content_type("application/javascript")
                                .body(include_str!("../static/app.js"))),
                            "style.css" => Ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok()
                                .content_type("text/css")
                                .body(include_str!("../static/style.css"))),
                            "js/state.js" => Ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok()
                                .content_type("application/javascript")
                                .body(include_str!("../static/js/state.js"))),
                            "js/utils.js" => Ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok()
                                .content_type("application/javascript")
                                .body(include_str!("../static/js/utils.js"))),
                            "js/api.js" => Ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok()
                                .content_type("application/javascript")
                                .body(include_str!("../static/js/api.js"))),
                            "js/components.js" => Ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok()
                                .content_type("application/javascript")
                                .body(include_str!("../static/js/components.js"))),
                            "js/translation.js" => Ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok()
                                .content_type("application/javascript")
                                .body(include_str!("../static/js/translation.js"))),
                            "js/dashboard.js" => Ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok()
                                .content_type("application/javascript")
                                .body(include_str!("../static/js/dashboard.js"))),
                            "js/auth.js" => Ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok()
                                .content_type("application/javascript")
                                .body(include_str!("../static/js/auth.js"))),
                            "js/payments.js" => Ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok()
                                .content_type("application/javascript")
                                .body(include_str!("../static/js/payments.js"))),
                            _ => Ok::<HttpResponse, actix_web::Error>(HttpResponse::NotFound().finish()),
                        }
                    }))
            )
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}

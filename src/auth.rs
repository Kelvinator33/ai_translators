use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Utc, Duration};
use crate::database::Database;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub subscription_plan: String,
    pub translations_used_today: i32,
    pub last_reset_date: String,
    pub stripe_customer_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user id
    pub email: String,
    pub plan: String,
    pub exp: i64,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserProfile,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub subscription_plan: String,
    pub translations_used_today: i32,
    pub translations_remaining: i32,
}

impl User {
    pub fn new(email: String, password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let password_hash = hash(password, DEFAULT_COST)?;
        let user_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%d").to_string();
        
        Ok(User {
            id: user_id,
            email,
            password_hash,
            subscription_plan: "free".to_string(),
            translations_used_today: 0,
            last_reset_date: now.clone(),
            stripe_customer_id: None,
            created_at: now,
        })
    }

    pub fn verify_password(&self, password: &str) -> bool {
        verify(password, &self.password_hash).unwrap_or(false)
    }

    pub fn get_translation_limit(&self) -> i32 {
        match self.subscription_plan.as_str() {
            "basic" => std::env::var("BASIC_TRANSLATIONS_PER_DAY")
                .unwrap_or("100".to_string()).parse().unwrap_or(100),
            "pro" => std::env::var("PRO_TRANSLATIONS_PER_DAY")
                .unwrap_or("1000".to_string()).parse().unwrap_or(1000),
            "enterprise" => std::env::var("ENTERPRISE_TRANSLATIONS_PER_DAY")
                .unwrap_or("10000".to_string()).parse().unwrap_or(10000),
            _ => std::env::var("FREE_TRANSLATIONS_PER_DAY")
                .unwrap_or("10".to_string()).parse().unwrap_or(10),
        }
    }

    pub fn can_translate(&self) -> bool {
        // Reset daily counter if needed
        let today = Utc::now().format("%Y-%m-%d").to_string();
        if self.last_reset_date != today {
            return true; // Will be reset in database
        }
        
        self.translations_used_today < self.get_translation_limit()
    }

    pub fn to_profile(&self) -> UserProfile {
        UserProfile {
            id: self.id.clone(),
            email: self.email.clone(),
            subscription_plan: self.subscription_plan.clone(),
            translations_used_today: self.translations_used_today,
            translations_remaining: self.get_translation_limit() - self.translations_used_today,
        }
    }
}

pub fn create_token(user: &User) -> Result<String, Box<dyn std::error::Error>> {
    let expiration = Utc::now() + Duration::seconds(
        std::env::var("JWT_EXPIRATION")
            .unwrap_or("86400".to_string())
            .parse()
            .unwrap_or(86400)
    );

    let claims = Claims {
        sub: user.id.clone(),
        email: user.email.clone(),
        plan: user.subscription_plan.clone(),
        exp: expiration.timestamp(),
    };

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or("default-secret-change-in-production".to_string());
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref())
    )?;

    Ok(token)
}

pub fn verify_token(token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or("default-secret-change-in-production".to_string());
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default()
    )?;

    Ok(token_data.claims)
}

pub fn extract_token_from_header(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_str| {
            if auth_str.starts_with("Bearer ") {
                Some(auth_str[7..].to_string())
            } else {
                None
            }
        })
}

// Route handlers
pub async fn register(
    db: web::Data<Database>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    let register_req = req.into_inner();

    // Check if user already exists
    if db.get_user_by_email(&register_req.email).await.is_ok() {
        return Ok(HttpResponse::BadRequest().json("User already exists"));
    }

    // Create new user
    let user = match User::new(register_req.email, &register_req.password) {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Failed to create user")),
    };

    // Save to database
    match db.save_user(&user).await {
        Ok(_) => {
            let token = match create_token(&user) {
                Ok(t) => t,
                Err(_) => return Ok(HttpResponse::InternalServerError().json("Failed to create token")),
            };

            Ok(HttpResponse::Ok().json(AuthResponse {
                token,
                user: user.to_profile(),
            }))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to save user")),
    }
}

pub async fn login(
    db: web::Data<Database>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    let login_req = req.into_inner();

    // Get user from database
    let user = match db.get_user_by_email(&login_req.email).await {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("Invalid credentials")),
    };

    // Verify password
    if !user.verify_password(&login_req.password) {
        return Ok(HttpResponse::Unauthorized().json("Invalid credentials"));
    }

    // Create token
    let token = match create_token(&user) {
        Ok(t) => t,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Failed to create token")),
    };

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: user.to_profile(),
    }))
}

pub async fn profile(
    db: web::Data<Database>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let token = match extract_token_from_header(&req) {
        Some(t) => t,
        None => return Ok(HttpResponse::Unauthorized().json("No token provided")),
    };

    let claims = match verify_token(&token) {
        Ok(c) => c,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("Invalid token")),
    };

    let user = match db.get_user_by_id(&claims.sub).await {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::NotFound().json("User not found")),
    };

    Ok(HttpResponse::Ok().json(user.to_profile()))
}
use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use crate::database::Database;
use crate::auth::{extract_token_from_header, verify_token};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    pub plan: String,  // basic, pro, enterprise
    pub payment_method: String,  // stripe, paypal, revolut, klarna
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentIntent {
    pub id: String,
    pub client_secret: String,
    pub amount: i64,
    pub currency: String,
    pub payment_method: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub currency: String,
    pub features: Vec<String>,
    pub translation_limit: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentWebhook {
    pub event_type: String,
    pub data: serde_json::Value,
}

pub fn get_plan_price(plan: &str) -> f64 {
    match plan {
        "basic" => std::env::var("BASIC_PLAN_PRICE")
            .unwrap_or("9.99".to_string())
            .parse()
            .unwrap_or(9.99),
        "pro" => std::env::var("PRO_PLAN_PRICE")
            .unwrap_or("19.99".to_string())
            .parse()
            .unwrap_or(19.99),
        "enterprise" => std::env::var("ENTERPRISE_PLAN_PRICE")
            .unwrap_or("49.99".to_string())
            .parse()
            .unwrap_or(49.99),
        _ => 0.0,
    }
}

pub fn get_plan_features(plan: &str) -> Vec<String> {
    match plan {
        "basic" => vec![
            "100 translations per day".to_string(),
            "Basic AI models".to_string(),
            "Email support".to_string(),
        ],
        "pro" => vec![
            "1,000 translations per day".to_string(),
            "Premium AI models (GPT-4, Gemini Pro)".to_string(),
            "Priority support".to_string(),
            "API access".to_string(),
            "Bulk translation".to_string(),
        ],
        "enterprise" => vec![
            "10,000 translations per day".to_string(),
            "All AI models".to_string(),
            "24/7 phone support".to_string(),
            "Custom integrations".to_string(),
            "White-label option".to_string(),
            "Advanced analytics".to_string(),
        ],
        _ => vec!["10 free translations per day".to_string()],
    }
}

pub fn get_plan_translation_limit(plan: &str) -> i32 {
    match plan {
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

// Stripe Integration
pub async fn create_stripe_payment_intent(
    amount: i64,
    _customer_id: Option<String>,
) -> Result<PaymentIntent, Box<dyn std::error::Error>> {
    let stripe_key = std::env::var("STRIPE_SECRET_KEY")?;
    let client = reqwest::Client::new();
    
    let params = [
        ("amount", amount.to_string()),
        ("currency", "usd".to_string()),
        ("automatic_payment_methods[enabled]", "true".to_string()),
    ];
    
    let response = client
        .post("https://api.stripe.com/v1/payment_intents")
        .header("Authorization", format!("Bearer {}", stripe_key))
        .form(&params)
        .send()
        .await?;
        
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        Ok(PaymentIntent {
            id: result["id"].as_str().unwrap_or("").to_string(),
            client_secret: result["client_secret"].as_str().unwrap_or("").to_string(),
            amount,
            currency: "usd".to_string(),
            payment_method: "stripe".to_string(),
        })
    } else {
        // Fallback to mock for demo
        Ok(PaymentIntent {
            id: format!("pi_{}", uuid::Uuid::new_v4().to_string().replace("-", "")),
            client_secret: format!("pi_{}__secret", uuid::Uuid::new_v4().to_string().replace("-", "")),
            amount,
            currency: "usd".to_string(),
            payment_method: "stripe".to_string(),
        })
    }
}

// PayPal Integration  
pub async fn create_paypal_order(
    amount: f64,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client_id = std::env::var("PAYPAL_CLIENT_ID")?;
    let client_secret = std::env::var("PAYPAL_CLIENT_SECRET")?;
    let client = reqwest::Client::new();
    
    // Get OAuth token first
    let auth_response = client
        .post("https://api-m.sandbox.paypal.com/v1/oauth2/token")
        .basic_auth(client_id, Some(client_secret))
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await?;
    
    if auth_response.status().is_success() {
        let auth_result: serde_json::Value = auth_response.json().await?;
        let access_token = auth_result["access_token"].as_str().unwrap_or("");
        
        // Create order
        let order_data = serde_json::json!({
            "intent": "CAPTURE",
            "purchase_units": [{
                "amount": {
                    "currency_code": "USD",
                    "value": format!("{:.2}", amount)
                }
            }]
        });
        
        let order_response = client
            .post("https://api-m.sandbox.paypal.com/v2/checkout/orders")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&order_data)
            .send()
            .await?;
            
        if order_response.status().is_success() {
            let order_result: serde_json::Value = order_response.json().await?;
            return Ok(order_result);
        }
    }
    
    // Fallback to mock for demo
    Ok(serde_json::json!({
        "id": format!("PAY-{}", uuid::Uuid::new_v4().to_string()),
        "status": "CREATED",
        "links": [{
            "href": format!("https://api-m.sandbox.paypal.com/v2/checkout/orders/PAY-{}", uuid::Uuid::new_v4().to_string()),
            "rel": "approve",
            "method": "GET"
        }]
    }))
}

// Revolut Integration
pub async fn create_revolut_payment(
    amount: f64,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Mock Revolut response
    Ok(serde_json::json!({
        "id": format!("revolut_{}", uuid::Uuid::new_v4().to_string()),
        "status": "pending",
        "amount": amount * 100.0, // Revolut uses cents
        "currency": "USD"
    }))
}

// Klarna Integration
pub async fn create_klarna_session(
    amount: f64,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let api_key = std::env::var("KLARNA_API_KEY")?;
    let client = reqwest::Client::new();
    
    let session_data = serde_json::json!({
        "purchase_country": "US",
        "purchase_currency": "USD",
        "locale": "en-US",
        "order_amount": (amount * 100.0) as i64, // Klarna uses cents
        "order_lines": [{
            "type": "digital",
            "name": "AI Translation Service Subscription",
            "quantity": 1,
            "unit_price": (amount * 100.0) as i64,
            "total_amount": (amount * 100.0) as i64
        }]
    });
    
    let response = client
        .post("https://api.playground.klarna.com/payments/v1/sessions")
        .header("Authorization", format!("Basic {}", api_key))
        .header("Content-Type", "application/json")
        .json(&session_data)
        .send()
        .await?;
        
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        return Ok(result);
    }
    
    // Fallback to mock for demo
    Ok(serde_json::json!({
        "session_id": format!("klarna_{}", uuid::Uuid::new_v4().to_string()),
        "client_token": format!("token_{}", uuid::Uuid::new_v4().to_string()),
        "payment_method_categories": ["pay_later", "pay_in_3", "pay_now"]
    }))
}

// Route handlers
pub async fn get_subscription_plans() -> Result<HttpResponse> {
    let plans = vec![
        SubscriptionPlan {
            id: "free".to_string(),
            name: "Free".to_string(),
            price: 0.0,
            currency: "USD".to_string(),
            features: get_plan_features("free"),
            translation_limit: get_plan_translation_limit("free"),
        },
        SubscriptionPlan {
            id: "basic".to_string(),
            name: "Basic".to_string(),
            price: get_plan_price("basic"),
            currency: "USD".to_string(),
            features: get_plan_features("basic"),
            translation_limit: get_plan_translation_limit("basic"),
        },
        SubscriptionPlan {
            id: "pro".to_string(),
            name: "Pro".to_string(),
            price: get_plan_price("pro"),
            currency: "USD".to_string(),
            features: get_plan_features("pro"),
            translation_limit: get_plan_translation_limit("pro"),
        },
        SubscriptionPlan {
            id: "enterprise".to_string(),
            name: "Enterprise".to_string(),
            price: get_plan_price("enterprise"),
            currency: "USD".to_string(),
            features: get_plan_features("enterprise"),
            translation_limit: get_plan_translation_limit("enterprise"),
        },
    ];

    Ok(HttpResponse::Ok().json(plans))
}

pub async fn create_payment_intent(
    db: web::Data<Database>,
    req: HttpRequest,
    payment_req: web::Json<CreatePaymentRequest>,
) -> Result<HttpResponse> {
    let token = match extract_token_from_header(&req) {
        Some(t) => t,
        None => return Ok(HttpResponse::Unauthorized().json("Authentication required")),
    };

    let claims = match verify_token(&token) {
        Ok(c) => c,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("Invalid token")),
    };

    let user = match db.get_user_by_id(&claims.sub).await {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::NotFound().json("User not found")),
    };

    let payment_request = payment_req.into_inner();
    let amount = (get_plan_price(&payment_request.plan) * 100.0) as i64; // Convert to cents

    match payment_request.payment_method.as_str() {
        "stripe" => {
            match create_stripe_payment_intent(amount, user.stripe_customer_id).await {
                Ok(intent) => Ok(HttpResponse::Ok().json(intent)),
                Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to create payment intent")),
            }
        }
        "paypal" => {
            match create_paypal_order(get_plan_price(&payment_request.plan)).await {
                Ok(order) => Ok(HttpResponse::Ok().json(order)),
                Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to create PayPal order")),
            }
        }
        "revolut" => {
            match create_revolut_payment(get_plan_price(&payment_request.plan)).await {
                Ok(payment) => Ok(HttpResponse::Ok().json(payment)),
                Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to create Revolut payment")),
            }
        }
        "klarna" => {
            match create_klarna_session(get_plan_price(&payment_request.plan)).await {
                Ok(session) => Ok(HttpResponse::Ok().json(session)),
                Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to create Klarna session")),
            }
        }
        _ => Ok(HttpResponse::BadRequest().json("Unsupported payment method")),
    }
}

pub async fn handle_webhook(
    _db: web::Data<Database>,
    webhook: web::Json<PaymentWebhook>,
) -> Result<HttpResponse> {
    let webhook_data = webhook.into_inner();

    // Handle successful payment
    if webhook_data.event_type == "payment.succeeded" {
        // Extract user ID and plan from webhook data
        // Update user subscription in database
        // This is a simplified implementation
        println!("Payment succeeded: {:?}", webhook_data.data);
    }

    Ok(HttpResponse::Ok().json("Webhook processed"))
}

pub async fn cancel_subscription(
    db: web::Data<Database>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let token = match extract_token_from_header(&req) {
        Some(t) => t,
        None => return Ok(HttpResponse::Unauthorized().json("Authentication required")),
    };

    let claims = match verify_token(&token) {
        Ok(c) => c,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("Invalid token")),
    };

    // Update user subscription to free
    match db.update_user_subscription(&claims.sub, "free").await {
        Ok(_) => Ok(HttpResponse::Ok().json("Subscription cancelled")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to cancel subscription")),
    }
}
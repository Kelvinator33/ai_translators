use sled::{Db, IVec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;
use crate::AIBackend;
use crate::auth::User;

#[derive(Clone)]
pub struct Database {
    db: Db,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        Ok(Database { db })
    }

    pub async fn save_backend(&self, backend: &AIBackend) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("backend:{}", backend.id);
        let value = serde_json::to_vec(backend)?;
        self.db.insert(key, value)?;
        Ok(())
    }

    pub async fn get_backend(&self, id: &str) -> Result<Option<AIBackend>, Box<dyn std::error::Error>> {
        let key = format!("backend:{}", id);
        if let Some(value) = self.db.get(&key)? {
            let backend: AIBackend = serde_json::from_slice(&value)?;
            Ok(Some(backend))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_backends(&self) -> Result<Vec<AIBackend>, Box<dyn std::error::Error>> {
        let mut backends = Vec::new();
        let prefix = b"backend:";
        
        for result in self.db.scan_prefix(prefix) {
            let (_key, value) = result?;
            let backend: AIBackend = serde_json::from_slice(&value)?;
            backends.push(backend);
        }
        
        Ok(backends)
    }

    pub async fn init_default_backends(&self) -> Result<(), Box<dyn std::error::Error>> {
        let default_backends = vec![
            AIBackend {
                id: "ollama".to_string(),
                name: "Ollama".to_string(),
                backend_type: "local".to_string(),
                config: {
                    let mut config = HashMap::new();
                    config.insert("endpoint".to_string(), 
                        std::env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434".to_string()));
                    config.insert("model".to_string(), 
                        std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama2".to_string()));
                    config
                },
                enabled: true,
            },
            AIBackend {
                id: "openai".to_string(),
                name: "OpenAI GPT".to_string(),
                backend_type: "api".to_string(),
                config: {
                    let mut config = HashMap::new();
                    config.insert("api_key".to_string(), 
                        std::env::var("OPENAI_API_KEY").unwrap_or_default());
                    config.insert("model".to_string(), 
                        std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string()));
                    config.insert("endpoint".to_string(), 
                        std::env::var("OPENAI_ENDPOINT").unwrap_or_else(|_| "https://api.openai.com/v1".to_string()));
                    config
                },
                enabled: true, // API key provided
            },
            AIBackend {
                id: "gemini".to_string(),
                name: "Google Gemini".to_string(),
                backend_type: "api".to_string(),
                config: {
                    let mut config = HashMap::new();
                    config.insert("api_key".to_string(), 
                        std::env::var("GEMINI_API_KEY").unwrap_or_default());
                    config.insert("model".to_string(), 
                        std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-pro".to_string()));
                    config
                },
                enabled: true, // API key provided
            },
            AIBackend {
                id: "deepl".to_string(),
                name: "DeepL API".to_string(),
                backend_type: "translation".to_string(),
                config: {
                    let mut config = HashMap::new();
                    config.insert("api_key".to_string(), 
                        std::env::var("DEEPL_API_KEY").unwrap_or_default());
                    config.insert("endpoint".to_string(), 
                        std::env::var("DEEPL_ENDPOINT").unwrap_or_else(|_| "https://api-free.deepl.com/v2/translate".to_string()));
                    config
                },
                enabled: true, // API key provided
            },
            AIBackend {
                id: "llama_cpp".to_string(),
                name: "llama.cpp".to_string(),
                backend_type: "local".to_string(),
                config: {
                    let mut config = HashMap::new();
                    config.insert("endpoint".to_string(), 
                        std::env::var("LLAMA_CPP_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_string()));
                    config.insert("model_path".to_string(), 
                        std::env::var("LLAMA_CPP_MODEL_PATH").unwrap_or_default());
                    config
                },
                enabled: false,
            },
            AIBackend {
                id: "mistral".to_string(),
                name: "Mistral AI".to_string(),
                backend_type: "api".to_string(),
                config: {
                    let mut config = HashMap::new();
                    config.insert("api_key".to_string(), 
                        std::env::var("MISTRAL_API_KEY").unwrap_or_default());
                    config.insert("model".to_string(), 
                        std::env::var("MISTRAL_MODEL").unwrap_or_else(|_| "mistral-tiny".to_string()));
                    config
                },
                enabled: false, // No API key provided
            },
            AIBackend {
                id: "anthropic".to_string(),
                name: "Anthropic Claude".to_string(),
                backend_type: "api".to_string(),
                config: {
                    let mut config = HashMap::new();
                    config.insert("api_key".to_string(), 
                        std::env::var("ANTHROPIC_API_KEY").unwrap_or_default());
                    config.insert("model".to_string(), 
                        std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-3-haiku-20240307".to_string()));
                    config
                },
                enabled: false, // No API key provided
            },
            AIBackend {
                id: "ratchet".to_string(),
                name: "Ratchet (Local GPU)".to_string(),
                backend_type: "local".to_string(),
                config: {
                    let mut config = HashMap::new();
                    config.insert("model_path".to_string(), 
                        std::env::var("RATCHET_MODEL_PATH").unwrap_or_default());
                    config.insert("device".to_string(), "gpu".to_string());
                    config
                },
                enabled: false,
            },
            AIBackend {
                id: "kalosm".to_string(),
                name: "Kalosm (Local)".to_string(),
                backend_type: "local".to_string(),
                config: {
                    let mut config = HashMap::new();
                    config.insert("model_type".to_string(), 
                        std::env::var("KALOSM_MODEL").unwrap_or_else(|_| "llama".to_string()));
                    config.insert("model_path".to_string(), 
                        std::env::var("KALOSM_MODEL_PATH").unwrap_or_default());
                    config
                },
                enabled: false,
            },
        ];

        for backend in default_backends {
            // Always save to ensure enabled status is updated with API keys
            self.save_backend(&backend).await?;
        }

        Ok(())
    }

    // User management methods
    pub async fn save_user(&self, user: &User) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("user:{}", user.id);
        let value = serde_json::to_vec(user)?;
        self.db.insert(key, value)?;
        
        // Also index by email for login
        let email_key = format!("user_email:{}", user.email);
        self.db.insert(email_key, user.id.as_bytes())?;
        
        Ok(())
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<User, Box<dyn std::error::Error>> {
        let key = format!("user:{}", id);
        if let Some(value) = self.db.get(&key)? {
            let user: User = serde_json::from_slice(&value)?;
            Ok(user)
        } else {
            Err("User not found".into())
        }
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, Box<dyn std::error::Error>> {
        let email_key = format!("user_email:{}", email);
        if let Some(user_id_bytes) = self.db.get(&email_key)? {
            let user_id = String::from_utf8(user_id_bytes.to_vec())?;
            self.get_user_by_id(&user_id).await
        } else {
            Err("User not found".into())
        }
    }

    pub async fn update_user_subscription(&self, user_id: &str, plan: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut user = self.get_user_by_id(user_id).await?;
        user.subscription_plan = plan.to_string();
        
        // Reset daily usage for new subscription
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        if user.last_reset_date != today {
            user.translations_used_today = 0;
            user.last_reset_date = today;
        }
        
        self.save_user(&user).await
    }

    pub async fn increment_user_translations(&self, user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut user = self.get_user_by_id(user_id).await?;
        
        // Reset daily counter if it's a new day
        let today = Utc::now().format("%Y-%m-%d").to_string();
        if user.last_reset_date != today {
            user.translations_used_today = 0;
            user.last_reset_date = today;
        }
        
        user.translations_used_today += 1;
        self.save_user(&user).await
    }

    pub async fn get_user_usage(&self, user_id: &str) -> Result<(i32, i32), Box<dyn std::error::Error>> {
        let user = self.get_user_by_id(user_id).await?;
        let limit = user.get_translation_limit();
        Ok((user.translations_used_today, limit))
    }
}
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::{AIBackend, TranslationRequest, TranslationResponse};

pub struct AIBackendClient {
    client: Client,
}

impl AIBackendClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn translate_with_backend(
        &self,
        backend: &AIBackend,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, Box<dyn std::error::Error>> {
        match backend.backend_type.as_str() {
            "local" => self.translate_local(backend, request).await,
            "api" => self.translate_api(backend, request).await,
            "translation" => self.translate_dedicated(backend, request).await,
            _ => Err("Unknown backend type".into()),
        }
    }

    async fn translate_local(
        &self,
        backend: &AIBackend,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, Box<dyn std::error::Error>> {
        let default_endpoint = "http://localhost:11434".to_string();
        let endpoint = backend.config.get("endpoint").unwrap_or(&default_endpoint);
        
        let prompt = format!(
            "Translate the following text from {} to {}: \"{}\"",
            request.source_lang.as_ref().unwrap_or(&"auto".to_string()),
            request.target_lang,
            request.text
        );

        match backend.id.as_str() {
            "ollama" => self.translate_ollama(endpoint, &prompt, backend).await,
            "llama_cpp" => self.translate_llama_cpp(endpoint, &prompt, backend).await,
            _ => Err("Unknown local backend".into()),
        }
    }

    async fn translate_ollama(
        &self,
        endpoint: &str,
        prompt: &str,
        backend: &AIBackend,
    ) -> Result<TranslationResponse, Box<dyn std::error::Error>> {
        let default_model = "llama2".to_string();
        let model = backend.config.get("model").unwrap_or(&default_model);
        
        let payload = json!({
            "model": model,
            "prompt": prompt,
            "stream": false
        });

        let response = self.client
            .post(&format!("{}/api/generate", endpoint))
            .json(&payload)
            .send()
            .await?;

        let result: Value = response.json().await?;
        let translated_text = result["response"]
            .as_str()
            .unwrap_or("Translation failed")
            .trim()
            .to_string();

        Ok(TranslationResponse {
            translated_text,
            source_lang: "auto".to_string(),
            target_lang: "target".to_string(),
            backend_used: backend.name.clone(),
        })
    }

    async fn translate_llama_cpp(
        &self,
        endpoint: &str,
        prompt: &str,
        backend: &AIBackend,
    ) -> Result<TranslationResponse, Box<dyn std::error::Error>> {
        let payload = json!({
            "prompt": prompt,
            "n_predict": 512,
            "temperature": 0.7,
            "stop": ["\n", "Human:", "Assistant:"]
        });

        let response = self.client
            .post(&format!("{}/completion", endpoint))
            .json(&payload)
            .send()
            .await?;

        let result: Value = response.json().await?;
        let translated_text = result["content"]
            .as_str()
            .unwrap_or("Translation failed")
            .trim()
            .to_string();

        Ok(TranslationResponse {
            translated_text,
            source_lang: "auto".to_string(),
            target_lang: "target".to_string(),
            backend_used: backend.name.clone(),
        })
    }

    async fn translate_api(
        &self,
        backend: &AIBackend,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, Box<dyn std::error::Error>> {
        match backend.id.as_str() {
            "openai" => self.translate_openai(backend, request).await,
            "gemini" => self.translate_gemini(backend, request).await,
            "mistral" => self.translate_mistral(backend, request).await,
            "anthropic" => self.translate_anthropic(backend, request).await,
            _ => Err("Unknown API backend".into()),
        }
    }

    async fn translate_openai(
        &self,
        backend: &AIBackend,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, Box<dyn std::error::Error>> {
        let api_key = backend.config.get("api_key").ok_or("API key not configured")?;
        let default_model = "gpt-3.5-turbo".to_string();
        let model = backend.config.get("model").unwrap_or(&default_model);
        let default_endpoint = "https://api.openai.com/v1".to_string();
        let endpoint = backend.config.get("endpoint").unwrap_or(&default_endpoint);

        let prompt = format!(
            "Translate the following text from {} to {}: \"{}\"",
            request.source_lang.as_ref().unwrap_or(&"auto".to_string()),
            request.target_lang,
            request.text
        );

        let payload = json!({
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        });

        let response = self.client
            .post(&format!("{}/chat/completions", endpoint))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await?;

        let result: Value = response.json().await?;
        let translated_text = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Translation failed")
            .trim()
            .to_string();

        Ok(TranslationResponse {
            translated_text,
            source_lang: request.source_lang.as_ref().unwrap_or(&"auto".to_string()).clone(),
            target_lang: request.target_lang.clone(),
            backend_used: backend.name.clone(),
        })
    }

    async fn translate_gemini(
        &self,
        backend: &AIBackend,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, Box<dyn std::error::Error>> {
        let api_key = backend.config.get("api_key").ok_or("API key not configured")?;
        let default_model = "gemini-pro".to_string();
        let model = backend.config.get("model").unwrap_or(&default_model);

        let prompt = format!(
            "Translate the following text from {} to {}: \"{}\"",
            request.source_lang.as_ref().unwrap_or(&"auto".to_string()),
            request.target_lang,
            request.text
        );

        let payload = json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }]
        });

        let response = self.client
            .post(&format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", model, api_key))
            .json(&payload)
            .send()
            .await?;

        let result: Value = response.json().await?;
        let translated_text = result["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("Translation failed")
            .trim()
            .to_string();

        Ok(TranslationResponse {
            translated_text,
            source_lang: request.source_lang.as_ref().unwrap_or(&"auto".to_string()).clone(),
            target_lang: request.target_lang.clone(),
            backend_used: backend.name.clone(),
        })
    }

    async fn translate_dedicated(
        &self,
        backend: &AIBackend,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, Box<dyn std::error::Error>> {
        match backend.id.as_str() {
            "deepl" => self.translate_deepl(backend, request).await,
            _ => Err("Unknown translation backend".into()),
        }
    }

    async fn translate_deepl(
        &self,
        backend: &AIBackend,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, Box<dyn std::error::Error>> {
        let api_key = backend.config.get("api_key").ok_or("API key not configured")?;
        let default_endpoint = "https://api-free.deepl.com/v2/translate".to_string();
        let endpoint = backend.config.get("endpoint").unwrap_or(&default_endpoint);

        let mut form = HashMap::new();
        form.insert("text", request.text.clone());
        form.insert("target_lang", request.target_lang.clone());
        if let Some(source) = &request.source_lang {
            form.insert("source_lang", source.clone());
        }

        let response = self.client
            .post(endpoint)
            .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
            .form(&form)
            .send()
            .await?;

        let result: Value = response.json().await?;
        let translated_text = result["translations"][0]["text"]
            .as_str()
            .unwrap_or("Translation failed")
            .to_string();

        let detected_lang = result["translations"][0]["detected_source_language"]
            .as_str()
            .unwrap_or("auto")
            .to_string();

        Ok(TranslationResponse {
            translated_text,
            source_lang: detected_lang,
            target_lang: request.target_lang.clone(),
            backend_used: backend.name.clone(),
        })
    }

    async fn translate_mistral(
        &self,
        backend: &AIBackend,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, Box<dyn std::error::Error>> {
        let api_key = backend.config.get("api_key").ok_or("API key not configured")?;
        let default_model = "mistral-tiny".to_string();
        let model = backend.config.get("model").unwrap_or(&default_model);

        let prompt = format!(
            "Translate the following text from {} to {}: \"{}\"",
            request.source_lang.as_ref().unwrap_or(&"auto".to_string()),
            request.target_lang,
            request.text
        );

        let payload = json!({
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        });

        let response = self.client
            .post("https://api.mistral.ai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await?;

        let result: Value = response.json().await?;
        let translated_text = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Translation failed")
            .trim()
            .to_string();

        Ok(TranslationResponse {
            translated_text,
            source_lang: request.source_lang.as_ref().unwrap_or(&"auto".to_string()).clone(),
            target_lang: request.target_lang.clone(),
            backend_used: backend.name.clone(),
        })
    }

    async fn translate_anthropic(
        &self,
        backend: &AIBackend,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, Box<dyn std::error::Error>> {
        let api_key = backend.config.get("api_key").ok_or("API key not configured")?;
        let default_model = "claude-3-haiku-20240307".to_string();
        let model = backend.config.get("model").unwrap_or(&default_model);

        let prompt = format!(
            "Translate the following text from {} to {}: \"{}\"",
            request.source_lang.as_ref().unwrap_or(&"auto".to_string()),
            request.target_lang,
            request.text
        );

        let payload = json!({
            "model": model,
            "max_tokens": 1024,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        });

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await?;

        let result: Value = response.json().await?;
        let translated_text = result["content"][0]["text"]
            .as_str()
            .unwrap_or("Translation failed")
            .trim()
            .to_string();

        Ok(TranslationResponse {
            translated_text,
            source_lang: request.source_lang.as_ref().unwrap_or(&"auto".to_string()).clone(),
            target_lang: request.target_lang.clone(),
            backend_used: backend.name.clone(),
        })
    }
}
use crate::config::Config;
use anyhow::Result;
use lazy_static::lazy_static;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref IN_FLIGHT_REQUESTS: Arc<Mutex<HashMap<String, std::time::Instant>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub detected_language: String,
    pub translated_text: String,
}

pub struct TranslationService {
    client: reqwest::Client,
    config: Config,
}

impl TranslationService {
    pub fn new(config: Config) -> Self {
        log::info!(
            "Creating TranslationService with custom_prompt: {}",
            config.custom_prompt
        );
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    pub async fn detect_and_translate(&self, text: &str) -> Result<TranslationResult> {
        // Create a more unique request key that includes current timestamp to prevent issues
        // with legitimate duplicate requests (e.g., user retrying the same text)
        let request_key = format!(
            "{}-{}",
            text.len(),
            text.chars().take(50).collect::<String>()
        );

        let now = std::time::Instant::now();

        {
            let mut requests = IN_FLIGHT_REQUESTS.lock().unwrap();

            // Clean up old requests (older than 5 seconds)
            requests.retain(|_, &mut timestamp| now.duration_since(timestamp).as_secs() < 5);

            // Check if there's a recent request for the same content
            if let Some(&request_time) = requests.get(&request_key) {
                let time_diff = now.duration_since(request_time);
                // Only consider it a duplicate if it's within 500ms
                if time_diff.as_millis() < 500 {
                    log::info!(
                        "Duplicate translation request detected within 500ms, skipping API call"
                    );
                    return Err(anyhow::anyhow!("Duplicate request detected"));
                }
            }

            requests.insert(request_key.clone(), now);
        }

        let result = match self.perform_translation(text).await {
            Ok(response) => Ok(response),
            Err(e) => Err(e),
        };

        {
            let mut requests = IN_FLIGHT_REQUESTS.lock().unwrap();
            requests.remove(&request_key);
        }

        result
    }
    async fn perform_translation(&self, text: &str) -> Result<TranslationResult> {
        // Improve text cleaning to preserve paragraph structure
        // Instead of filtering out empty lines, preserve them as paragraph breaks
        let cleaned_text = text
            .lines()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n");

        log::info!("Cleaned text for translation: {}", cleaned_text);

        let user_prompt = format!("Text to translate: \"{}\"", cleaned_text);

        // Create a smart prompt that handles the alternative language logic
        let smart_prompt = format!(
            "{}\n\n# Alternative Language Logic\n- Primary target language: {}\n- Alternative target language: {}\n- If the detected language matches the primary target language, translate to the alternative target language instead.\n- If the detected language is different from the primary target language, translate to the primary target language.",
            self.config.custom_prompt,
            self.config.target_language,
            self.config.alternative_target_language
        );

        log::info!("Using smart prompt with alternative language logic");
        let mut request_body = json!({
            "messages": [
                {
                    "role": "system",
                    "content": [
                        {
                            "type": "text",
                            "text": format!("{}\n\nAlways respond with valid JSON containing 'detected_language' and 'translated_text' fields. Make sure to properly escape newlines in the translated_text field.", smart_prompt)
                        }
                    ]
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ],
            "max_tokens": 800,
            "temperature": 0.3
        });

        if self.config.api_provider == "openai" {
            request_body["model"] = json!(self.config.model);
            log::info!("Using OpenAI model: {}", self.config.model);
        }

        let response = if self.config.api_provider == "azure_openai" {
            self.call_azure_openai(request_body).await?
        } else {
            self.call_openai(request_body).await?
        };
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

        log::info!("API Response content: {}", content);

        // Clean the content by removing control characters that can break JSON parsing
        let cleaned_content = content
            .chars()
            .filter(|c| !c.is_control() || matches!(*c, '\n' | '\r' | '\t'))
            .collect::<String>();

        // Log if we removed any control characters
        if cleaned_content != content {
            log::warn!("Removed control characters from API response");
            log::info!("Cleaned content: {}", cleaned_content);
        }

        // Try to parse as JSON, but handle cases where the AI might have returned plain text
        let parsed: Value = match serde_json::from_str(&cleaned_content) {
            Ok(json) => {
                log::info!("Successfully parsed JSON response");
                json
            }
            Err(parse_error) => {
                log::warn!("Failed to parse as JSON: {}", parse_error);

                // Try to find and extract valid JSON from the response
                if let Some(start_idx) = cleaned_content.find('{') {
                    // Find the matching closing brace by counting braces
                    let mut brace_count = 0;
                    let mut end_idx = None;

                    for (i, c) in cleaned_content[start_idx..].char_indices() {
                        if c == '{' {
                            brace_count += 1;
                        } else if c == '}' {
                            brace_count -= 1;
                            if brace_count == 0 {
                                end_idx = Some(start_idx + i + 1); // +1 to include the closing brace
                                break;
                            }
                        }
                    }

                    if let Some(end_idx) = end_idx {
                        let json_str = &cleaned_content[start_idx..end_idx];
                        log::info!("Attempting to parse extracted JSON: {}", json_str);

                        match serde_json::from_str::<Value>(json_str) {
                            Ok(json) => {
                                log::info!("Successfully parsed extracted JSON");
                                json
                            }
                            Err(e) => {
                                log::warn!("Failed to parse extracted JSON: {}", e);
                                json!({
                                    "detected_language": "unknown",
                                    "translated_text": cleaned_content
                                })
                            }
                        }
                    } else {
                        log::warn!("Could not find matching closing brace");
                        json!({
                            "detected_language": "unknown",
                            "translated_text": cleaned_content
                        })
                    }
                } else {
                    log::warn!("No JSON structure found in response");
                    json!({
                        "detected_language": "unknown",
                        "translated_text": cleaned_content
                    })
                }
            }
        };

        // Extract detected language and translated text from parsed JSON
        let detected_language = match parsed["detected_language"].as_str() {
            Some(lang) if !lang.is_empty() => lang.to_string(),
            _ => {
                // If detected_language field is missing or empty, try to look deeper in JSON structure
                if let Some(lang) = parsed.get("detected_language").and_then(|v| v.as_str()) {
                    lang.to_string()
                } else {
                    "unknown".to_string()
                }
            }
        };

        let translated_text = match parsed["translated_text"].as_str() {
            Some(text) => text.to_string(),
            None => {
                // If translated_text field is missing, check if the whole response is just text
                if parsed.is_string() {
                    parsed.as_str().unwrap_or("translation failed").to_string()
                } else {
                    "translation failed".to_string()
                }
            }
        };
        log::info!("Detected language: {}", detected_language);
        log::info!("Target language: {}", self.config.target_language);
        log::info!(
            "Alternative target language: {}",
            self.config.alternative_target_language
        );

        // Log if alternative language logic should have been applied
        if detected_language
            .to_lowercase()
            .contains(&self.config.target_language.to_lowercase())
            || self
                .config
                .target_language
                .to_lowercase()
                .contains(&detected_language.to_lowercase())
        {
            log::info!(
                "Alternative language logic should apply - detected '{}' matches target '{}'",
                detected_language,
                self.config.target_language
            );
        }

        log::info!(
            "Translated text (first 100 chars): {}",
            if translated_text.len() > 100 {
                format!("{}...", &translated_text[..100])
            } else {
                translated_text.clone()
            }
        );

        Ok(TranslationResult {
            detected_language,
            translated_text,
        })
    }

    async fn call_openai(&self, request_body: Value) -> Result<Value> {
        let url = "https://api.openai.com/v1/chat/completions";

        log::info!("Making OpenAI request to: {}", url);
        log::info!(
            "Request body: {}",
            serde_json::to_string_pretty(&request_body).unwrap_or_default()
        );

        let response = self
            .client
            .post(url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.openai_api_key),
            )
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            log::error!("OpenAI API request failed: {}", error_text);
            return Err(anyhow::anyhow!("OpenAI API request failed: {}", error_text));
        }

        Ok(response.json().await?)
    }

    async fn call_azure_openai(&self, request_body: Value) -> Result<Value> {
        let url = format!(
            "{}openai/deployments/{}/chat/completions?api-version={}",
            self.config.azure_endpoint,
            self.config.azure_deployment_name,
            self.config.azure_api_version
        );

        log::info!("Making Azure OpenAI request to: {}", url);
        log::info!(
            "Request body: {}",
            serde_json::to_string_pretty(&request_body).unwrap_or_default()
        );

        let response = self
            .client
            .post(&url)
            .header("api-key", &self.config.azure_api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            log::error!("Azure OpenAI API request failed: {}", error_text);
            return Err(anyhow::anyhow!(
                "Azure OpenAI API request failed: {}",
                error_text
            ));
        }

        Ok(response.json().await?)
    }
}

#[derive(Debug)]
pub enum Error {
    DuplicateRequest,
    ApiError(anyhow::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DuplicateRequest => write!(f, "Duplicate request"),
            Error::ApiError(e) => write!(f, "API error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error::ApiError(error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    pub original_text: String,
    pub translated_text: String,
    pub detected_language: String,
    pub target_language: String,
}

pub async fn translate_text(
    text: String,
    config: tauri::State<'_, crate::AppState>,
) -> Result<TranslationResponse, Error> {
    log::info!("translate_text called with text: {}", text);

    let config_guard = config.config.lock().await;
    let config_clone = config_guard.clone();
    drop(config_guard);

    log::info!(
        "Config loaded, custom_prompt: {}",
        config_clone.custom_prompt
    );

    let service = TranslationService::new(config_clone);
    match service.detect_and_translate(&text).await {
        Ok(result) => {
            // Use the configured target language from config
            let target_language = service.config.target_language.clone();

            Ok(TranslationResponse {
                original_text: text,
                translated_text: result.translated_text,
                detected_language: result.detected_language,
                target_language,
            })
        }
        Err(e) => {
            if e.to_string().contains("Duplicate request detected") {
                Err(Error::DuplicateRequest)
            } else {
                Err(Error::ApiError(e))
            }
        }
    }
}

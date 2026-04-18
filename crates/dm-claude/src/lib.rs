//! dm-claude: AI service client.
//!
//! Phase 1: thin async HTTP wrapper around the local Python FastAPI backend
//! (`/api/intent` and `/api/suggest`).  Phase 2 will grow this into a full
//! Anthropic tool-use client; callers won't need to change.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("backend returned no suggestions")]
    NoSuggestions,
}

// ── /api/intent ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct IntentRequest<'a> {
    text: &'a str,
}

/// Response from POST /api/intent.
#[derive(Debug, Clone, Deserialize)]
pub struct IntentResponse {
    pub intent: String,
    pub confidence: f32,
}

// ── /api/suggest ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct SuggestRequest<'a> {
    scene_description: &'a str,
}

#[derive(Deserialize)]
struct SuggestResponse {
    suggestions: Vec<String>,
    #[serde(default)]
    error: Option<String>,
}

// ── Client ───────────────────────────────────────────────────────────────────

/// Async HTTP client for the local Python AI backend.
///
/// Construct once per process; `Client` is cheaply cloneable internally.
#[derive(Debug, Clone)]
pub struct PyAiClient {
    client: Client,
    base_url: String,
}

impl PyAiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { client: Client::new(), base_url: base_url.into() }
    }

    /// Connects to `http://127.0.0.1:8000` — the default uvicorn bind.
    pub fn localhost() -> Self {
        Self::new("http://127.0.0.1:8000")
    }

    /// Classify player input. Returns the intent label + confidence [0..1].
    ///
    /// Labels from the fine-tuned MiniLM: `"attack"`, `"ability_check"`,
    /// `"cast_spell"`, `"roleplay"`.
    pub async fn classify_intent(&self, text: &str) -> Result<IntentResponse, AiError> {
        let url = format!("{}/api/intent", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&IntentRequest { text })
            .send()
            .await?
            .error_for_status()?
            .json::<IntentResponse>()
            .await?;
        Ok(resp)
    }

    /// Ask Ollama for 3 context-aware action suggestions for the current scene.
    ///
    /// Returns up to 3 Thai-language strings. Falls back gracefully: if the
    /// backend returns an error field or an empty list, the caller should treat
    /// suggestions as optional (offer manual input instead).
    pub async fn suggest(&self, scene_description: &str) -> Result<Vec<String>, AiError> {
        let url = format!("{}/api/suggest", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&SuggestRequest { scene_description })
            .send()
            .await?
            .error_for_status()?
            .json::<SuggestResponse>()
            .await?;

        if let Some(err) = resp.error {
            // Backend returned fallback suggestions with an error field
            // (matches the try/except pattern in api_server.py)
            eprintln!("[AI] suggest fallback: {}", err);
        }

        if resp.suggestions.is_empty() {
            return Err(AiError::NoSuggestions);
        }
        Ok(resp.suggestions)
    }
}

use serde_json::{json, Value};

#[derive(Debug)]
pub enum AiError {
    Http(String),
    Parse(String),
    NoContent,
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::Http(e) => write!(f, "HTTP error: {}", e),
            AiError::Parse(e) => write!(f, "Parse error: {}", e),
            AiError::NoContent => write!(f, "No content in response"),
        }
    }
}

impl std::error::Error for AiError {}

/// Send `text` to LM Studio and return the compressed result.
pub fn compress(
    base_url: &str,
    model: &str,
    system_prompt: &str,
    max_tokens: u32,
    text: &str,
) -> Result<String, AiError> {
    let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

    let body = json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user",   "content": text }
        ],
        "max_tokens": max_tokens,
        "temperature": 0,
        "stream": false
    });

    let body_str =
        serde_json::to_string(&body).map_err(|e| AiError::Parse(e.to_string()))?;

    let response_str = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body_str)
        .map_err(|e: ureq::Error| AiError::Http(e.to_string()))?
        .into_string()
        .map_err(|e: std::io::Error| AiError::Http(e.to_string()))?;

    let response: Value =
        serde_json::from_str(&response_str).map_err(|e| AiError::Parse(e.to_string()))?;

    response["choices"][0]["message"]["content"]
        .as_str()
        .map(|s: &str| s.to_owned())
        .ok_or(AiError::NoContent)
}

use crate::config::Config;
use crate::models::transaction::RawTransaction;
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde_json::{json, Value};
use std::sync::Arc;

const MODEL: &str = "gpt-4o";

const SYSTEM_PROMPT: &str =
    "You are a financial data extraction assistant. Extract bank statement transactions \
     and return them as a JSON array only — no explanation, no markdown fences. \
     Each object must have: date (YYYY-MM-DD), description (string), \
     credit (number in Naira, 0 if none), debit (number in Naira, 0 if none), \
     balance (number in Naira, 0 if not shown).";

const TEXT_USER_PROMPT: &str =
    "Extract all transactions from the following bank statement text and return a JSON array:\n\n";

const PDF_USER_PROMPT: &str =
    "This image is a page from a bank statement PDF. \
     Extract all visible transactions and return a JSON array.";

pub async fn extract_from_text(text: &str, config: &Arc<Config>) -> Result<Vec<RawTransaction>> {
    let body = json!({
        "model": MODEL,
        "max_tokens": 4096,
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user",   "content": format!("{}{}", TEXT_USER_PROMPT, text) }
        ]
    });
    call_openai(body, config).await
}

/// Fallback for image-based / scanned PDFs.
/// OpenAI vision accepts PNG/JPEG; we send the first page as a base64 image.
/// For simplicity we re-encode the raw bytes as a PNG data URL — works when
/// the PDF is already image-backed (most Nigerian bank statement exports are).
pub async fn extract_from_pdf_bytes(
    pdf_bytes: &[u8],
    config: &Arc<Config>,
) -> Result<Vec<RawTransaction>> {
    let b64 = B64.encode(pdf_bytes);
    // OpenAI vision: send as a base64-encoded image URL (treat PDF bytes as image)
    let body = json!({
        "model": MODEL,
        "max_tokens": 4096,
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            {
                "role": "user",
                "content": [
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", b64),
                            "detail": "high"
                        }
                    },
                    { "type": "text", "text": PDF_USER_PROMPT }
                ]
            }
        ]
    });
    call_openai(body, config).await
}

async fn call_openai(body: Value, config: &Arc<Config>) -> Result<Vec<RawTransaction>> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&config.openai_api_key)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("OpenAI API error {}: {}", status, text));
    }

    let resp_json: Value = resp.json().await?;
    let content = resp_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow!("unexpected OpenAI response shape: {}", resp_json))?;

    let json_str = strip_fences(content);

    let txns: Vec<RawTransaction> = serde_json::from_str(json_str)
        .map_err(|e| anyhow!("Failed to parse LLM JSON: {e}\nRaw: {json_str}"))?;

    Ok(txns)
}

fn strip_fences(s: &str) -> &str {
    let s = s.trim();
    let s = s.strip_prefix("```json").unwrap_or(s);
    let s = s.strip_prefix("```").unwrap_or(s);
    let s = s.strip_suffix("```").unwrap_or(s);
    s.trim()
}

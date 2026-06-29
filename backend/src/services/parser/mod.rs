mod llm;
mod pdf;

use crate::config::Config;
use crate::models::transaction::RawTransaction;
use anyhow::Result;
use std::sync::Arc;

pub async fn parse_statement(
    pdf_bytes: &[u8],
    config: &Arc<Config>,
) -> Result<Vec<RawTransaction>> {
    // Try to extract text from PDF first
    let text = pdf::extract_text(pdf_bytes).unwrap_or_default();

    if text.trim().len() < 100 {
        // Too little text — the PDF might be scanned/image-based
        tracing::info!("PDF text too short ({}b), using LLM direct extraction", text.len());
        return llm::extract_from_pdf_bytes(pdf_bytes, config).await;
    }

    // We have text — ask the LLM to parse it into structured transactions
    tracing::info!("Extracted {}b of text from PDF, sending to LLM", text.len());
    llm::extract_from_text(&text, config).await
}

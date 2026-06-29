use anyhow::Result;
use lopdf::Document;

/// Extract raw text from PDF bytes using lopdf.
/// Returns Err (or empty string) if the PDF is scanned/image-based.
pub fn extract_text(pdf_bytes: &[u8]) -> Result<String> {
    let doc = Document::load_mem(pdf_bytes)?;
    let mut all_text = String::new();

    let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    for page_num in pages {
        match doc.extract_text(&[page_num]) {
            Ok(text) => {
                all_text.push_str(&text);
                all_text.push('\n');
            }
            Err(e) => {
                tracing::warn!("Failed to extract text from page {}: {}", page_num, e);
            }
        }
    }

    Ok(all_text)
}

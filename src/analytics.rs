// src/analytics.rs

pub struct TokenMetadata {
    pub raw_tokens: usize,
    pub compressed_tokens: usize,
    pub savings_percentage: f64,
}

/// Measures exact local token counts completely offline using the tiktoken cl100k model
pub fn calculate_savings(raw_text: &str, compressed_text: &str) -> TokenMetadata {
    // Initialize the cl100k_base engine (powers GPT-4 and Claude's context limits)
    let bpe = tiktoken_rs::cl100k_base().unwrap();
    
    // Count exact token vector lengths
    let raw_tokens = bpe.encode_with_special_tokens(raw_text).len();
    let compressed_tokens = bpe.encode_with_special_tokens(compressed_text).len();

    let savings_percentage = if raw_tokens > 0 {
        ((raw_tokens - compressed_tokens) as f64 / raw_tokens as f64) * 100.0
    } else {
        0.0
    };

    TokenMetadata {
        raw_tokens,
        compressed_tokens,
        savings_percentage,
    }
}
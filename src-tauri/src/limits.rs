//! Max lengths for user-editable text. Keep in sync with `src/lib/limits.ts`.

pub const DOCUMENT_TITLE: usize = 120;
pub const DOCUMENT_SUMMARY: usize = 500;
pub const PREREQUISITE: usize = 200;
pub const PREREQUISITES_MAX: usize = 10;
pub const STEP_TITLE: usize = 80;
pub const STEP_BODY: usize = 2000;
pub const AUDIENCE: usize = 300;
pub const LANGUAGE: usize = 50;
pub const PRODUCT_NAME: usize = 80;
pub const VOCABULARY_TERM: usize = 80;
pub const VOCABULARY_EXPLANATION: usize = 200;
pub const MODEL_ID: usize = 128;
pub const BASE_URL: usize = 2048;
pub const API_KEY: usize = 512;

pub fn clamp(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max).collect()
}

pub fn clamp_trim(s: &str, max: usize) -> String {
    clamp(s.trim(), max)
}

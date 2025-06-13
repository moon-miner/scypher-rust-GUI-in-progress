// src/bip39/mod.rs - Módulo BIP39 principal

pub mod wordlist;
pub mod validation;
pub mod conversion;

use crate::error::Result;

// Re-exportar funciones principales para fácil acceso
pub use wordlist::{BIP39_WORDLIST, word_to_index, index_to_word, is_valid_word};
pub use validation::{validate_seed_phrase, validate_word_count, validate_words, analyze_seed_phrase, is_valid_seed_phrase};
pub use conversion::{phrase_to_bits, bits_to_phrase, entropy_to_phrase, phrase_to_entropy, phrase_to_hex, hex_to_phrase};

/// Validar formato de seed phrase BIP39 (función principal)
pub fn validate_seed_phrase_complete(seed_phrase: &str) -> Result<()> {
    validation::validate_seed_phrase(seed_phrase)
}

/// Verificar checksum BIP39 (implementación actualizada)
pub fn verify_checksum(seed_phrase: &str) -> Result<bool> {
    match validation::validate_checksum(seed_phrase) {
        Ok(()) => Ok(true),
        Err(crate::error::SCypherError::InvalidChecksum) => Ok(false),
        Err(e) => Err(e),
    }
}

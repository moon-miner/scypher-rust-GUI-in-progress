// src/bip39/wordlist.rs
//! Lista de palabras BIP39
//! TODO: Implementar en Fase 2

pub const WORDLIST: &[&str] = &["abandon", "ability"]; // Placeholder truncado

// src/bip39/validation.rs
//! Validaciones BIP39
//! TODO: Implementar en Fase 2

use crate::error::Result;

pub fn validate_words(_words: &[&str]) -> Result<()> {
    // Placeholder
    Ok(())
}

// src/bip39/conversion.rs
//! Conversiones binario↔palabras
//! TODO: Implementar en Fase 2

use crate::error::Result;

pub fn words_to_bits(_words: &str) -> Result<Vec<u8>> {
    // Placeholder
    Ok(vec![0u8; 16])
}

pub fn bits_to_words(_bits: &[u8]) -> Result<String> {
    // Placeholder
    Ok("abandon ability".to_string())
}
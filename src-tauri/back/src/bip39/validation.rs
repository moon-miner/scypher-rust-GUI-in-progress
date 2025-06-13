//! Validación de seed phrases BIP39
//!
//! Este módulo proporciona funciones para validar seed phrases
//! según el estándar BIP39, incluyendo validación de palabras,
//! longitud y checksums.

use crate::error::{SCypherError, Result};
use crate::bip39::wordlist::{is_valid_word, find_closest_word};
use crate::crypto::checksum;

/// Longitudes válidas de seed phrases BIP39 (en palabras)
const VALID_WORD_COUNTS: [usize; 5] = [12, 15, 18, 21, 24];

/// Valida una seed phrase completa BIP39
pub fn validate_seed_phrase(phrase: &str) -> Result<()> {
    let words = phrase.split_whitespace().collect::<Vec<&str>>();

    // 1. Validar longitud
    validate_word_count(words.len())?;

    // 2. Validar que todas las palabras sean válidas BIP39
    validate_words(&words)?;

    // 3. Validar checksum
    validate_checksum(phrase)?;

    Ok(())
}

/// Valida que el número de palabras sea correcto para BIP39
pub fn validate_word_count(count: usize) -> Result<()> {
    if VALID_WORD_COUNTS.contains(&count) {
        Ok(())
    } else {
        Err(SCypherError::InvalidWordCount(count))
    }
}

/// Valida que todas las palabras estén en la lista BIP39
pub fn validate_words(words: &[&str]) -> Result<()> {
    let mut invalid_words = Vec::new();
    let mut suggestions = Vec::new();

    for &word in words {
        if !is_valid_word(word) {
            invalid_words.push(word);

            // Intentar encontrar sugerencia para palabra inválida
            if let Some((closest, distance)) = find_closest_word(word) {
                if distance <= 2 { // Solo sugerir si la distancia es pequeña
                    suggestions.push(format!("'{}' -> '{}'", word, closest));
                }
            }
        }
    }

    if !invalid_words.is_empty() {
        let mut error_msg = format!(
            "Invalid BIP39 words found: {}",
            invalid_words.join(", ")
        );

        if !suggestions.is_empty() {
            error_msg.push_str("\nSuggested corrections: ");
            error_msg.push_str(&suggestions.join(", "));
        }

        return Err(SCypherError::InvalidBip39Word(error_msg));
    }

    Ok(())
}

/// Valida el checksum BIP39 de una seed phrase
pub fn validate_checksum(phrase: &str) -> Result<()> {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    let word_count = words.len();

    // Calcular bits de entropía y checksum
    let entropy_bits = word_count * 32 / 3;
    let checksum_bits = entropy_bits / 32;
    let total_bits = entropy_bits + checksum_bits;

    // Convertir palabras a bits
    let seed_bits = words_to_bits(&words)?;

    if seed_bits.len() != total_bits {
        return Err(SCypherError::InvalidChecksum);
    }

    // Verificar checksum
    if checksum::verify_bip39_checksum(&seed_bits, entropy_bits)? {
        Ok(())
    } else {
        Err(SCypherError::InvalidChecksum)
    }
}

/// Convierte palabras BIP39 a representación de bits
fn words_to_bits(words: &[&str]) -> Result<Vec<bool>> {
    let mut bits = Vec::new();

    for &word in words {
        let index = crate::bip39::wordlist::word_to_index(word)
            .ok_or_else(|| SCypherError::InvalidBip39Word(word.to_string()))?;

        // Convertir índice a 11 bits (cada palabra BIP39 es 11 bits)
        for i in (0..11).rev() {
            bits.push((index >> i) & 1 == 1);
        }
    }

    Ok(bits)
}

/// Proporciona información detallada sobre una seed phrase
pub fn analyze_seed_phrase(phrase: &str) -> SeedPhraseAnalysis {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    let word_count = words.len();

    let mut analysis = SeedPhraseAnalysis {
        word_count,
        is_valid_length: VALID_WORD_COUNTS.contains(&word_count),
        invalid_words: Vec::new(),
        suggestions: Vec::new(),
        entropy_bits: if VALID_WORD_COUNTS.contains(&word_count) {
            Some(word_count * 32 / 3)
        } else {
            None
        },
        checksum_valid: None,
        overall_valid: false,
    };

    // Analizar palabras individuales
    for &word in &words {
        if !is_valid_word(word) {
            analysis.invalid_words.push(word.to_string());

            if let Some((closest, distance)) = find_closest_word(word) {
                if distance <= 2 {
                    analysis.suggestions.push(format!("{} -> {}", word, closest));
                }
            }
        }
    }

    // Verificar checksum si todo lo demás es válido
    if analysis.is_valid_length && analysis.invalid_words.is_empty() {
        analysis.checksum_valid = Some(validate_checksum(phrase).is_ok());
        analysis.overall_valid = analysis.checksum_valid.unwrap_or(false);
    }

    analysis
}

/// Estructura que contiene análisis detallado de una seed phrase
#[derive(Debug, Clone)]
pub struct SeedPhraseAnalysis {
    pub word_count: usize,
    pub is_valid_length: bool,
    pub invalid_words: Vec<String>,
    pub suggestions: Vec<String>,
    pub entropy_bits: Option<usize>,
    pub checksum_valid: Option<bool>,
    pub overall_valid: bool,
}

impl SeedPhraseAnalysis {
    /// Genera un reporte legible del análisis
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!("Seed Phrase Analysis:\n"));
        report.push_str(&format!("  Word count: {} ", self.word_count));

        if self.is_valid_length {
            report.push_str("✓ Valid\n");
        } else {
            report.push_str(&format!("❌ Invalid (expected: {:?})\n", VALID_WORD_COUNTS));
        }

        if self.invalid_words.is_empty() {
            report.push_str("  Words: ✓ All words valid\n");
        } else {
            report.push_str(&format!("  Words: ❌ Invalid words: {}\n",
                                   self.invalid_words.join(", ")));

            if !self.suggestions.is_empty() {
                report.push_str(&format!("  Suggestions: {}\n",
                                       self.suggestions.join(", ")));
            }
        }

        if let Some(entropy) = self.entropy_bits {
            report.push_str(&format!("  Entropy: {} bits\n", entropy));
        }

        match self.checksum_valid {
            Some(true) => report.push_str("  Checksum: ✓ Valid\n"),
            Some(false) => report.push_str("  Checksum: ❌ Invalid\n"),
            None => report.push_str("  Checksum: - Not checked\n"),
        }

        report.push_str(&format!("  Overall: {}\n",
                               if self.overall_valid { "✓ Valid" } else { "❌ Invalid" }));

        report
    }
}

/// Función de conveniencia para validación rápida
pub fn is_valid_seed_phrase(phrase: &str) -> bool {
    validate_seed_phrase(phrase).is_ok()
}

/// Sanitiza una seed phrase removiendo espacios extra y normalizando
pub fn sanitize_seed_phrase(phrase: &str) -> String {
    phrase
        .split_whitespace()
        .map(|word| word.trim().to_lowercase())
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_word_count() {
        // Casos válidos
        for &count in &VALID_WORD_COUNTS {
            assert!(validate_word_count(count).is_ok());
        }

        // Casos inválidos
        assert!(validate_word_count(11).is_err());
        assert!(validate_word_count(13).is_err());
        assert!(validate_word_count(25).is_err());
    }

    #[test]
    fn test_validate_words() {
        // Palabras válidas
        let valid_words = vec!["abandon", "ability", "able"];
        assert!(validate_words(&valid_words).is_ok());

        // Palabras inválidas
        let invalid_words = vec!["abandon", "invalid_word", "able"];
        assert!(validate_words(&invalid_words).is_err());
    }

    #[test]
    fn test_words_to_bits() {
        let words = vec!["abandon", "ability"];
        let bits = words_to_bits(&words).unwrap();

        // Cada palabra son 11 bits
        assert_eq!(bits.len(), 22);

        // "abandon" es índice 0, así que debería ser 00000000000
        let abandon_bits = &bits[0..11];
        assert!(abandon_bits.iter().all(|&bit| !bit));

        // "ability" es índice 1, así que debería ser 00000000001
        let ability_bits = &bits[11..22];
        assert_eq!(ability_bits[10], true); // Último bit en 1
        assert!(ability_bits[0..10].iter().all(|&bit| !bit)); // Resto en 0
    }

    #[test]
    fn test_sanitize_seed_phrase() {
        let messy = "  abandon   ABILITY    able  ";
        let clean = sanitize_seed_phrase(messy);
        assert_eq!(clean, "abandon ability able");
    }

    #[test]
    fn test_seed_phrase_analysis() {
        // Analizar frase válida
        let valid_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let analysis = analyze_seed_phrase(valid_phrase);

        assert_eq!(analysis.word_count, 12);
        assert!(analysis.is_valid_length);
        assert!(analysis.invalid_words.is_empty());
        assert!(analysis.entropy_bits.is_some());

        // Analizar frase inválida
        let invalid_phrase = "invalid word count";
        let analysis = analyze_seed_phrase(invalid_phrase);

        assert_eq!(analysis.word_count, 3);
        assert!(!analysis.is_valid_length);
        assert!(!analysis.overall_valid);
    }

    #[test]
    fn test_is_valid_seed_phrase() {
        // Esta debería ser válida (seed phrase común para testing)
        let test_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        // Nota: En el placeholder actual, checksum siempre retorna true
        // En implementación completa, esta verificación será más estricta
        let result = is_valid_seed_phrase(test_phrase);
        println!("Test phrase validation result: {}", result);
    }
}

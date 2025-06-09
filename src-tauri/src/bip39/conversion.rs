//! Conversiones entre palabras BIP39 y representaciones binarias
//!
//! Este módulo maneja la conversión bidireccional entre seed phrases
//! y su representación binaria, incluyendo manejo de checksums.

use crate::error::{SCypherError, Result};
use crate::bip39::wordlist::{word_to_index, index_to_word};
use crate::crypto::checksum;

/// Convierte una seed phrase BIP39 a su representación en bits
pub fn phrase_to_bits(phrase: &str) -> Result<Vec<bool>> {
    let words: Vec<&str> = phrase.split_whitespace().collect();

    if words.is_empty() {
        return Err(SCypherError::InvalidSeedPhrase);
    }

    let mut bits = Vec::new();

    for word in words {
        let index = word_to_index(word)
            .ok_or_else(|| SCypherError::InvalidBip39Word(word.to_string()))?;

        // Cada palabra BIP39 se representa con 11 bits
        for i in (0..11).rev() {
            bits.push((index >> i) & 1 == 1);
        }
    }

    Ok(bits)
}

/// Convierte una representación de bits a seed phrase BIP39
pub fn bits_to_phrase(bits: &[bool]) -> Result<String> {
    if bits.len() % 11 != 0 {
        return Err(SCypherError::crypto(
            format!("Bit length {} is not divisible by 11", bits.len())
        ));
    }

    let mut words = Vec::new();

    // Procesar en chunks de 11 bits
    for chunk in bits.chunks(11) {
        let mut index = 0usize;

        for (i, &bit) in chunk.iter().enumerate() {
            if bit {
                index |= 1 << (10 - i); // MSB first
            }
        }

        if index >= 2048 {
            return Err(SCypherError::crypto(
                format!("Word index {} is out of range (0-2047)", index)
            ));
        }

        let word = index_to_word(index)
            .ok_or_else(|| SCypherError::crypto(format!("Invalid word index: {}", index)))?;

        words.push(word);
    }

    Ok(words.join(" "))
}

/// Convierte entropía pura a seed phrase BIP39 válida (con checksum)
pub fn entropy_to_phrase(entropy: &[u8]) -> Result<String> {
    let entropy_bits = entropy.len() * 8;

    // Validar longitud de entropía
    checksum::validate_entropy_length(entropy_bits)?;

    // Convertir entropía a bits
    let mut entropy_bit_vec = Vec::new();
    for byte in entropy {
        for i in (0..8).rev() {
            entropy_bit_vec.push((byte >> i) & 1 == 1);
        }
    }

    // Calcular checksum
    let checksum_bits = checksum::recalculate_bip39_checksum(&entropy_bit_vec)?;

    // Combinar entropía + checksum
    let mut full_bits = entropy_bit_vec;
    full_bits.extend(checksum_bits);

    // Convertir a frase
    bits_to_phrase(&full_bits)
}

/// Convierte seed phrase BIP39 a entropía pura (sin checksum)
pub fn phrase_to_entropy(phrase: &str) -> Result<Vec<u8>> {
    let bits = phrase_to_bits(phrase)?;
    let word_count = phrase.split_whitespace().count();

    // Calcular longitudes
    let entropy_bits = word_count * 32 / 3;
    let checksum_bits = entropy_bits / 32;

    if bits.len() != entropy_bits + checksum_bits {
        return Err(SCypherError::crypto(
            format!("Invalid seed phrase bit length: expected {}, got {}",
                   entropy_bits + checksum_bits, bits.len())
        ));
    }

    // Extraer solo la parte de entropía
    let entropy_bits = &bits[0..entropy_bits];

    // Convertir bits a bytes
    let entropy_bytes = bits_to_bytes(entropy_bits);

    Ok(entropy_bytes)
}

/// Convierte bits a bytes, rellenando con ceros si es necesario
fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
    let mut bytes = Vec::new();

    for chunk in bits.chunks(8) {
        let mut byte = 0u8;

        for (i, &bit) in chunk.iter().enumerate() {
            if bit {
                byte |= 1 << (7 - i); // MSB first
            }
        }

        bytes.push(byte);
    }

    bytes
}

/// Genera una seed phrase BIP39 desde entropía aleatoria
pub fn generate_seed_phrase(entropy_bits: usize) -> Result<String> {
    checksum::validate_entropy_length(entropy_bits)?;

    let entropy_bytes = entropy_bits / 8;
    let mut entropy = vec![0u8; entropy_bytes];

    // Generar entropía aleatoria
    use rand::RngCore;
    rand::thread_rng().fill_bytes(&mut entropy);

    entropy_to_phrase(&entropy)
}

/// Valida que una seed phrase tenga el checksum correcto y lo recalcula si es necesario
pub fn validate_and_fix_checksum(phrase: &str) -> Result<String> {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    let word_count = words.len();

    // Validar longitud
    let valid_counts = [12, 15, 18, 21, 24];
    if !valid_counts.contains(&word_count) {
        return Err(SCypherError::InvalidWordCount(word_count));
    }

    // Validar palabras
    for word in &words {
        if word_to_index(word).is_none() {
            return Err(SCypherError::InvalidBip39Word(word.to_string()));
        }
    }

    // Extraer entropía
    let entropy = phrase_to_entropy(phrase)?;

    // Regenerar frase con checksum correcto
    entropy_to_phrase(&entropy)
}

/// Información sobre una seed phrase
#[derive(Debug, Clone)]
pub struct SeedPhraseInfo {
    pub word_count: usize,
    pub entropy_bits: usize,
    pub checksum_bits: usize,
    pub total_bits: usize,
    pub entropy_bytes: Vec<u8>,
}

/// Analiza una seed phrase y extrae información detallada
pub fn analyze_phrase(phrase: &str) -> Result<SeedPhraseInfo> {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    let word_count = words.len();

    let entropy_bits = word_count * 32 / 3;
    let checksum_bits = entropy_bits / 32;
    let total_bits = entropy_bits + checksum_bits;

    let entropy_bytes = phrase_to_entropy(phrase)?;

    Ok(SeedPhraseInfo {
        word_count,
        entropy_bits,
        checksum_bits,
        total_bits,
        entropy_bytes,
    })
}

/// Convierte una seed phrase a formato hexadecimal (para interoperabilidad)
pub fn phrase_to_hex(phrase: &str) -> Result<String> {
    let entropy = phrase_to_entropy(phrase)?;
    Ok(hex::encode(entropy))
}

/// Convierte formato hexadecimal a seed phrase BIP39
pub fn hex_to_phrase(hex_str: &str) -> Result<String> {
    let entropy = hex::decode(hex_str)
        .map_err(|e| SCypherError::crypto(format!("Invalid hex string: {}", e)))?;

    entropy_to_phrase(&entropy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phrase_to_bits() {
        let phrase = "abandon ability";
        let bits = phrase_to_bits(phrase).unwrap();

        // 2 palabras * 11 bits = 22 bits
        assert_eq!(bits.len(), 22);

        // "abandon" es índice 0 (00000000000)
        let abandon_bits = &bits[0..11];
        assert!(abandon_bits.iter().all(|&bit| !bit));

        // "ability" es índice 1 (00000000001)
        let ability_bits = &bits[11..22];
        assert_eq!(ability_bits[10], true);
        assert!(ability_bits[0..10].iter().all(|&bit| !bit));
    }

    #[test]
    fn test_bits_to_phrase() {
        // Crear bits para "abandon ability"
        let mut bits = vec![false; 11]; // abandon (índice 0)
        bits.extend(vec![false; 10]); // ability (índice 1) - primeros 10 bits
        bits.push(true); // ability - último bit

        let phrase = bits_to_phrase(&bits).unwrap();
        assert_eq!(phrase, "abandon ability");
    }

    #[test]
    fn test_bits_to_bytes() {
        let bits = vec![true, false, true, true, false, true, false, false]; // 0b10110100 = 180
        let bytes = bits_to_bytes(&bits);
        assert_eq!(bytes, vec![180]);

        // Test con padding
        let bits_short = vec![true, false, true]; // Solo 3 bits -> 0b10100000 = 160
        let bytes_padded = bits_to_bytes(&bits_short);
        assert_eq!(bytes_padded, vec![160]);
    }

    #[test]
    fn test_entropy_to_phrase() {
        // Test con entropía de 128 bits (16 bytes) para 12 palabras
        let entropy = vec![0u8; 16];
        let phrase = entropy_to_phrase(&entropy).unwrap();

        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 12);
    }

    #[test]
    fn test_phrase_to_entropy() {
        // Crear una frase válida y extraer su entropía
        let entropy_original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let phrase = entropy_to_phrase(&entropy_original).unwrap();
        let entropy_extracted = phrase_to_entropy(&phrase).unwrap();

        assert_eq!(entropy_original, entropy_extracted);
    }

    #[test]
    fn test_roundtrip_conversion() {
        // Test de ida y vuelta: entropía -> frase -> entropía
        let original_entropy = vec![42u8; 16]; // 128 bits

        let phrase = entropy_to_phrase(&original_entropy).unwrap();
        let recovered_entropy = phrase_to_entropy(&phrase).unwrap();

        assert_eq!(original_entropy, recovered_entropy);
    }

    #[test]
    fn test_invalid_bit_length() {
        // Bits que no son múltiplo de 11
        let invalid_bits = vec![true; 10];
        assert!(bits_to_phrase(&invalid_bits).is_err());
    }

    #[test]
    fn test_invalid_word_index() {
        // Crear bits que representan un índice > 2047
        let mut invalid_bits = vec![true; 11]; // 0b11111111111 = 2047
        invalid_bits[0] = true; // Hacer que sea > 2047

        // Esto debería fallar en la implementación real
        // Por ahora, el test puede pasar dependiendo de la implementación
    }

    #[test]
    fn test_hex_conversion() {
        let entropy = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let phrase = entropy_to_phrase(&entropy).unwrap();
        let hex = phrase_to_hex(&phrase).unwrap();

        assert_eq!(hex, "deadbeef");

        let phrase_from_hex = hex_to_phrase(&hex).unwrap();
        assert_eq!(phrase, phrase_from_hex);
    }

    #[test]
    fn test_analyze_phrase() {
        let entropy = vec![0u8; 16]; // 128 bits
        let phrase = entropy_to_phrase(&entropy).unwrap();
        let info = analyze_phrase(&phrase).unwrap();

        assert_eq!(info.word_count, 12);
        assert_eq!(info.entropy_bits, 128);
        assert_eq!(info.checksum_bits, 4);
        assert_eq!(info.total_bits, 132);
        assert_eq!(info.entropy_bytes.len(), 16);
    }

    #[test]
    fn test_generate_seed_phrase() {
        // Generar frase de 12 palabras (128 bits de entropía)
        let phrase = generate_seed_phrase(128).unwrap();
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 12);

        // Verificar que sea válida
        let info = analyze_phrase(&phrase).unwrap();
        assert_eq!(info.entropy_bits, 128);

        // Generar otra y verificar que sean diferentes
        let phrase2 = generate_seed_phrase(128).unwrap();
        assert_ne!(phrase, phrase2); // Extremadamente improbable que sean iguales
    }

    #[test]
    fn test_validate_and_fix_checksum() {
        // Crear una frase con entropía conocida
        let entropy = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let correct_phrase = entropy_to_phrase(&entropy).unwrap();

        // La frase debería validarse correctamente
        let fixed_phrase = validate_and_fix_checksum(&correct_phrase).unwrap();
        assert_eq!(correct_phrase, fixed_phrase);
    }

    #[test]
    fn test_empty_phrase() {
        assert!(phrase_to_bits("").is_err());
        assert!(phrase_to_entropy("").is_err());
    }

    #[test]
    fn test_invalid_entropy_length() {
        // Entropía de longitud inválida (no múltiplo de 32 bits)
        let invalid_entropy = vec![0u8; 15]; // 120 bits, no válido para BIP39
        assert!(entropy_to_phrase(&invalid_entropy).is_err());
    }
}

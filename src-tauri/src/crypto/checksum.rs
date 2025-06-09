//! Manejo de checksums BIP39
//!
//! BIP39 usa SHA256 para calcular checksums que validan la integridad
//! de las frases semilla. Este módulo maneja el cálculo y verificación.

use sha2::{Sha256, Digest};
use crate::error::{SCypherError, Result};

/// Calcula el checksum BIP39 para datos de entropía
///
/// # Parámetros
/// - `entropy`: Datos de entropía en bytes
///
/// # Retorna
/// Los primeros bits del hash SHA256 como checksum
pub fn calculate_checksum(entropy: &[u8]) -> Result<Vec<u8>> {
    if entropy.is_empty() {
        return Err(SCypherError::crypto("Entropy cannot be empty".to_string()));
    }

    // Calcular SHA256 de la entropía
    let mut hasher = Sha256::new();
    hasher.update(entropy);
    let hash = hasher.finalize();

    // BIP39 usa los primeros ENT/32 bits del hash como checksum
    // donde ENT es la longitud de entropía en bits
    let entropy_bits = entropy.len() * 8;
    let checksum_bits = entropy_bits / 32;
    let checksum_bytes = (checksum_bits + 7) / 8; // Redondear hacia arriba

    if checksum_bytes > hash.len() {
        return Err(SCypherError::crypto("Invalid entropy length".to_string()));
    }

    Ok(hash[..checksum_bytes].to_vec())
}

/// Extrae bits específicos de un array de bytes
///
/// # Parámetros
/// - `bytes`: Array de bytes fuente
/// - `start_bit`: Bit de inicio (0-indexado)
/// - `num_bits`: Número de bits a extraer
pub fn extract_bits(bytes: &[u8], start_bit: usize, num_bits: usize) -> Result<Vec<bool>> {
    let total_bits = bytes.len() * 8;

    if start_bit + num_bits > total_bits {
        return Err(SCypherError::crypto(
            format!("Bit range out of bounds: {}+{} > {}", start_bit, num_bits, total_bits)
        ));
    }

    let mut bits = Vec::with_capacity(num_bits);

    for i in 0..num_bits {
        let bit_index = start_bit + i;
        let byte_index = bit_index / 8;
        let bit_offset = 7 - (bit_index % 8); // MSB first

        let bit = (bytes[byte_index] >> bit_offset) & 1 == 1;
        bits.push(bit);
    }

    Ok(bits)
}

/// Convierte bits a bytes, rellenando con ceros si es necesario
pub fn bits_to_bytes_padded(bits: &[bool]) -> Vec<u8> {
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

/// Verifica si un checksum BIP39 es válido
///
/// # Parámetros
/// - `seed_bits`: Bits completos de la seed (entropía + checksum)
/// - `entropy_bits`: Número de bits de entropía (sin checksum)
pub fn verify_bip39_checksum(seed_bits: &[bool], entropy_bits: usize) -> Result<bool> {
    if seed_bits.len() < entropy_bits {
        return Err(SCypherError::crypto("Seed bits shorter than entropy".to_string()));
    }

    let checksum_bits = entropy_bits / 32;

    if seed_bits.len() != entropy_bits + checksum_bits {
        return Err(SCypherError::crypto("Invalid seed length for BIP39".to_string()));
    }

    // Extraer entropía y checksum
    let entropy_part = &seed_bits[..entropy_bits];
    let checksum_part = &seed_bits[entropy_bits..];

    // Convertir entropía a bytes
    let entropy_bytes = bits_to_bytes_padded(entropy_part);

    // Calcular checksum esperado
    let expected_checksum = calculate_checksum(&entropy_bytes)?;
    let expected_checksum_bits = extract_bits(&expected_checksum, 0, checksum_bits)?;

    // Comparar checksums
    Ok(checksum_part == expected_checksum_bits)
}

/// Recalcula el checksum BIP39 para una entropía dada
/// Útil después de transformaciones XOR para mantener validez BIP39
pub fn recalculate_bip39_checksum(entropy_bits: &[bool]) -> Result<Vec<bool>> {
    let entropy_bytes = bits_to_bytes_padded(entropy_bits);
    let checksum_bytes = calculate_checksum(&entropy_bytes)?;
    let checksum_bit_count = entropy_bits.len() / 32;

    extract_bits(&checksum_bytes, 0, checksum_bit_count)
}

/// Obtiene la longitud de checksum esperada para una longitud de entropía dada
pub fn get_checksum_length(entropy_bits: usize) -> usize {
    entropy_bits / 32
}

/// Valida que una longitud de entropía sea válida para BIP39
pub fn validate_entropy_length(entropy_bits: usize) -> Result<()> {
    // BIP39 válido: 128, 160, 192, 224, 256 bits (ENT)
    // Que corresponden a 12, 15, 18, 21, 24 palabras
    let valid_lengths = [128, 160, 192, 224, 256];

    if valid_lengths.contains(&entropy_bits) {
        Ok(())
    } else {
        Err(SCypherError::crypto(
            format!("Invalid entropy length: {} bits (valid: {:?})", entropy_bits, valid_lengths)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_checksum() {
        let entropy = vec![0x12, 0x34, 0x56, 0x78]; // 32 bits
        let checksum = calculate_checksum(&entropy).unwrap();

        // Para 32 bits de entropía, checksum debe ser 1 bit (1 byte mínimo)
        assert!(!checksum.is_empty());

        // Debe ser determinista
        let checksum2 = calculate_checksum(&entropy).unwrap();
        assert_eq!(checksum, checksum2);
    }

    #[test]
    fn test_extract_bits() {
        let bytes = vec![0b10110100, 0b11001010]; // 16 bits total

        // Extraer los primeros 4 bits
        let bits = extract_bits(&bytes, 0, 4).unwrap();
        assert_eq!(bits, vec![true, false, true, true]); // 1011

        // Extraer bits del medio
        let bits = extract_bits(&bytes, 4, 4).unwrap();
        assert_eq!(bits, vec![false, true, false, false]); // 0100
    }

    #[test]
    fn test_bits_to_bytes_padded() {
        let bits = vec![true, false, true, true, false, true, false, false];
        let bytes = bits_to_bytes_padded(&bits);
        assert_eq!(bytes, vec![0b10110100]);

        // Test con padding
        let bits_short = vec![true, false, true]; // Solo 3 bits
        let bytes_padded = bits_to_bytes_padded(&bits_short);
        assert_eq!(bytes_padded, vec![0b10100000]); // Rellenado con ceros
    }

    #[test]
    fn test_validate_entropy_length() {
        // Longitudes válidas
        assert!(validate_entropy_length(128).is_ok()); // 12 palabras
        assert!(validate_entropy_length(256).is_ok()); // 24 palabras

        // Longitudes inválidas
        assert!(validate_entropy_length(100).is_err());
        assert!(validate_entropy_length(300).is_err());
    }

    #[test]
    fn test_get_checksum_length() {
        assert_eq!(get_checksum_length(128), 4);  // 128/32 = 4 bits
        assert_eq!(get_checksum_length(256), 8);  // 256/32 = 8 bits
        assert_eq!(get_checksum_length(160), 5);  // 160/32 = 5 bits
    }

    #[test]
    fn test_recalculate_bip39_checksum() {
        // Crear entropía de prueba (128 bits = 16 bytes)
        let entropy_bits: Vec<bool> = (0..128).map(|i| i % 2 == 0).collect();

        let checksum = recalculate_bip39_checksum(&entropy_bits).unwrap();
        assert_eq!(checksum.len(), 4); // 128/32 = 4 bits de checksum

        // Verificar que sea determinista
        let checksum2 = recalculate_bip39_checksum(&entropy_bits).unwrap();
        assert_eq!(checksum, checksum2);
    }

    #[test]
    fn test_verify_bip39_checksum() {
        // Crear entropía y checksum válidos
        let entropy_bits: Vec<bool> = (0..128).map(|i| i % 3 == 0).collect();
        let checksum_bits = recalculate_bip39_checksum(&entropy_bits).unwrap();

        // Combinar entropía + checksum
        let mut seed_bits = entropy_bits.clone();
        seed_bits.extend(checksum_bits);

        // Debe ser válido
        assert!(verify_bip39_checksum(&seed_bits, 128).unwrap());

        // Modificar un bit del checksum - debe ser inválido
        let mut invalid_seed = seed_bits.clone();
        if invalid_seed.len() > 130 {
            invalid_seed[130] = !invalid_seed[130]; // Cambiar bit en checksum
            assert!(!verify_bip39_checksum(&invalid_seed, 128).unwrap());
        }
    }

    #[test]
    fn test_empty_entropy() {
        let empty: Vec<u8> = vec![];
        assert!(calculate_checksum(&empty).is_err());
    }

    #[test]
    fn test_extract_bits_bounds() {
        let bytes = vec![0xFF, 0x00]; // 16 bits

        // Casos válidos
        assert!(extract_bits(&bytes, 0, 8).is_ok());
        assert!(extract_bits(&bytes, 8, 8).is_ok());
        assert!(extract_bits(&bytes, 0, 16).is_ok());

        // Casos inválidos (fuera de rango)
        assert!(extract_bits(&bytes, 0, 17).is_err());
        assert!(extract_bits(&bytes, 16, 1).is_err());
        assert!(extract_bits(&bytes, 10, 8).is_err());
    }
}

//! Operaciones XOR para cifrado simétrico
//! 
//! El cifrado XOR es perfectamente simétrico: aplicar la misma operación
//! dos veces con la misma clave devuelve los datos originales.

use crate::error::{SCypherError, Result};

/// Aplica operación XOR entre datos y keystream
/// 
/// # Parámetros
/// - `data`: Datos a cifrar/descifrar
/// - `keystream`: Flujo de claves de la misma longitud que los datos
/// 
/// # Retorna
/// Vector de bytes con el resultado XOR
/// 
/// # Errores
/// - Si las longitudes no coinciden
pub fn xor_data(data: &[u8], keystream: &[u8]) -> Result<Vec<u8>> {
    if data.len() != keystream.len() {
        return Err(SCypherError::crypto(
            format!("Data length ({}) doesn't match keystream length ({})", 
                   data.len(), keystream.len())
        ));
    }
    
    let result: Vec<u8> = data
        .iter()
        .zip(keystream.iter())
        .map(|(a, b)| a ^ b)
        .collect();
    
    Ok(result)
}

/// Aplica XOR bit a bit entre dos vectores de bits (representados como Vec<bool>)
/// Útil para operaciones a nivel de bits individuales
pub fn xor_bits(bits_a: &[bool], bits_b: &[bool]) -> Result<Vec<bool>> {
    if bits_a.len() != bits_b.len() {
        return Err(SCypherError::crypto(
            format!("Bit vector lengths don't match: {} vs {}", 
                   bits_a.len(), bits_b.len())
        ));
    }
    
    let result: Vec<bool> = bits_a
        .iter()
        .zip(bits_b.iter())
        .map(|(a, b)| a ^ b)
        .collect();
    
    Ok(result)
}

/// Convierte bytes a representación de bits (Vec<bool>)
pub fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
    let mut bits = Vec::with_capacity(bytes.len() * 8);
    
    for byte in bytes {
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1 == 1);
        }
    }
    
    bits
}

/// Convierte representación de bits (Vec<bool>) a bytes
/// Rellena con ceros si no es múltiplo de 8
pub fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
    let mut bytes = Vec::new();
    
    // Procesar en chunks de 8 bits
    for chunk in bits.chunks(8) {
        let mut byte = 0u8;
        
        for (i, &bit) in chunk.iter().enumerate() {
            if bit {
                byte |= 1 << (7 - i);
            }
        }
        
        bytes.push(byte);
    }
    
    bytes
}

/// Utilidad para mostrar bytes en formato hexadecimal (para debugging)
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Utilidad para convertir hexadecimal a bytes
pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>> {
    hex::decode(hex_str)
        .map_err(|e| SCypherError::crypto(format!("Invalid hex string: {}", e)))
}

/// Verificar que la operación XOR sea reversible (para tests)
pub fn verify_xor_reversibility(original: &[u8], keystream: &[u8]) -> Result<bool> {
    let encrypted = xor_data(original, keystream)?;
    let decrypted = xor_data(&encrypted, keystream)?;
    
    Ok(original == decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_data() {
        let data = vec![0b10101010, 0b11110000];
        let key = vec![0b01010101, 0b00001111];
        
        let result = xor_data(&data, &key).unwrap();
        assert_eq!(result, vec![0b11111111, 0b11111111]);
        
        // Verificar reversibilidad
        let restored = xor_data(&result, &key).unwrap();
        assert_eq!(restored, data);
    }

    #[test]
    fn test_xor_data_length_mismatch() {
        let data = vec![1, 2, 3];
        let key = vec![1, 2]; // Longitud diferente
        
        assert!(xor_data(&data, &key).is_err());
    }

    #[test]
    fn test_bytes_to_bits() {
        let bytes = vec![0b10101010]; // 170 en decimal
        let bits = bytes_to_bits(&bytes);
        
        assert_eq!(bits.len(), 8);
        assert_eq!(bits, vec![true, false, true, false, true, false, true, false]);
    }

    #[test] 
    fn test_bits_to_bytes() {
        let bits = vec![true, false, true, false, true, false, true, false];
        let bytes = bits_to_bytes(&bits);
        
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 0b10101010);
    }

    #[test]
    fn test_xor_bits() {
        let bits_a = vec![true, false, true, false];
        let bits_b = vec![false, false, true, true];
        
        let result = xor_bits(&bits_a, &bits_b).unwrap();
        assert_eq!(result, vec![true, false, false, true]);
    }

    #[test]
    fn test_bytes_hex_conversion() {
        let bytes = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let hex = bytes_to_hex(&bytes);
        assert_eq!(hex, "deadbeef");
        
        let restored = hex_to_bytes(&hex).unwrap();
        assert_eq!(restored, bytes);
    }

    #[test]
    fn test_xor_reversibility() {
        let original = vec![1, 2, 3, 4, 5];
        let keystream = vec![255, 128, 64, 32, 16];
        
        assert!(verify_xor_reversibility(&original, &keystream).unwrap());
    }

    #[test]
    fn test_empty_data() {
        let empty: Vec<u8> = vec![];
        let result = xor_data(&empty, &empty).unwrap();
        assert!(result.is_empty());
    }
}
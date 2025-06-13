// src/crypto/mod.rs - Módulo criptográfico principal

pub mod keystream;
pub mod xor;
pub mod checksum;

use crate::error::Result;

/// Función principal para transformar seed phrase usando XOR
/// Esta es la función que une todos los componentes criptográficos
pub fn transform_seed(
    seed_phrase: &str,
    password: &str,
    iterations: u32,
    memory_cost: u32,
) -> Result<String> {
    // Validar parámetros Argon2id
    keystream::validate_argon2_params(iterations, memory_cost)?;

    // Convertir seed phrase a bits usando BIP39
    let seed_bits = crate::bip39::conversion::phrase_to_bits(seed_phrase)?;

    // Separar entropía y checksum
    let word_count = seed_phrase.split_whitespace().count();
    let entropy_bits = word_count * 32 / 3;  // Bits de entropía según BIP39
    let checksum_bits = entropy_bits / 32;   // Bits de checksum según BIP39

    if seed_bits.len() != entropy_bits + checksum_bits {
        return Err(crate::error::SCypherError::crypto(
            "Invalid seed phrase bit length".to_string()
        ));
    }

    // Extraer SOLO la parte de entropía (ignorar checksum actual)
    let entropy_part = &seed_bits[0..entropy_bits];

    // Convertir entropía a bytes para XOR
    let entropy_bytes = crate::crypto::checksum::bits_to_bytes_padded(entropy_part);

    // Generar keystream del tamaño de la entropía
    let keystream = keystream::derive_keystream(password, entropy_bytes.len(), iterations, memory_cost)?;

    // Aplicar XOR solo a la entropía
    let encrypted_entropy_bytes = xor::xor_data(&entropy_bytes, &keystream)?;

    // Convertir entropía cifrada de vuelta a bits
    let mut encrypted_entropy_bits = Vec::new();
    for byte in &encrypted_entropy_bytes {
        for i in (0..8).rev() {
            encrypted_entropy_bits.push((byte >> i) & 1 == 1);
        }
    }

    // Truncar a la longitud exacta de entropía
    encrypted_entropy_bits.truncate(entropy_bits);

    // Recalcular checksum BIP39 para la nueva entropía
    let new_checksum_bits = crate::crypto::checksum::recalculate_bip39_checksum(&encrypted_entropy_bits)?;

    // Combinar entropía cifrada + nuevo checksum
    let mut final_bits = encrypted_entropy_bits;
    final_bits.extend(new_checksum_bits);

    // Convertir de vuelta a seed phrase BIP39
    let result_phrase = crate::bip39::conversion::bits_to_phrase(&final_bits)?;

    Ok(result_phrase)
}

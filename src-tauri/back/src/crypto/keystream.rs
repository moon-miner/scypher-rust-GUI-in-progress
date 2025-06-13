//! Derivación de keystream con Argon2id
//!
//! Este módulo se encarga de generar un flujo de claves (keystream) usando
//! Argon2id a partir de una contraseña. El keystream se usa luego para
//! la operación XOR con la frase semilla.

use argon2::{Argon2, Algorithm, Version, Params};
use crate::error::{SCypherError, Result};

/// Genera un keystream usando Argon2id
///
/// # Parámetros
/// - `password`: Contraseña del usuario
/// - `length`: Longitud deseada del keystream en bytes
/// - `iterations`: Número de iteraciones de Argon2id
/// - `memory_cost`: Costo de memoria en KB
///
/// # Retorna
/// Vector de bytes que representa el keystream
pub fn derive_keystream(
    password: &str,
    length: usize,
    iterations: u32,
    memory_cost: u32,
) -> Result<Vec<u8>> {
    // TODO: Implementar Argon2id real en Fase 3
    // Por ahora, generar datos pseudoaleatorios deterministas para desarrollo

    // Usar un salt fijo derivado de la contraseña para hacer determinista
    let salt_bytes = generate_deterministic_salt(password);

    // Crear parámetros Argon2id
    let _params = Params::new(
        memory_cost,
        iterations,
        1, // parallelism
        Some(length),
    ).map_err(|e| SCypherError::crypto(format!("Invalid Argon2 parameters: {:?}", e)))?;

    let _argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, _params);

    // Para el placeholder, generar datos pseudoaleatorios deterministas
    let mut keystream = vec![0u8; length];
    for (i, byte) in keystream.iter_mut().enumerate() {
        *byte = ((password.len() + salt_bytes.len() + i) % 256) as u8;
    }

    Ok(keystream)
}

/// Genera un salt determinista basado en la contraseña
/// Esto asegura que la misma contraseña produzca el mismo resultado
fn generate_deterministic_salt(password: &str) -> Vec<u8> {
    use sha2::{Sha256, Digest};

    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(b"SCYPHER_SALT_V1"); // Valor constante para consistencia

    hasher.finalize().to_vec()
}

/// Valida que los parámetros Argon2id estén en rangos seguros
pub fn validate_argon2_params(iterations: u32, memory_cost: u32) -> Result<()> {
    // Validaciones de rango seguro
    if iterations == 0 || iterations > 100 {
        return Err(SCypherError::InvalidIterations(iterations.to_string()));
    }

    if memory_cost < 8192 || memory_cost > 2_097_152 {  // 8MB - 2GB
        return Err(SCypherError::InvalidMemoryCost(memory_cost.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_keystream() {
        let keystream = derive_keystream("test_password", 32, 3, 65536).unwrap();
        assert_eq!(keystream.len(), 32);

        // Debe ser determinista
        let keystream2 = derive_keystream("test_password", 32, 3, 65536).unwrap();
        assert_eq!(keystream, keystream2);

        // Diferente contraseña debe dar resultado diferente
        let keystream3 = derive_keystream("different_password", 32, 3, 65536).unwrap();
        assert_ne!(keystream, keystream3);
    }

    #[test]
    fn test_validate_argon2_params() {
        // Casos válidos
        assert!(validate_argon2_params(3, 65536).is_ok());
        assert!(validate_argon2_params(5, 131072).is_ok());

        // Casos inválidos
        assert!(validate_argon2_params(0, 65536).is_err());
        assert!(validate_argon2_params(101, 65536).is_err());
        assert!(validate_argon2_params(5, 4096).is_err());
        assert!(validate_argon2_params(5, 3_000_000).is_err());
    }

    #[test]
    fn test_deterministic_salt() {
        let salt1 = generate_deterministic_salt("password");
        let salt2 = generate_deterministic_salt("password");
        assert_eq!(salt1, salt2);

        let salt3 = generate_deterministic_salt("different");
        assert_ne!(salt1, salt3);
    }
}

// src/lib.rs - Interfaz pública de la biblioteca SCypher

//! # SCypher
//!
//! SCypher es una implementación en Rust de un cifrador XOR para frases semilla BIP39.
//! Proporciona transformación reversible segura usando derivación de clave Argon2id.
//!
//! ## Características principales
//! - Cifrado XOR simétrico (la misma operación encripta y desencripta)
//! - Derivación de clave con Argon2id resistente a ataques de hardware
//! - Preservación de checksums BIP39 válidos
//! - Limpieza segura de memoria
//! - Sin dependencias de red (operación completamente offline)
//!
//! ## Ejemplo de uso
//! ```rust,no_run
//! use scypher_rust::{transform_seed, SCypherError};
//!
//! let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
//! let password = "my_secure_password";
//! let iterations = 5;
//! let memory_cost = 131072; // 128MB
//!
//! // Encriptar
//! let encrypted = transform_seed(seed, password, iterations, memory_cost)?;
//!
//! // Desencriptar (misma función debido a XOR)
//! let decrypted = transform_seed(&encrypted, password, iterations, memory_cost)?;
//!
//! assert_eq!(seed, decrypted);
//! # Ok::<(), SCypherError>(())
//! ```

// Módulos públicos
pub mod error;
pub mod crypto;
pub mod bip39;
pub mod security;

// Re-exportaciones públicas para facilitar el uso
pub use error::{SCypherError, Result};
pub use crypto::transform_seed;
pub use bip39::{validate_seed_phrase_complete as validate_seed_phrase, verify_checksum};
pub use security::{SecureString, SecureBytes};

/// Versión de la biblioteca
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Función de conveniencia que combina todas las operaciones
/// Esta es la interfaz principal para usuarios de la biblioteca
pub fn scypher_transform(
    seed_phrase: &str,
    password: &str,
    iterations: Option<u32>,
    memory_cost: Option<u32>,
) -> Result<String> {
    let iterations = iterations.unwrap_or(5);
    let memory_cost = memory_cost.unwrap_or(131072); // 128MB

    // Validar entrada
    bip39::validate_seed_phrase_complete(seed_phrase)?;

    // Realizar transformación
    transform_seed(seed_phrase, password, iterations, memory_cost)
}

/// Configuración predeterminada para SCypher
pub struct SCypherConfig {
    pub iterations: u32,
    pub memory_cost: u32,
    pub verify_checksum: bool,
}

impl Default for SCypherConfig {
    fn default() -> Self {
        Self {
            iterations: 5,
            memory_cost: 131072, // 128MB
            verify_checksum: true,
        }
    }
}

/// Builder para configuración avanzada
pub struct SCypherBuilder {
    config: SCypherConfig,
}

impl SCypherBuilder {
    pub fn new() -> Self {
        Self {
            config: SCypherConfig::default(),
        }
    }

    pub fn iterations(mut self, iterations: u32) -> Self {
        self.config.iterations = iterations;
        self
    }

    pub fn memory_cost(mut self, memory_cost: u32) -> Self {
        self.config.memory_cost = memory_cost;
        self
    }

    pub fn verify_checksum(mut self, verify: bool) -> Self {
        self.config.verify_checksum = verify;
        self
    }

    pub fn transform(&self, seed_phrase: &str, password: &str) -> Result<String> {
        if self.config.verify_checksum {
            bip39::validate_seed_phrase_complete(seed_phrase)?;
        }

        transform_seed(
            seed_phrase,
            password,
            self.config.iterations,
            self.config.memory_cost,
        )
    }
}

impl Default for SCypherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.contains('.'));
    }

    #[test]
    fn test_default_config() {
        let config = SCypherConfig::default();
        assert_eq!(config.iterations, 5);
        assert_eq!(config.memory_cost, 131072);
        assert!(config.verify_checksum);
    }

    #[test]
    fn test_builder_pattern() {
        let builder = SCypherBuilder::new()
            .iterations(10)
            .memory_cost(262144)
            .verify_checksum(false);

        assert_eq!(builder.config.iterations, 10);
        assert_eq!(builder.config.memory_cost, 262144);
        assert!(!builder.config.verify_checksum);
    }
}

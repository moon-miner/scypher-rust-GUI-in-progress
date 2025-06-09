// src/error.rs - Manejo centralizado de errores para SCypher

use std::fmt;
use serde::{Serialize, Deserialize};

/// Enum principal que representa todos los posibles errores en SCypher
#[derive(Debug, Serialize, Deserialize)]
pub enum SCypherError {
    // Errores de entrada y validación
    InvalidSeedPhrase,
    InvalidWordCount(usize),           // Guarda el número de palabras encontradas
    InvalidBip39Word(String),          // Guarda la palabra inválida
    InvalidChecksum,

    // Errores de entrada del usuario
    InvalidPassword,
    PasswordMismatch,
    InvalidIterations(String),         // Guarda el valor inválido
    InvalidMemoryCost(String),         // Guarda el valor inválido

    // Errores criptográficos
    CryptoError(String),               // Errores de Argon2 u otras operaciones crypto
    KeyDerivationFailed,

    // Errores de E/O
    IoError(String),                   // Convertimos std::io::Error a String para Serialize
    FileError(String),

    // Errores del sistema
    InsufficientMemory,
    UnsupportedPlatform,
}

impl fmt::Display for SCypherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Errores de validación BIP39
            SCypherError::InvalidSeedPhrase => {
                write!(f, "Invalid seed phrase format")
            }
            SCypherError::InvalidWordCount(count) => {
                write!(f, "Invalid word count: found {} words (expected: 12, 15, 18, 21, or 24)", count)
            }
            SCypherError::InvalidBip39Word(word) => {
                write!(f, "Word '{}' is not in the BIP39 wordlist", word)
            }
            SCypherError::InvalidChecksum => {
                write!(f, "Invalid BIP39 checksum - seed phrase may be corrupted")
            }

            // Errores de entrada del usuario
            SCypherError::InvalidPassword => {
                write!(f, "Password does not meet security requirements")
            }
            SCypherError::PasswordMismatch => {
                write!(f, "Passwords do not match")
            }
            SCypherError::InvalidIterations(val) => {
                write!(f, "Invalid iteration count '{}' (must be a positive number)", val)
            }
            SCypherError::InvalidMemoryCost(val) => {
                write!(f, "Invalid memory cost '{}' (must be a positive number in KB)", val)
            }

            // Errores criptográficos
            SCypherError::CryptoError(msg) => {
                write!(f, "Cryptographic error: {}", msg)
            }
            SCypherError::KeyDerivationFailed => {
                write!(f, "Failed to derive encryption key")
            }

            // Errores de E/O
            SCypherError::IoError(msg) => {
                write!(f, "IO error: {}", msg)
            }
            SCypherError::FileError(msg) => {
                write!(f, "File error: {}", msg)
            }

            // Errores del sistema
            SCypherError::InsufficientMemory => {
                write!(f, "Insufficient system memory for secure operation")
            }
            SCypherError::UnsupportedPlatform => {
                write!(f, "This platform is not supported")
            }
        }
    }
}

impl std::error::Error for SCypherError {}

// Conversión automática desde std::io::Error (convertimos a String)
impl From<std::io::Error> for SCypherError {
    fn from(error: std::io::Error) -> Self {
        SCypherError::IoError(error.to_string())
    }
}

// Conversión desde errores de Argon2
impl From<argon2::Error> for SCypherError {
    fn from(error: argon2::Error) -> Self {
        SCypherError::CryptoError(format!("Argon2 error: {:?}", error))
    }
}

// La conversión a InvokeError se hace automáticamente por Tauri
// ya que SCypherError implementa Serialize

/// Tipo Result personalizado para SCypher
/// Esto nos permite escribir `Result<T>` en lugar de `Result<T, SCypherError>`
pub type Result<T> = std::result::Result<T, SCypherError>;

/// Funciones helper para crear errores comunes de manera más fácil
impl SCypherError {
    /// Crear error de palabra BIP39 inválida
    pub fn invalid_word<S: Into<String>>(word: S) -> Self {
        SCypherError::InvalidBip39Word(word.into())
    }

    /// Crear error criptográfico con mensaje personalizado
    pub fn crypto<S: Into<String>>(msg: S) -> Self {
        SCypherError::CryptoError(msg.into())
    }

    /// Crear error de archivo con mensaje personalizado
    pub fn file<S: Into<String>>(msg: S) -> Self {
        SCypherError::FileError(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = SCypherError::InvalidWordCount(10);
        assert!(error.to_string().contains("10"));

        let error = SCypherError::InvalidBip39Word("invalid".to_string());
        assert!(error.to_string().contains("invalid"));
    }

    #[test]
    fn test_error_helpers() {
        let error = SCypherError::invalid_word("test");
        match error {
            SCypherError::InvalidBip39Word(word) => assert_eq!(word, "test"),
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_serialization() {
        let error = SCypherError::InvalidSeedPhrase;
        // Test que se puede serializar (requerido por Tauri)
        let serialized = serde_json::to_string(&error);
        assert!(serialized.is_ok());
    }
}

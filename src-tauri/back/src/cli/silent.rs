//! Modo silent para scripting sin prompts interactivos

use std::io::{self, Read};
use crate::error::{SCypherError, Result};

/// Lee seed phrase desde stdin sin prompts
pub fn read_seed_from_stdin() -> Result<String> {
    let mut buffer = String::new();

    // Leer solo la primera línea para la seed phrase
    io::stdin()
        .read_line(&mut buffer)
        .map_err(|e| SCypherError::IoError(e))?;

    // Limpiar input: remover salto de línea y espacios extra
    let cleaned = buffer.trim().to_string();

    if cleaned.is_empty() {
        return Err(SCypherError::InvalidSeedPhrase);
    }

    Ok(cleaned)
}

/// Lee contraseña desde stdin sin prompts
pub fn read_password_from_stdin() -> Result<String> {
    let mut password = String::new();

    io::stdin()
        .read_line(&mut password)
        .map_err(|e| SCypherError::IoError(e))?;

    let password = password.trim().to_string();

    if password.is_empty() {
        return Err(SCypherError::InvalidPassword);
    }

    Ok(password)
}

/// Detecta automáticamente si el input es un archivo o una frase
pub fn detect_input_type(input: &str) -> InputType {
    // Regla 1: Si existe como archivo, es archivo
    if std::path::Path::new(input).exists() {
        return InputType::File(input.to_string());
    }

    // Regla 2: Si contiene extensión común, probablemente archivo
    if input.contains('.') && has_file_extension(input) {
        return InputType::ProbableFile(input.to_string());
    }

    // Regla 3: Contar palabras válidas BIP39
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.len() >= 12 && words.len() <= 24 {
        let valid_count = words.iter()
            .filter(|&word| crate::bip39::is_valid_word(word))
            .count();

        let validity_ratio = valid_count as f64 / words.len() as f64;
        if validity_ratio >= 0.8 {
            return InputType::SeedPhrase(input.to_string());
        }
    }

    // Por defecto: tratar como frase
    InputType::SeedPhrase(input.to_string())
}

/// Verifica si el input tiene una extensión de archivo común
fn has_file_extension(input: &str) -> bool {
    let common_extensions = [".txt", ".seed", ".dat", ".key", ".phrase", ".bip39"];

    common_extensions.iter().any(|ext| {
        input.to_lowercase().ends_with(ext)
    })
}

/// Tipos de input detectados
#[derive(Debug, Clone)]
pub enum InputType {
    File(String),
    ProbableFile(String),
    SeedPhrase(String),
}

/// Lee número de iteraciones desde stdin sin prompts
pub fn read_iterations_from_stdin() -> Result<u32> {
    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .map_err(|e| SCypherError::IoError(e))?;

    let iterations_str = input.trim();

    iterations_str.parse::<u32>()
        .map_err(|_| SCypherError::InvalidIterations(iterations_str.to_string()))
}

impl InputType {
    /// Obtiene el contenido según el tipo detectado
    pub fn get_content(&self) -> Result<String> {
        match self {
            InputType::File(path) => {
                crate::cli::read_seed_from_file(path)
            }
            InputType::ProbableFile(path) => {
                match crate::cli::read_seed_from_file(path) {
                    Ok(content) => Ok(content),
                    Err(_) => Ok(path.clone()),
                }
            }
            InputType::SeedPhrase(phrase) => Ok(phrase.clone()),
        }
    }
}

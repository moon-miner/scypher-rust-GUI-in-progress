use tauri::command;
use serde::{Deserialize, Serialize};
use crate::error::{SCypherError, Result};

#[derive(Serialize, Deserialize)]
pub struct SeedValidation {
    pub valid: bool,
    pub word_count: usize,
    pub message: String,
    pub status: String, // "valid", "invalid", "progress", "empty"
}

#[derive(Serialize, Deserialize)]
pub struct ProcessResult {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
}

/// Validar frase semilla BIP39 completa
#[command]
pub fn validate_seed_phrase(phrase: String) -> SeedValidation {
    let word_count = phrase.split_whitespace().count();

    if phrase.trim().is_empty() {
        return SeedValidation {
            valid: false,
            word_count: 0,
            message: "Ready to input seed phrase • AUTO mode active".to_string(),
            status: "empty".to_string(),
        };
    }

    match crate::bip39::validate_seed_phrase_complete(&phrase) {
        Ok(()) => SeedValidation {
            valid: true,
            word_count,
            message: format!("✅ Valid BIP39 seed phrase ({} words) with correct checksum", word_count),
            status: "valid".to_string(),
        },
        Err(SCypherError::InvalidWordCount(count)) => SeedValidation {
            valid: false,
            word_count: count,
            message: format!("Invalid word count: found {} words (expected: 12, 15, 18, 21, or 24)", count),
            status: "invalid".to_string(),
        },
        Err(SCypherError::InvalidBip39Word(word)) => SeedValidation {
            valid: false,
            word_count,
            message: format!("Invalid BIP39 word: '{}'", word),
            status: "invalid".to_string(),
        },
        Err(SCypherError::InvalidChecksum) => SeedValidation {
            valid: false,
            word_count,
            message: "Invalid BIP39 checksum - seed phrase may be corrupted".to_string(),
            status: "invalid".to_string(),
        },
        Err(e) => SeedValidation {
            valid: false,
            word_count,
            message: format!("Validation error: {}", e),
            status: "invalid".to_string(),
        },
    }
}

/// Transformar frase semilla usando XOR
#[command]
pub fn transform_seed_phrase(
    phrase: String,
    password: String,
    iterations: u32,
    memory_cost: u32,
) -> ProcessResult {
    match crate::crypto::transform_seed(&phrase, &password, iterations, memory_cost) {
        Ok(result) => ProcessResult {
            success: true,
            result: Some(result),
            error: None,
        },
        Err(e) => ProcessResult {
            success: false,
            result: None,
            error: Some(e.to_string()),
        },
    }
}

/// Obtener lista completa de palabras BIP39
#[command]
pub fn get_bip39_wordlist() -> Vec<String> {
    crate::bip39::BIP39_WORDLIST.iter().map(|s| s.to_string()).collect()
}

/// Validar palabra individual BIP39
#[command]
pub fn validate_bip39_word(word: String) -> bool {
    crate::bip39::is_valid_word(&word)
}

/// Obtener sugerencias para palabra incorrecta
#[command]
pub fn get_word_suggestions(word: String) -> Vec<String> {
    if let Some((closest, distance)) = crate::bip39::wordlist::find_closest_word(&word) {
        if distance <= 2 {
            vec![closest.to_string()]
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    }
}

/// Leer archivo de semilla (sin async para Tauri v1)
#[command]
pub fn read_seed_file(path: String) -> Result<String> {
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let cleaned = content
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<&str>>()
                .join(" ");
            Ok(cleaned)
        }
        Err(e) => Err(SCypherError::file(format!("Cannot read file: {}", e))),
    }
}

/// Guardar resultado en archivo (sin async para Tauri v1)
#[command]
pub fn save_result_file(content: String, path: String) -> Result<()> {
    use std::fs;
    fs::write(&path, &content)
        .map_err(|e| SCypherError::file(format!("Cannot write file: {}", e)))?;

    // Establecer permisos seguros en sistemas Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)
            .map_err(|e| SCypherError::file(format!("Cannot read metadata: {}", e)))?
            .permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)
            .map_err(|e| SCypherError::file(format!("Cannot set permissions: {}", e)))?;
    }

    Ok(())
}

/// Abrir diálogo moderno de archivo (usa XDG Portal en Linux)
#[command]
pub async fn open_file_dialog() -> Result<Option<String>> {
    use rfd::AsyncFileDialog;

    let file = AsyncFileDialog::new()
        .add_filter("Text files", &["txt"])
        .add_filter("All files", &["*"])
        .set_title("Select seed phrase file")
        .pick_file()
        .await;

    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}

/// Abrir diálogo moderno de guardar archivo (usa XDG Portal en Linux)
#[command]
pub async fn save_file_dialog() -> Result<Option<String>> {
    use rfd::AsyncFileDialog;

    let file = AsyncFileDialog::new()
        .add_filter("Text files", &["txt"])
        .set_file_name("scypher_result.txt")
        .set_title("Save transformation result")
        .save_file()
        .await;

    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}

/// Generar nueva frase semilla BIP39 válida
#[command]
pub fn generate_seed_phrase(word_count: serde_json::Value) -> Result<String> {
    // Parsear el word_count de manera flexible
    let count: usize = match word_count {
        serde_json::Value::Number(n) => {
            if let Some(num) = n.as_u64() {
                num as usize
            } else {
                return Err(SCypherError::crypto("Invalid word count number".to_string()));
            }
        }
        serde_json::Value::String(s) => {
            s.parse::<usize>()
                .map_err(|_| SCypherError::crypto(format!("Cannot parse '{}' as number", s)))?
        }
        _ => return Err(SCypherError::crypto("Word count must be a number or string".to_string())),
    };

    // Validar rango de palabras válidas para BIP39
    let valid_counts = [12, 15, 18, 21, 24];
    if !valid_counts.contains(&count) {
        return Err(SCypherError::InvalidWordCount(count));
    }

    // Calcular entropía necesaria según BIP39
    let entropy_bits = count * 32 / 3;  // 128, 160, 192, 224, 256 bits
    let entropy_bytes = entropy_bits / 8;  // 16, 20, 24, 28, 32 bytes

    // Generar entropía criptográficamente segura
    let mut entropy = vec![0u8; entropy_bytes];
    use rand::RngCore;
    rand::thread_rng().fill_bytes(&mut entropy);

    // Convertir entropía a frase BIP39 válida (con checksum correcto)
    // USAR PATH COMPLETO para evitar problemas de scope
    crate::bip39::conversion::entropy_to_phrase(&entropy)
}

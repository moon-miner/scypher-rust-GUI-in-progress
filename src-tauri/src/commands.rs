use tauri::command;
use tokio::task;
use serde::{Deserialize, Serialize};
use crate::error::{SCypherError, Result};
use crate::addresses::{derive_addresses as derive_addr, AddressSet};

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

/// Transformar frase semilla usando XOR - VERSIÓN ASYNC NO BLOQUEANTE
#[command]
pub async fn transform_seed_phrase(
    phrase: String,
    password: String,
    iterations: u32,
    memory_cost: u32,
) -> ProcessResult {
    // Ejecutar Argon2id en thread separado para no bloquear UI
    let result = task::spawn_blocking(move || {
        // LA MISMA LÓGICA CRIPTOGRÁFICA EXACTA - SIN CAMBIOS
        crate::crypto::transform_seed(&phrase, &password, iterations, memory_cost)
    }).await;

    match result {
        Ok(Ok(transformed)) => ProcessResult {
            success: true,
            result: Some(transformed),
            error: None,
        },
        Ok(Err(e)) => ProcessResult {
            success: false,
            result: None,
            error: Some(e.to_string()),
        },
        Err(e) => ProcessResult {
            success: false,
            result: None,
            error: Some(format!("Task error: {}", e)),
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
    crate::bip39::conversion::entropy_to_phrase(&entropy)
}

/// Derivar direcciones HD Wallet con configuración individual por red
#[command]
pub fn derive_addresses_with_config(
    seed_phrase: String,
    passphrase: Option<String>,
    network_configs: std::collections::HashMap<String, crate::addresses::NetworkConfig>,
) -> Result<AddressSet> {
    crate::addresses::derive_addresses_with_config(
        &seed_phrase,
        passphrase.as_deref(),
        network_configs
    )
}

/// Derivar direcciones HD Wallet para múltiples redes (ACTUALIZADA)
#[command]
pub fn derive_addresses(
    seed_phrase: String,
    passphrase: Option<String>,
    networks: Vec<String>,
    address_count: u32, // NUEVO PARÁMETRO
) -> Result<AddressSet> {
    // Validar address_count
    let count = if address_count < 1 { 1 } else if address_count > 100 { 100 } else { address_count };

    // Crear configuración usando el count especificado
    let mut network_configs = std::collections::HashMap::new();
    for network in networks {
        network_configs.insert(network, crate::addresses::NetworkConfig {
            count,
            use_passphrase: true, // Será aplicado solo a redes que lo soporten
        });
    }

    crate::addresses::derive_addresses_with_config(
        &seed_phrase,
        passphrase.as_deref(),
        network_configs
    )
}

/// Validar que una red sea soportada
#[command]
pub fn validate_network(network: String) -> bool {
    matches!(network.as_str(),
        "bitcoin" | "ethereum" | "ergo" |
        "bsc" | "polygon" | "cardano" |
        "dogecoin" | "litecoin" | "solana"
    )
}

/// Obtener información sobre redes soportadas
#[command]
pub fn get_supported_networks() -> Vec<NetworkInfo> {
    vec![
        // REDES EXISTENTES
        NetworkInfo {
            id: "bitcoin".to_string(),
            name: "Bitcoin".to_string(),
            symbol: "₿".to_string(),
            coin_type: 0,
            description: "Bitcoin mainnet addresses".to_string(),
        },
        NetworkInfo {
            id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            symbol: "Ξ".to_string(),
            coin_type: 60,
            description: "Ethereum mainnet addresses".to_string(),
        },
        NetworkInfo {
            id: "ergo".to_string(),
            name: "Ergo".to_string(),
            symbol: "⚡".to_string(),
            coin_type: 429,
            description: "Ergo platform addresses".to_string(),
        },

        // NUEVAS REDES
        NetworkInfo {
            id: "bsc".to_string(),
            name: "Binance Smart Chain".to_string(),
            symbol: "🟡".to_string(),
            coin_type: 60, // Mismo que Ethereum (EVM compatible)
            description: "BSC mainnet addresses (EVM compatible)".to_string(),
        },
        NetworkInfo {
            id: "polygon".to_string(),
            name: "Polygon".to_string(),
            symbol: "🔷".to_string(),
            coin_type: 60, // Mismo que Ethereum (EVM compatible)
            description: "Polygon mainnet addresses (EVM compatible)".to_string(),
        },
        NetworkInfo {
            id: "cardano".to_string(),
            name: "Cardano".to_string(),
            symbol: "₳".to_string(),
            coin_type: 1815,
            description: "Cardano Shelley addresses".to_string(),
        },
        NetworkInfo {
            id: "dogecoin".to_string(),
            name: "Dogecoin".to_string(),
            symbol: "Ð".to_string(),
            coin_type: 3,
            description: "Dogecoin mainnet addresses".to_string(),
        },
        NetworkInfo {
            id: "litecoin".to_string(),
            name: "Litecoin".to_string(),
            symbol: "Ł".to_string(),
            coin_type: 2,
            description: "Litecoin mainnet addresses".to_string(),
        },
        NetworkInfo {
            id: "solana".to_string(),
            name: "Solana".to_string(),
            symbol: "◎".to_string(),
            coin_type: 501,
            description: "Solana mainnet addresses".to_string(),
        },
    ]
}

/// Información sobre una red soportada
#[derive(Serialize, Deserialize)]
pub struct NetworkInfo {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub coin_type: u32,
    pub description: String,
}

// Prevenir ventana de consola en builds de release de Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

// Importar tus módulos existentes
mod crypto;
mod bip39;
mod cli;
mod security;
mod error;

// Re-exportar funciones principales
pub use error::{SCypherError, Result};
pub use crypto::transform_seed;

fn main() {
    // Configurar limpieza de seguridad
    security::setup_security_cleanup();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::validate_seed_phrase,
            commands::transform_seed_phrase,
            commands::get_bip39_wordlist,
            commands::validate_bip39_word,
            commands::get_word_suggestions,
            commands::read_seed_file,
            commands::save_result_file,
            commands::open_file_dialog,
            commands::save_file_dialog,
            commands::generate_seed_phrase,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

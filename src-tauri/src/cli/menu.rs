// src/cli/menu.rs - Sistema de menús interactivo

use crate::cli::display::{self, colors};
use crate::error::Result;
use std::process;

/// Opciones del menú principal
#[derive(Debug, Clone, Copy)]
pub enum MainMenuChoice {
    ProcessSeed = 1,
    Help = 2,
    Exit = 3,
}

/// Opciones del submenú de ayuda
#[derive(Debug, Clone, Copy)]
pub enum HelpMenuChoice {
    License = 1,
    Details = 2,
    Examples = 3,
    Compatibility = 4,
    ReturnToMain = 5,
}

/// Opciones del menú post-procesamiento
#[derive(Debug, Clone, Copy)]
pub enum PostProcessChoice {
    SaveToFile = 1,
    ReturnToMain = 2,
    Exit = 3,
}

/// Opciones después de guardar archivo
#[derive(Debug, Clone, Copy)]
pub enum PostSaveChoice {
    ReturnToMain = 1,
    Exit = 2,
}

/// Estado del sistema de menús para controlar flujo
#[derive(Debug, Clone)]
pub struct MenuState {
    pub should_exit: bool,
    pub return_to_main: bool,
    pub processed_result: Option<String>,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            should_exit: false,
            return_to_main: false,
            processed_result: None,
        }
    }
}

/// Mostrar y manejar el menú principal
pub fn show_main_menu() -> Result<MainMenuChoice> {
    loop {
        display::clear_screen();
        display::show_banner();

        // Opciones del menú
        println!("{}Main Menu:{}", colors::SUCCESS, colors::RESET);
        println!("1. Encrypt/Decrypt seed phrase");
        println!("2. Help/License/Details");
        println!("3. Exit");
        println!();

        let choice = display::read_user_input("Select option [1-3]: ");
        println!();

        match choice.as_str() {
            "1" => return Ok(MainMenuChoice::ProcessSeed),
            "2" => return Ok(MainMenuChoice::Help),
            "3" | "" => return Ok(MainMenuChoice::Exit),
            _ => {
                println!("{}Invalid option. Please select 1-3.{}", colors::ERROR, colors::RESET);
                println!();
                display::wait_for_enter();
            }
        }
    }
}

/// Mostrar y manejar el submenú de ayuda/licencia
pub fn show_help_submenu() -> Result<HelpMenuChoice> {
    loop {
        display::clear_screen();
        println!("{}Help/License/Details{}", colors::BRIGHT, colors::RESET);
        println!("{}===================={}", colors::FRAME, colors::RESET);
        println!();
        println!("1. Show license and disclaimer");
        println!("2. Show detailed cipher explanation");
        println!("3. Show usage examples");
        println!("4. Show system compatibility");
        println!("5. Return to main menu");
        println!();

        let choice = display::read_user_input("Select option [1-5]: ");
        println!();

        match choice.as_str() {
            "1" => return Ok(HelpMenuChoice::License),
            "2" => return Ok(HelpMenuChoice::Details),
            "3" => return Ok(HelpMenuChoice::Examples),
            "4" => return Ok(HelpMenuChoice::Compatibility),
            "5" | "" => return Ok(HelpMenuChoice::ReturnToMain),
            _ => {
                println!("{}Invalid option. Please select 1-5.{}", colors::ERROR, colors::RESET);
                println!();
                display::wait_for_enter();
            }
        }
    }
}

/// Manejar el submenú de ayuda con navegación completa
pub fn handle_help_submenu() -> Result<bool> {
    loop {
        match show_help_submenu()? {
            HelpMenuChoice::License => {
                display::show_license_text();
            }
            HelpMenuChoice::Details => {
                display::show_cipher_details();
            }
            HelpMenuChoice::Examples => {
                display::show_usage_examples();
            }
            HelpMenuChoice::Compatibility => {
                display::show_compatibility_info();
            }
            HelpMenuChoice::ReturnToMain => {
                return Ok(false); // No salir, volver al menú principal
            }
        }
    }
}

/// Mostrar menú post-procesamiento después de una operación exitosa
pub fn show_post_processing_menu(result: &str) -> Result<PostProcessChoice> {
    loop {
        println!();
        println!("{}What would you like to do next?{}", colors::SUCCESS, colors::RESET);
        println!("1. Save result to file");
        println!("2. Return to main menu");
        println!("3. Exit");
        println!();

        let choice = display::read_user_input("Select option [1-3]: ");
        println!();

        match choice.as_str() {
            "1" => return Ok(PostProcessChoice::SaveToFile),
            "2" => return Ok(PostProcessChoice::ReturnToMain),
            "3" | "" => return Ok(PostProcessChoice::Exit),
            _ => {
                println!("{}Invalid option. Please select 1-3.{}", colors::ERROR, colors::RESET);
                println!();
                display::wait_for_enter();
            }
        }
    }
}

/// Manejar guardado de resultado en archivo
pub fn handle_save_result(result: &str) -> Result<bool> {
    loop {
        println!("{}Enter filename to save result:{}", colors::PRIMARY, colors::RESET);
        let save_file = display::read_user_input("> ");
        println!();

        // Validar entrada
        if save_file.is_empty() {
            println!("{}Error: Filename cannot be empty{}", colors::ERROR, colors::RESET);
            println!();
            display::wait_for_enter();
            continue;
        }

        // Auto-añadir extensión .txt si no está presente
        let save_file = if save_file.ends_with(".txt") {
            save_file
        } else {
            format!("{}.txt", save_file)
        };

        // Intentar guardar el archivo usando la función del módulo output
        match crate::cli::output::save_to_file(result, &save_file) {
            Ok(()) => {
                println!("{}✓ Result successfully saved to {}{}",
                         colors::SUCCESS, save_file, colors::RESET);

                // Mostrar menú post-guardado
                return handle_post_save_menu();
            }
            Err(e) => {
                println!("{}Error: Failed to save file: {}{}", colors::ERROR, e, colors::RESET);
                println!();
                display::wait_for_enter();
                continue;
            }
        }
    }
}

/// Mostrar menú después de guardar archivo exitosamente
pub fn show_post_save_menu() -> Result<PostSaveChoice> {
    loop {
        println!();
        println!("{}File saved successfully. What would you like to do next?{}",
                 colors::SUCCESS, colors::RESET);
        println!("1. Return to main menu");
        println!("2. Exit");
        println!();

        let choice = display::read_user_input("Select option [1-2]: ");
        println!();

        match choice.as_str() {
            "1" => return Ok(PostSaveChoice::ReturnToMain),
            "2" | "" => return Ok(PostSaveChoice::Exit),
            _ => {
                println!("{}Invalid option. Please select 1-2.{}", colors::ERROR, colors::RESET);
                println!();
                display::wait_for_enter();
            }
        }
    }
}

/// Manejar menú post-guardado
pub fn handle_post_save_menu() -> Result<bool> {
    match show_post_save_menu()? {
        PostSaveChoice::ReturnToMain => Ok(false), // No salir
        PostSaveChoice::Exit => Ok(true),         // Salir
    }
}

/// Manejar el menú post-procesamiento completo
pub fn handle_post_processing_menu(result: &str) -> Result<bool> {
    loop {
        match show_post_processing_menu(result)? {
            PostProcessChoice::SaveToFile => {
                if handle_save_result(result)? {
                    return Ok(true); // Usuario eligió salir después de guardar
                }
                // Si no salió, volver al menú principal
                return Ok(false);
            }
            PostProcessChoice::ReturnToMain => {
                display::clear_screen();
                return Ok(false); // Volver al menú principal
            }
            PostProcessChoice::Exit => {
                println!("{}Exiting...{}", colors::DIM, colors::RESET);
                std::thread::sleep(std::time::Duration::from_millis(1000));
                display::clear_screen();
                return Ok(true); // Salir
            }
        }
    }
}

/// Función principal del sistema de menús - maneja todo el flujo
pub fn run_interactive_menu() -> Result<MenuState> {
    let mut state = MenuState::default();

    loop {
        match show_main_menu()? {
            MainMenuChoice::ProcessSeed => {
                // Retornar al main para que ejecute el procesamiento
                state.return_to_main = true;
                return Ok(state);
            }
            MainMenuChoice::Help => {
                if handle_help_submenu()? {
                    // Si help submenu retorna true, significa salir
                    state.should_exit = true;
                    return Ok(state);
                }
                // Si retorna false, continuar en el loop del menú principal
            }
            MainMenuChoice::Exit => {
                println!("{}Exiting...{}", colors::DIM, colors::RESET);
                std::thread::sleep(std::time::Duration::from_millis(1000));
                display::clear_screen();
                crate::security::secure_cleanup();
                process::exit(0);
            }
        }
    }
}

/// Función utilitaria para manejo de errores en menús
pub fn handle_menu_error(error_message: &str) {
    println!("{}✗ Error: {}{}", colors::ERROR, error_message, colors::RESET);
    println!();
    display::wait_for_enter();
    display::clear_screen();
}

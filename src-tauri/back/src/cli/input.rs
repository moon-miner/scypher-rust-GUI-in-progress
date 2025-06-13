// src/cli/input.rs - Manejo seguro de entrada del usuario

use std::io::{self, Write};
use rpassword::read_password;
use crate::error::{SCypherError, Result};

const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_SEED_LENGTH: usize = 1000; // Límite razonable para frases semilla

/// Lee la frase semilla de forma interactiva
pub fn read_seed_interactive(is_decrypt_mode: bool) -> Result<String> {
    let prompt = if is_decrypt_mode {
        "\nEnter encrypted seed phrase to decrypt:"
    } else {
        "\nEnter seed phrase to encrypt:"
    };

    println!("{}", prompt);
    print!("> ");
    io::stdout().flush().map_err(SCypherError::from)?;

    // Leer una sola línea directamente
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(SCypherError::from)?;

    let seed_phrase = input.trim().to_string();

    // Verificar si es un archivo
    if seed_phrase.ends_with(".txt") && std::path::Path::new(&seed_phrase).exists() {
        println!("Reading from file: {}", seed_phrase);
        return read_seed_from_file(&seed_phrase);
    }

    if seed_phrase.is_empty() {
        return Err(SCypherError::InvalidSeedPhrase);
    }

    validate_seed_input(&seed_phrase)?;
    Ok(seed_phrase)
}

/// Lee la frase semilla desde un archivo
pub fn read_seed_from_file(file_path: &str) -> Result<String> {
    println!("Reading seed phrase from file: {}", file_path);

    let content = std::fs::read_to_string(file_path)
        .map_err(|e| SCypherError::file(format!("Cannot read file '{}': {}", file_path, e)))?;

    // Limpiar contenido: remover saltos de línea excesivos y espacios
    let seed_phrase = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join(" ");

    if seed_phrase.is_empty() {
        return Err(SCypherError::file("File is empty or contains no valid content".to_string()));
    }

    validate_seed_input(&seed_phrase)?;

    println!("✓ Successfully read {} words from file\n", seed_phrase.split_whitespace().count());
    Ok(seed_phrase)
}

/// Lee la contraseña de forma segura (sin mostrar en pantalla)
pub fn read_password_secure() -> Result<String> {
    println!("Password Requirements:");
    println!("• Minimum {} characters", MIN_PASSWORD_LENGTH);
    println!("• Use a strong, unique password");
    println!("• Remember: same password needed for decryption\n");

    loop {
        print!("Enter password: ");
        io::stdout().flush().map_err(SCypherError::from)?;

        let password = read_password_with_asterisks()?;
        println!(); // Nueva línea después de la entrada

        print!("Confirm password: ");
        io::stdout().flush().map_err(SCypherError::from)?;

        let password_confirm = read_password_with_asterisks()?;
        println!(); // Nueva línea después de la confirmación

        if password != password_confirm {
            println!("❌ Password mismatch. Please try again.\n");
            continue;
        }

        if password.len() < MIN_PASSWORD_LENGTH {
            println!("❌ Password too short (minimum {} characters). Please try again.\n", MIN_PASSWORD_LENGTH);
            continue;
        }

        println!("✓ Password confirmed\n");
        return Ok(password);
    }
}

/// Función mejorada para leer contraseña con asteriscos
fn read_password_with_asterisks() -> Result<String> {
    use std::io::Read;

    let mut password = String::new();

    // Configurar terminal para modo raw
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let stdin_fd = io::stdin().as_raw_fd();

        // Obtener configuración actual del terminal
        let mut termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(stdin_fd, &mut termios) } != 0 {
            // Si falla, usar rpassword como fallback
            return Ok(rpassword::read_password().map_err(|e|
                SCypherError::crypto(format!("Failed to read password: {}", e)))?);
        }

        // Guardar configuración original
        let original_termios = termios;

        // Deshabilitar echo y modo canónico
        termios.c_lflag &= !(libc::ECHO | libc::ICANON);

        if unsafe { libc::tcsetattr(stdin_fd, libc::TCSANOW, &termios) } != 0 {
            return Ok(rpassword::read_password().map_err(|e|
                SCypherError::crypto(format!("Failed to read password: {}", e)))?);
        }

        // Leer caracteres uno por uno
        let stdin = io::stdin();
        for byte in stdin.bytes() {
            match byte {
                Ok(b'\n') | Ok(b'\r') => break,
                Ok(127) | Ok(8) => { // Backspace o DEL
                    if !password.is_empty() {
                        password.pop();
                        print!("\x08 \x08"); // Borrar asterisco
                        io::stdout().flush().unwrap_or(());
                    }
                }
                Ok(b) if b >= 32 && b <= 126 => { // Caracteres imprimibles
                    password.push(b as char);
                    print!("*");
                    io::stdout().flush().unwrap_or(());
                }
                Ok(_) => {} // Ignorar otros caracteres de control
                Err(_) => break,
            }
        }

        // Restaurar configuración original del terminal
        unsafe { libc::tcsetattr(stdin_fd, libc::TCSANOW, &original_termios) };
    }

    #[cfg(not(unix))]
    {
        // En Windows o otros sistemas, usar rpassword como fallback
        return Ok(rpassword::read_password().map_err(|e|
            SCypherError::crypto(format!("Failed to read password: {}", e)))?);
    }

    Ok(password)
}

/// Validar entrada de frase semilla
fn validate_seed_input(seed_phrase: &str) -> Result<()> {
    // Verificar longitud máxima
    if seed_phrase.len() > MAX_SEED_LENGTH {
        return Err(SCypherError::InvalidSeedPhrase);
    }

    // Verificar que no esté vacía
    if seed_phrase.trim().is_empty() {
        return Err(SCypherError::InvalidSeedPhrase);
    }

    // Si parece ser un archivo, no validar como seed phrase
    if seed_phrase.ends_with(".txt") || seed_phrase.contains("/") || seed_phrase.contains("\\") {
        return Ok(()); // Los archivos se validan en otra función
    }

    // Verificar caracteres básicos (solo letras, números y espacios)
    if !seed_phrase.chars().all(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace()) {
        return Err(SCypherError::InvalidSeedPhrase);
    }

    // Contar palabras
    let word_count = seed_phrase.split_whitespace().count();

    // Verificar que tenga al menos una palabra
    if word_count == 0 {
        return Err(SCypherError::InvalidSeedPhrase);
    }

    // BIP39 especifica estos números de palabras como válidos
    let valid_word_counts = [12, 15, 18, 21, 24];
    if !valid_word_counts.contains(&word_count) {
        return Err(SCypherError::InvalidWordCount(word_count));
    }

    Ok(())
}

/// Utilidad para leer confirmación del usuario (sí/no)
pub fn read_confirmation(prompt: &str) -> Result<bool> {
    loop {
        print!("{} (y/n): ", prompt);
        io::stdout().flush().map_err(SCypherError::from)?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(SCypherError::from)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Please answer 'y' for yes or 'n' for no."),
        }
    }
}

/// Utilidad para leer un número entero con validación
pub fn read_number<T>(prompt: &str, min: T, max: T) -> Result<T>
where
    T: std::str::FromStr + std::cmp::PartialOrd + std::fmt::Display + Copy,
    T::Err: std::fmt::Display,
{
    loop {
        print!("{} ({}-{}): ", prompt, min, max);
        io::stdout().flush().map_err(SCypherError::from)?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(SCypherError::from)?;

        match input.trim().parse::<T>() {
            Ok(num) if num >= min && num <= max => return Ok(num),
            Ok(num) => println!("Number must be between {} and {}, got {}", min, max, num),
            Err(e) => println!("Invalid number: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_seed_input() {
        // Casos válidos
        assert!(validate_seed_input("word1 word2 word3 word4 word5 word6 word7 word8 word9 word10 word11 word12").is_ok());

        // Casos inválidos
        assert!(validate_seed_input("").is_err());                    // Vacío
        assert!(validate_seed_input("   ").is_err());                // Solo espacios
        assert!(validate_seed_input("word1").is_err());              // Solo 1 palabra
        assert!(validate_seed_input("word1 word2 word3 word4 word5 word6 word7 word8 word9 word10 word11").is_err()); // 11 palabras
        assert!(validate_seed_input("word1 word2! word3").is_err()); // Caracteres especiales
    }

    #[test]
    fn test_word_count_validation() {
        let valid_counts = [12, 15, 18, 21, 24];
        for count in valid_counts {
            let words = (0..count).map(|i| format!("word{}", i)).collect::<Vec<_>>().join(" ");
            assert!(validate_seed_input(&words).is_ok());
        }

        // Casos inválidos
        let invalid_counts = [1, 5, 13, 20, 25, 30];
        for count in invalid_counts {
            let words = (0..count).map(|i| format!("word{}", i)).collect::<Vec<_>>().join(" ");
            assert!(validate_seed_input(&words).is_err());
        }
    }
}

// src/cli/output.rs - Manejo de salida y archivos

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use crate::error::{SCypherError, Result};
use crate::cli::input::read_confirmation;

const DEFAULT_EXTENSION: &str = ".txt";
const FILE_PERMISSIONS: u32 = 0o600; // Solo lectura/escritura para el propietario

/// Mostrar resultado y opcionalmente guardarlo en archivo
pub fn output_result(result: &str, output_file: Option<&String>) -> Result<()> {
    // Siempre mostrar el resultado en pantalla
    println!("Result:");
    println!("─────────────────────────────────────────────────────────────");
    println!("{}", result);
    println!("─────────────────────────────────────────────────────────────");

    // Guardar en archivo si se especificó
    if let Some(file_path) = output_file {
        let final_path = ensure_extension(file_path);
        save_to_file(result, &final_path)?;
        println!("\n✓ Result saved to: {}", final_path);
    } else {
        // Preguntar si quiere guardar en archivo
        if read_confirmation("\nDo you want to save the result to a file?")? {
            print!("Enter filename (without extension): ");
            io::stdout().flush().map_err(SCypherError::from)?;

            let mut filename = String::new();
            io::stdin().read_line(&mut filename).map_err(SCypherError::from)?;
            let filename = filename.trim();

            if !filename.is_empty() {
                let file_path = ensure_extension(filename);
                save_to_file(result, &file_path)?;
                println!("✓ Result saved to: {}", file_path);
            }
        }
    }

    Ok(())
}

/// Guardar contenido en archivo con permisos seguros
pub fn save_to_file(content: &str, file_path: &str) -> Result<()> {
    use std::path::Path;

    if file_path.is_empty() {
        return Err(SCypherError::file("File path is empty".to_string()));
    }

    let path = Path::new(file_path);

    // Manejar correctamente el directorio padre
    let parent_dir = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new(".") // Si no hay padre o es vacío, usar directorio actual
    };

    // Verificar que el directorio padre existe
    if !parent_dir.exists() {
        return Err(SCypherError::file(
            format!("Directory '{}' does not exist", parent_dir.display())
        ));
    }

    if !parent_dir.is_dir() {
        return Err(SCypherError::file(
            format!("'{}' is not a directory", parent_dir.display())
        ));
    }

    // Escribir archivo
    fs::write(file_path, content)
        .map_err(|e| SCypherError::file(format!("Cannot write to '{}': {}", file_path, e)))?;

    // Establecer permisos seguros (solo en sistemas Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(file_path)
            .map_err(|e| SCypherError::file(format!("Cannot read file metadata: {}", e)))?
            .permissions();
        perms.set_mode(FILE_PERMISSIONS);
        fs::set_permissions(file_path, perms)
            .map_err(|e| SCypherError::file(format!("Cannot set file permissions: {}", e)))?;
    }

    Ok(())
}

/// Asegurar que el archivo tenga la extensión correcta
fn ensure_extension(file_path: &str) -> String {
    if file_path.ends_with(DEFAULT_EXTENSION) {
        file_path.to_string()
    } else {
        format!("{}{}", file_path, DEFAULT_EXTENSION)
    }
}

/// Validar que una ruta de archivo es segura para escritura
pub fn validate_output_path(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);

    // Verificar que no sea un directorio
    if path.is_dir() {
        return Err(SCypherError::file(
            format!("'{}' is a directory, not a file", file_path)
        ));
    }

    // Verificar que no contenga caracteres peligrosos
    if file_path.contains("..") || file_path.contains("//") {
        return Err(SCypherError::file(
            "File path contains unsafe characters".to_string()
        ));
    }

    // Verificar longitud razonable
    if file_path.len() > 250 {
        return Err(SCypherError::file(
            "File path is too long".to_string()
        ));
    }

    Ok(())
}

/// Mostrar información de archivo antes de guardarlo
pub fn show_file_info(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);

    println!("File Information:");
    println!("• Path: {}", file_path);
    println!("• Directory: {}", path.parent().unwrap_or(Path::new(".")).display());
    println!("• Filename: {}", path.file_name().unwrap().to_string_lossy());

    if path.exists() {
        let metadata = fs::metadata(path)
            .map_err(|e| SCypherError::file(format!("Cannot read file metadata: {}", e)))?;

        println!("• Status: File exists (will be overwritten)");
        println!("• Size: {} bytes", metadata.len());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            println!("• Permissions: {:o}", metadata.permissions().mode() & 0o777);
        }
    } else {
        println!("• Status: New file (will be created)");
    }

    Ok(())
}

/// Utilidades para formateo de salida
pub mod format {
    /// Crear una línea separadora
    pub fn separator_line(length: usize) -> String {
        "─".repeat(length)
    }

    /// Formatear texto en columnas
    pub fn in_columns(text: &str, columns: usize) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut words_in_line = 0;

        for word in words {
            if words_in_line >= columns {
                lines.push(current_line.trim().to_string());
                current_line.clear();
                words_in_line = 0;
            }

            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
            words_in_line += 1;
        }

        if !current_line.is_empty() {
            lines.push(current_line.trim().to_string());
        }

        lines
    }

    /// Formatear resultado para display bonito
    pub fn format_seed_phrase(phrase: &str) -> String {
        let words: Vec<&str> = phrase.split_whitespace().collect();
        let mut formatted = String::new();

        // Mostrar en grupos de 4 palabras por línea (común para seed phrases)
        for (i, word) in words.iter().enumerate() {
            if i > 0 && i % 4 == 0 {
                formatted.push('\n');
            }
            formatted.push_str(&format!("{:2}. {:<12} ", i + 1, word));
        }

        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;

    #[test]
    fn test_ensure_extension() {
        assert_eq!(ensure_extension("test"), "test.txt");
        assert_eq!(ensure_extension("test.txt"), "test.txt");
        assert_eq!(ensure_extension("path/test"), "path/test.txt");
    }

    #[test]
    fn test_validate_output_path() {
        // Casos válidos
        assert!(validate_output_path("test.txt").is_ok());
        assert!(validate_output_path("path/test.txt").is_ok());

        // Casos inválidos
        assert!(validate_output_path("../test.txt").is_err());
        assert!(validate_output_path("test//test.txt").is_err());
        assert!(validate_output_path(&"x".repeat(300)).is_err()); // Muy largo
    }

    #[test]
    fn test_format_columns() {
        let text = "word1 word2 word3 word4 word5 word6";
        let result = format::in_columns(text, 3);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "word1 word2 word3");
        assert_eq!(result[1], "word4 word5 word6");
    }

    #[test]
    fn test_format_seed_phrase() {
        let phrase = "abandon ability able about";
        let formatted = format::format_seed_phrase(phrase);
        assert!(formatted.contains("1. abandon"));
        assert!(formatted.contains("4. about"));
    }

    #[test]
    fn test_save_to_file() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("scypher_test.txt");
        let test_content = "test content for scypher";

        // Limpiar archivo de prueba si existe
        let _ = fs::remove_file(&test_file);

        // Probar guardado
        let result = save_to_file(test_content, test_file.to_str().unwrap());
        assert!(result.is_ok());

        // Verificar contenido
        let saved_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(saved_content, test_content);

        // Limpiar
        let _ = fs::remove_file(&test_file);
    }
}

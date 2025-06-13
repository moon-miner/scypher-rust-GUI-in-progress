//! Protecciones del entorno de ejecución
//!
//! Este módulo maneja la configuración segura del entorno donde se ejecuta
//! SCypher, incluyendo variables de entorno y configuraciones del sistema.

use std::collections::HashMap;
use crate::error::{SCypherError, Result};

/// Lista de variables de entorno potencialmente peligrosas
const DANGEROUS_ENV_VARS: &[&str] = &[
    "LD_PRELOAD",
    "LD_LIBRARY_PATH",
    "LD_AUDIT",
    "DYLD_INSERT_LIBRARIES",
    "DYLD_LIBRARY_PATH",
    "BASH_ENV",
    "ENV",
    "SHELL",
    "IFS",
];

/// Variables de entorno que podrían contener información sensible
const SENSITIVE_ENV_VARS: &[&str] = &[
    "SCYPHER_PASSWORD",
    "SCYPHER_SEED",
    "SCYPHER_KEY",
    "WALLET_PASSWORD",
    "PRIVATE_KEY",
    "MNEMONIC",
    "SEED_PHRASE",
    "RECOVERY_PHRASE",
];

/// Configurar entorno seguro para la ejecución
pub fn setup_secure_environment() -> Result<()> {
    validate_environment_safety()?;
    clean_sensitive_variables();
    configure_secure_umask();
    validate_execution_context()?;

    Ok(())
}

/// Validar que el entorno de ejecución es seguro
pub fn validate_environment_safety() -> Result<()> {
    let mut warnings = Vec::new();
    let mut critical_issues = Vec::new();

    // Verificar variables peligrosas
    for &var in DANGEROUS_ENV_VARS {
        if std::env::var(var).is_ok() {
            warnings.push(format!("Potentially dangerous environment variable found: {}", var));
        }
    }

    // Verificar si estamos en un entorno virtualizado/containerizado
    if is_running_in_container() {
        warnings.push("Running in containerized environment".to_string());
    }

    // Verificar si hay depuradores activos
    if is_debugger_present() {
        critical_issues.push("Debugger or profiler detected".to_string());
    }

    // Verificar PATH seguro
    if let Ok(path) = std::env::var("PATH") {
        if path.contains(".") || path.contains("..") {
            warnings.push("PATH contains relative directories".to_string());
        }
    }

    // Reportar advertencias
    for warning in &warnings {
        eprintln!("Warning: {}", warning);
    }

    // Fallar en problemas críticos
    if !critical_issues.is_empty() {
        return Err(SCypherError::crypto(format!(
            "Critical security issues detected: {}",
            critical_issues.join(", ")
        )));
    }

    Ok(())
}

/// Limpiar variables de entorno sensibles
pub fn clean_sensitive_variables() {
    for &var in SENSITIVE_ENV_VARS {
        std::env::remove_var(var);
    }

    // También limpiar variables temporales comunes
    let temp_vars = ["TMPDIR", "TEMP", "TMP"];
    for &var in &temp_vars {
        if let Ok(value) = std::env::var(var) {
            // Verificar si la ubicación temporal es segura
            if !is_secure_temp_dir(&value) {
                std::env::remove_var(var);
            }
        }
    }
}

/// Configurar umask segura
pub fn configure_secure_umask() {
    #[cfg(unix)]
    {
        use libc::umask;

        unsafe {
            // Configurar umask 077 (solo propietario puede leer/escribir)
            umask(0o077);
        }
    }
}

/// Validar contexto de ejecución
pub fn validate_execution_context() -> Result<()> {
    // Verificar permisos del usuario
    #[cfg(unix)]
    {
        use libc::{getuid, geteuid};

        unsafe {
            let real_uid = getuid();
            let effective_uid = geteuid();

            // Advertir si hay diferencia entre UID real y efectivo
            if real_uid != effective_uid {
                eprintln!("Warning: Running with different real and effective UIDs");
            }

            // Advertir si se ejecuta como root sin necesidad
            if effective_uid == 0 {
                eprintln!("Warning: Running as root - consider using a regular user account");
            }
        }
    }

    // Verificar que no estamos en un entorno de desarrollo
    if is_development_environment() {
        eprintln!("Warning: Development environment detected");
    }

    Ok(())
}

/// Detectar si estamos ejecutando en un contenedor
fn is_running_in_container() -> bool {
    // Verificar indicadores comunes de contenedores
    std::path::Path::new("/.dockerenv").exists() ||
    std::path::Path::new("/run/.containerenv").exists() ||
    std::env::var("container").is_ok() ||
    check_cgroup_for_container()
}

/// Verificar cgroups para detectar contenedores
fn check_cgroup_for_container() -> bool {
    if let Ok(cgroup) = std::fs::read_to_string("/proc/1/cgroup") {
        cgroup.contains("docker") ||
        cgroup.contains("lxc") ||
        cgroup.contains("kubepods") ||
        cgroup.contains("containerd")
    } else {
        false
    }
}

/// Detectar presencia de depuradores
fn is_debugger_present() -> bool {
    #[cfg(target_os = "linux")]
    {
        // Verificar /proc/self/status para TracerPid
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("TracerPid:") {
                    if let Some(pid_str) = line.split_whitespace().nth(1) {
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            return pid != 0;
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // En macOS, verificar usando sysctl
        use std::process::Command;

        if let Ok(output) = Command::new("sysctl")
            .args(&["-n", "kern.proc.pid", &std::process::id().to_string()])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            return output_str.contains("P_TRACED");
        }
    }

    // Verificar variables de entorno de depuradores comunes
    let debugger_vars = [
        "RUST_GDB", "RUST_LLDB", "DEBUGGER",
        "VALGRIND_LIB", "MSAN_OPTIONS", "ASAN_OPTIONS"
    ];

    for &var in &debugger_vars {
        if std::env::var(var).is_ok() {
            return true;
        }
    }

    false
}

/// Verificar si un directorio temporal es seguro
fn is_secure_temp_dir(path: &str) -> bool {
    let path = std::path::Path::new(path);

    // Verificar que existe y es un directorio
    if !path.exists() || !path.is_dir() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(metadata) = path.metadata() {
            let permissions = metadata.permissions();
            let mode = permissions.mode();

            // Verificar que no sea world-writable
            if mode & 0o002 != 0 {
                return false;
            }

            // Verificar que el sticky bit esté configurado en directorios compartidos
            if mode & 0o001 != 0 && mode & 0o1000 == 0 {
                return false;
            }
        }
    }

    true
}

/// Detectar entorno de desarrollo
fn is_development_environment() -> bool {
    // Verificar variables de entorno de desarrollo
    let dev_vars = [
        "CARGO_MANIFEST_DIR", "RUST_SRC_PATH", "RUSTUP_HOME",
        "CARGO_HOME", "RUST_BACKTRACE", "RUST_LOG"
    ];

    for &var in &dev_vars {
        if std::env::var(var).is_ok() {
            return true;
        }
    }

    // Verificar directorios de desarrollo comunes
    let current_dir = std::env::current_dir().unwrap_or_default();
    let current_path = current_dir.to_string_lossy();

    current_path.contains("/target/") ||
    current_path.contains("/.cargo/") ||
    current_path.contains("/src/") ||
    std::path::Path::new("Cargo.toml").exists()
}

/// Obtener información del entorno para auditoría
pub fn get_environment_info() -> HashMap<String, String> {
    let mut info = HashMap::new();

    // Información básica del sistema
    info.insert("os".to_string(), std::env::consts::OS.to_string());
    info.insert("arch".to_string(), std::env::consts::ARCH.to_string());

    // Información del usuario
    #[cfg(unix)]
    {
        unsafe {
            info.insert("uid".to_string(), libc::getuid().to_string());
            info.insert("gid".to_string(), libc::getgid().to_string());
        }
    }

    // Estado del entorno
    info.insert("container".to_string(), is_running_in_container().to_string());
    info.insert("debugger".to_string(), is_debugger_present().to_string());
    info.insert("development".to_string(), is_development_environment().to_string());

    // Información de directorio actual
    if let Ok(current_dir) = std::env::current_dir() {
        info.insert("working_dir".to_string(), current_dir.to_string_lossy().to_string());
    }

    info
}

/// Configurar entorno limpio antes de operaciones sensibles
pub fn setup_clean_environment() -> Result<()> {
    // Limpiar variables sensibles
    clean_sensitive_variables();

    // Configurar variables mínimas necesarias
    std::env::set_var("LC_ALL", "C");
    std::env::set_var("LANG", "C");

    // Limpiar PATH a mínimo necesario
    let secure_path = "/usr/local/bin:/usr/bin:/bin";
    std::env::set_var("PATH", secure_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_validation() {
        // Test básico que no debería fallar en entorno normal
        assert!(validate_environment_safety().is_ok());
    }

    #[test]
    fn test_clean_sensitive_variables() {
        // Configurar variable sensible para test
        std::env::set_var("SCYPHER_PASSWORD", "test_password");

        // Limpiar
        clean_sensitive_variables();

        // Verificar que se limpió
        assert!(std::env::var("SCYPHER_PASSWORD").is_err());
    }

    #[test]
    fn test_get_environment_info() {
        let info = get_environment_info();

        // Verificar que contiene información básica
        assert!(info.contains_key("os"));
        assert!(info.contains_key("arch"));
    }

    #[test]
    fn test_secure_temp_dir() {
        // Test con directorio que sabemos que existe
        assert!(!is_secure_temp_dir("/nonexistent/path"));

        // Test con directorio actual (debería ser relativamente seguro)
        if let Ok(current) = std::env::current_dir() {
            // No siempre será seguro, pero no debería causar panic
            let _ = is_secure_temp_dir(&current.to_string_lossy());
        }
    }
}

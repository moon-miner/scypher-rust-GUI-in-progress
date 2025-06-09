// src/security/mod.rs - Módulo de seguridad y limpieza de memoria

pub mod memory;
pub mod process;
pub mod environment;

use std::sync::atomic::{AtomicBool, Ordering};
use zeroize::Zeroize;
use crate::error::Result;

// Flag global para rastrear si la limpieza está configurada
static CLEANUP_CONFIGURED: AtomicBool = AtomicBool::new(false);

/// Configurar limpieza de seguridad al inicio de la aplicación
pub fn setup_security_cleanup() {
    if CLEANUP_CONFIGURED.load(Ordering::Relaxed) {
        return; // Ya configurado
    }

    // Configurar handler para limpieza en caso de señales de terminación
    let _ = ctrlc::set_handler(move || {
        eprintln!("\nReceived termination signal. Performing secure cleanup...");
        secure_cleanup();
        std::process::exit(130); // 128 + 2 (SIGINT)
    });

    CLEANUP_CONFIGURED.store(true, Ordering::Relaxed);
}

/// Configurar protecciones completas de seguridad
pub fn setup_comprehensive_security() -> Result<()> {
    // Configurar protecciones de entorno
    environment::setup_secure_environment()?;

    // Configurar protecciones de proceso
    process::setup_process_protections()?;

    // Configurar límites de memoria
    memory::configure_memory_limits()?;

    // Intentar deshabilitar swap para el proceso
    if let Err(_) = memory::disable_swap_for_process() {
        eprintln!("Warning: Could not disable swap for process - sensitive data may be written to disk");
    }

    // Configurar limpieza de señales
    setup_security_cleanup();

    Ok(())
}

/// Limpieza segura de memoria al final de la aplicación
pub fn secure_cleanup() {
    // Limpiar variables de entorno sensibles si las hay
    clear_environment_variables();

    // Limpiar información del proceso
    process::cleanup_process_info();

    // Sobrescribir stack con ceros (mejor esfuerzo)
    let mut dummy_buffer = vec![0u8; 4096];
    dummy_buffer.zeroize();

    // Nota: En Rust, la limpieza automática de memoria es más segura
    // que en otros lenguajes debido al ownership system
}

/// Limpiar variables de entorno que podrían contener datos sensibles
fn clear_environment_variables() {
    // Variables que podrían contener información sensible
    let sensitive_vars = [
        "SCYPHER_PASSWORD",
        "SCYPHER_SEED",
        "TMPDIR",
        "TEMP",
        "TMP",
    ];

    for var in &sensitive_vars {
        std::env::remove_var(var);
    }
}

/// Wrapper seguro para strings sensibles
/// Implementa Drop para limpieza automática
pub struct SecureString {
    data: Vec<u8>,
}

impl SecureString {
    /// Crear nueva cadena segura
    pub fn new(s: &str) -> Self {
        Self {
            data: s.as_bytes().to_vec(),
        }
    }

    /// Obtener referencia como str (usar con cuidado)
    pub fn as_str(&self) -> &str {
        // SAFETY: Mantenemos la invariante de que data contiene UTF-8 válido
        unsafe { std::str::from_utf8_unchecked(&self.data) }
    }

    /// Obtener bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Longitud en bytes
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Verificar si está vacía
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // Sobrescribir con ceros antes de liberar
        self.data.zeroize();
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Estructura para manejar datos binarios sensibles con memoria bloqueada
pub struct SecureBytes {
    data: memory::LockedBuffer,
}

impl SecureBytes {
    /// Crear nuevo vector de bytes seguro
    pub fn new(data: Vec<u8>) -> Result<Self> {
        let locked_buffer = memory::LockedBuffer::from_vec(data)
            .map_err(|e| crate::error::SCypherError::crypto(format!("Failed to create secure buffer: {}", e)))?;

        Ok(Self { data: locked_buffer })
    }

    /// Crear desde slice
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        Self::new(slice.to_vec())
    }

    /// Crear buffer vacío de tamaño específico
    pub fn with_capacity(size: usize) -> Result<Self> {
        let locked_buffer = memory::LockedBuffer::new(size)
            .map_err(|e| crate::error::SCypherError::crypto(format!("Failed to create secure buffer: {}", e)))?;

        Ok(Self { data: locked_buffer })
    }

    /// Obtener referencia a los datos
    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// Obtener referencia mutable a los datos
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.data.as_mut_slice()
    }

    /// Longitud
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Verificar si está vacío
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Verificar si la memoria está bloqueada
    pub fn is_memory_locked(&self) -> bool {
        self.data.is_locked()
    }
}

/// Utilidades para operaciones seguras en memoria
pub mod utils {
    use super::*;

    /// Comparación constante en tiempo para evitar timing attacks
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result = 0u8;
        for (byte_a, byte_b) in a.iter().zip(b.iter()) {
            result |= byte_a ^ byte_b;
        }

        result == 0
    }

    /// Generar bytes aleatorios seguros
    pub fn secure_random_bytes(len: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = vec![0u8; len];
        rng.fill_bytes(&mut bytes);
        bytes
    }

    /// Limpiar un buffer con datos aleatorios antes de sobrescribir con ceros
    pub fn secure_wipe(buffer: &mut [u8]) {
        use rand::RngCore;
        let mut rng = rand::thread_rng();

        // Primer pase: datos aleatorios
        rng.fill_bytes(buffer);

        // Segundo pase: ceros
        buffer.zeroize();
    }

    /// Crear SecureString desde entrada de usuario
    pub fn secure_string_from_input(prompt: &str) -> Result<SecureString> {
        use rpassword::read_password;
        use std::io::{self, Write};

        print!("{}", prompt);
        io::stdout().flush().map_err(crate::error::SCypherError::from)?;

        let password = read_password()
            .map_err(|e| crate::error::SCypherError::crypto(format!("Failed to read secure input: {}", e)))?;

        Ok(SecureString::new(&password))
    }
}

/// Verificar el estado general de seguridad del sistema
pub fn security_audit() -> SecurityAuditReport {
    let mut report = SecurityAuditReport::new();

    // Auditar entorno
    if let Err(e) = environment::validate_environment_safety() {
        report.add_critical_issue(format!("Environment validation failed: {}", e));
    }

    // Verificar integridad del proceso
    if !process::check_process_integrity() {
        report.add_critical_issue("Process integrity check failed - debugger detected".to_string());
    }

    // Verificar límites de memoria
    let (current_limit, _max_limit) = memory::check_memory_lock_limits();
    if current_limit == 0 {
        report.add_warning("No memory locking limits configured".to_string());
    } else if current_limit < 64 * 1024 * 1024 {
        report.add_warning(format!("Low memory lock limit: {} bytes", current_limit));
    }

    // Verificar información del entorno
    let env_info = environment::get_environment_info();
    if env_info.get("container").unwrap_or(&"false".to_string()) == "true" {
        report.add_info("Running in containerized environment".to_string());
    }

    if env_info.get("development").unwrap_or(&"false".to_string()) == "true" {
        report.add_warning("Running in development environment".to_string());
    }

    report
}

/// Reporte de auditoría de seguridad
pub struct SecurityAuditReport {
    critical_issues: Vec<String>,
    warnings: Vec<String>,
    info: Vec<String>,
}

impl SecurityAuditReport {
    fn new() -> Self {
        Self {
            critical_issues: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        }
    }

    fn add_critical_issue(&mut self, issue: String) {
        self.critical_issues.push(issue);
    }

    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    fn add_info(&mut self, info: String) {
        self.info.push(info);
    }

    /// Verificar si hay problemas críticos
    pub fn has_critical_issues(&self) -> bool {
        !self.critical_issues.is_empty()
    }

    /// Obtener todos los problemas críticos
    pub fn critical_issues(&self) -> &[String] {
        &self.critical_issues
    }

    /// Obtener todas las advertencias
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Obtener toda la información
    pub fn info(&self) -> &[String] {
        &self.info
    }

    /// Generar reporte legible
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        if !self.critical_issues.is_empty() {
            report.push_str("CRITICAL ISSUES:\n");
            for issue in &self.critical_issues {
                report.push_str(&format!("  ❌ {}\n", issue));
            }
            report.push('\n');
        }

        if !self.warnings.is_empty() {
            report.push_str("WARNINGS:\n");
            for warning in &self.warnings {
                report.push_str(&format!("  ⚠️  {}\n", warning));
            }
            report.push('\n');
        }

        if !self.info.is_empty() {
            report.push_str("INFORMATION:\n");
            for info in &self.info {
                report.push_str(&format!("  ℹ️  {}\n", info));
            }
        }

        if report.is_empty() {
            report.push_str("✅ No security issues detected\n");
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_string() {
        let original = "sensitive data";
        let secure = SecureString::new(original);

        assert_eq!(secure.as_str(), original);
        assert_eq!(secure.len(), original.len());
        assert!(!secure.is_empty());
    }

    #[test]
    fn test_secure_bytes() {
        let data = vec![1, 2, 3, 4, 5];
        let secure = SecureBytes::new(data.clone()).unwrap();

        assert_eq!(secure.as_slice(), &data);
        assert_eq!(secure.len(), data.len());
        assert!(!secure.is_empty());
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(utils::constant_time_eq(b"hello", b"hello"));
        assert!(!utils::constant_time_eq(b"hello", b"world"));
        assert!(!utils::constant_time_eq(b"hello", b"hell"));  // Diferente longitud
    }

    #[test]
    fn test_secure_random_bytes() {
        let bytes1 = utils::secure_random_bytes(16);
        let bytes2 = utils::secure_random_bytes(16);

        assert_eq!(bytes1.len(), 16);
        assert_eq!(bytes2.len(), 16);
        assert_ne!(bytes1, bytes2); // Extremadamente improbable que sean iguales
    }

    #[test]
    fn test_security_audit() {
        let report = security_audit();
        // No debería causar panic
        let _report_text = report.generate_report();
    }
}

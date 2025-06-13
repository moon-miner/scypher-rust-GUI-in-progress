//! Protecciones de seguridad a nivel de proceso
//!
//! Este módulo proporciona protecciones específicas para procesos que manejan
//! datos sensibles como claves privadas de criptomonedas.

use std::ffi::CString;
use crate::error::{SCypherError, Result};

/// Configurar protecciones básicas de proceso
pub fn setup_process_protections() -> Result<()> {
    disable_core_dumps()?;
    setup_anti_debugging()?;
    configure_process_isolation()?;

    Ok(())
}

/// Deshabilitar core dumps para prevenir filtración de datos sensibles
pub fn disable_core_dumps() -> Result<()> {
    #[cfg(unix)]
    {
        use libc::{setrlimit, rlimit, RLIMIT_CORE};

        let rlim = rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };

        unsafe {
            if setrlimit(RLIMIT_CORE, &rlim) != 0 {
                return Err(SCypherError::crypto(
                    "Failed to disable core dumps".to_string()
                ));
            }
        }
    }

    #[cfg(not(unix))]
    {
        // En sistemas Windows, usar SetErrorMode
        #[cfg(windows)]
        {
            use winapi::um::errhandlingapi::SetErrorMode;
            use winapi::um::winbase::SEM_NOGPFAULTERRORBOX;

            unsafe {
                SetErrorMode(SEM_NOGPFAULTERRORBOX);
            }
        }
    }

    Ok(())
}

/// Configurar protecciones anti-debugging básicas
pub fn setup_anti_debugging() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use libc::{prctl, PR_SET_DUMPABLE};

        unsafe {
            // Prevenir que otros procesos hagan ptrace a este proceso
            if prctl(PR_SET_DUMPABLE, 0, 0, 0, 0) != 0 {
                return Err(SCypherError::crypto(
                    "Failed to set anti-debugging protection".to_string()
                ));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use libc::{ptrace, PT_DENY_ATTACH};

        unsafe {
            // En macOS, usar PT_DENY_ATTACH
            if ptrace(PT_DENY_ATTACH, 0, 0, 0) != 0 {
                // No es crítico si falla en macOS
                eprintln!("Warning: Could not set anti-debugging protection on macOS");
            }
        }
    }

    Ok(())
}

/// Configurar aislamiento de proceso donde sea posible
pub fn configure_process_isolation() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        // Intentar usar seccomp para restringir syscalls peligrosas
        setup_seccomp_filter()?;
    }

    // Configurar umask restrictiva
    #[cfg(unix)]
    {
        use libc::umask;
        unsafe {
            umask(0o077); // Solo el propietario puede leer/escribir
        }
    }

    Ok(())
}

/// Configurar filtro seccomp para restringir syscalls (Linux)
#[cfg(target_os = "linux")]
fn setup_seccomp_filter() -> Result<()> {
    // Implementación básica - en producción se usaría libseccomp
    // Por ahora, solo reportamos que está disponible

    // Verificar si seccomp está disponible
    if std::path::Path::new("/proc/sys/kernel/seccomp").exists() {
        // Sistema soporta seccomp
        // En implementación completa, configurar whitelist de syscalls

        // Por ahora, solo configurar prctl básico
        use libc::{prctl, PR_SET_NO_NEW_PRIVS};

        unsafe {
            if prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                return Err(SCypherError::crypto(
                    "Failed to set no new privileges".to_string()
                ));
            }
        }
    }

    Ok(())
}

/// Verificar integridad del proceso (detección de debugging activo)
pub fn check_process_integrity() -> bool {
    #[cfg(target_os = "linux")]
    {
        // Verificar si hay debuggers attachados leyendo /proc/self/status
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("TracerPid:") {
                    let tracer_pid: u32 = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);

                    if tracer_pid != 0 {
                        return false; // Debugger detectado
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
            .args(&["kern.proc.pid", &std::process::id().to_string()])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Verificar flags de debugging en la salida
            if output_str.contains("P_TRACED") {
                return false;
            }
        }
    }

    true
}

/// Limpiar información del proceso al salir
pub fn cleanup_process_info() {
    #[cfg(unix)]
    {
        // Cambiar título del proceso para limpiar información sensible
        if let Ok(name) = CString::new("cleaned_process") {
            unsafe {
                libc::prctl(libc::PR_SET_NAME, name.as_ptr(), 0, 0, 0);
            }
        }
    }

    // Limpiar variables de entorno sensibles
    std::env::remove_var("SCYPHER_PASSWORD");
    std::env::remove_var("SCYPHER_SEED");
    std::env::remove_var("RUST_LOG");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disable_core_dumps() {
        // Test que core dumps están deshabilitados
        assert!(disable_core_dumps().is_ok());
    }

    #[test]
    fn test_process_integrity_check() {
        // En condiciones normales, no debería haber debugger
        assert!(check_process_integrity());
    }

    #[test]
    fn test_cleanup_process_info() {
        // Test que cleanup no cause panic
        cleanup_process_info();
    }
}

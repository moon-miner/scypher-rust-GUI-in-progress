//! Limpieza segura de memoria
//!
//! Este módulo proporciona utilidades para el manejo seguro de memoria,
//! incluyendo limpieza de datos sensibles y verificaciones de integridad.

use zeroize::Zeroize;

/// Limpiar buffer de memoria de forma segura
/// Sobrescribe con datos aleatorios antes de poner en ceros
pub fn secure_clear(buffer: &mut [u8]) {
    // Primer pase: datos aleatorios
    use rand::RngCore;
    rand::thread_rng().fill_bytes(buffer);

    // Segundo pase: ceros
    buffer.zeroize();
}

/// Verificar integridad de memoria básica
/// Retorna true si la memoria parece estar íntegra
pub fn check_memory_integrity() -> bool {
    // Test básico: allocar y verificar que podemos escribir/leer
    let mut test_buffer = vec![0u8; 1024];

    // Escribir patrón
    for (i, byte) in test_buffer.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }

    // Verificar patrón
    let is_intact = test_buffer.iter().enumerate().all(|(i, &byte)| {
        byte == (i % 256) as u8
    });

    // Limpiar buffer de prueba
    secure_clear(&mut test_buffer);

    is_intact
}

/// Limpieza profunda de un vector
pub fn deep_clear_vec<T: Zeroize>(vec: &mut Vec<T>) {
    // Limpiar cada elemento
    for item in vec.iter_mut() {
        item.zeroize();
    }

    // Limpiar y reducir capacidad
    vec.clear();
    vec.shrink_to_fit();
}

/// Wrapper para strings que se autolimpian
pub struct SecureBuffer {
    data: Vec<u8>,
}

impl SecureBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
        }
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            data: slice.to_vec(),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        secure_clear(&mut self.data);
    }
}

impl Zeroize for SecureBuffer {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

/// Bloquear memoria para prevenir que vaya a swap
pub fn lock_memory(ptr: *mut u8, size: usize) -> Result<(), std::io::Error> {
    #[cfg(unix)]
    {
        use libc::mlock;

        unsafe {
            if mlock(ptr as *const libc::c_void, size) != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
    }

    #[cfg(windows)]
    {
        use winapi::um::memoryapi::VirtualLock;

        unsafe {
            if VirtualLock(ptr as *mut libc::c_void, size) == 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
    }

    Ok(())
}

/// Desbloquear memoria bloqueada
pub fn unlock_memory(ptr: *mut u8, size: usize) -> Result<(), std::io::Error> {
    #[cfg(unix)]
    {
        use libc::munlock;

        unsafe {
            if munlock(ptr as *const libc::c_void, size) != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
    }

    #[cfg(windows)]
    {
        use winapi::um::memoryapi::VirtualUnlock;

        unsafe {
            if VirtualUnlock(ptr as *mut libc::c_void, size) == 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
    }

    Ok(())
}

/// Buffer seguro con memoria bloqueada
pub struct LockedBuffer {
    data: Vec<u8>,
    locked: bool,
}

impl LockedBuffer {
    /// Crear nuevo buffer bloqueado en memoria
    pub fn new(size: usize) -> std::io::Result<Self> {
        let mut data = vec![0u8; size];

        // Intentar bloquear la memoria
        let locked = lock_memory(data.as_mut_ptr(), size).is_ok();

        if !locked {
            eprintln!("Warning: Could not lock memory - data may be swapped to disk");
        }

        Ok(Self { data, locked })
    }

    /// Crear desde datos existentes
    pub fn from_vec(mut data: Vec<u8>) -> std::io::Result<Self> {
        let size = data.len();
        let locked = lock_memory(data.as_mut_ptr(), size).is_ok();

        if !locked {
            eprintln!("Warning: Could not lock memory - data may be swapped to disk");
        }

        Ok(Self { data, locked })
    }

    /// Obtener slice de solo lectura
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Obtener slice mutable
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Longitud del buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Verificar si está vacío
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Verificar si la memoria está bloqueada
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

impl Drop for LockedBuffer {
    fn drop(&mut self) {
        // Limpiar contenido
        secure_clear(&mut self.data);

        // Desbloquear memoria si estaba bloqueada
        if self.locked {
            let _ = unlock_memory(self.data.as_mut_ptr(), self.data.len());
        }
    }
}

impl Zeroize for LockedBuffer {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

/// Prevenir que las páginas actuales vayan a swap
pub fn disable_swap_for_process() -> Result<(), std::io::Error> {
    #[cfg(target_os = "linux")]
    {
        use libc::mlockall;
        use libc::{MCL_CURRENT, MCL_FUTURE};

        unsafe {
            // Bloquear todas las páginas actuales y futuras
            if mlockall(MCL_CURRENT | MCL_FUTURE) != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("Warning: Process-wide memory locking not supported on this platform");
    }

    Ok(())
}

/// Verificar límites de memoria bloqueada
pub fn check_memory_lock_limits() -> (usize, usize) {
    #[cfg(unix)]
    {
        use libc::{getrlimit, rlimit, RLIMIT_MEMLOCK};

        let mut rlim = rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };

        unsafe {
            if getrlimit(RLIMIT_MEMLOCK, &mut rlim) == 0 {
                return (rlim.rlim_cur as usize, rlim.rlim_max as usize);
            }
        }
    }

    (0, 0) // Default en caso de error o plataforma no soportada
}

/// Configurar límites de memoria bloqueada si es posible
pub fn configure_memory_limits() -> Result<(), std::io::Error> {
    #[cfg(unix)]
    {
        use libc::{setrlimit, rlimit, RLIMIT_MEMLOCK};

        let (current, max) = check_memory_lock_limits();

        // Si el límite actual es muy bajo, intentar aumentarlo
        if current < 64 * 1024 * 1024 { // 64MB
            let new_limit = std::cmp::min(max, 128 * 1024 * 1024); // 128MB

            let rlim = rlimit {
                rlim_cur: new_limit as u64,
                rlim_max: max as u64,
            };

            unsafe {
                if setrlimit(RLIMIT_MEMLOCK, &rlim) != 0 {
                    eprintln!("Warning: Could not increase memory lock limit");
                }
            }
        }
    }

    Ok(())
}

// Agregar estos tests adicionales a la sección #[cfg(test)]

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_locked_buffer_creation() {
        let buffer = LockedBuffer::new(1024);
        assert!(buffer.is_ok());

        let buffer = buffer.unwrap();
        assert_eq!(buffer.len(), 1024);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_locked_buffer_from_vec() {
        let data = vec![1, 2, 3, 4, 5];
        let buffer = LockedBuffer::from_vec(data);
        assert!(buffer.is_ok());

        let buffer = buffer.unwrap();
        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.as_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_memory_lock_limits() {
        let (current, max) = check_memory_lock_limits();
        // En la mayoría de sistemas, debería haber algún límite
        // Pero en contenedores puede ser 0, así que no podemos asumir valores específicos
        assert!(max >= current);
    }

    #[test]
    fn test_configure_memory_limits() {
        // Test que no debería fallar (puede mostrar warnings)
        assert!(configure_memory_limits().is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_clear() {
        let mut buffer = vec![0xFF; 32];
        secure_clear(&mut buffer);

        // Buffer debería estar lleno de ceros
        assert!(buffer.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_check_memory_integrity() {
        // Esta función siempre debería retornar true en condiciones normales
        assert!(check_memory_integrity());
    }

    #[test]
    fn test_deep_clear_vec() {
        let mut vec = vec![vec![1u8, 2, 3], vec![4, 5, 6]];
        deep_clear_vec(&mut vec);

        assert!(vec.is_empty());
        assert_eq!(vec.capacity(), 0); // shrink_to_fit debería reducir capacidad
    }

    #[test]
    fn test_secure_buffer() {
        let mut buffer = SecureBuffer::new(16);
        buffer.as_mut_slice().fill(0xFF);

        assert_eq!(buffer.len(), 16);
        assert!(!buffer.is_empty());
        assert!(buffer.as_slice().iter().all(|&b| b == 0xFF));

        // Al salir del scope, el drop debería limpiar automáticamente
    }

    #[test]
    fn test_secure_buffer_from_slice() {
        let data = b"sensitive data";
        let buffer = SecureBuffer::from_slice(data);

        assert_eq!(buffer.as_slice(), data);
        assert_eq!(buffer.len(), data.len());
    }
}

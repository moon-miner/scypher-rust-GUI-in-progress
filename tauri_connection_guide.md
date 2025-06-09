# 🔗 Guía de Conexión Frontend-Backend para SCypher GUI

## 📋 Información del Proyecto

**Proyecto:** SCypher GUI v3.0  
**Framework:** Tauri v1.x  
**Backend:** Rust  
**Frontend:** HTML/CSS/JavaScript (Vanilla)  
**Protocolo:** Tauri IPC (Inter-Process Communication)  

---

## 🚨 PROBLEMA COMÚN: "Tauri not available" o "invoke is not a function"

### ❌ Síntomas del problema:
- Console log: `"❌ Tauri not available, falling back to mock"`
- Error: `"invoke is not a function"`
- Error: `"window.__TAURI__ is undefined"`
- La aplicación funciona solo con mock backend

### ✅ SOLUCIÓN CONFIRMADA (Ya implementada):

#### **Frontend (JavaScript) - Importación correcta:**

```javascript
// MÉTODO 1: Import dinámico desde CDN (FUNCIONA)
async function loadTauriInvoke() {
    try {
        const { invoke: tauriInvoke } = await import('https://unpkg.com/@tauri-apps/api@1/tauri');
        console.log('✅ Imported invoke from @tauri-apps/api');
        return tauriInvoke;
    } catch (error) {
        console.log('❌ CDN import failed:', error);
        return null;
    }
}

// FUNCIÓN DE DETECCIÓN ROBUSTA
async function detectTauri() {
    const tauriInvoke = await loadTauriInvoke();
    
    if (tauriInvoke) {
        invoke = tauriInvoke;  // Variable global
        tauriAvailable = true;
        return true;
    }
    
    return false;
}
```

#### **Backend (Rust) - Configuración commands.rs:**

```rust
// Importaciones necesarias
use tauri::command;
use serde::{Deserialize, Serialize};
use crate::error::{SCypherError, Result};

// ESTRUCTURA DE COMANDO ESTÁNDAR
#[derive(Serialize, Deserialize)]
pub struct CommandResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

// MANEJO DE TIPOS FLEXIBLES (CRÍTICO)
#[command]
pub fn example_command(flexible_param: serde_json::Value) -> Result<String> {
    // Parsear parámetro de manera flexible
    let parsed_value: usize = match flexible_param {
        serde_json::Value::Number(n) => {
            if let Some(num) = n.as_u64() {
                num as usize
            } else {
                return Err(SCypherError::crypto("Invalid number".to_string()));
            }
        }
        serde_json::Value::String(s) => {
            s.parse::<usize>()
                .map_err(|_| SCypherError::crypto(format!("Cannot parse '{}'", s)))?
        }
        _ => return Err(SCypherError::crypto("Invalid parameter type".to_string())),
    };
    
    // Tu lógica aquí...
    Ok(format!("Processed: {}", parsed_value))
}
```

#### **main.rs - Registro de comandos:**

```rust
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // Registra TODOS los comandos aquí
            commands::validate_seed_phrase,
            commands::transform_seed_phrase,
            commands::get_bip39_wordlist,
            commands::validate_bip39_word,
            commands::generate_seed_phrase,
            // Agrega nuevos comandos aquí
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## 🎯 PATRÓN DE IMPLEMENTACIÓN PARA NUEVAS CARACTERÍSTICAS

### **1. Backend (Rust) - Agregar nuevo comando:**

```rust
// En src-tauri/src/commands.rs

#[command]
pub fn new_feature_command(
    param1: String,
    param2: serde_json::Value,  // Usar para tipos flexibles
    param3: Option<String>,     // Usar para parámetros opcionales
) -> Result<YourResponseType> {
    // Validar parámetros
    // Procesar lógica
    // Retornar resultado
    Ok(result)
}
```

### **2. Registrar en main.rs:**

```rust
// En src-tauri/src/main.rs
.invoke_handler(tauri::generate_handler![
    // ... comandos existentes ...
    commands::new_feature_command,  // ← AGREGAR AQUÍ
])
```

### **3. Frontend (JavaScript) - Usar comando:**

```javascript
// Llamar al comando desde el frontend
async function useNewFeature() {
    try {
        const result = await invoke('new_feature_command', {
            param1: "valor",
            param2: 123,           // Puede ser string o number
            param3: null           // Opcional
        });
        
        console.log('✅ Success:', result);
        return result;
    } catch (error) {
        console.error('❌ Error:', error);
        throw error;
    }
}
```

---

## 🔧 CONFIGURACIÓN TAURI CRÍTICA

### **tauri.conf.json - Configuración mínima:**

```json
{
  "tauri": {
    "allowlist": {
      "all": false,
      "fs": {
        "readFile": true,
        "writeFile": true,
        "exists": true
      },
      "dialog": {
        "open": true,
        "save": true
      },
      "clipboard": {
        "all": true
      }
    }
  }
}
```

### **Cargo.toml - Dependencias Tauri:**

```toml
[dependencies]
tauri = { version = "1.8", features = [ 
    "dialog-open", 
    "fs-read-file", 
    "fs-exists", 
    "clipboard-all", 
    "dialog-save", 
    "fs-write-file"
] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## 🧪 TESTING Y DEBUGGING

### **Frontend - Test de conexión:**

```javascript
// Función de diagnóstico completo
async function fullDiagnostic() {
    console.log('=== TAURI DIAGNOSTIC ===');
    console.log('window.__TAURI__:', window.__TAURI__);
    console.log('invoke function:', typeof invoke);
    console.log('Tauri available:', tauriAvailable);
    
    // Test de comando simple
    try {
        const result = await invoke('validate_bip39_word', { word: 'abandon' });
        console.log('✅ Backend test successful:', result);
    } catch (error) {
        console.error('❌ Backend test failed:', error);
    }
}
```

### **Comando de test en backend:**

```rust
// Comando simple para testing
#[command]
pub fn health_check() -> Result<String> {
    Ok("Backend healthy".to_string())
}
```

---

## 📚 ERRORES COMUNES Y SOLUCIONES

### **Error 1:** `"command not found"`
**Causa:** Comando no registrado en `main.rs`  
**Solución:** Agregar a `tauri::generate_handler![]`

### **Error 2:** `"invalid args for command"`
**Causa:** Nombre de parámetro no coincide entre frontend y backend  
**Solución:** Verificar nombres exactos: `{ word_count: value }` ↔ `word_count: Type`

### **Error 3:** `"Cannot parse parameter"`
**Causa:** Tipo de dato incompatible  
**Solución:** Usar `serde_json::Value` para tipos flexibles

### **Error 4:** `"Tauri not available"`
**Causa:** Import incorrecto de `invoke`  
**Solución:** Usar el método de import dinámico mostrado arriba

---

## ✅ CHECKLIST PARA NUEVAS IMPLEMENTACIONES

- [ ] **Backend:** Comando implementado en `commands.rs`
- [ ] **Backend:** Comando registrado en `main.rs`
- [ ] **Backend:** Tipos de parámetros compatibles con frontend
- [ ] **Frontend:** Import correcto de `invoke`
- [ ] **Frontend:** Nombres de parámetros coinciden exactamente
- [ ] **Testing:** Comando funciona con datos reales
- [ ] **Error handling:** Manejo apropiado de errores

---

## 🎯 PATRÓN EXITOSO CONFIRMADO

**Este patrón ya está funcionando en SCypher:**

```javascript
// ✅ ESTO FUNCIONA
const result = await invoke('validate_seed_phrase', { 
    phrase: "abandon abandon abandon..." 
});

// ✅ ESTO TAMBIÉN FUNCIONA  
const wordlist = await invoke('get_bip39_wordlist');

// ✅ TIPOS FLEXIBLES FUNCIONAN
const generated = await invoke('generate_seed_phrase', { 
    word_count: "12"  // String o number, ambos funcionan
});
```

---

## 🚀 NOTAS PARA LA PRÓXIMA IA

1. **NO cambiar** el método de import de `invoke` - ya funciona
2. **USAR** `serde_json::Value` para parámetros que pueden ser string o number
3. **REGISTRAR** todos los comandos nuevos en `main.rs`
4. **PROBAR** siempre con datos reales antes de entregar
5. **MANTENER** la estructura de error handling existente

---

## 📄 ARCHIVO DE REFERENCIA

**Archivo principal de conexión:** `src/index.html` (líneas 280-350)  
**Comandos backend:** `src-tauri/src/commands.rs`  
**Registro de comandos:** `src-tauri/src/main.rs`  

---

**✅ Este documento garantiza que el patrón de conexión Tauri funcione correctamente en futuras implementaciones.**